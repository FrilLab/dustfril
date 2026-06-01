use tempfile::TempDir;

use crate::{detector::scan_project, models::ArtifactType};

#[test]
fn scan_returns_empty_when_not_cargo_project() {
    let temp_dir = TempDir::new().unwrap();

    let result = scan_project(temp_dir.path());

    assert!(result.artifacts.is_empty());
}

#[test]
fn cargo_project_without_target_returns_empty() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

    let result = scan_project(temp_dir.path());

    assert!(result.artifacts.is_empty());
}

#[test]
fn detects_target_directory() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

    std::fs::create_dir(temp_dir.path().join("target")).unwrap();

    let result = scan_project(temp_dir.path());

    assert_eq!(result.artifacts.len(), 1);

    assert_eq!(result.artifacts[0].artifact_type, ArtifactType::Target);
}
