use std::{
    fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::error::{DustError, DustResult};

/// Collect all directories in a filesystem tree.
///
/// Invalid or symbolic-link roots and traversal errors are returned instead of
/// being silently omitted from the result.
pub fn walk_dirs(root: &Path) -> DustResult<Vec<PathBuf>> {
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

    WalkDir::new(root)
        .into_iter()
        .map(|entry| {
            let entry = entry.map_err(|error| DustError::Io(io::Error::other(error)))?;
            Ok(entry
                .file_type()
                .is_dir()
                .then(|| entry.path().to_path_buf()))
        })
        .filter_map(|entry| entry.transpose())
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

        let dirs = walk_dirs(temp_dir.path()).unwrap();

        assert!(dirs.iter().any(|path| path == temp_dir.path()));
        assert!(dirs.iter().any(|path| path == &temp_dir.path().join("a")));
        assert!(dirs.iter().any(|path| path == &nested));
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
}
