use std::path::{Path, PathBuf};

pub fn validate_path(path: &Path) -> bool {
    if !path.exists() {
        eprintln!("Path does not exist: {}", path.display());

        return false;
    }

    true
}

pub fn resolve_path(path: &Option<PathBuf>) -> PathBuf {
    path.clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"))
}
