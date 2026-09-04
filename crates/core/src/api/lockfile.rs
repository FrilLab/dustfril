use std::path::Path;

use crate::{
    error::DustResult,
    lockfile,
    models::{LockfileCheck, LockfileKind},
};

/// Checks lockfiles expected by a project and additional supported lockfiles
/// found at its root.
pub fn check_lockfiles(root: &Path, expected: &[LockfileKind]) -> DustResult<Vec<LockfileCheck>> {
    lockfile::check_lockfiles(root, expected)
}

/// Checks one supported lockfile at a project root.
pub fn check_lockfile(root: &Path, kind: LockfileKind) -> DustResult<LockfileCheck> {
    lockfile::check_lockfile(root, kind)
}

/// Checks lockfiles inferred from project manifests.
pub fn check_lockfile_integrity(root: &Path) -> DustResult<Vec<LockfileCheck>> {
    lockfile::check_lockfile_integrity(root)
}

/// Spelled-out alias for callers that use “lock file” as two words.
pub fn check_lock_file_integrity(root: &Path) -> DustResult<Vec<LockfileCheck>> {
    check_lockfile_integrity(root)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn api_exposes_missing_lockfile_check() {
        let temp_dir = TempDir::new().unwrap();

        let checks = check_lockfiles(temp_dir.path(), &[LockfileKind::CargoLock]).unwrap();

        assert_eq!(checks[0].status, crate::models::LockfileStatus::Missing);
    }
}
