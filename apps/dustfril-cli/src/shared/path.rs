use std::path::{Path, PathBuf};

/// Returns `true` when the given path exists on disk.
pub fn validate_path(path: &Path) -> bool {
    if !path.exists() {
        eprintln!("Path does not exist: {}", path.display());

        return false;
    }

    true
}

/// Resolves an optional CLI path to an explicit filesystem path.
///
/// When no path is provided, the current working directory is used.
pub fn resolve_path(path: &Option<PathBuf>) -> PathBuf {
    path.clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"))
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
        let missing = std::env::temp_dir().join("dustfril-cli-missing-path");
        assert!(!validate_path(&missing));
    }

    #[test]
    fn resolve_path_returns_explicit_path_when_provided() {
        let path = PathBuf::from("/tmp/dustfril-explicit");
        assert_eq!(resolve_path(&Some(path.clone())), path);
    }

    #[test]
    fn resolve_path_uses_current_dir_when_not_provided() {
        assert_eq!(resolve_path(&None), std::env::current_dir().unwrap());
    }
}
