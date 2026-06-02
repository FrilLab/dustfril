use std::path::PathBuf;

use crate::models::{ArtifactType, CleanupRecommendation};

#[derive(Debug)]
pub struct CleanupCandidate {
    pub path: PathBuf,

    pub artifact_type: ArtifactType,

    pub size_bytes: u64,

    pub age_days: Option<u64>,

    pub recommendation: CleanupRecommendation,
}
