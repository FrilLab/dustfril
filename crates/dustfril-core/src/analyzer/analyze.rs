use std::path::Path;
use std::time::SystemTime;

use crate::error::DustResult;
use crate::models::{
    AnalysisResult, Artifact, ArtifactAnalysis, CleanupRecommendation, ScanResult,
};
use rayon::prelude::*;
use walkdir::WalkDir;

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
        let mut artifacts: Vec<ArtifactAnalysis> = scan_result
            .artifacts
            .into_par_iter()
            .map(Self::analyze_artifact)
            .collect();

        artifacts.sort_by_key(|artifact| std::cmp::Reverse(artifact.size_bytes));

        let total_size_bytes = artifacts.iter().map(|a| a.size_bytes).sum();

        Ok(AnalysisResult {
            artifacts,
            total_size_bytes,
        })
    }

    fn analyze_artifact(artifact: Artifact) -> ArtifactAnalysis {
        let size_bytes = calculate_directory_size(&artifact.path);
        let last_modified = find_latest_modified(&artifact.path);
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

fn calculate_directory_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .metadata()
                .ok()
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len())
        })
        .sum()
}

fn find_latest_modified(path: &Path) -> Option<SystemTime> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max()
}
