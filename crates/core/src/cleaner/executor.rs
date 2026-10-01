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

    let mut candidates = Vec::with_capacity(plan.candidates.len());
    for candidate in &plan.candidates {
        if let Err(reason) = validate_candidate(candidate) {
            result.failed_paths.push(CleanupFailure {
                path: candidate.path.clone(),
                reason,
            });
        } else {
            candidates.push(candidate.clone());
        }
    }

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

pub(crate) fn is_protected_path(path: &Path) -> bool {
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

/// Executes validated directory roots for a Core-owned cleanup provider.
/// Provider ownership is checked both before path normalization and again
/// immediately before deletion; the shared protected-path and symlink rules
/// remain in force for every cleanup surface.
pub(crate) fn execute_directory_paths<F>(
    paths: &[(std::path::PathBuf, u64)],
    mode: DeleteMode,
    is_provider_path: F,
) -> CleanupResult
where
    F: Fn(&Path) -> bool,
{
    execute_directory_paths_with(paths, mode, is_provider_path, delete_path)
}

fn execute_directory_paths_with<F, D>(
    paths: &[(std::path::PathBuf, u64)],
    mode: DeleteMode,
    is_provider_path: F,
    delete: D,
) -> CleanupResult
where
    F: Fn(&Path) -> bool,
    D: Fn(&Path, DeleteMode) -> io::Result<()>,
{
    let mut result = CleanupResult::default();
    let mut candidates = Vec::with_capacity(paths.len());

    for (path, size_bytes) in paths {
        if let Err(reason) = validate_provider_path(path, &is_provider_path) {
            result.failed_paths.push(CleanupFailure {
                path: path.clone(),
                reason,
            });
        } else {
            candidates.push((path.clone(), *size_bytes));
        }
    }

    candidates.sort_by_key(|(path, _)| path.components().count());
    let mut roots = Vec::<std::path::PathBuf>::with_capacity(candidates.len());
    candidates.retain(|(path, _)| {
        let covered = roots
            .iter()
            .any(|root| crate::models::path_contains(root, path));
        if !covered {
            roots.push(path.clone());
        }
        !covered
    });

    for (path, size_bytes) in candidates {
        if let Err(reason) = validate_provider_path(&path, &is_provider_path) {
            result.failed_paths.push(CleanupFailure { path, reason });
            continue;
        }

        match delete(&path, mode) {
            Ok(()) => {
                result.deleted_paths.push(path);
                result.freed_size_bytes = result.freed_size_bytes.saturating_add(size_bytes);
            }
            Err(error) => result.failed_paths.push(CleanupFailure {
                path,
                reason: failure_reason(&error),
            }),
        }
    }

    result
}

fn validate_provider_path<F>(path: &Path, is_provider_path: &F) -> Result<(), CleanupFailureReason>
where
    F: Fn(&Path) -> bool,
{
    let metadata = fs::symlink_metadata(path).map_err(|error| failure_reason(&error))?;
    if metadata.file_type().is_symlink() {
        return Err(CleanupFailureReason::SymbolicLink);
    }
    if !metadata.is_dir() {
        return Err(CleanupFailureReason::UnsafePath);
    }

    let canonical_path = fs::canonicalize(path).map_err(|_| CleanupFailureReason::UnsafePath)?;
    if is_protected_path(&canonical_path) || !is_provider_path(path) {
        return Err(CleanupFailureReason::UnsafePath);
    }

    Ok(())
}

#[cfg(test)]
mod provider_cleanup_tests {
    use std::{cell::Cell, io};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn trash_failure_is_reported_without_a_permanent_delete_retry() {
        let temp = TempDir::new().unwrap();
        let cache = temp.path().join("registry");
        fs::create_dir(&cache).unwrap();
        fs::write(cache.join("entry"), b"keep me").unwrap();
        let attempts = Cell::new(0);

        let result = execute_directory_paths_with(
            &[(cache.clone(), 7)],
            DeleteMode::Trash,
            |_| true,
            |_, mode| {
                attempts.set(attempts.get() + 1);
                assert_eq!(mode, DeleteMode::Trash);
                Err(io::Error::other("Trash is unavailable"))
            },
        );

        assert_eq!(attempts.get(), 1);
        assert!(cache.exists());
        assert!(result.deleted_paths.is_empty());
        assert_eq!(result.failed_paths.len(), 1);
    }

    #[test]
    fn trash_mode_moves_a_disposable_cache_to_the_trash_backend() {
        let temp = TempDir::new().unwrap();
        let cache = temp.path().join("registry");
        let trash_bin = temp.path().join("fake-trash");
        let trashed_cache = trash_bin.join("registry");
        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&trash_bin).unwrap();
        fs::write(cache.join("entry"), b"fixture contents").unwrap();

        let result = execute_directory_paths_with(
            &[(cache.clone(), 16)],
            DeleteMode::Trash,
            |_| true,
            |path, mode| {
                assert_eq!(mode, DeleteMode::Trash);
                fs::rename(path, &trashed_cache)
            },
        );

        assert!(!cache.exists());
        assert_eq!(
            fs::read(trashed_cache.join("entry")).unwrap(),
            b"fixture contents"
        );
        assert_eq!(result.deleted_paths, vec![cache]);
        assert!(result.failed_paths.is_empty());
    }
}
