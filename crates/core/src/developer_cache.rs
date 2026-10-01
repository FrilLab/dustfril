use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::BaseDirs;

use crate::{
    cleaner::executor::execute_directory_paths,
    error::{DustError, DustResult},
    fs::measure_directory,
    models::{
        DeleteMode, DeveloperCache, DeveloperCacheDiscovery, DeveloperCacheKind,
        DeveloperCacheScope, DeveloperCacheSelection, DeveloperCacheSupportState,
    },
};

const MAX_DISCOVERY_WARNINGS: usize = 8;
const CLEANUP_IMPACT: &str = "Reclaims disk space. Cargo may need to download sources or rebuild Git checkouts again. Cleaning a global cache can reduce offline availability and affects all projects using this Cargo home.";

pub fn discover() -> DustResult<DeveloperCacheDiscovery> {
    let mut discovery = DeveloperCacheDiscovery::default();
    let Some((cargo_home, evidence)) = resolve_cargo_home(&mut discovery) else {
        return Ok(discovery);
    };

    Ok(discover_at(&cargo_home, &evidence))
}

fn discover_at(cargo_home: &Path, evidence: &str) -> DeveloperCacheDiscovery {
    let mut discovery = DeveloperCacheDiscovery::default();
    let mut seen = Vec::<PathBuf>::new();
    for kind in [
        DeveloperCacheKind::CargoRegistry,
        DeveloperCacheKind::CargoGit,
    ] {
        let path = cargo_home.join(kind.directory_name());
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                record_warning(
                    &mut discovery,
                    format!("Cannot inspect {}: {error}", path.display()),
                );
                continue;
            }
        };

        if metadata.file_type().is_symlink() {
            record_warning(
                &mut discovery,
                format!("Skipped symbolic-link cache root: {}", path.display()),
            );
            continue;
        }
        if !metadata.is_dir() {
            record_warning(
                &mut discovery,
                format!("Skipped non-directory cache root: {}", path.display()),
            );
            continue;
        }

        let canonical_path = match fs::canonicalize(&path) {
            Ok(path) => path,
            Err(error) => {
                record_warning(
                    &mut discovery,
                    format!("Cannot resolve {}: {error}", path.display()),
                );
                continue;
            }
        };
        if crate::cleaner::executor::is_protected_path(&canonical_path) {
            record_warning(
                &mut discovery,
                format!("Skipped protected cache root: {}", path.display()),
            );
            continue;
        }
        if seen.iter().any(|existing| {
            existing == &canonical_path
                || existing.starts_with(&canonical_path)
                || canonical_path.starts_with(existing)
        }) {
            record_warning(
                &mut discovery,
                format!("Skipped overlapping cache root: {}", path.display()),
            );
            continue;
        }
        seen.push(canonical_path);

        let measurement = measure_directory(&path);
        discovery.caches.push(DeveloperCache {
            kind,
            tool: kind.tool_name().to_owned(),
            name: kind.cache_name().to_owned(),
            path,
            scope: DeveloperCacheScope::Global,
            size_bytes: measurement.size_bytes,
            evidence: evidence.to_owned(),
            support_state: DeveloperCacheSupportState::Supported,
            cleanup_impact: CLEANUP_IMPACT.to_owned(),
            measurement_failures: measurement.failures,
            failure_samples: measurement.failure_samples,
        });
    }

    discovery
}

pub fn cleanup(
    selections: &[DeveloperCacheSelection],
    mode: DeleteMode,
) -> DustResult<crate::models::CleanupResult> {
    let mut discovery = DeveloperCacheDiscovery::default();
    let Some((cargo_home, _)) = resolve_cargo_home(&mut discovery) else {
        return Err(DustError::InvalidCleanupSelection(
            "Cargo cache location is unavailable".to_owned(),
        ));
    };

    cleanup_at(&cargo_home, selections, mode)
}

fn cleanup_at(
    cargo_home: &Path,
    selections: &[DeveloperCacheSelection],
    mode: DeleteMode,
) -> DustResult<crate::models::CleanupResult> {
    let provider_paths: Vec<_> = [
        DeveloperCacheKind::CargoRegistry,
        DeveloperCacheKind::CargoGit,
    ]
    .into_iter()
    .map(|kind| (kind, cargo_home.join(kind.directory_name())))
    .collect();
    let mut requested = Vec::with_capacity(selections.len());

    for selection in selections {
        let Some((_, provider_path)) = provider_paths
            .iter()
            .find(|(kind, path)| *kind == selection.kind && *path == selection.path)
        else {
            return Err(DustError::InvalidCleanupSelection(
                selection.path.display().to_string(),
            ));
        };

        if requested
            .iter()
            .any(|(requested_path, _): &(PathBuf, u64)| requested_path == provider_path)
        {
            continue;
        }
        requested.push((
            provider_path.clone(),
            measure_directory(provider_path).size_bytes,
        ));
    }

    Ok(execute_directory_paths(&requested, mode, |path| {
        provider_paths
            .iter()
            .any(|(_, provider_path)| provider_path == path)
    }))
}

