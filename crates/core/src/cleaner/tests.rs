use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::{
    cleaner::{create_cleanup_plan, create_cleanup_plan_from_selection, execute_cleanup},
    models::*,
};

fn artifact(recommendation: CleanupRecommendation) -> ArtifactAnalysis {
    ArtifactAnalysis {
        artifact: Artifact::new(PathBuf::from("target"), Ecosystem::Rust),
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
                project: ProjectIdentity::default(),
                size_bytes: 10,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
            CleanupCandidate {
                path: PathBuf::from("b"),
                ecosystem: Ecosystem::Node,
                project: ProjectIdentity::default(),
                size_bytes: 20,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
        ],
    };

    assert_eq!(plan.reclaimable_size_bytes(), 30);
}

#[test]
fn safe_to_clean_becomes_candidate() {
    let artifact = ArtifactAnalysis {
        artifact: Artifact::new(PathBuf::from("target"), Ecosystem::Rust),
        size_bytes: 100,
        last_modified: None,
        age_days: Some(200),
        recommendation: CleanupRecommendation::SafeToClean,
    };

    let analysis = AnalysisResult {
        artifacts: vec![artifact],

        total_size_bytes: 100,
        ..AnalysisResult::default()
    };

    let plan = create_cleanup_plan(analysis).unwrap();

    assert_eq!(plan.candidates.len(), 1,);

    assert_eq!(plan.reclaimable_size_bytes(), 100,);
}

