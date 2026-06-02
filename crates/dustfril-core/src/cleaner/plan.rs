use crate::models::{AnalysisResult, CleanupCandidate, CleanupPlan, CleanupRecommendation};

pub fn create_cleanup_plan(analysis: AnalysisResult) -> CleanupPlan {
    let mut plan = CleanupPlan::default();

    for artifact in analysis.artifacts {
        if artifact.recommendation == CleanupRecommendation::SafeToClean {
            plan.reclaimable_size_bytes += artifact.size_bytes;

            // Flatten the analysis into a cleanup candidate
            plan.candidates.push(CleanupCandidate {
                path: artifact.artifact.path.clone(),

                artifact_type: artifact.artifact.artifact_type.clone(),

                size_bytes: artifact.size_bytes,

                age_days: artifact.age_days,

                recommendation: artifact.recommendation,
            });
        }
    }

    plan
}
