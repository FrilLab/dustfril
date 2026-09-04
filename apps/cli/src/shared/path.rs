use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Returns `true` when the given path is an accessible directory.
pub fn validate_path(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            eprintln!("Path must not be a symbolic link: {}", path.display());
            false
        }
        Ok(metadata) if metadata.is_dir() => true,
        Ok(_) => {
            eprintln!("Path is not a directory: {}", path.display());
            false
        }
        Err(error) => {
            eprintln!("Cannot access path {}: {error}", path.display());
            false
        }
    }
}

/// Resolves an optional CLI path to an explicit filesystem path.
///
/// When no path is provided, the current working directory is used.
pub fn resolve_path(path: &Option<PathBuf>) -> io::Result<PathBuf> {
    path.clone().map(Ok).unwrap_or_else(std::env::current_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_returns_true_for_existing_path() {
        assert!(validate_path(std::env::current_dir().unwrap().as_path()));
    }

    #[test]
    fn validate_path_returns_false_for_missing_path() {
        let missing = tempfile::tempdir().unwrap().path().join("missing");
        assert!(!validate_path(&missing));
    }

    #[test]
    fn validate_path_returns_false_for_a_file() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file");
        std::fs::write(&file, "not a directory").unwrap();

        assert!(!validate_path(&file));
    }

    #[cfg(unix)]
    #[test]
    fn validate_path_returns_false_for_a_symbolic_link() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("link");
        std::fs::create_dir(&target).unwrap();
        symlink(&target, &link).unwrap();

        assert!(!validate_path(&link));
    }

    #[test]
    fn resolve_path_returns_explicit_path_when_provided() {
        let path = PathBuf::from("/tmp/dustfril-explicit");
        assert_eq!(resolve_path(&Some(path.clone())).unwrap(), path);
    }

    #[test]
    fn resolve_path_uses_current_dir_when_not_provided() {
        assert_eq!(
            resolve_path(&None).unwrap(),
            std::env::current_dir().unwrap()
        );
    }
}
