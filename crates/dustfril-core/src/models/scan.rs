use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Supported project ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ecosystem {
    Rust,
    Node,
    Java,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),

            Self::Node => write!(f, "Node"),

            Self::Java => write!(f, "Java"),
        }
    }
}

/// Result of scanning a filesystem tree for removable artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanResult {
    /// Artifact paths discovered during the scan.
    pub artifacts: Vec<Artifact>,
}

/// A removable artifact discovered for a supported ecosystem.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    /// Filesystem path to the removable artifact.
    pub path: PathBuf,
    /// Ecosystem that owns the artifact.
    pub ecosystem: Ecosystem,
}

impl Artifact {
    /// Creates a scanned artifact entry for the given path and ecosystem.
    pub fn new(path: PathBuf, ecosystem: Ecosystem) -> Self {
        Self { path, ecosystem }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_display_matches_cli_labels() {
        assert_eq!(Ecosystem::Rust.to_string(), "Rust");
        assert_eq!(Ecosystem::Node.to_string(), "Node");
        assert_eq!(Ecosystem::Java.to_string(), "Java");
    }

    #[test]
    fn artifact_new_preserves_fields() {
        let artifact = Artifact::new(PathBuf::from("target"), Ecosystem::Rust);

        assert_eq!(artifact.path, PathBuf::from("target"));
        assert_eq!(artifact.ecosystem, Ecosystem::Rust);
    }
}
