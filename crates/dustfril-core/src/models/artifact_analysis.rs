use crate::models::{ArtifactLocation, CleanupRecommendation};
use std::time::SystemTime;

/// Analyzed artifact information
#[derive(Debug)]
pub struct ArtifactAnalysis {
    pub artifact: ArtifactLocation,
    pub size_bytes: u64,
    // permission denied, broken symlink, network filesystem failed to detect
    pub last_modified: Option<SystemTime>,
    pub age_days: Option<u64>,

    pub recommendation: CleanupRecommendation,
}
