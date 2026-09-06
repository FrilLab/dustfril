use crate::{
    analyzer, cleaner,
    error::DustResult,
    models::{
        AnalysisResult, ArtifactSelection, CleanupPlan, CleanupResult, DeleteMode, ScanResult,
    },
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

/// Builds a cleanup plan from explicit identities in an analyzed result.
///
/// The selection is validated against Core's analyzed scanner-owned artifacts;
/// it cannot introduce a raw filesystem path or alter recommendation metadata.
pub fn build_plan_from_analysis_with_selection(
    analysis: &AnalysisResult,
    selected: &[ArtifactSelection],
) -> DustResult<CleanupPlan> {
    cleaner::create_cleanup_plan_from_selection(analysis, selected)
}

/// Executes a cleanup plan and reports deleted and failed paths.
pub fn execute(plan: &CleanupPlan, mode: DeleteMode) -> DustResult<CleanupResult> {
    cleaner::execute_cleanup(plan, mode)
}
#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::models::{
        Artifact, ArtifactAnalysis, ArtifactSelection, CleanupCandidate, CleanupRecommendation,
        Ecosystem,
    };

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
    fn explicit_selection_builds_a_plan_for_a_keep_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        std::fs::create_dir(&node_modules).unwrap();
        let analysis = AnalysisResult {
            artifacts: vec![ArtifactAnalysis {
                artifact: Artifact::new(node_modules.clone(), Ecosystem::Node),
                size_bytes: 42,
                last_modified: None,
                age_days: Some(5),
                recommendation: CleanupRecommendation::Keep,
            }],
            total_size_bytes: 42,
            ..AnalysisResult::default()
        };

        let plan = build_plan_from_analysis_with_selection(
            &analysis,
            &[ArtifactSelection {
                path: node_modules.clone(),
                ecosystem: Ecosystem::Node,
            }],
        )
        .unwrap();

        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(plan.candidates[0].path, node_modules);
        assert_eq!(
            plan.candidates[0].project,
            analysis.artifacts[0].artifact.project
        );
        assert_eq!(
            plan.candidates[0].recommendation,
            CleanupRecommendation::Keep
        );
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
                project: crate::models::ProjectIdentity::default(),
                size_bytes: 5,
                age_days: Some(120),
                recommendation: crate::models::CleanupRecommendation::SafeToClean,
            }],
        };

        let result = execute(&plan, DeleteMode::Permanent).unwrap();

        assert!(!target.exists());
        assert_eq!(result.deleted_paths, vec![target]);
        assert!(result.failed_paths.is_empty());
        assert_eq!(result.freed_size_bytes, 5);
    }

    #[test]
    fn project_identity_and_size_accounting_survive_the_cleanup_pipeline() {
        let workspace = TempDir::new().unwrap();
        let project = workspace.path().join("web");
        let node_modules = project.join("node_modules");
        let nested = node_modules.join("package-a").join("node_modules");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(project.join("package.json"), "{}\n").unwrap();
        std::fs::write(node_modules.join("outer.bin"), b"outer").unwrap();
        std::fs::write(nested.join("nested.bin"), b"nested").unwrap();

        let scan = crate::api::scan(workspace.path(), &[Ecosystem::Node]).unwrap();
        assert_eq!(scan.artifacts.len(), 1);

        let mut analysis = crate::api::analyze(scan).unwrap();
        assert_eq!(analysis.total_size_bytes, 11);
        analysis.artifacts[0].recommendation = CleanupRecommendation::SafeToClean;

        let plan = build_plan_from_analysis(analysis).unwrap();
        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(plan.candidates[0].project.display_name, "web");
        assert_eq!(plan.reclaimable_size_bytes(), 11);

        let result = execute(&plan, DeleteMode::Permanent).unwrap();
        assert_eq!(result.freed_size_bytes, 11);
        assert_eq!(result.deleted_paths, vec![node_modules]);
    }
}
