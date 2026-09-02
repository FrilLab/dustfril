use std::path::{Path, PathBuf};

use crate::{
    artifact_snapshot,
    error::DustResult,
    models::{AnalysisResult, ArtifactSnapshot, ArtifactSnapshotResult, Ecosystem},
};

pub use crate::artifact_snapshot::ArtifactSnapshotStore;

/// Returns the OS-specific local path used for artifact snapshots.
pub fn artifact_snapshot_path() -> std::io::Result<PathBuf> {
    artifact_snapshot::default_state_path()
}

/// Builds one generated-artifact snapshot from existing analysis metadata.
pub fn create_artifact_snapshot(
    workspace_path: &Path,
    analysis: &AnalysisResult,
) -> ArtifactSnapshot {
    ArtifactSnapshot::from_analysis(workspace_path, analysis)
}

/// Persists one explicit generated-artifact snapshot and compares it with the previous one.
pub fn record_artifact_snapshot(
    workspace_path: &Path,
    analysis: &AnalysisResult,
) -> DustResult<ArtifactSnapshotResult> {
    let path = artifact_snapshot_path()?;
    ArtifactSnapshotStore::new(path).record(workspace_path, analysis)
}

/// Persists one snapshot while preserving artifacts outside the selected scan scope.
pub fn record_artifact_snapshot_with_ecosystems(
    workspace_path: &Path,
    analysis: &AnalysisResult,
    selected_ecosystems: &[Ecosystem],
) -> DustResult<ArtifactSnapshotResult> {
    let path = artifact_snapshot_path()?;
    ArtifactSnapshotStore::new(path).record_with_ecosystems(
        workspace_path,
        analysis,
        selected_ecosystems,
    )
}

/// Loads all retained generated-artifact snapshots.
pub fn load_artifact_snapshots() -> DustResult<Vec<ArtifactSnapshot>> {
    let path = artifact_snapshot_path()?;
    ArtifactSnapshotStore::new(path).load_all()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;
    use crate::models::{Artifact, ArtifactAnalysis, CleanupRecommendation, Ecosystem};

    #[test]
    fn snapshot_api_builds_from_analysis_without_requiring_artifact_paths() {
        let workspace = TempDir::new().unwrap();
        let analysis = AnalysisResult {
            artifacts: vec![ArtifactAnalysis {
                artifact: Artifact::new(workspace.path().join("target"), Ecosystem::Rust),
                size_bytes: 42,
                last_modified: None,
                age_days: None,
                recommendation: CleanupRecommendation::Keep,
            }],
            total_size_bytes: 42,
        };

        let snapshot = create_artifact_snapshot(workspace.path(), &analysis);

        assert_eq!(snapshot.artifacts[0].path, PathBuf::from("target"));
        assert_eq!(snapshot.artifacts[0].size_bytes, 42);
    }
}
