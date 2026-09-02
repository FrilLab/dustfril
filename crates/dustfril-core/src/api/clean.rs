use crate::{
    analyzer, cleaner,
    error::DustResult,
    models::{AnalysisResult, CleanupPlan, CleanupResult, DeleteMode, ScanResult},
};

/// Builds a cleanup plan from scanned artifacts using analyzer recommendations.
pub fn build_plan(scan: ScanResult) -> DustResult<CleanupPlan> {
    let analysis = analyzer::Analyzer::analyze(scan)?;
    build_plan_from_analysis(analysis)
}

/// Builds a cleanup plan from an analysis that has already been computed.
pub fn build_plan_from_analysis(analysis: AnalysisResult) -> DustResult<CleanupPlan> {
    cleaner::create_cleanup_plan(analysis)
}

/// Executes a cleanup plan and reports deleted and failed paths.
pub fn execute(plan: &CleanupPlan, mode: DeleteMode) -> DustResult<CleanupResult> {
    cleaner::execute_cleanup(plan, mode)
}
#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::models::{Artifact, CleanupCandidate, Ecosystem};

    #[test]
    fn build_plan_returns_empty_when_artifact_is_recent() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("artifact.bin"), b"hello").unwrap();

        let scan = ScanResult {
            artifacts: vec![Artifact::new(
                temp_dir.path().to_path_buf(),
                Ecosystem::Rust,
            )],
            ..ScanResult::default()
        };

        let plan = build_plan(scan).unwrap();

        assert!(plan.candidates.is_empty());
    }

    #[test]
    fn build_plan_from_analysis_returns_expected_candidates() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("artifact.bin"), b"hello").unwrap();
        let scan = ScanResult {
            artifacts: vec![Artifact::new(
                temp_dir.path().to_path_buf(),
                Ecosystem::Rust,
            )],
            ..ScanResult::default()
        };

        let analysis = crate::api::analyze(scan).unwrap();
        let plan = build_plan_from_analysis(analysis).unwrap();

        assert!(plan.candidates.is_empty());
    }

    #[test]
    fn execute_removes_candidate_path() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("artifact.bin"), b"hello").unwrap();

        let plan = CleanupPlan {
            candidates: vec![CleanupCandidate {
                path: target.clone(),
                ecosystem: Ecosystem::Rust,
                size_bytes: 5,
                age_days: Some(120),
            }],
        };

        let result = execute(&plan, DeleteMode::Permanent).unwrap();

        assert!(!target.exists());
        assert_eq!(result.deleted_paths, vec![target]);
        assert!(result.failed_paths.is_empty());
        assert_eq!(result.freed_size_bytes, 5);
    }
}
