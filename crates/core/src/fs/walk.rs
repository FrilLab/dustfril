use std::{
    fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    error::{DustError, DustResult},
    models::ScanAccessSummary,
};

/// Collect all directories in a filesystem tree.
///
/// Invalid or symbolic-link roots and traversal errors are returned instead of
/// being silently omitted from the result.
pub fn walk_dirs(root: &Path) -> DustResult<Vec<PathBuf>> {
    walk_dirs_with_summary(root, &mut ScanAccessSummary::default())
}

/// Collects directories while recording traversal access in an existing
/// scan-level summary.
///
/// The traversal does not follow symbolic links. A traversal error remains an
/// error, preserving the scanner's existing failure semantics, but is added to
/// the in-memory summary before it is returned.
pub fn walk_dirs_with_summary(
    root: &Path,
    summary: &mut ScanAccessSummary,
) -> DustResult<Vec<PathBuf>> {
    walk_dirs_with_summary_and_boundary(root, summary, |_, _| false)
}

/// Collects directories while pruning directories that are opaque to the
/// caller's discovery policy.
///
/// Boundary directories are visited and counted, but are not returned and
/// their descendants are never enumerated. This lets a scanner report an
/// artifact found in a parent project without discovering projects inside the
/// artifact itself.
pub fn walk_dirs_with_summary_and_boundary<F>(
    root: &Path,
    summary: &mut ScanAccessSummary,
    mut is_boundary: F,
) -> DustResult<Vec<PathBuf>>
where
    F: FnMut(&Path, &mut ScanAccessSummary) -> bool,
{
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(DustError::InvalidPath(root.to_path_buf()));
        }
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Err(DustError::InvalidPath(root.to_path_buf())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(DustError::InvalidPath(root.to_path_buf()));
        }
        Err(error) => return Err(DustError::Io(error)),
    }

    let mut directories = Vec::new();

    let mut entries = WalkDir::new(root).into_iter();
    while let Some(next_entry) = entries.next() {
        let entry = next_entry;
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                let reason = error.to_string();
                if let Some(path) = error.path() {
                    summary.record_failure(path, &reason);
                } else {
                    summary.record_failure(root, &reason);
                }
                return Err(DustError::Io(io::Error::other(error)));
            }
        };

        let file_type = entry.file_type();

        if file_type.is_symlink() {
            summary.record_symlink_skipped();
            continue;
        }

        if file_type.is_dir() {
            summary.record_directory();
            if is_boundary(entry.path(), summary) {
                entries.skip_current_dir();
            } else {
                directories.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(directories)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn walk_dirs_returns_root_and_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();

        let dirs = walk_dirs(temp_dir.path()).unwrap();

        assert!(dirs.iter().any(|path| path == temp_dir.path()));
        assert!(dirs.iter().any(|path| path == &temp_dir.path().join("a")));
        assert!(dirs.iter().any(|path| path == &nested));
    }

    #[test]
    fn walk_dirs_with_summary_counts_visited_directories() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("nested")).unwrap();
        let mut summary = ScanAccessSummary::new(temp_dir.path());

        let dirs = walk_dirs_with_summary(temp_dir.path(), &mut summary).unwrap();

        assert_eq!(dirs.len(), 2);
        assert_eq!(summary.directories_visited, 2);
        assert_eq!(summary.symlinks_skipped, 0);
        assert_eq!(summary.failures, 0);
    }

    #[test]
    fn walk_dirs_rejects_missing_roots() {
        let temp_dir = TempDir::new().unwrap();
        let missing = temp_dir.path().join("missing");

        assert!(matches!(
            walk_dirs(&missing),
            Err(DustError::InvalidPath(path)) if path == missing
        ));
    }

    #[test]
    fn walk_dirs_rejects_file_roots() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("file");
        std::fs::write(&file, "not a directory").unwrap();

        assert!(matches!(
            walk_dirs(&file),
            Err(DustError::InvalidPath(path)) if path == file
        ));
    }

    #[cfg(unix)]
    #[test]
    fn walk_dirs_rejects_symlink_roots() {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().unwrap();
        let real_root = temp_dir.path().join("real-root");
        let link = temp_dir.path().join("link");
        std::fs::create_dir(&real_root).unwrap();
        symlink(&real_root, &link).unwrap();

        assert!(matches!(
            walk_dirs(&link),
            Err(DustError::InvalidPath(path)) if path == link
        ));
    }

    #[cfg(unix)]
    #[test]
    fn walk_dirs_with_summary_counts_skipped_symbolic_links() {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        std::fs::create_dir(&target).unwrap();
        symlink(&target, temp_dir.path().join("link")).unwrap();
        let mut summary = ScanAccessSummary::new(temp_dir.path());

        walk_dirs_with_summary(temp_dir.path(), &mut summary).unwrap();

        assert_eq!(summary.directories_visited, 2);
        assert_eq!(summary.symlinks_skipped, 1);
    }
}
