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

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

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

#[test]
fn execute_cleanup_rejects_unsafe_path() {
    let temp = TempDir::new().unwrap();

    let unsafe_dir = temp.path().join("my_folder");
    std::fs::create_dir(&unsafe_dir).unwrap();

    let plan = CleanupPlan {
        candidates: vec![CleanupCandidate {
            path: unsafe_dir.clone(),
            ecosystem: Ecosystem::Rust,
            size_bytes: 0,
            age_days: None,
        }],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(unsafe_dir.exists());

    assert!(result.deleted_paths.is_empty());

    assert_eq!(result.failed_paths.len(), 1);

    assert_eq!(
        result.failed_paths[0].reason,
        CleanupFailureReason::UnsafePath
    );
}

#[test]
fn execute_cleanup_rejects_a_regular_file_with_an_artifact_name() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target");
    fs::write(&target, "not a directory").unwrap();

    let plan = CleanupPlan {
        candidates: vec![CleanupCandidate {
            path: target.clone(),
            ecosystem: Ecosystem::Rust,
            size_bytes: 0,
            age_days: None,
        }],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(target.exists());
    assert_eq!(
        result.failed_paths[0].reason,
        CleanupFailureReason::UnsafePath
    );
}

#[cfg(unix)]
#[test]
fn execute_cleanup_rejects_symbolic_link_candidates() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let real_target = temp.path().join("real-target");
    let link = temp.path().join("target");
    fs::create_dir(&real_target).unwrap();
    symlink(&real_target, &link).unwrap();

    let plan = CleanupPlan {
        candidates: vec![CleanupCandidate {
            path: link.clone(),
            ecosystem: Ecosystem::Rust,
            size_bytes: 0,
            age_days: None,
        }],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(link.exists());
    assert!(real_target.exists());
    assert_eq!(
        result.failed_paths[0].reason,
        CleanupFailureReason::SymbolicLink
    );
}

#[cfg(unix)]
#[test]
fn protected_path_check_rejects_direct_children_of_filesystem_root() {
    assert!(super::executor::is_protected_path(&PathBuf::from(
        "/target"
    )));
}
