use std::{fs, io, path::Path};

use crate::{
    error::DustResult,
    models::{CleanupFailure, CleanupFailureReason, CleanupPlan, CleanupResult, DeleteMode},
};

/// Deletes all paths in a cleanup plan and summarizes reclaimed space.
pub fn execute_cleanup(plan: &CleanupPlan, mode: DeleteMode) -> DustResult<CleanupResult> {
    let mut result = CleanupResult::default();

    for candidate in &plan.candidates {
        match delete_path(&candidate.path, mode) {
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

pub fn delete_path(path: &Path, mode: DeleteMode) -> io::Result<()> {
    match mode {
        DeleteMode::Trash => move_to_trash(path),
        DeleteMode::Permanent => permanently_delete(path),
    }
}

fn permanently_delete(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn move_to_trash(path: &Path) -> io::Result<()> {
    trash::delete(path).map_err(io::Error::other)
}

fn failure_reason(error: &io::Error) -> CleanupFailureReason {
    match error.kind() {
        io::ErrorKind::PermissionDenied => CleanupFailureReason::PermissionDenied,
        io::ErrorKind::NotFound => CleanupFailureReason::NotFound,
        _ => CleanupFailureReason::Other(error.to_string()),
    }
}
