use std::fs;
use tempfile::TempDir;

use crate::analyzer::{calculate_age_days, calculate_directory_size, find_latest_modified};
use crate::{
    analyzer::analyze,
    models::{Artifact, ArtifactType, ScanResult},
};

#[test]
fn analyze_empty_scan_result() {
    let result = analyze(ScanResult::default()).unwrap();

    assert_eq!(result.total_size_bytes, 0);

    assert!(result.artifacts.is_empty());
}

#[test]
fn calculate_directory_size_returns_total_size() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("a.txt"), vec![0_u8; 100]).unwrap();

    fs::write(temp_dir.path().join("b.txt"), vec![0_u8; 200]).unwrap();

    let size = calculate_directory_size(temp_dir.path());

    assert_eq!(size, 300);
}

#[test]
fn analyze_returns_total_size() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("a.txt"), vec![0_u8; 100]).unwrap();

    let artifact = Artifact {
        path: temp_dir.path().to_path_buf(),
        artifact_type: ArtifactType::Target,
    };

    let scan_result = ScanResult {
        artifacts: vec![artifact],
    };

    let result = analyze(scan_result).unwrap();

    assert_eq!(result.total_size_bytes, 100);

    assert_eq!(result.artifacts.len(), 1);
}

#[test]
fn find_latest_modified_returns_some() {
    let temp_dir = TempDir::new().unwrap();

    let modified = find_latest_modified(temp_dir.path());

    assert!(modified.is_some());
}

#[test]
fn analyze_sorts_by_size_descending() {
    let small = TempDir::new().unwrap();
    let large = TempDir::new().unwrap();

    fs::write(small.path().join("small.bin"), vec![0_u8; 100]).unwrap();

    fs::write(large.path().join("large.bin"), vec![0_u8; 200]).unwrap();

    let scan_result = ScanResult {
        artifacts: vec![
            Artifact {
                path: small.path().to_path_buf(),
                artifact_type: ArtifactType::Target,
            },
            Artifact {
                path: large.path().to_path_buf(),
                artifact_type: ArtifactType::CargoRegistry,
            },
        ],
    };

    let result = analyze(scan_result).unwrap();

    assert_eq!(result.artifacts[0].size_bytes, 200);

    assert_eq!(result.artifacts[1].size_bytes, 100);
}

use std::time::{Duration, SystemTime};

#[test]
fn calculate_age_days_returns_correct_days() {
    let modified = SystemTime::now() - Duration::from_secs(10 * 86_400);

    let age = calculate_age_days(Some(modified));

    assert_eq!(age, Some(10),);
}
