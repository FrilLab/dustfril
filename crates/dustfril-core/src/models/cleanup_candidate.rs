use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{ArtifactType, CleanupRecommendation};

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub path: PathBuf,

    pub artifact_type: ArtifactType,

    pub size_bytes: u64,

    pub age_days: Option<u64>,

    pub recommendation: CleanupRecommendation,
}
