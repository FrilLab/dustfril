use std::fs;

use crate::{
    error::DustResult,
    models::{CleanupPlan, CleanupResult},
};

pub fn execute_cleanup(plan: &CleanupPlan) -> DustResult<CleanupResult> {
    let mut result = CleanupResult {
        deleted_paths: Vec::new(),
        failed_paths: Vec::new(),
        freed_size_bytes: 0,
    };

    for candidate in &plan.candidates {
        match if candidate.path.is_dir() {
            fs::remove_dir_all(&candidate.path)
        } else {
            fs::remove_file(&candidate.path)
        } {
            Ok(_) => {
                result.deleted_paths.push(candidate.path.clone());
                result.freed_size_bytes += candidate.size_bytes;
            }

            Err(_) => {
                result.failed_paths.push(candidate.path.clone());
            }
        }
    }

    Ok(result)
}
