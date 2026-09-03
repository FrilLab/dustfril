use std::{fs, io, path::Path};

use directories::BaseDirs;

use crate::{
    error::DustResult,
    models::{
        CleanupCandidate, CleanupFailure, CleanupFailureReason, CleanupPlan, CleanupResult,
        DeleteMode,
    },
    scanner::detector_for,
};

use super::plan::normalize_candidates;

/// Deletes all paths in a cleanup plan and summarizes reclaimed space.
pub fn execute_cleanup(plan: &CleanupPlan, mode: DeleteMode) -> DustResult<CleanupResult> {
    let mut result = CleanupResult::default();

    let mut candidates = plan.candidates.clone();
    normalize_candidates(&mut candidates);

    for candidate in &candidates {
        if let Err(reason) = validate_candidate(candidate) {
            result.failed_paths.push(CleanupFailure {
                path: candidate.path.clone(),
                reason,
            });
            continue;
        }

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

fn delete_path(path: &Path, mode: DeleteMode) -> io::Result<()> {
    match mode {
        DeleteMode::Trash => move_to_trash(path),
        DeleteMode::Permanent => permanently_delete(path),
    }
}

fn permanently_delete(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;

    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to delete a symbolic link",
        ));
    }

    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to delete a non-directory artifact",
        ));
    }

    fs::remove_dir_all(path)
}

fn move_to_trash(path: &Path) -> io::Result<()> {
    trash::delete(path).map_err(|error| io::Error::other(error.to_string()))?;

    if path.exists() {
        return Err(io::Error::other(
            "trash operation completed but the artifact is still present",
        ));
    }

    Ok(())
}

fn failure_reason(error: &io::Error) -> CleanupFailureReason {
    match error.kind() {
        io::ErrorKind::PermissionDenied => CleanupFailureReason::PermissionDenied,
        io::ErrorKind::NotFound => CleanupFailureReason::NotFound,
        _ => CleanupFailureReason::Other(error.to_string()),
    }
}

fn validate_candidate(candidate: &CleanupCandidate) -> Result<(), CleanupFailureReason> {
    let metadata = fs::symlink_metadata(&candidate.path).map_err(|e| failure_reason(&e))?;
    if metadata.file_type().is_symlink() {
        return Err(CleanupFailureReason::SymbolicLink);
    }

    if !metadata.is_dir() {
        return Err(CleanupFailureReason::UnsafePath);
    }

    if !is_safe_cleanup_candidate(candidate) {
        return Err(CleanupFailureReason::UnsafePath);
    }

    Ok(())
}

fn is_safe_cleanup_candidate(candidate: &CleanupCandidate) -> bool {
    let Some(name) = candidate.path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    let Some(detector) = detector_for(candidate.ecosystem) else {
        return false;
    };

    if !detector.artifact_paths().contains(&name) {
        return false;
    }

    let Ok(canonical_path) = fs::canonicalize(&candidate.path) else {
        return false;
    };

    // Never remove a directory directly below a filesystem root, the current
    // working directory, or the user's home directory. The plan does not carry
    // a project root, so these checks provide a conservative final boundary at
    // execution time.
    if is_protected_path(&canonical_path) {
        return false;
    }

    true
}

pub(super) fn is_protected_path(path: &Path) -> bool {
    path.parent()
        .map(|parent| parent.parent().is_none())
        .unwrap_or(true)
        || std::env::current_dir()
            .map(|current_dir| path == current_dir)
            .unwrap_or(false)
        || BaseDirs::new()
            .map(|base_dirs| path == base_dirs.home_dir())
            .unwrap_or(false)
}
