use std::path::Path;
use std::time::SystemTime;

use crate::error::DustResult;
use crate::fs::measure_directory;
use crate::models::{
    AnalysisResult, Artifact, ArtifactAnalysis, RecommendationPolicy, ScanResult,
    normalize_artifacts,
};
use rayon::prelude::*;

pub struct Analyzer;

impl Analyzer {
    /// Computes per-artifact size and freshness metadata for a scan result.
    pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
        Self::analyze_with_policy(scan_result, RecommendationPolicy::default())
    }

    /// Computes per-artifact metadata using the supplied cleanup policy.
    pub fn analyze_with_policy(
        scan_result: ScanResult,
        policy: RecommendationPolicy,
    ) -> DustResult<AnalysisResult> {
        let analyzed_artifacts: Vec<(ArtifactAnalysis, u64)> =
            normalize_artifacts(scan_result.artifacts)
                .into_par_iter()
                .map(|artifact| Self::analyze_artifact(artifact, policy))
                .collect();

        let measurement_failures = analyzed_artifacts
            .iter()
            .map(|(_, failures)| *failures)
            .fold(0, u64::saturating_add);
        let mut artifacts: Vec<ArtifactAnalysis> = analyzed_artifacts
            .into_iter()
            .map(|(artifact, _)| artifact)
            .collect();
        artifacts.sort_by_key(|artifact| std::cmp::Reverse(artifact.size_bytes));

        let total_size_bytes = artifacts
            .iter()
            .map(|artifact| artifact.size_bytes)
            .fold(0, u64::saturating_add);

        Ok(AnalysisResult {
            artifacts,
            total_size_bytes,
            measurement_failures,
        })
    }

    fn analyze_artifact(
        artifact: Artifact,
        policy: RecommendationPolicy,
    ) -> (ArtifactAnalysis, u64) {
        let (size_bytes, last_modified, measurement_failures) =
            calculate_artifact_metadata(&artifact.path);
        let age_days = calculate_age_days(last_modified);
        let recommendation = policy.recommendation(age_days);

        (
            ArtifactAnalysis {
                artifact,
                size_bytes,
                last_modified,
                age_days,
                recommendation,
            },
            measurement_failures,
        )
    }
}

fn calculate_age_days(modified: Option<SystemTime>) -> Option<u64> {
    let modified = modified?;
    let duration = SystemTime::now().duration_since(modified).ok()?;

    const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

    Some(duration.as_secs() / SECONDS_PER_DAY)
}

fn calculate_artifact_metadata(path: &Path) -> (u64, Option<SystemTime>, u64) {
    let measurement = measure_directory(path);
    (
        measurement.size_bytes,
        measurement.latest_modified,
        measurement.failures,
    )
}

#[cfg(test)]
mod tests {
    use super::calculate_age_days;
    use crate::analyzer::Analyzer;
    use crate::models::{CleanupRecommendation, RecommendationPolicy};
    use tempfile::TempDir;

    #[test]
    fn cleanup_recommendations_use_documented_age_boundaries() {
        let policy = RecommendationPolicy::default();

        assert_eq!(
            policy.recommendation(None),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(policy.recommendation(Some(14)), CleanupRecommendation::Keep);
        assert_eq!(
            policy.recommendation(Some(15)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(29)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(30)),
            CleanupRecommendation::SafeToClean
        );
    }

    #[test]
    fn future_modification_times_have_unknown_age() {
        assert_eq!(
            calculate_age_days(Some(
                std::time::SystemTime::now() + std::time::Duration::from_secs(1),
            )),
            None
        );
    }

    #[test]
    fn missing_artifact_path_is_reported_as_a_measurement_failure() {
        let root = TempDir::new().unwrap();
        let missing_path = root.path().join("missing-artifact");
        let analysis = Analyzer::analyze(crate::models::ScanResult {
            artifacts: vec![crate::models::Artifact::new(
                missing_path,
                crate::models::Ecosystem::Rust,
            )],
            ..crate::models::ScanResult::default()
        })
        .unwrap();

        assert_eq!(analysis.total_size_bytes, 0);
        assert_eq!(analysis.measurement_failures, 1);
    }
}
