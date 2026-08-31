use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Lockfiles whose integrity can be checked by DustFril.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockfileKind {
    PackageLockJson,
    PnpmLockYaml,
    BunLock,
    CargoLock,
}

impl LockfileKind {
    /// Returns every lockfile supported by the v1 integrity check.
    pub const fn all() -> &'static [Self] {
        &[
            Self::PackageLockJson,
            Self::PnpmLockYaml,
            Self::BunLock,
            Self::CargoLock,
        ]
    }

    /// Returns the filename used by this lockfile kind.
    pub const fn filename(self) -> &'static str {
        match self {
            Self::PackageLockJson => "package-lock.json",
            Self::PnpmLockYaml => "pnpm-lock.yaml",
            Self::BunLock => "bun.lock",
            Self::CargoLock => "Cargo.lock",
        }
    }

    /// Returns the ecosystem associated with this lockfile.
    pub const fn ecosystem(self) -> crate::models::Ecosystem {
        match self {
            Self::PackageLockJson | Self::PnpmLockYaml | Self::BunLock => {
                crate::models::Ecosystem::Node
            }
            Self::CargoLock => crate::models::Ecosystem::Rust,
        }
    }

    /// Finds a supported lockfile kind by its exact filename.
    pub fn from_filename(filename: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|kind| kind.filename() == filename)
    }
}

impl fmt::Display for LockfileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.filename())
    }
}

/// Integrity state for a supported lockfile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockfileStatus {
    /// The expected lockfile is not a regular file.
    Missing,
    /// A tracked lockfile differs from HEAD or the index.
    Modified,
    /// The lockfile exists in the worktree but is not tracked by Git.
    Untracked,
    /// The lockfile exists and has no reported Git changes.
    Clean,
}

impl fmt::Display for LockfileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Missing => "Missing",
            Self::Modified => "Modified",
            Self::Untracked => "Untracked",
            Self::Clean => "Clean",
        };

        f.write_str(value)
    }
}

/// Integrity information for one lockfile at a project root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockfileCheck {
    /// Path checked by the integrity scanner.
    pub path: PathBuf,
    /// Format and filename of the lockfile.
    pub kind: LockfileKind,
    /// Status observed for the lockfile.
    pub status: LockfileStatus,
}

impl LockfileCheck {
    /// Creates an integrity result for a lockfile path.
    pub fn new(path: PathBuf, kind: LockfileKind, status: LockfileStatus) -> Self {
        Self { path, kind, status }
    }
}

/// Alias emphasizing that a check is an integrity result.
pub type LockfileIntegrity = LockfileCheck;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_lockfile_filenames_are_stable() {
        assert_eq!(
            LockfileKind::PackageLockJson.filename(),
            "package-lock.json"
        );
        assert_eq!(LockfileKind::PnpmLockYaml.filename(), "pnpm-lock.yaml");
        assert_eq!(LockfileKind::BunLock.filename(), "bun.lock");
        assert_eq!(LockfileKind::CargoLock.filename(), "Cargo.lock");
    }

    #[test]
    fn from_filename_rejects_unsupported_lockfiles() {
        assert_eq!(
            LockfileKind::from_filename("package-lock.json"),
            Some(LockfileKind::PackageLockJson)
        );
        assert_eq!(LockfileKind::from_filename("bun.lockb"), None);
    }

    #[test]
    fn status_display_matches_report_labels() {
        assert_eq!(LockfileStatus::Missing.to_string(), "Missing");
        assert_eq!(LockfileStatus::Modified.to_string(), "Modified");
        assert_eq!(LockfileStatus::Untracked.to_string(), "Untracked");
        assert_eq!(LockfileStatus::Clean.to_string(), "Clean");
    }
}
