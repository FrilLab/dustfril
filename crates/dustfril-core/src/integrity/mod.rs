//! Non-executing executable-integrity inspection and baseline comparison.
//!
//! This module only reads filesystem metadata and executable bytes. It never
//! launches a requested tool, asks it for a version, or changes the inspected
//! path.

mod baseline;
mod comparator;
mod hasher;
mod resolver;

pub use baseline::BaselineStore;
pub use resolver::{ResolvedExecutable, ToolResolver};

use std::path::Path;

use crate::{
    error::{DustError, DustResult},
    models::{
        ExecutableObservation, IntegrityBaseline, IntegrityCheck, IntegrityFailureKind,
        IntegrityReport, IntegrityStatus, ToolSpec,
    },
};

/// The initial set of representative development tools.
pub fn default_tools() -> Vec<ToolSpec> {
    ["node", "bun", "cargo", "rustc", "git", "java", "gradle"]
        .into_iter()
        .map(ToolSpec::from)
        .collect()
}

/// Returns the default local state path and creates its parent data directory.
pub fn state_path() -> std::io::Result<std::path::PathBuf> {
    baseline::default_state_path()
}

/// Inspects one selected tool without comparing or persisting a baseline.
pub fn inspect_tool(
    tool: &ToolSpec,
    resolver: &ToolResolver,
) -> Result<ExecutableObservation, crate::models::IntegrityFailure> {
    let resolved = resolver.resolve(tool)?;
    hasher::observe(resolved)
}

/// Resolves and compares selected tools using the process environment's PATH.
pub fn scan(tools: &[ToolSpec], baseline_path: &Path) -> DustResult<IntegrityReport> {
    let resolver = ToolResolver::from_environment().map_err(DustError::Io)?;
    scan_with_resolver(tools, &resolver, baseline_path)
}

/// Resolves and compares selected tools using an explicit resolver.
pub fn scan_with_resolver(
    tools: &[ToolSpec],
    resolver: &ToolResolver,
    baseline_path: &Path,
) -> DustResult<IntegrityReport> {
    let store = BaselineStore::new(baseline_path);

    store.update(|baseline| {
        let checks = tools
            .iter()
            .map(|tool| check_tool(tool, resolver, baseline))
            .collect();

        Ok(IntegrityReport { checks })
    })
}

