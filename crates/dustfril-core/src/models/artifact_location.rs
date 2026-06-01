use std::path::PathBuf;

use super::ArtifactType;

#[derive(Debug, Clone)]
pub struct ArtifactLocation {
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
}

impl ArtifactLocation {
    pub fn new(path: PathBuf, artifact_type: ArtifactType) -> Self {
        Self {
            path,
            artifact_type,
        }
    }
}