fn resolve_cargo_home(discovery: &mut DeveloperCacheDiscovery) -> Option<(PathBuf, String)> {
    match env::var_os("CARGO_HOME") {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                record_warning(
                    discovery,
                    "Skipped Cargo caches because CARGO_HOME is not an absolute path.".to_owned(),
                );
                return None;
            }
            Some((
                path,
                "Resolved from the absolute CARGO_HOME environment variable.".to_owned(),
            ))
        }
        None => BaseDirs::new()
            .map(|base_dirs| {
                (
                    base_dirs.home_dir().join(".cargo"),
                    "Resolved from the platform user home and Cargo's default .cargo directory."
                        .to_owned(),
                )
            })
            .or_else(|| {
                record_warning(
                    discovery,
                    "Skipped Cargo cache discovery because the user home could not be resolved."
                        .to_owned(),
                );
                None
            }),
    }
}

fn record_warning(discovery: &mut DeveloperCacheDiscovery, warning: String) {
    if discovery.warnings.len() < MAX_DISCOVERY_WARNINGS {
        discovery.warnings.push(warning);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn make_cache(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path).unwrap();
        fs::write(path.join("fixture.crate"), bytes).unwrap();
    }

    #[test]
    fn discovery_reports_only_supported_cargo_roots_and_measures_bytes() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("custom-cargo-home");
        make_cache(&home.join("registry"), b"12345");
        make_cache(&home.join("git"), b"abc");
        make_cache(&home.join(".npm"), b"unrelated");
        make_cache(&home.join("store"), b"unrelated");

        let discovery = discover_at(&home, "Resolved from the CARGO_HOME fixture.");

        assert_eq!(discovery.caches.len(), 2);
        assert_eq!(discovery.caches[0].kind, DeveloperCacheKind::CargoRegistry);
        assert_eq!(discovery.caches[0].size_bytes, 5);
        assert_eq!(discovery.caches[0].scope, DeveloperCacheScope::Global);
        assert_eq!(
            discovery.caches[0].support_state,
            DeveloperCacheSupportState::Supported
        );
        assert!(discovery.caches[0].evidence.contains("CARGO_HOME"));
        assert_eq!(discovery.caches[1].kind, DeveloperCacheKind::CargoGit);
        assert_eq!(discovery.caches[1].size_bytes, 3);
    }

    #[test]
    fn discovery_treats_missing_cache_paths_as_an_empty_state() {
        let temp = TempDir::new().unwrap();
        let discovery = discover_at(temp.path(), "fixture");

        assert!(discovery.caches.is_empty());
        assert!(discovery.warnings.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn discovery_skips_symlink_cache_roots_and_does_not_follow_nested_links() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let home = temp.path().join("cargo-home");
        let external = temp.path().join("external");
        make_cache(&external, b"external");
        fs::create_dir_all(&home).unwrap();
        symlink(&external, home.join("registry")).unwrap();

        let git = home.join("git");
        fs::create_dir_all(&git).unwrap();
        symlink(external.join("fixture.crate"), git.join("external-link")).unwrap();

        let discovery = discover_at(&home, "fixture");

        assert_eq!(discovery.caches.len(), 1);
        assert_eq!(discovery.caches[0].kind, DeveloperCacheKind::CargoGit);
        assert_eq!(discovery.caches[0].size_bytes, 0);
        assert_eq!(discovery.caches[0].measurement_failures, 0);
        assert!(
            discovery
                .warnings
                .iter()
                .any(|warning| warning.contains("symbolic-link"))
        );
    }

    #[test]
    fn cleanup_revalidates_provider_identity_and_removes_only_the_selected_root() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("cargo-home");
        let registry = home.join("registry");
        let git = home.join("git");
        make_cache(&registry, b"registry-bytes");
        make_cache(&git, b"git-bytes");
        let selection = DeveloperCacheSelection {
            kind: DeveloperCacheKind::CargoRegistry,
            path: registry.clone(),
        };

        let result = cleanup_at(&home, &[selection], DeleteMode::Permanent).unwrap();

        assert_eq!(result.deleted_paths, vec![registry.clone()]);
        assert_eq!(result.freed_size_bytes, b"registry-bytes".len() as u64);
        assert!(!registry.exists());
        assert!(git.exists());
    }

    #[test]
    fn cleanup_rejects_paths_outside_the_current_provider_roots() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("cargo-home");
        let lookalike = temp.path().join("registry");
        make_cache(&lookalike, b"preserve");
        let selection = DeveloperCacheSelection {
            kind: DeveloperCacheKind::CargoRegistry,
            path: lookalike.clone(),
        };

        let result = cleanup_at(&home, &[selection], DeleteMode::Permanent);

        assert!(matches!(result, Err(DustError::InvalidCleanupSelection(_))));
        assert!(lookalike.exists());
    }

    #[test]
    fn repeated_cache_selection_is_measured_and_removed_only_once() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("cargo-home");
        let registry = home.join("registry");
        make_cache(&registry, b"one measurement");
        let selection = DeveloperCacheSelection {
            kind: DeveloperCacheKind::CargoRegistry,
            path: registry.clone(),
        };

        let result = cleanup_at(
            &home,
            &[selection.clone(), selection],
            DeleteMode::Permanent,
        )
        .unwrap();

        assert_eq!(result.deleted_paths, vec![registry]);
        assert_eq!(result.freed_size_bytes, b"one measurement".len() as u64);
    }
}
