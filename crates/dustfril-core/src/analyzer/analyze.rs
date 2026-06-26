use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::error::DustResult;
use crate::models::{
    AnalysisResult, Artifact, ArtifactAnalysis, CleanupRecommendation, ScanResult,
};
use rayon::prelude::*;

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
        let artifacts: Vec<ArtifactAnalysis> = scan_result
            .artifacts
            .into_par_iter()
            .map(Self::analyze_artifact)
            .collect();

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
    let mut total_size = 0;

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_directory_size(&entry.path());
        }
    }

    total_size
}

fn find_latest_modified(path: &Path) -> Option<SystemTime> {
    let mut latest = fs::metadata(path).ok()?.modified().ok();

    let Ok(entries) = fs::read_dir(path) else {
        return latest;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let current = if metadata.is_dir() {
            find_latest_modified(&path)
        } else {
            metadata.modified().ok()
        };

        if let Some(current) = current
            && latest.is_none_or(|existing| current > existing)
        {
            latest = Some(current);
        }
    }

    latest
}
