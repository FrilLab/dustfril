use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Supported project ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

/// Returns the effective ecosystems supported by the security scanner.
///
/// An empty selection means the scanner's default Node and Rust scope. Other
/// ecosystems are ignored, matching the scanner's execution semantics.
pub(crate) fn effective_security_ecosystems(selected: &[Ecosystem]) -> Vec<Ecosystem> {
    if selected.is_empty() {
        return vec![Ecosystem::Node, Ecosystem::Rust];
    }

    let mut effective = Vec::new();
    for ecosystem in selected.iter().copied() {
        if matches!(ecosystem, Ecosystem::Node | Ecosystem::Rust) && !effective.contains(&ecosystem)
        {
            effective.push(ecosystem);
        }
    }

    effective
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

    #[test]
    fn security_scope_matches_scanner_defaults_and_filters_unsupported_ecosystems() {
        assert_eq!(
            effective_security_ecosystems(&[]),
            vec![Ecosystem::Node, Ecosystem::Rust]
        );
        assert_eq!(
            effective_security_ecosystems(&[
                Ecosystem::Java,
                Ecosystem::Rust,
                Ecosystem::Node,
                Ecosystem::Rust,
            ]),
            vec![Ecosystem::Rust, Ecosystem::Node]
        );
        assert!(effective_security_ecosystems(&[Ecosystem::Java]).is_empty());
    }
}
