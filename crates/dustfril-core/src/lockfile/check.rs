use std::{fs, path::Path};

use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    lockfile::git::{GitStatusProvider, Libgit2StatusProvider},
    models::{Ecosystem, LockfileCheck, LockfileKind, LockfileStatus},
};

/// Checks explicitly expected lockfiles and any additional supported
/// lockfiles present at the project root.
pub fn check_lockfiles(root: &Path, expected: &[LockfileKind]) -> DustResult<Vec<LockfileCheck>> {
    check_lockfiles_with_provider(root, expected, &Libgit2StatusProvider)
}

/// Checks one supported lockfile, including a missing-file result.
pub fn check_lockfile(root: &Path, kind: LockfileKind) -> DustResult<LockfileCheck> {
    check_lockfiles(root, &[kind])?
        .into_iter()
        .find(|check| check.kind == kind)
        .ok_or_else(|| DustError::InvalidPath(root.to_path_buf()))
}

/// Checks lockfiles expected from the project manifests.
///
/// `Cargo.toml` expects `Cargo.lock`. For Node projects, the optional
/// `packageManager` field selects npm, pnpm, or bun. Without that field, an
/// existing supported Node lockfile is used; when none exists, npm's
/// `package-lock.json` is the default expectation. Additional supported
/// lockfiles are always included so stale or untracked alternatives are
/// visible.
pub fn check_lockfile_integrity(root: &Path) -> DustResult<Vec<LockfileCheck>> {
    let expected = infer_expected_lockfiles(root)?;

    check_lockfiles(root, &expected)
}

fn check_lockfiles_with_provider(
    root: &Path,
    expected: &[LockfileKind],
    git: &impl GitStatusProvider,
) -> DustResult<Vec<LockfileCheck>> {
    if !root.is_dir() {
        return Err(DustError::InvalidPath(root.to_path_buf()));
    }

    let mut kinds = if expected.is_empty() {
        existing_lockfiles(root)
    } else {
        expected.to_vec()
    };

    for kind in LockfileKind::all() {
        if root.join(kind.filename()).exists() && !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }

    deduplicate(&mut kinds);

    kinds
        .into_iter()
        .map(|kind| {
            let path = root.join(kind.filename());
            let status = if !path.is_file() {
                LockfileStatus::Missing
            } else {
                git.status(root, Path::new(kind.filename()))?
                    .unwrap_or(LockfileStatus::Clean)
            };

            Ok(LockfileCheck::new(path, kind, status))
        })
        .collect()
}

fn existing_lockfiles(root: &Path) -> Vec<LockfileKind> {
    LockfileKind::all()
        .iter()
        .copied()
        .filter(|kind| root.join(kind.filename()).exists())
        .collect()
}

fn infer_expected_lockfiles(root: &Path) -> DustResult<Vec<LockfileKind>> {
    if !root.is_dir() {
        return Err(DustError::InvalidPath(root.to_path_buf()));
    }

    let mut expected = Vec::new();

    if root.join("Cargo.toml").is_file() {
        expected.push(LockfileKind::CargoLock);
    }

    if root.join("package.json").is_file() {
        let node_lockfiles = existing_lockfiles(root)
            .into_iter()
            .filter(|kind| kind.ecosystem() == Ecosystem::Node)
            .collect::<Vec<_>>();

        if let Some(kind) = package_manager_lockfile(&root.join("package.json")) {
            expected.push(kind);
        } else if node_lockfiles.is_empty() {
            expected.push(LockfileKind::PackageLockJson);
        } else {
            expected.extend(node_lockfiles);
        }
    }

    if expected.is_empty() {
        expected.extend(existing_lockfiles(root));
    }

    deduplicate(&mut expected);

    Ok(expected)
}