fn check_tool(
    tool: &ToolSpec,
    resolver: &ToolResolver,
    baseline: &mut IntegrityBaseline,
) -> IntegrityCheck {
    let previous_observation = baseline.observations.get(&tool.name).cloned();

    match inspect_tool(tool, resolver) {
        Ok(observation) => {
            let status = previous_observation
                .as_ref()
                .map_or(IntegrityStatus::NewBaseline, |previous| {
                    comparator::compare(previous, &observation)
                });

            // A successful observation becomes the next comparison point. A
            // failed observation never removes or overwrites the last good
            // record, so missing/unreadable tools remain diagnosable.
            baseline
                .observations
                .insert(tool.name.clone(), observation.clone());

            IntegrityCheck {
                requested_tool: tool.name.clone(),
                status,
                observation: Some(observation),
                previous_observation,
                failure: None,
            }
        }
        Err(failure) => {
            let status = if failure.kind == IntegrityFailureKind::NotFound {
                IntegrityStatus::Missing
            } else {
                IntegrityStatus::InspectionFailed
            };

            IntegrityCheck {
                requested_tool: tool.name.clone(),
                status,
                observation: None,
                previous_observation,
                failure: Some(failure),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    use super::*;
    use crate::models::{IntegrityFailureKind, IntegrityStatus};

    fn tool(name: &str) -> ToolSpec {
        ToolSpec::from(name)
    }

    fn write_tool(directory: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = directory.join(name);
        fs::write(&path, bytes).unwrap();
        make_executable(&path);
        path
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    fn scan_one(tool_name: &str, resolver: &ToolResolver, baseline_path: &Path) -> IntegrityCheck {
        scan_with_resolver(&[tool(tool_name)], resolver, baseline_path)
            .unwrap()
            .checks
            .into_iter()
            .next()
            .unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn default_tools_include_the_initial_developer_tool_set() {
        assert_eq!(
            default_tools()
                .into_iter()
                .map(|tool| tool.name)
                .collect::<Vec<_>>(),
            ["node", "bun", "cargo", "rustc", "git", "java", "gradle"]
        );
    }

    #[test]
    fn first_observation_is_baseline_then_bytes_are_compared() {
        let temp = TempDir::new().unwrap();
        let tool_path = write_tool(temp.path(), "node", b"first version");
        let state_path = temp.path().join("integrity.json");
        let resolver = ToolResolver::from_paths([temp.path().to_path_buf()]);

        let first_report = scan_with_resolver(&[tool("node")], &resolver, &state_path).unwrap();
        assert!(!first_report.has_changes());
        let first = first_report.checks.into_iter().next().unwrap();
        assert_eq!(first.status, IntegrityStatus::NewBaseline);
        assert_eq!(first.observation.as_ref().unwrap().size_bytes, 13);
        assert_eq!(
            first.observation.as_ref().unwrap().sha256,
            sha256(b"first version")
        );
        assert!(first.failure.is_none());

        let unchanged = scan_one("node", &resolver, &state_path);
        assert_eq!(unchanged.status, IntegrityStatus::Unchanged);

        fs::write(&tool_path, b"replacement version").unwrap();
        let changed = scan_one("node", &resolver, &state_path);
        assert_eq!(changed.status, IntegrityStatus::ContentChanged);
        assert_eq!(
            changed.previous_observation.unwrap().sha256,
            sha256(b"first version")
        );
        assert_eq!(
            changed.observation.unwrap().sha256,
            sha256(b"replacement version")
        );
    }

    #[test]
    fn path_resolution_change_is_distinguished_from_content_change() {
        let temp = TempDir::new().unwrap();
        let first_directory = temp.path().join("first");
        let second_directory = temp.path().join("second");
        fs::create_dir(&first_directory).unwrap();
        fs::create_dir(&second_directory).unwrap();
        write_tool(&first_directory, "node", b"same bytes");
        write_tool(&second_directory, "node", b"same bytes");
        let state_path = temp.path().join("integrity.json");

        let first_resolver = ToolResolver::from_paths([first_directory.clone()]);
        assert_eq!(
            scan_one("node", &first_resolver, &state_path).status,
            IntegrityStatus::NewBaseline
        );

        let second_resolver = ToolResolver::from_paths([second_directory.clone()]);
        let changed = scan_one("node", &second_resolver, &state_path);
        assert_eq!(changed.status, IntegrityStatus::ResolvedPathChanged);
        assert_eq!(
            changed.observation.unwrap().resolved_path,
            second_directory.join("node")
        );
    }

    #[cfg(unix)]
    #[test]
    fn path_lookup_skips_a_non_executable_shadow_candidate() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let shadow_directory = temp.path().join("shadow");
        let executable_directory = temp.path().join("executable");
        fs::create_dir(&shadow_directory).unwrap();
        fs::create_dir(&executable_directory).unwrap();
        let shadow = write_tool(&shadow_directory, "node", b"not runnable");
        write_tool(&executable_directory, "node", b"runnable");
        let mut permissions = fs::metadata(&shadow).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&shadow, permissions).unwrap();
        let resolver = ToolResolver::from_paths([shadow_directory, executable_directory.clone()]);

        let observation = inspect_tool(&tool("node"), &resolver).unwrap();

        assert_eq!(
            observation.canonical_path,
            fs::canonicalize(executable_directory.join("node")).unwrap()
        );
    }

    #[test]
    fn missing_tool_is_reported_without_erasing_previous_baseline() {
        let temp = TempDir::new().unwrap();
        let state_path = temp.path().join("integrity.json");
        let directory = temp.path().join("tools");
        fs::create_dir(&directory).unwrap();
        write_tool(&directory, "node", b"node");
        let resolver = ToolResolver::from_paths([directory.clone()]);

        assert_eq!(
            scan_one("node", &resolver, &state_path).status,
            IntegrityStatus::NewBaseline
        );
        fs::remove_file(directory.join("node")).unwrap();

        let missing_report = scan_with_resolver(&[tool("node")], &resolver, &state_path).unwrap();
        assert!(missing_report.has_changes());
        let missing = missing_report.checks.into_iter().next().unwrap();
        assert_eq!(missing.status, IntegrityStatus::Missing);
        assert_eq!(
            missing.failure.unwrap().kind,
            IntegrityFailureKind::NotFound
        );
        assert_eq!(
            BaselineStore::new(&state_path)
                .load()
                .unwrap()
                .observations
                .len(),
            1
        );
    }

    #[test]
    fn non_regular_target_is_an_inspection_failure() {
        let temp = TempDir::new().unwrap();
        let directory = temp.path().join("tools");
        fs::create_dir(&directory).unwrap();
        fs::create_dir(directory.join("node")).unwrap();
        let resolver = ToolResolver::from_paths([directory]);

        let check = scan_one("node", &resolver, &temp.path().join("integrity.json"));
        assert_eq!(check.status, IntegrityStatus::InspectionFailed);
        assert_eq!(
            check.failure.unwrap().kind,
            IntegrityFailureKind::NonRegularFile
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_observation_records_target_and_detects_target_change() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let directory = temp.path().join("tools");
        fs::create_dir(&directory).unwrap();
        let first_target = write_tool(&directory, "node-v1", b"v1");
        let second_target = write_tool(&directory, "node-v2", b"v2");
        let link = directory.join("node");
        symlink(&first_target, &link).unwrap();
        let resolver = ToolResolver::from_paths([directory.clone()]);
        let state_path = temp.path().join("integrity.json");

        let first = scan_one("node", &resolver, &state_path);
        assert_eq!(first.status, IntegrityStatus::NewBaseline);
        let observation = first.observation.unwrap();
        assert_eq!(
            observation.canonical_path,
            fs::canonicalize(&first_target).unwrap()
        );
        assert_eq!(observation.symlink_target, Some(first_target.clone()));

        fs::remove_file(&link).unwrap();
        symlink(&second_target, &link).unwrap();
        let changed = scan_one("node", &resolver, &state_path);
        assert_eq!(changed.status, IntegrityStatus::ResolvedPathChanged);
        assert_eq!(
            changed.observation.unwrap().canonical_path,
            fs::canonicalize(second_target).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn broken_symlink_is_not_silently_skipped() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let directory = temp.path().join("tools");
        fs::create_dir(&directory).unwrap();
        symlink(directory.join("missing-target"), directory.join("node")).unwrap();
        let resolver = ToolResolver::from_paths([directory]);

        let check = scan_one("node", &resolver, &temp.path().join("integrity.json"));
        assert_eq!(check.status, IntegrityStatus::InspectionFailed);
        assert_eq!(
            check.failure.unwrap().kind,
            IntegrityFailureKind::BrokenSymlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_is_reported_as_an_inspection_failure() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let directory = temp.path().join("tools");
        fs::create_dir(&directory).unwrap();
        symlink(directory.join("loop-b"), directory.join("loop-a")).unwrap();
        symlink(directory.join("loop-a"), directory.join("loop-b")).unwrap();
        let resolver = ToolResolver::from_paths([directory]);

        let check = scan_one("loop-a", &resolver, &temp.path().join("integrity.json"));
        assert_eq!(check.status, IntegrityStatus::InspectionFailed);
        assert_eq!(
            check.failure.unwrap().kind,
            IntegrityFailureKind::SymlinkLoop
        );
    }

    #[test]
    fn large_fixture_is_hashed_to_the_expected_sha256() {
        let temp = TempDir::new().unwrap();
        let bytes = (0..(2 * 1024 * 1024))
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        write_tool(temp.path(), "node", &bytes);
        let resolver = ToolResolver::from_paths([temp.path().to_path_buf()]);

        let observation = inspect_tool(&tool("node"), &resolver).unwrap();
        assert_eq!(observation.size_bytes, bytes.len() as u64);
        assert_eq!(observation.sha256, sha256(&bytes));
    }

    #[test]
    fn baseline_reload_preserves_multiple_tools_when_one_is_updated() {
        let temp = TempDir::new().unwrap();
        let first_directory = temp.path().join("first");
        let second_directory = temp.path().join("second");
        fs::create_dir(&first_directory).unwrap();
        fs::create_dir(&second_directory).unwrap();
        let node_path = write_tool(&first_directory, "node", b"node");
        write_tool(&second_directory, "git", b"git");
        let state_path = temp.path().join("integrity.json");
        let resolver = ToolResolver::from_paths([first_directory, second_directory]);

        let report =
            scan_with_resolver(&[tool("node"), tool("git")], &resolver, &state_path).unwrap();
        assert!(
            report
                .checks
                .iter()
                .all(|check| { check.status == IntegrityStatus::NewBaseline })
        );

        fs::write(node_path, b"node replacement").unwrap();
        let report = scan_with_resolver(&[tool("node")], &resolver, &state_path).unwrap();
        assert_eq!(report.checks[0].status, IntegrityStatus::ContentChanged);

        let baseline = BaselineStore::new(&state_path).load().unwrap();
        assert_eq!(baseline.observations.len(), 2);
        assert!(baseline.observations.contains_key("git"));
        assert_eq!(baseline.version, crate::models::INTEGRITY_STATE_VERSION);
    }

    #[test]
    fn malformed_and_unsupported_state_are_explicit_errors() {
        let temp = TempDir::new().unwrap();
        let state_path = temp.path().join("integrity.json");
        let store = BaselineStore::new(&state_path);

        fs::write(&state_path, "not json").unwrap();
        assert!(matches!(
            store.load(),
            Err(crate::error::DustError::IntegrityState(_))
        ));

        fs::write(&state_path, r#"{"version":2,"observations":{}}"#).unwrap();
        assert!(matches!(
            store.load(),
            Err(crate::error::DustError::IntegrityState(message))
                if message.contains("unsupported executable-integrity state version: 2")
        ));
    }

    #[test]
    fn failed_atomic_save_keeps_the_existing_destination_untouched() {
        let temp = TempDir::new().unwrap();
        let destination = temp.path().join("state-directory");
        fs::create_dir(&destination).unwrap();

        let result = BaselineStore::new(&destination).save(&IntegrityBaseline::default());

        assert!(result.is_err());
        assert!(destination.is_dir());
        assert_eq!(
            fs::read_dir(temp.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
                .count(),
            0
        );
    }

    #[test]
    fn inspecting_a_fake_executable_never_launches_it() {
        let temp = TempDir::new().unwrap();
        let marker = temp.path().join("launched");
        let fake = format!("#!/bin/sh\ntouch {}\n", marker.display());
        write_tool(temp.path(), "node", fake.as_bytes());
        let resolver = ToolResolver::from_paths([temp.path().to_path_buf()]);

        inspect_tool(&tool("node"), &resolver).unwrap();

        assert!(!marker.exists());
    }

    #[test]
    fn invalid_tool_name_is_reported_without_filesystem_access() {
        let temp = TempDir::new().unwrap();
        let resolver = ToolResolver::from_paths(Vec::<std::path::PathBuf>::new());

        let failure = inspect_tool(&tool("  "), &resolver).unwrap_err();
        assert_eq!(failure.kind, IntegrityFailureKind::InvalidToolName);
        assert!(!temp.path().join("integrity.json").exists());
    }
}
