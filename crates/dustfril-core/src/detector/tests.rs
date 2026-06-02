use tempfile::TempDir;

use crate::{
    detector::{project::find_projects, scan::scan_workspace, scan_project},
    models::ArtifactType,
};

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

#[test]
fn finds_projects_recursively() {
    let temp_dir = TempDir::new().unwrap();

    let dustfril = temp_dir.path().join("dustfril");

    let roguekit = temp_dir.path().join("roguekit");

    std::fs::create_dir_all(&dustfril).unwrap();
    std::fs::create_dir_all(&roguekit).unwrap();

    std::fs::write(dustfril.join("Cargo.toml"), "[package]").unwrap();
    std::fs::write(roguekit.join("Cargo.toml"), "[package]").unwrap();

    let projects = find_projects(temp_dir.path());

    assert_eq!(projects.len(), 2,);

    assert!(projects.iter().any(|p| p.root == dustfril));
    assert!(projects.iter().any(|p| p.root == roguekit));
}

#[test]
fn scan_workspace_finds_targets_from_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();

    let project_a = temp_dir.path().join("project_a");
    let project_b = temp_dir.path().join("project_b");

    std::fs::create_dir_all(&project_a).unwrap();
    std::fs::create_dir_all(&project_b).unwrap();

    std::fs::write(project_a.join("Cargo.toml"), "[package]").unwrap();
    std::fs::write(project_b.join("Cargo.toml"), "[package]").unwrap();

    std::fs::create_dir(project_a.join("target")).unwrap();
    std::fs::create_dir(project_b.join("target")).unwrap();

    let result = scan_workspace(temp_dir.path());

    assert_eq!(result.artifacts.len(), 2);

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.path == project_a.join("target") && a.artifact_type == ArtifactType::Target)
    );

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.path == project_b.join("target") && a.artifact_type == ArtifactType::Target)
    );
}
