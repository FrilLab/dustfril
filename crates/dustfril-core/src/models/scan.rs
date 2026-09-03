use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Maximum number of representative scan failures retained in an access
/// summary. The total failure count remains authoritative when more failures
/// occur.
pub const MAX_SCAN_FAILURE_SAMPLES: usize = 8;

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

/// A bounded summary of filesystem access performed by one artifact scan.
///
/// `files_inspected` counts regular metadata files that the artifact detector
/// actually checked. It intentionally does not count every unrelated file
/// encountered by directory enumeration. At present the artifact detector
/// checks supported project metadata files, so `metadata_files_inspected` is
/// the same count; the separate fields leave room for future content-aware
/// detectors without changing the contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanAccessSummary {
    /// Workspace root used for the scan.
    pub root: PathBuf,
    /// Directories successfully visited by the directory traversal.
    pub directories_visited: u64,
    /// Regular files whose metadata/content was actually inspected by a
    /// detector.
    pub files_inspected: u64,
    /// Supported project metadata files inspected by a detector.
    pub metadata_files_inspected: u64,
    /// Artifact directories discovered by the detectors.
    pub artifact_candidates: u64,
    /// Symbolic links skipped by the non-following traversal policy.
    pub symlinks_skipped: u64,
    /// Total traversal or detector-access failures observed.
    pub failures: u64,
    /// Bounded representative failures. This is not a complete failure log.
    pub failure_samples: Vec<ScanAccessFailure>,
}

impl Default for ScanAccessSummary {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            directories_visited: 0,
            files_inspected: 0,
            metadata_files_inspected: 0,
            artifact_candidates: 0,
            symlinks_skipped: 0,
            failures: 0,
            failure_samples: Vec::new(),
        }
    }
}

impl ScanAccessSummary {
    /// Starts an access summary for a workspace root.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..Self::default()
        }
    }

    /// Records a directory returned by the existing traversal.
    pub(crate) fn record_directory(&mut self) {
        self.directories_visited = self.directories_visited.saturating_add(1);
    }

    /// Records a supported metadata file whose metadata was inspected.
    pub(crate) fn record_metadata_file(&mut self) {
        self.files_inspected = self.files_inspected.saturating_add(1);
        self.metadata_files_inspected = self.metadata_files_inspected.saturating_add(1);
    }

    /// Records an artifact candidate found by a detector.
    pub(crate) fn record_artifact_candidate(&mut self) {
        self.artifact_candidates = self.artifact_candidates.saturating_add(1);
    }

    /// Records a symbolic link omitted by the traversal policy.
    pub(crate) fn record_symlink_skipped(&mut self) {
        self.symlinks_skipped = self.symlinks_skipped.saturating_add(1);
    }

    /// Records a failure while retaining only a bounded diagnostic sample.
    pub(crate) fn record_failure(&mut self, path: &std::path::Path, reason: &str) {
        self.failures = self.failures.saturating_add(1);

        if self.failure_samples.len() < MAX_SCAN_FAILURE_SAMPLES {
            let sample_path = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();
            self.failure_samples.push(ScanAccessFailure {
                path: sample_path,
                reason: reason.to_owned(),
            });
        }
    }

    /// Returns a copy safe to include in persisted activity details.
    ///
    /// Scanner-created summaries are already bounded, but applying the limit
    /// again protects the persistence boundary if a caller constructs a
    /// `ScanResult` manually.
    pub(crate) fn bounded(&self) -> Self {
        let mut bounded = self.clone();
        bounded.failure_samples.truncate(MAX_SCAN_FAILURE_SAMPLES);
        bounded
    }
}

/// One representative traversal or detector-access failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanAccessFailure {
    /// Path relative to the scan root when it can be made relative.
    pub path: PathBuf,
    /// Filesystem error description.
    pub reason: String,
}

/// Result of scanning a filesystem tree for removable artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanResult {
    /// Artifact paths discovered during the scan.
    pub artifacts: Vec<Artifact>,
    /// Bounded filesystem access summary collected during this scan.
    #[serde(default)]
    pub access_summary: ScanAccessSummary,
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

/// Removes duplicate and covered artifacts while retaining the shallowest
/// artifact that represents each filesystem subtree.
pub(crate) fn normalize_artifacts(mut artifacts: Vec<Artifact>) -> Vec<Artifact> {
    artifacts.sort_by_key(|artifact| artifact.path.components().count());

    let mut normalized = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        if normalized
            .iter()
            .any(|selected: &Artifact| crate::models::path_contains(&selected.path, &artifact.path))
        {
            continue;
        }

        normalized.push(artifact);
    }

    normalized
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
    fn scan_result_deserializes_legacy_payload_without_access_summary() {
        let result: ScanResult = serde_json::from_value(serde_json::json!({
            "artifacts": []
        }))
        .unwrap();

        assert!(result.artifacts.is_empty());
        assert_eq!(result.access_summary, ScanAccessSummary::default());
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
