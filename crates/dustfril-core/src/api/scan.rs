use std::path::Path;

use crate::{
    error::DustResult,
    models::{Ecosystem, ScanResult},
    scanner,
};

/// Scans a filesystem tree for removable artifacts in supported ecosystems.
///
/// When `ecosystems` is empty, all registered detectors are used.
pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    scanner::scan(root, ecosystems)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn scan_returns_detected_rust_artifact() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let target = temp_dir.path().join("target");
        std::fs::create_dir_all(&target).unwrap();

        let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].path, target);
        assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Rust);
    }
}
