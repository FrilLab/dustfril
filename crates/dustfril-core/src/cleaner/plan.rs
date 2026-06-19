use crate::{
    error::DustResult,
    models::{AnalysisResult, CleanupCandidate, CleanupPlan, CleanupRecommendation},
};

pub fn create_cleanup_plan(analysis: AnalysisResult) -> DustResult<CleanupPlan> {
    let mut plan = CleanupPlan::default();

    for artifact_analysis in analysis.artifacts {
        if artifact_analysis.recommendation == CleanupRecommendation::SafeToClean {
            // Flatten the analysis into a cleanup candidate
            plan.candidates.push(CleanupCandidate {
                path: artifact_analysis.artifact.path.clone(),

                artifact_type: artifact_analysis.artifact.artifact_type,

                size_bytes: artifact_analysis.size_bytes,

                age_days: artifact_analysis.age_days,

                recommendation: artifact_analysis.recommendation,
            });
        }
    }

    Ok(plan)
}
