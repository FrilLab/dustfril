use std::path::Path;

use crate::models::{ArtifactLocation, ArtifactType};

pub fn detect(root: &Path) -> Option<ArtifactLocation> {
    let target_path = root.join("target");

    if !target_path.exists() {
        return None;
    }

    Some(ArtifactLocation {
        path: target_path,
        artifact_type: ArtifactType::Target,
    })
}
