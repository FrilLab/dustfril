use std::fs;

use crate::models::{ArtifactType, CleanupPlan, CleanupResult};

pub fn execute_cleanup(plan: &CleanupPlan) -> CleanupResult {
    let mut result = CleanupResult {
        deleted_paths: vec![],
        failed_paths: vec![],
        freed_size_bytes: 0,
    };

    for candidate in &plan.candidates {
        match candidate.artifact_type {
            ArtifactType::Target | ArtifactType::CargoRegistry | ArtifactType::CargoGit => {
                if fs::remove_dir_all(&candidate.path).is_ok() {
                    result.deleted_paths.push(candidate.path.clone());

                    result.freed_size_bytes += candidate.size_bytes;
                } else {
                    result.failed_paths.push(candidate.path.clone());
                }
            }
        }
    }

    result
}
