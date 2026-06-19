use std::path::Path;

use crate::models::{Artifact, ArtifactType};

pub fn detect(root: &Path) -> Option<Artifact> {
    let target_path = root.join("target");

    if !target_path.exists() {
        return None;
    }

    Some(Artifact {
        path: target_path,
        artifact_type: ArtifactType::Target,
    })
}
