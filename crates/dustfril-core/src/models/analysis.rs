use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::{Artifact, CleanupRecommendation};

/// Detailed analysis for a single scanned artifact.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactAnalysis {
    /// The original artifact found during scanning.
    pub artifact: Artifact,
    /// Total size of all files contained by the artifact path.
    pub size_bytes: u64,
    /// Best-effort latest modification time.
    ///
    /// This may be `None` when metadata cannot be read, such as on permission
    /// errors, broken symlinks, or unusual filesystems.
    pub last_modified: Option<SystemTime>,
    /// Age in days derived from `last_modified`.
    pub age_days: Option<u64>,
    /// Cleanup recommendation derived from the observed age.
    pub recommendation: CleanupRecommendation,
}

/// Aggregate analysis output for all detected artifacts.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Per-artifact analysis records.
    pub artifacts: Vec<ArtifactAnalysis>,
    /// Sum of all analyzed artifact sizes.
    pub total_size_bytes: u64,
}
