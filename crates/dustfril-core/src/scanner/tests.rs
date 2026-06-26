use tempfile::TempDir;

use crate::{models::Ecosystem, scanner::scan};

#[test]
fn scan_returns_empty_when_no_projects() {
    let temp_dir = TempDir::new().unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert!(result.artifacts.is_empty());
}

#[test]
fn scan_detects_rust_project() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);

    let artifact = &result.artifacts[0];

    assert_eq!(artifact.ecosystem, Ecosystem::Rust);
    assert_eq!(artifact.path, temp_dir.path());
}

#[test]
fn scan_detects_node_project() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
}

#[test]
fn scan_detects_java_project() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("pom.xml"), "<project></project>").unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Java);
}

#[test]
fn scan_detects_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    std::fs::write(rust.join("Cargo.toml"), "[package]").unwrap();
    std::fs::write(node.join("package.json"), "{}").unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 2);

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Rust)
    );

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Node)
    );
}

#[test]
fn scan_filters_rust_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    std::fs::write(rust.join("Cargo.toml"), "[package]").unwrap();
    std::fs::write(node.join("package.json"), "{}").unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Rust);
}

#[test]
fn scan_filters_node_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    std::fs::write(rust.join("Cargo.toml"), "[package]").unwrap();
    std::fs::write(node.join("package.json"), "{}").unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
}

#[test]
fn scan_with_unknown_filter_returns_empty() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

    assert!(result.artifacts.is_empty());
}
