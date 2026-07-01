use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{ArtifactAnalysis, Ecosystem};

/// Plan containing artifact paths that are safe to remove.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupPlan {
    /// Individual removal candidates.
    pub candidates: Vec<CleanupCandidate>,
}

impl CleanupPlan {
    /// Returns the total number of bytes that can be reclaimed by this plan.
    pub fn reclaimable_size_bytes(&self) -> u64 {
        self.candidates
            .iter()
            .map(|candidate| candidate.size_bytes)
            .sum()
    }
}

/// Summary of an attempted cleanup operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Paths successfully deleted.
    pub deleted_paths: Vec<PathBuf>,
    /// Paths that could not be deleted.
    pub failed_paths: Vec<CleanupFailure>,
    /// Total bytes reclaimed from deleted paths.
    pub freed_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupFailure {
    pub path: PathBuf,
    pub reason: CleanupFailureReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CleanupFailureReason {
    PermissionDenied,
    NotFound,
    UnsafePath,
    SymbolicLink,
    Other(String),
}

impl fmt::Display for CleanupFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::NotFound => write!(f, "Not found"),
            Self::UnsafePath => write!(f, "Unsafe path"),
            Self::SymbolicLink => write!(f, "Symbolic link"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeleteMode {
    #[default]
    Trash,
    Permanent,
}

/// Suggested user action for an analyzed artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupRecommendation {
    Keep,
    NeedsReview,
    SafeToClean,
}

impl fmt::Display for CleanupRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keep => write!(f, "Keep"),

            Self::NeedsReview => write!(f, "NeedsReview"),

            Self::SafeToClean => write!(f, "SafeToClean"),
        }
    }
}

/// A single artifact selected for removal.
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupCandidate {
    /// Filesystem path to remove.
    pub path: PathBuf,
    /// Ecosystem that owns the artifact.
    pub ecosystem: Ecosystem,
    /// Estimated reclaimable size for this candidate.
    pub size_bytes: u64,
    /// Age in days when known.
    pub age_days: Option<u64>,
}

impl From<ArtifactAnalysis> for CleanupCandidate {
    fn from(analysis: ArtifactAnalysis) -> Self {
        Self {
            path: analysis.artifact.path,
            ecosystem: analysis.artifact.ecosystem,
            size_bytes: analysis.size_bytes,
            age_days: analysis.age_days,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::models::{Artifact, ArtifactAnalysis};

    #[test]
    fn cleanup_recommendation_display_is_stable() {
        assert_eq!(CleanupRecommendation::Keep.to_string(), "Keep");
        assert_eq!(
            CleanupRecommendation::NeedsReview.to_string(),
            "NeedsReview"
        );
        assert_eq!(
            CleanupRecommendation::SafeToClean.to_string(),
            "SafeToClean"
        );
    }

    #[test]
    fn cleanup_candidate_from_analysis_preserves_expected_fields() {
        let analysis = ArtifactAnalysis {
            artifact: Artifact::new(PathBuf::from("target"), Ecosystem::Rust),
            size_bytes: 42,
            last_modified: Some(SystemTime::UNIX_EPOCH),
            age_days: Some(120),
            recommendation: CleanupRecommendation::SafeToClean,
        };

        let candidate = CleanupCandidate::from(analysis);

        assert_eq!(candidate.path, PathBuf::from("target"));
        assert_eq!(candidate.ecosystem, Ecosystem::Rust);
        assert_eq!(candidate.size_bytes, 42);
        assert_eq!(candidate.age_days, Some(120));
    }
}
