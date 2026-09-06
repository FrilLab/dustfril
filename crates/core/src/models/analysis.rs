use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::{Artifact, CleanupRecommendation};

/// Detailed analysis for a single scanned artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Per-artifact analysis records.
    pub artifacts: Vec<ArtifactAnalysis>,
    /// Sum of all analyzed artifact sizes.
    pub total_size_bytes: u64,
    /// Number of filesystem entries whose size or modification metadata could
    /// not be measured while analyzing the artifacts.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub measurement_failures: u64,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

/// Removes duplicate and covered analysis records before any aggregate is
/// calculated or cleanup selection is exposed.
pub(crate) fn normalize_artifact_analyses(
    mut artifacts: Vec<ArtifactAnalysis>,
) -> Vec<ArtifactAnalysis> {
    artifacts.sort_by_key(|artifact| artifact.artifact.path.components().count());

    let mut normalized = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        if normalized.iter().any(|selected: &ArtifactAnalysis| {
            crate::models::path_contains(&selected.artifact.path, &artifact.artifact.path)
        }) {
            continue;
        }

        normalized.push(artifact);
    }

    normalized
}