fn package_manager_lockfile(path: &Path) -> Option<LockfileKind> {
    let content = fs::read_to_string(path).ok()?;
    let package_json: Value = serde_json::from_str(&content).ok()?;
    let package_manager = package_json.get("packageManager")?.as_str()?;

    if package_manager.starts_with("pnpm@") {
        Some(LockfileKind::PnpmLockYaml)
    } else if package_manager.starts_with("bun@") {
        Some(LockfileKind::BunLock)
    } else if package_manager.starts_with("npm@") {
        Some(LockfileKind::PackageLockJson)
    } else {
        None
    }
}

fn deduplicate(kinds: &mut Vec<LockfileKind>) {
    let mut seen = Vec::with_capacity(kinds.len());
    kinds.retain(|kind| {
        if seen.contains(kind) {
            false
        } else {
            seen.push(*kind);
            true
        }
    });
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use git2::{Repository, Signature};
    use tempfile::TempDir;

    use super::*;

    fn git_repository_with_committed_lockfile() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repository = Repository::init(temp_dir.path()).unwrap();
        fs::write(temp_dir.path().join("Cargo.lock"), "version = 3\n").unwrap();

        let mut index = repository.index().unwrap();
        index.add_path(Path::new("Cargo.lock")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repository.find_tree(tree_id).unwrap();
        let signature = Signature::now("DustFril Test", "test@dustfril.invalid").unwrap();
        repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Add lockfile",
                &tree,
                &[],
            )
            .unwrap();

        temp_dir
    }

    #[test]
    fn explicit_expected_lockfile_reports_missing() {
        let temp_dir = TempDir::new().unwrap();

        let checks = check_lockfiles(temp_dir.path(), &[LockfileKind::CargoLock]).unwrap();

        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, LockfileStatus::Missing);
    }

    #[test]
    fn non_git_existing_lockfile_is_clean() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("bun.lock"), "lockfile\n").unwrap();

        let check = check_lockfile(temp_dir.path(), LockfileKind::BunLock).unwrap();

        assert_eq!(check.status, LockfileStatus::Clean);
    }

    #[test]
    fn non_git_lockfile_cannot_be_reported_as_untracked() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: 9\n",
        )
        .unwrap();

        let check = check_lockfile(temp_dir.path(), LockfileKind::PnpmLockYaml).unwrap();

        assert_ne!(check.status, LockfileStatus::Untracked);
        assert_eq!(check.status, LockfileStatus::Clean);
    }

    #[test]
    fn git_status_distinguishes_clean_modified_and_untracked_lockfiles() {
        let temp_dir = git_repository_with_committed_lockfile();

        let clean = check_lockfile(temp_dir.path(), LockfileKind::CargoLock).unwrap();
        assert_eq!(clean.status, LockfileStatus::Clean);

        fs::write(temp_dir.path().join("Cargo.lock"), "version = 4\n").unwrap();
        let modified = check_lockfile(temp_dir.path(), LockfileKind::CargoLock).unwrap();
        assert_eq!(modified.status, LockfileStatus::Modified);

        fs::write(temp_dir.path().join("package-lock.json"), "{}\n").unwrap();
        let untracked = check_lockfile(temp_dir.path(), LockfileKind::PackageLockJson).unwrap();
        assert_eq!(untracked.status, LockfileStatus::Untracked);
    }

    #[test]
    fn integrity_check_infers_manifest_expectations() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();

        let checks = check_lockfile_integrity(temp_dir.path()).unwrap();

        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].kind, LockfileKind::PnpmLockYaml);
        assert_eq!(checks[0].status, LockfileStatus::Missing);
    }

    #[test]
    fn integrity_check_includes_additional_supported_lockfiles() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(temp_dir.path().join("Cargo.lock"), "version = 3\n").unwrap();
        fs::write(temp_dir.path().join("bun.lock"), "lockfile\n").unwrap();

        let checks = check_lockfile_integrity(temp_dir.path()).unwrap();

        assert_eq!(checks.len(), 2);
        assert!(checks.iter().any(|check| {
            check.kind == LockfileKind::CargoLock && check.status == LockfileStatus::Clean
        }));
        assert!(checks.iter().any(|check| {
            check.kind == LockfileKind::BunLock && check.status == LockfileStatus::Clean
        }));
    }
}
