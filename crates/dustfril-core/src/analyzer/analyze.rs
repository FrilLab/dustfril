use crate::{
    analyzer::{
        calculate_age_days, calculate_directory_size, find_latest_modified, recommend_cleanup,
    },
    error::DustResult,
    models::{AnalysisResult, ArtifactAnalysis, ScanResult},
};

pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
    let mut result = AnalysisResult::default();

    for artifact in scan_result.artifacts {
        let size_bytes = calculate_directory_size(&artifact.path);
        let last_modified = find_latest_modified(&artifact.path);
        let age_days = calculate_age_days(last_modified);
        let recommendation = recommend_cleanup(age_days);

        result.total_size_bytes += size_bytes;

        result.artifacts.push(ArtifactAnalysis {
            artifact,
            size_bytes,
            last_modified,
            age_days,
            recommendation,
        });
    }

    result
        .artifacts
        .sort_by_key(|b| std::cmp::Reverse(b.size_bytes));

    Ok(result)
}
