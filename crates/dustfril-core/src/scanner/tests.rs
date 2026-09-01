use tempfile::TempDir;

use crate::{
    models::Ecosystem,
    scanner::{
        detector::{Detector, RustDetector},
        scan,
    },
};

fn create_rust_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    let target = root.join("target");
    std::fs::create_dir_all(&target).unwrap();
    target
}

fn create_node_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("package.json"), "{}").unwrap();
    let node_modules = root.join("node_modules");
    std::fs::create_dir_all(&node_modules).unwrap();
    node_modules
}

fn create_java_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("pom.xml"), "<project></project>").unwrap();
    let build = root.join("build");
    std::fs::create_dir_all(&build).unwrap();
    build
}

#[test]
fn scan_returns_empty_when_no_projects() {
    let temp_dir = TempDir::new().unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert!(result.artifacts.is_empty());
}

#[test]
fn scan_detects_rust_project() {
    let temp_dir = TempDir::new().unwrap();

    let target = create_rust_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);

    let artifact = &result.artifacts[0];

    assert_eq!(artifact.ecosystem, Ecosystem::Rust);
    assert_eq!(artifact.path, target);
}

#[test]
fn scan_detects_node_project() {
    let temp_dir = TempDir::new().unwrap();

    let node_modules = create_node_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
    assert_eq!(result.artifacts[0].path, node_modules);
}

#[test]
fn scan_detects_java_project() {
    let temp_dir = TempDir::new().unwrap();

    let build = create_java_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Java);
    assert_eq!(result.artifacts[0].path, build);
}

#[test]
fn scan_detects_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    let rust_target = create_rust_artifact(&rust);
    let node_modules = create_node_artifact(&node);

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 2);

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Rust && a.path == rust_target)
    );

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Node && a.path == node_modules)
    );
}

#[test]
fn scan_filters_rust_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    let rust_target = create_rust_artifact(&rust);
    create_node_artifact(&node);

    let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Rust);
    assert_eq!(result.artifacts[0].path, rust_target);
}

#[test]
fn scan_filters_node_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    create_rust_artifact(&rust);
    let node_modules = create_node_artifact(&node);

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
    assert_eq!(result.artifacts[0].path, node_modules);
}

#[test]
fn scan_with_unknown_filter_returns_empty() {
    let temp_dir = TempDir::new().unwrap();

    create_rust_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

    assert!(result.artifacts.is_empty());
}

#[test]
fn rust_detector_reports_target_as_safe_artifact() {
    let detector = RustDetector;

    assert_eq!(detector.artifact_paths(), &["target"]);
}

#[cfg(unix)]
#[test]
fn scanner_does_not_return_symbolic_link_artifacts() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let real_target = TempDir::new().unwrap();
    std::fs::write(root.path().join("Cargo.toml"), "[package]").unwrap();
    symlink(real_target.path(), root.path().join("target")).unwrap();

    let result = scan(root.path(), &[Ecosystem::Rust]).unwrap();

    assert!(result.artifacts.is_empty());
}
