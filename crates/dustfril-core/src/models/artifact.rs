use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::ArtifactType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
}

impl Artifact {
    pub fn new(path: PathBuf, artifact_type: ArtifactType) -> Self {
        Self {
            path,
            artifact_type,
        }
    }
}