#[test]
fn keep_is_not_candidate() {
    let artifact = ArtifactAnalysis {
        artifact: Artifact::new(PathBuf::from("target"), Ecosystem::Rust),
        size_bytes: 100,
        last_modified: None,
        age_days: Some(5),
        recommendation: CleanupRecommendation::Keep,
    };

    let analysis = AnalysisResult {
        artifacts: vec![artifact],
        total_size_bytes: 100,
        ..AnalysisResult::default()
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
            project: ProjectIdentity::default(),
            size_bytes: size,
            age_days: Some(365),
            recommendation: CleanupRecommendation::SafeToClean,
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
        project: ProjectIdentity::default(),
        size_bytes: 5,
        age_days: Some(100),
        recommendation: CleanupRecommendation::SafeToClean,
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
fn execute_cleanup_skips_a_covered_child_candidate() {
    let temp_dir = TempDir::new().unwrap();
    let outer = temp_dir.path().join("node_modules");
    let nested = outer.join("package-a").join("node_modules");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("artifact.bin"), b"nested").unwrap();

    let plan = CleanupPlan {
        candidates: vec![
            CleanupCandidate {
                path: nested,
                ecosystem: Ecosystem::Node,
                project: ProjectIdentity::default(),
                size_bytes: 6,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
            CleanupCandidate {
                path: outer.clone(),
                ecosystem: Ecosystem::Node,
                project: ProjectIdentity::default(),
                size_bytes: 6,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
        ],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(!outer.exists());
    assert_eq!(result.deleted_paths, vec![outer]);
    assert!(result.failed_paths.is_empty());
    assert_eq!(result.freed_size_bytes, 6);
}

#[test]
fn execute_cleanup_validates_candidates_before_collapsing_paths() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("artifact.bin"), b"valid target").unwrap();

    let plan = CleanupPlan {
        candidates: vec![
            CleanupCandidate {
                path: target.clone(),
                ecosystem: Ecosystem::Node,
                project: ProjectIdentity::default(),
                size_bytes: 0,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
            CleanupCandidate {
                path: target.clone(),
                ecosystem: Ecosystem::Rust,
                project: ProjectIdentity::default(),
                size_bytes: 12,
                age_days: None,
                recommendation: CleanupRecommendation::SafeToClean,
            },
        ],
    };

    let result = execute_cleanup(&plan, DeleteMode::Permanent).unwrap();

    assert!(!target.exists());
    assert_eq!(result.deleted_paths, vec![target.clone()]);
    assert_eq!(result.freed_size_bytes, 12);
    assert_eq!(result.failed_paths.len(), 1);
    assert_eq!(result.failed_paths[0].path, target);
    assert_eq!(
        result.failed_paths[0].reason,
        CleanupFailureReason::UnsafePath
    );
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
        ..AnalysisResult::default()
    };

    let plan = create_cleanup_plan(analysis).unwrap();

    assert_eq!(plan.candidates.len(), 1);
}

#[test]
fn recommendation_controls_default_selection_not_eligibility() {
    let analysis = AnalysisResult {
        artifacts: vec![
            ArtifactAnalysis {
                artifact: Artifact::new("/project/target".into(), Ecosystem::Rust),
                size_bytes: 100,
                last_modified: None,
                age_days: Some(100),
                recommendation: CleanupRecommendation::SafeToClean,
            },
            ArtifactAnalysis {
                artifact: Artifact::new("/project/node_modules".into(), Ecosystem::Node),
                size_bytes: 20,
                last_modified: None,
                age_days: Some(60),
                recommendation: CleanupRecommendation::NeedsReview,
            },
            ArtifactAnalysis {
                artifact: Artifact::new("/project/build".into(), Ecosystem::Java),
                size_bytes: 30,
                last_modified: None,
                age_days: Some(5),
                recommendation: CleanupRecommendation::Keep,
            },
        ],
        total_size_bytes: 150,
        ..AnalysisResult::default()
    };

    let default_plan = create_cleanup_plan(analysis.clone()).unwrap();
    assert_eq!(default_plan.candidates.len(), 1);
    assert_eq!(
        default_plan.candidates[0].recommendation,
        CleanupRecommendation::SafeToClean
    );

    let selected_plan = create_cleanup_plan_from_selection(
        &analysis,
        &analysis
            .artifacts
            .iter()
            .map(|artifact| ArtifactSelection {
                path: artifact.artifact.path.clone(),
                ecosystem: artifact.artifact.ecosystem,
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    assert_eq!(selected_plan.candidates.len(), 3);
}

#[test]
fn explicit_selection_allows_review_and_keep_recommendations() {
    let root = TempDir::new().unwrap();
    let review = root.path().join("review").join("node_modules");
    let keep = root.path().join("keep").join("node_modules");
    fs::create_dir_all(&review).unwrap();
    fs::create_dir_all(&keep).unwrap();

    let analysis = AnalysisResult {
        artifacts: vec![
            ArtifactAnalysis {
                artifact: Artifact::new(review.clone(), Ecosystem::Node),
                size_bytes: 20,
                last_modified: None,
                age_days: None,
                recommendation: CleanupRecommendation::NeedsReview,
            },
            ArtifactAnalysis {
                artifact: Artifact::new(keep.clone(), Ecosystem::Node),
                size_bytes: 30,
                last_modified: None,
                age_days: None,
                recommendation: CleanupRecommendation::Keep,
            },
        ],
        total_size_bytes: 50,
        ..AnalysisResult::default()
    };

    let plan = create_cleanup_plan_from_selection(
        &analysis,
        &[
            ArtifactSelection {
                path: review,
                ecosystem: Ecosystem::Node,
            },
            ArtifactSelection {
                path: keep,
                ecosystem: Ecosystem::Node,
            },
        ],
    )
    .unwrap();

    assert_eq!(plan.candidates.len(), 2);
    assert_eq!(plan.reclaimable_size_bytes(), 50);
    assert_eq!(
        plan.candidates[0].recommendation,
        CleanupRecommendation::NeedsReview
    );
    assert_eq!(
        plan.candidates[1].recommendation,
        CleanupRecommendation::Keep
    );
}

#[test]
fn cleanup_plan_normalizes_ancestor_and_descendant_candidates() {
    let outer = PathBuf::from("/workspace/project/node_modules");
    let nested = outer.join("foo").join("node_modules");
    let mut candidates = vec![
        CleanupCandidate {
            path: nested,
            ecosystem: Ecosystem::Node,
            project: ProjectIdentity::default(),
            size_bytes: 20,
            age_days: None,
            recommendation: CleanupRecommendation::SafeToClean,
        },
        CleanupCandidate {
            path: outer.clone(),
            ecosystem: Ecosystem::Node,
            project: ProjectIdentity::default(),
            size_bytes: 100,
            age_days: None,
            recommendation: CleanupRecommendation::SafeToClean,
        },
    ];

    super::plan::normalize_candidates(&mut candidates);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].path, outer);
}

#[test]
fn cleanup_reports_failed_path() {
    let temp_dir = TempDir::new().unwrap();
    let missing = temp_dir.path().join("missing");

    let candidate = CleanupCandidate {
        path: missing,
        ecosystem: Ecosystem::Rust,
        project: ProjectIdentity::default(),
        size_bytes: 100,
        age_days: None,
        recommendation: CleanupRecommendation::SafeToClean,
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
            project: ProjectIdentity::default(),
            size_bytes: 100,
            age_days: Some(365),
            recommendation: CleanupRecommendation::SafeToClean,
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
            project: ProjectIdentity::default(),
            size_bytes: 0,
            age_days: None,
            recommendation: CleanupRecommendation::SafeToClean,
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
            project: ProjectIdentity::default(),
            size_bytes: 0,
            age_days: None,
            recommendation: CleanupRecommendation::SafeToClean,
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
            project: ProjectIdentity::default(),
            size_bytes: 0,
            age_days: None,
            recommendation: CleanupRecommendation::SafeToClean,
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
