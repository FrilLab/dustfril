use std::time::SystemTime;
use std::{fs, path::Path};

use crate::error::DustResult;
use crate::models::{
    AnalysisResult, Artifact, ArtifactAnalysis, RecommendationPolicy, ScanResult,
    normalize_artifacts,
};
use rayon::prelude::*;
use walkdir::WalkDir;

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
        let mut artifacts: Vec<ArtifactAnalysis> = normalize_artifacts(scan_result.artifacts)
            .into_par_iter()
            .map(|artifact| Self::analyze_artifact(artifact, policy))
            .collect();

        artifacts.sort_by_key(|artifact| std::cmp::Reverse(artifact.size_bytes));

        let total_size_bytes = artifacts
            .iter()
            .map(|artifact| artifact.size_bytes)
            .fold(0, u64::saturating_add);

        Ok(AnalysisResult {
            artifacts,
            total_size_bytes,
        })
    }

    fn analyze_artifact(artifact: Artifact, policy: RecommendationPolicy) -> ArtifactAnalysis {
        let (size_bytes, last_modified) = calculate_artifact_metadata(&artifact.path);
        let age_days = calculate_age_days(last_modified);
        let recommendation = policy.recommendation(age_days);

        ArtifactAnalysis {
            artifact,
            size_bytes,
            last_modified,
            age_days,
            recommendation,
        }
    }
}

fn calculate_age_days(modified: Option<SystemTime>) -> Option<u64> {
    let modified = modified?;
    let duration = SystemTime::now().duration_since(modified).ok()?;

    const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

    Some(duration.as_secs() / SECONDS_PER_DAY)
}

fn calculate_artifact_metadata(path: &Path) -> (u64, Option<SystemTime>) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = fs::symlink_metadata(entry.path()).ok()?;
            let size = if metadata.is_file() {
                metadata.len()
            } else {
                0
            };
            let modified = metadata.modified().ok();

            Some((size, modified))
        })
        .fold(
            (0, None),
            |(size, latest_modified), (entry_size, modified)| {
                (
                    size.saturating_add(entry_size),
                    latest_modified.max(modified),
                )
            },
        )
}

#[cfg(test)]
mod tests {
    use super::calculate_age_days;
    use crate::models::{CleanupRecommendation, RecommendationPolicy};

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
}
