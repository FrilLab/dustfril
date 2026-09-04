use std::{fs, path::PathBuf};

use tempfile::TempDir;

use crate::{analyzer::Analyzer, models::*};

fn scan_result(path: PathBuf, ecosystem: Ecosystem) -> ScanResult {
    ScanResult {
        artifacts: vec![Artifact::new(path, ecosystem)],
        ..ScanResult::default()
    }
}

#[test]
fn analyze_empty_scan_result() {
    let analysis = Analyzer::analyze(ScanResult::default()).unwrap();

    assert!(analysis.artifacts.is_empty());
    assert_eq!(analysis.total_size_bytes, 0);
}

#[test]
fn analyze_calculates_directory_size() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.txt"), b"hello").unwrap();
    fs::write(dir.path().join("b.txt"), b"world").unwrap();

    let analysis =
        Analyzer::analyze(scan_result(dir.path().to_path_buf(), Ecosystem::Rust)).unwrap();

    assert_eq!(analysis.artifacts.len(), 1);

    assert_eq!(analysis.total_size_bytes, 10);

    assert_eq!(analysis.artifacts[0].size_bytes, 10);
}

#[test]
fn analyze_sets_cleanup_recommendation() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("file.txt"), b"hello").unwrap();

    let analysis =
        Analyzer::analyze(scan_result(dir.path().to_path_buf(), Ecosystem::Rust)).unwrap();

    assert_eq!(
        analysis.artifacts[0].recommendation,
        CleanupRecommendation::Keep
    );
}

#[test]
fn analyze_sets_last_modified() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("file.txt"), b"hello").unwrap();

    let analysis =
        Analyzer::analyze(scan_result(dir.path().to_path_buf(), Ecosystem::Rust)).unwrap();

    assert!(analysis.artifacts[0].last_modified.is_some());
}

#[test]
fn analyze_sets_age_days() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("file.txt"), b"hello").unwrap();

    let analysis =
        Analyzer::analyze(scan_result(dir.path().to_path_buf(), Ecosystem::Rust)).unwrap();

    assert!(analysis.artifacts[0].age_days.is_some());
}

#[test]
fn analyze_multiple_artifacts() {
    let root = TempDir::new().unwrap();

    let rust = root.path().join("rust");
    let node = root.path().join("node");

    fs::create_dir_all(&rust).unwrap();
    fs::create_dir_all(&node).unwrap();

    fs::write(rust.join("a.txt"), b"12345").unwrap();
    fs::write(node.join("b.txt"), b"1234567890").unwrap();

    let analysis = Analyzer::analyze(ScanResult {
        artifacts: vec![
            Artifact::new(rust, Ecosystem::Rust),
            Artifact::new(node, Ecosystem::Node),
        ],
        ..ScanResult::default()
    })
    .unwrap();

    assert_eq!(analysis.artifacts.len(), 2);

    assert_eq!(analysis.total_size_bytes, 15);
}

#[test]
fn analyze_preserves_artifact_metadata() {
    let dir = TempDir::new().unwrap();

    let analysis =
        Analyzer::analyze(scan_result(dir.path().to_path_buf(), Ecosystem::Node)).unwrap();

    let artifact = &analysis.artifacts[0];

    assert_eq!(artifact.artifact.ecosystem, Ecosystem::Node);

    assert_eq!(artifact.artifact.path, dir.path());
}

#[cfg(unix)]
#[test]
fn analyze_does_not_count_files_reached_through_symbolic_links() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("outside.txt"), b"outside").unwrap();
    symlink(
        outside.path().join("outside.txt"),
        root.path().join("link.txt"),
    )
    .unwrap();

    let analysis =
        Analyzer::analyze(scan_result(root.path().to_path_buf(), Ecosystem::Rust)).unwrap();

    assert_eq!(analysis.artifacts[0].size_bytes, 0);
    assert_eq!(analysis.total_size_bytes, 0);
}

#[test]
fn analyze_does_not_double_count_a_covered_artifact() {
    let root = TempDir::new().unwrap();
    let outer = root.path().join("node_modules");
    let nested = outer.join("package-a").join("node_modules");

    fs::create_dir_all(&nested).unwrap();
    fs::write(outer.join("outer.txt"), b"outer").unwrap();
    fs::write(nested.join("nested.txt"), b"nested").unwrap();

    let analysis = Analyzer::analyze(ScanResult {
        artifacts: vec![
            Artifact::new(outer.clone(), Ecosystem::Node),
            Artifact::new(nested, Ecosystem::Node),
        ],
        ..ScanResult::default()
    })
    .unwrap();

    assert_eq!(analysis.artifacts.len(), 1);
    assert_eq!(analysis.artifacts[0].artifact.path, outer);
    assert_eq!(
        analysis.artifacts[0].artifact.project.root,
        outer.parent().unwrap()
    );
    assert_eq!(analysis.total_size_bytes, 11);
}
