use crate::{
    error::DustResult,
    models::{AnalysisResult, CleanupCandidate, CleanupPlan, CleanupRecommendation},
};

/// Converts analyzed artifacts into a cleanup plan by keeping only safe candidates.
pub fn create_cleanup_plan(analysis: AnalysisResult) -> DustResult<CleanupPlan> {
    let mut plan = CleanupPlan::default();

    for artifact in analysis.artifacts {
        if artifact.recommendation != CleanupRecommendation::SafeToClean {
            continue;
        }

        plan.candidates.push(CleanupCandidate {
            path: artifact.artifact.path,
            ecosystem: artifact.artifact.ecosystem,
            size_bytes: artifact.size_bytes,
            age_days: artifact.age_days,
        });
    }

    Ok(plan)
}
