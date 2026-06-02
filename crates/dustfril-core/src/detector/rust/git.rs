use std::{env, path::Path};

use crate::models::{ArtifactLocation, ArtifactType};

pub fn detect() -> Option<ArtifactLocation> {
    // TODO:
    // Replace HOME lookup with dirs::home_dir()
    // for cross-platform support.
    let Ok(home_dir) = env::var("HOME") else {
        return None;
    };

    let git_path = Path::new(&home_dir).join(".cargo").join("git");

    if !git_path.exists() {
        return None;
    }

    Some(ArtifactLocation {
        path: git_path,
        artifact_type: ArtifactType::CargoGit,
    })
}
