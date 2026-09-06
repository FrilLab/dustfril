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

/// Loads the retained snapshot comparisons for one canonical workspace.
///
/// This is a read-only query. It never scans the workspace, calculates
/// artifact sizes, or writes snapshot state.
pub fn load_artifact_snapshot_history(
    workspace_path: &Path,
) -> DustResult<Vec<ArtifactSnapshotResult>> {
    let path = artifact_snapshot_path()?;
    let snapshots = ArtifactSnapshotStore::new(path).load_workspace(workspace_path)?;
    Ok(artifact_snapshot::artifact_snapshot_history(snapshots))
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
            ..AnalysisResult::default()
        };

        let snapshot = create_artifact_snapshot(workspace.path(), &analysis);

        assert_eq!(snapshot.artifacts[0].path, PathBuf::from("target"));
        assert_eq!(snapshot.artifacts[0].size_bytes, 42);
    }

    #[test]
    fn snapshot_history_query_is_read_only_and_core_computed() {
        let workspace = TempDir::new().unwrap();
        let store = ArtifactSnapshotStore::new(workspace.path().join("snapshots.json"));
        let first = ArtifactSnapshot::from_analysis_at(
            workspace.path().display().to_string(),
            workspace.path(),
            &AnalysisResult::default(),
            chrono::DateTime::UNIX_EPOCH,
        );
        let second = ArtifactSnapshot {
            timestamp: chrono::DateTime::UNIX_EPOCH + chrono::Duration::seconds(1),
            ..first.clone()
        };

        store.record_snapshot(first).unwrap();
        store.record_snapshot(second).unwrap();

        let history = artifact_snapshot::artifact_snapshot_history(store.load_all().unwrap());

        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0].status,
            crate::models::ArtifactSnapshotStatus::BaselineCreated
        );
        assert_eq!(
            history[1].status,
            crate::models::ArtifactSnapshotStatus::Compared
        );
        assert!(history[1].changes.is_empty());
    }
}
