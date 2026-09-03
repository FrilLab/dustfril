use std::path::PathBuf;

use crate::{
    error::{DustError, DustResult},
    models::{
        AnalysisResult, ArtifactAnalysis, ArtifactSelection, CleanupCandidate, CleanupPlan,
        normalize_artifact_analyses, path_contains,
    },
    scanner::detector_for,
};

/// Converts analyzed artifacts into the default cleanup plan.
///
/// Recommendation is intentionally used only for default selection here.
/// `NeedsReview` and `Keep` artifacts remain eligible through
/// `create_cleanup_plan_from_selection`.
pub fn create_cleanup_plan(analysis: AnalysisResult) -> DustResult<CleanupPlan> {
    let artifacts = normalize_artifact_analyses(analysis.artifacts);
    let selected = artifacts
        .iter()
        .filter(|artifact| {
            artifact.recommendation.selected_by_default() && is_cleanup_eligible(artifact)
        })
        .map(|artifact| ArtifactSelection {
            path: artifact.artifact.path.clone(),
            ecosystem: artifact.artifact.ecosystem,
        })
        .collect::<Vec<_>>();

    create_cleanup_plan_from_artifacts(&artifacts, &selected)
}

/// Builds a cleanup plan from explicit identities in an existing analysis.
///
/// A selection can name only an artifact that Core already analyzed. Core
/// copies all candidate metadata from that analysis, including its original
/// recommendation, so callers cannot inject an arbitrary path or rewrite its
/// cleanup metadata.
pub fn create_cleanup_plan_from_selection(
    analysis: &AnalysisResult,
    selected: &[ArtifactSelection],
) -> DustResult<CleanupPlan> {
    let artifacts = normalize_artifact_analyses(analysis.artifacts.clone());
    create_cleanup_plan_from_artifacts(&artifacts, selected)
}

fn create_cleanup_plan_from_artifacts(
    artifacts: &[ArtifactAnalysis],
    selected: &[ArtifactSelection],
) -> DustResult<CleanupPlan> {
    let mut candidates = Vec::with_capacity(selected.len());

    for selection in selected {
        let Some(artifact) = artifacts.iter().find(|artifact| {
            artifact.artifact.path == selection.path
                && artifact.artifact.ecosystem == selection.ecosystem
        }) else {
            return Err(DustError::InvalidCleanupSelection(
                selection.path.display().to_string(),
            ));
        };

        if !is_cleanup_eligible(artifact) {
            return Err(DustError::InvalidCleanupSelection(
                selection.path.display().to_string(),
            ));
        }

        candidates.push(CleanupCandidate::from(artifact.clone()));
    }

    normalize_candidates(&mut candidates);

    Ok(CleanupPlan { candidates })
}

fn is_cleanup_eligible(artifact: &ArtifactAnalysis) -> bool {
    let Some(name) = artifact
        .artifact
        .path
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return false;
    };

    detector_for(artifact.artifact.ecosystem)
        .is_some_and(|detector| detector.artifact_paths().contains(&name))
}

pub(super) fn normalize_candidates(candidates: &mut Vec<CleanupCandidate>) {
    candidates.sort_by_key(|candidate| candidate.path.components().count());

    let mut normalized_paths = Vec::<PathBuf>::with_capacity(candidates.len());
    candidates.retain(|candidate| {
        let covered = normalized_paths
            .iter()
            .any(|ancestor| path_contains(ancestor, &candidate.path));
        if !covered {
            normalized_paths.push(candidate.path.clone());
        }
        !covered
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Artifact, CleanupRecommendation, Ecosystem};

    fn analysis(path: &str, recommendation: CleanupRecommendation) -> ArtifactAnalysis {
        ArtifactAnalysis {
            artifact: Artifact::new(PathBuf::from(path), Ecosystem::Node),
            size_bytes: 10,
            last_modified: None,
            age_days: None,
            recommendation,
        }
    }

    #[test]
    fn explicit_selection_preserves_non_recommended_artifacts() {
        let analysis = AnalysisResult {
            artifacts: vec![analysis(
                "/workspace/node_modules",
                CleanupRecommendation::Keep,
            )],
            total_size_bytes: 10,
        };

        let plan = create_cleanup_plan_from_selection(
            &analysis,
            &[ArtifactSelection {
                path: PathBuf::from("/workspace/node_modules"),
                ecosystem: Ecosystem::Node,
            }],
        )
        .unwrap();

        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(
            plan.candidates[0].recommendation,
            CleanupRecommendation::Keep
        );
    }

    #[test]
    fn explicit_selection_rejects_an_unanalyzed_path() {
        let analysis = AnalysisResult {
            artifacts: vec![analysis(
                "/workspace/node_modules",
                CleanupRecommendation::Keep,
            )],
            total_size_bytes: 10,
        };

        let result = create_cleanup_plan_from_selection(
            &analysis,
            &[ArtifactSelection {
                path: PathBuf::from("/workspace/target"),
                ecosystem: Ecosystem::Rust,
            }],
        );

        assert!(matches!(result, Err(DustError::InvalidCleanupSelection(_))));
    }

    #[test]
    fn explicit_selection_cannot_reach_a_covered_artifact() {
        let analysis = AnalysisResult {
            artifacts: vec![
                analysis("/workspace/node_modules", CleanupRecommendation::Keep),
                analysis(
                    "/workspace/node_modules/package-a/node_modules",
                    CleanupRecommendation::SafeToClean,
                ),
            ],
            total_size_bytes: 20,
        };

        let result = create_cleanup_plan_from_selection(
            &analysis,
            &[
                ArtifactSelection {
                    path: PathBuf::from("/workspace/node_modules"),
                    ecosystem: Ecosystem::Node,
                },
                ArtifactSelection {
                    path: PathBuf::from("/workspace/node_modules/package-a/node_modules"),
                    ecosystem: Ecosystem::Node,
                },
            ],
        );

        assert!(matches!(
            result,
            Err(DustError::InvalidCleanupSelection(path))
                if path == "/workspace/node_modules/package-a/node_modules"
        ));
    }
}
