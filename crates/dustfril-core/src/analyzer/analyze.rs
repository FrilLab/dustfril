use std::time::SystemTime;
use std::{fs, path::Path};

use crate::error::DustResult;
use crate::models::{
    AnalysisResult, Artifact, ArtifactAnalysis, CleanupRecommendation, ScanResult,
    normalize_artifacts,
};
use rayon::prelude::*;
use walkdir::WalkDir;

pub struct Analyzer;

impl Analyzer {
    /// Computes per-artifact size and freshness metadata for a scan result.
    pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
        let mut artifacts: Vec<ArtifactAnalysis> = normalize_artifacts(scan_result.artifacts)
            .into_par_iter()
            .map(Self::analyze_artifact)
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

    fn analyze_artifact(artifact: Artifact) -> ArtifactAnalysis {
        let (size_bytes, last_modified) = calculate_artifact_metadata(&artifact.path);
        let age_days = calculate_age_days(last_modified);
        let recommendation = recommend_cleanup(age_days);

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

fn recommend_cleanup(age_days: Option<u64>) -> CleanupRecommendation {
    let Some(days) = age_days else {
        return CleanupRecommendation::NeedsReview;
    };

    const KEEP_DAYS: u64 = 30;
    const REVIEW_DAYS: u64 = 90;

    if days <= KEEP_DAYS {
        CleanupRecommendation::Keep
    } else if days <= REVIEW_DAYS {
        CleanupRecommendation::NeedsReview
    } else {
        CleanupRecommendation::SafeToClean
    }
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
    use super::{calculate_age_days, recommend_cleanup};
    use crate::models::CleanupRecommendation;

    #[test]
    fn cleanup_recommendations_use_documented_age_boundaries() {
        assert_eq!(recommend_cleanup(None), CleanupRecommendation::NeedsReview);
        assert_eq!(recommend_cleanup(Some(30)), CleanupRecommendation::Keep);
        assert_eq!(
            recommend_cleanup(Some(31)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            recommend_cleanup(Some(90)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            recommend_cleanup(Some(91)),
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
