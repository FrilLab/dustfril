use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Collect all directories in filesystem tree.
/// No logic, no filtering.
pub fn walk_dirs(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect()
}
