use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::{
    cleaner::{create_cleanup_plan, execute_cleanup},
    models::*,
};

fn artifact(recommendation: CleanupRecommendation) -> ArtifactAnalysis {
    ArtifactAnalysis {
        artifact: Artifact {
            path: PathBuf::from("target"),
            ecosystem: Ecosystem::Rust,
        },
        size_bytes: 100,
        last_modified: None,
        age_days: Some(100),
        recommendation,
    }
}

#[test]
fn create_empty_cleanup_plan() {
    let plan = create_cleanup_plan(AnalysisResult::default()).unwrap();

    assert!(plan.candidates.is_empty());
}

#[test]
fn reclaimable_size_bytes_returns_sum() {
    let plan = CleanupPlan {
        candidates: vec![
            CleanupCandidate {
                path: PathBuf::from("a"),
                ecosystem: Ecosystem::Rust,
                size_bytes: 10,
                age_days: None,
            },
            CleanupCandidate {
                path: PathBuf::from("b"),
                ecosystem: Ecosystem::Node,
                size_bytes: 20,
                age_days: None,
            },
        ],
    };

    assert_eq!(plan.reclaimable_size_bytes(), 30);
}

#[test]
fn safe_to_clean_becomes_candidate() {
    let artifact = ArtifactAnalysis {
        artifact: Artifact {
            path: PathBuf::from("target"),
            ecosystem: Ecosystem::Rust,
        },
        size_bytes: 100,
        last_modified: None,
        age_days: Some(200),
        recommendation: CleanupRecommendation::SafeToClean,
    };

    let analysis = AnalysisResult {
        artifacts: vec![artifact],

        total_size_bytes: 100,
    };

    let plan = create_cleanup_plan(analysis).unwrap();

    assert_eq!(plan.candidates.len(), 1,);

    assert_eq!(plan.reclaimable_size_bytes(), 100,);
}

#[test]
fn keep_is_not_candidate() {
    let artifact = ArtifactAnalysis {
        artifact: Artifact {
            path: PathBuf::from("target"),
            ecosystem: Ecosystem::Rust,
        },
        size_bytes: 100,
        last_modified: None,
        age_days: Some(5),
        recommendation: CleanupRecommendation::Keep,
    };

    let analysis = AnalysisResult {
        artifacts: vec![artifact],
        total_size_bytes: 100,
    };

    let plan = create_cleanup_plan(analysis).unwrap();

    assert!(plan.candidates.is_empty());
}

#[test]
fn execute_cleanup_deletes_directory() {
    let temp = TempDir::new().unwrap();

    let target = temp.path().join("target");
    fs::create_dir(&target).unwrap();

    fs::write(target.join("a.txt"), "hello").unwrap();
    fs::write(target.join("b.txt"), "world").unwrap();

    let size = fs::metadata(target.join("a.txt")).unwrap().len()
        + fs::metadata(target.join("b.txt")).unwrap().len();

    let plan = CleanupPlan {
        candidates: vec![CleanupCandidate {
            path: target.clone(),
            ecosystem: Ecosystem::Rust,
            size_bytes: size,
            age_days: Some(365),
        }],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(!target.exists());

    assert_eq!(result.deleted_paths.len(), 1);
    assert!(result.failed_paths.is_empty());
    assert_eq!(result.freed_size_bytes, size);
}

#[test]
fn execute_cleanup_removes_target_directory() {
    let temp_dir = TempDir::new().unwrap();

    let target_dir = temp_dir.path().join("target");

    fs::create_dir_all(&target_dir).unwrap();

    fs::write(target_dir.join("test.bin"), b"hello").unwrap();

    assert!(target_dir.exists());

    let candidate = CleanupCandidate {
        path: target_dir.clone(),
        ecosystem: Ecosystem::Rust,
        size_bytes: 5,
        age_days: Some(100),
    };

    let plan = CleanupPlan {
        candidates: vec![candidate],
    };

    let result = execute_cleanup(&plan, DeleteMode::default()).unwrap();

    assert!(!target_dir.exists());
    assert_eq!(result.deleted_paths.len(), 1);
    assert_eq!(result.failed_paths.len(), 0);
}

#[test]
fn create_cleanup_plan_filters_safe_to_clean() {
    let analysis = AnalysisResult {
        artifacts: vec![
            artifact(CleanupRecommendation::SafeToClean),
            artifact(CleanupRecommendation::Keep),
            artifact(CleanupRecommendation::NeedsReview),
        ],
        total_size_bytes: 300,
    };

    let plan = create_cleanup_plan(analysis).unwrap();

    assert_eq!(plan.candidates.len(), 1);
}

#[test]
fn cleanup_reports_failed_path() {
    let temp_dir = TempDir::new().unwrap();
    let missing = temp_dir.path().join("missing");

    let candidate = CleanupCandidate {
        path: missing,
        ecosystem: Ecosystem::Rust,
        size_bytes: 100,
        age_days: None,
    };

    let plan = CleanupPlan {
        candidates: vec![candidate],
    };
    let result = execute_cleanup(&plan, DeleteMode::default()).unwrap();

    assert_eq!(result.deleted_paths.len(), 0);
    assert_eq!(result.failed_paths.len(), 1);
    assert_eq!(result.freed_size_bytes, 0);
    assert_eq!(plan.reclaimable_size_bytes(), 100);
}

#[test]
fn execute_cleanup_reports_missing_path() {
    let temp = TempDir::new().unwrap();

    let missing = temp.path().join("target");

    let plan = CleanupPlan {
        candidates: vec![CleanupCandidate {
            path: missing.clone(),
            ecosystem: Ecosystem::Rust,
            size_bytes: 100,
            age_days: Some(365),
        }],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(result.deleted_paths.is_empty());

    assert_eq!(result.failed_paths.len(), 1);

    assert_eq!(
        result.failed_paths[0].reason,
        CleanupFailureReason::NotFound
    );
}
