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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn walk_dirs_returns_root_and_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();

        let dirs = walk_dirs(temp_dir.path());

        assert!(dirs.iter().any(|path| path == temp_dir.path()));
        assert!(dirs.iter().any(|path| path == &temp_dir.path().join("a")));
        assert!(dirs.iter().any(|path| path == &nested));
    }
}
