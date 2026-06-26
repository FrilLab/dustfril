use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::{Artifact, CleanupRecommendation};

/// Analyzed artifact information
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactAnalysis {
    pub artifact: Artifact,
    pub size_bytes: u64,
    // permission denied, broken symlink, network filesystem failed to detect
    pub last_modified: Option<SystemTime>,
    pub age_days: Option<u64>,
    pub recommendation: CleanupRecommendation,
}

/// Total Analysis Results
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub artifacts: Vec<ArtifactAnalysis>,
    pub total_size_bytes: u64,
}
