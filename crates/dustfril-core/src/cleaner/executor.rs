use std::fs;

use crate::{
    error::DustResult,
    models::{CleanupFailure, CleanupFailureReason, CleanupPlan, CleanupResult},
};

/// Deletes all paths in a cleanup plan and summarizes reclaimed space.
pub fn execute_cleanup(plan: &CleanupPlan) -> DustResult<CleanupResult> {
    let mut result = CleanupResult::default();

    for candidate in &plan.candidates {
        let delete_result = if candidate.path.is_dir() {
            fs::remove_dir_all(&candidate.path)
        } else {
            fs::remove_file(&candidate.path)
        };

        match delete_result {
            Ok(_) => {
                result.deleted_paths.push(candidate.path.clone());
                result.freed_size_bytes += candidate.size_bytes;
            }

            Err(error) => {
                result.failed_paths.push(CleanupFailure {
                    path: candidate.path.clone(),
                    reason: failure_reason(&error),
                });
            }
        }
    }

    Ok(result)
}

fn failure_reason(error: &std::io::Error) -> CleanupFailureReason {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => CleanupFailureReason::PermissionDenied,
        std::io::ErrorKind::NotFound => CleanupFailureReason::NotFound,
        _ => CleanupFailureReason::Other(error.to_string()),
    }
}
