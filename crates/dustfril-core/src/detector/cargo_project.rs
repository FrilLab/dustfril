use std::path::Path;

/// Cargo project detection and artifact scanning.
pub fn is_cargo_project(root: &Path) -> bool {
    root.join("Cargo.toml").exists()
}
