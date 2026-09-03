use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Ecosystem;

/// Stable identity for a discovered project.
///
/// The root is the directory selected by an ecosystem detector. The display
/// name intentionally follows the directory name so it matches what a user
/// sees in their workspace; manifest package names can be added later without
/// changing the artifact, analysis, or cleanup models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdentity {
    /// Filesystem root discovered for the project.
    pub root: PathBuf,
    /// Human-readable project name derived from `root`.
    pub display_name: String,
    /// Ecosystem that identified the project.
    pub ecosystem: Ecosystem,
}

impl ProjectIdentity {
    /// Creates an identity using the discovered root directory name.
    pub fn new(root: PathBuf, ecosystem: Ecosystem) -> Self {
        // Resolve relative roots without following symlinks. Project identity
        // should be stable for `.` while retaining the path spelling used by
        // the scan and cleanup boundaries.
        let root = std::path::absolute(&root).unwrap_or(root);
        let display_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| root.display().to_string());

        Self {
            root,
            display_name,
            ecosystem,
        }
    }

    /// Returns whether this is the empty compatibility value used when a
    /// legacy serialized artifact omitted project metadata.
    pub(crate) fn is_empty(&self) -> bool {
        self.root.as_os_str().is_empty() && self.display_name.is_empty()
    }

    /// Builds the compatibility identity used by `Artifact::new` for callers
    /// that construct an artifact without going through a detector.
    pub(crate) fn from_artifact_path(path: &Path, ecosystem: Ecosystem) -> Self {
        let root = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        Self::new(root, ecosystem)
    }
}

impl Default for ProjectIdentity {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            display_name: String::new(),
            ecosystem: Ecosystem::Rust,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_root_uses_the_absolute_directory_identity() {
        let identity = ProjectIdentity::new(PathBuf::from("."), Ecosystem::Rust);
        let absolute_root = std::path::absolute(".").unwrap();

        assert_eq!(identity.root, absolute_root);
        assert_eq!(
            identity.display_name,
            absolute_root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| absolute_root.to_str().unwrap())
        );
    }
}
