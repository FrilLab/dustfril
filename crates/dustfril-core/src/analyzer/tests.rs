use std::fs;
use tempfile::TempDir;

use crate::analyzer::{
    calculate_age_days, calculate_directory_size, format_size, get_latest_modified,
    recommend_cleanup,
};
use crate::{
    analyzer::analyze,
    models::{ArtifactLocation, ArtifactType, CleanupRecommendation, ScanResult},
};

#[test]
fn analyze_empty_scan_result() {
    let result = analyze(ScanResult::default());

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

    let artifact = ArtifactLocation {
        path: temp_dir.path().to_path_buf(),
        artifact_type: ArtifactType::Target,
    };

    let scan_result = ScanResult {
        artifacts: vec![artifact],
    };

    let result = analyze(scan_result);

    assert_eq!(result.total_size_bytes, 100);

    assert_eq!(result.artifacts.len(), 1);
}

#[test]
fn get_latest_modified_returns_some() {
    let temp_dir = TempDir::new().unwrap();

    let modified = get_latest_modified(temp_dir.path());

    assert!(modified.is_some());
}

#[test]
fn format_size_bytes() {
    assert_eq!(format_size(512), "512 B");
}

#[test]
fn format_size_kilobytes() {
    assert_eq!(format_size(2048), "2.00 KB");
}

#[test]
fn format_size_megabytes() {
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
}

#[test]
fn format_size_gigabytes() {
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn analyze_sorts_by_size_descending() {
    let small = TempDir::new().unwrap();
    let large = TempDir::new().unwrap();

    fs::write(small.path().join("small.bin"), vec![0_u8; 100]).unwrap();

    fs::write(large.path().join("large.bin"), vec![0_u8; 200]).unwrap();

    let scan_result = ScanResult {
        artifacts: vec![
            ArtifactLocation {
                path: small.path().to_path_buf(),
                artifact_type: ArtifactType::Target,
            },
            ArtifactLocation {
                path: large.path().to_path_buf(),
                artifact_type: ArtifactType::CargoRegistry,
            },
        ],
    };

    let result = analyze(scan_result);

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

#[test]
fn keep_when_recent() {
    assert_eq!(recommend_cleanup(Some(10)), CleanupRecommendation::Keep);
}

#[test]
fn review_when_middle_age() {
    assert_eq!(recommend_cleanup(Some(60)), CleanupRecommendation::Review);
}

#[test]
fn safe_to_clean_when_old() {
    assert_eq!(
        recommend_cleanup(Some(180)),
        CleanupRecommendation::SafeToClean
    );
}
