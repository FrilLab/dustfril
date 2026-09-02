use std::{
    cmp::Ordering,
    fmt,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{AnalysisResult, Ecosystem};

/// Version of the on-disk generated-artifact snapshot format.
pub const ARTIFACT_SNAPSHOT_STATE_VERSION: u32 = 1;

/// Maximum number of snapshots retained for one workspace.
pub const MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE: usize = 32;

/// The result of comparing one explicit artifact snapshot with its predecessor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSnapshotResult {
    /// Whether this operation established a baseline or compared two snapshots.
    pub status: ArtifactSnapshotStatus,
    /// The snapshot produced by the explicit operation.
    pub snapshot: ArtifactSnapshot,
    /// The previous snapshot for this workspace, if one existed.
    pub previous_snapshot: Option<ArtifactSnapshot>,
    /// Deterministically ordered changes between the previous and current state.
    pub changes: Vec<ArtifactSizeChange>,
}

impl ArtifactSnapshotResult {
    /// Returns true when at least one artifact was added, removed, or resized.
    pub fn has_changes(&self) -> bool {
        self.changes.iter().any(ArtifactSizeChange::has_change)
    }
}

/// Whether an explicit snapshot created a baseline or produced a comparison.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactSnapshotStatus {
    /// No previous snapshot existed for this workspace.
    BaselineCreated,
    /// The current snapshot was compared with the previous snapshot.
    Compared,
}

impl fmt::Display for ArtifactSnapshotStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaselineCreated => f.write_str("Baseline created"),
            Self::Compared => f.write_str("Compared"),
        }
    }
}

/// The generated-artifact state observed during one explicit snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSnapshot {
    /// Stable local identity of the workspace, normally its canonical path.
    pub workspace_id: String,
    /// Time at which the existing analysis result was captured.
    pub timestamp: DateTime<Utc>,
    /// Generated artifacts represented by the analysis result.
    pub artifacts: Vec<ArtifactSnapshotArtifact>,
}

impl ArtifactSnapshot {
    /// Builds a snapshot from an existing analysis without traversing the filesystem.
    pub fn from_analysis(workspace_path: &Path, analysis: &AnalysisResult) -> Self {
        let workspace_id = workspace_path
            .canonicalize()
            .unwrap_or_else(|_| lexical_normalize(workspace_path));
        Self::from_analysis_at(
            workspace_id.display().to_string(),
            workspace_path,
            analysis,
            Utc::now(),
        )
    }

    /// Builds a snapshot with a caller-provided timestamp for deterministic callers/tests.
    pub fn from_analysis_at(
        workspace_id: impl Into<String>,
        workspace_path: &Path,
        analysis: &AnalysisResult,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let mut artifacts = analysis
            .artifacts
            .iter()
            .filter(|artifact| {
                is_scanner_owned_artifact(artifact.artifact.ecosystem, &artifact.artifact.path)
            })
            .map(|artifact| ArtifactSnapshotArtifact {
                path: relative_artifact_path(workspace_path, &artifact.artifact.path),
                ecosystem: artifact.artifact.ecosystem,
                size_bytes: artifact.size_bytes,
                last_modified: artifact.last_modified,
                age_days: artifact.age_days,
            })
            .collect::<Vec<_>>();

        artifacts.sort_by(ArtifactSnapshotArtifact::compare_identity);

        Self {
            workspace_id: workspace_id.into(),
            timestamp,
            artifacts,
        }
    }
}

/// One scanner-owned artifact's size and existing freshness metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSnapshotArtifact {
    /// Deterministic path relative to the workspace when possible.
    pub path: PathBuf,
    pub ecosystem: Ecosystem,
    pub size_bytes: u64,
    pub last_modified: Option<std::time::SystemTime>,
    pub age_days: Option<u64>,
}

impl ArtifactSnapshotArtifact {
    /// Returns the stable identity used when matching entries across snapshots.
    pub fn identity_key(&self) -> String {
        format!("{}:{}", self.ecosystem, self.path.display())
    }

    fn compare_identity(left: &Self, right: &Self) -> Ordering {
        left.path
            .cmp(&right.path)
            .then_with(|| left.ecosystem.cmp(&right.ecosystem))
    }
}

/// The factual size state transition for one artifact identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSizeChange {
    pub path: PathBuf,
    pub ecosystem: Ecosystem,
    pub kind: ArtifactChangeKind,
    pub previous_size_bytes: Option<u64>,
    pub current_size_bytes: Option<u64>,
    /// Exact signed byte delta. New and removed artifacts use their current or
    /// previous size respectively; unchanged artifacts have a zero delta.
    pub delta_bytes: i128,
}

impl ArtifactSizeChange {
    pub fn has_change(&self) -> bool {
        self.kind != ArtifactChangeKind::Unchanged
    }
}

/// The storage change classification for one generated artifact.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactChangeKind {
    New,
    Removed,
    SizeIncreased,
    SizeDecreased,
    Unchanged,
}

impl fmt::Display for ArtifactChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::New => "New",
            Self::Removed => "Removed",
            Self::SizeIncreased => "Size increased",
            Self::SizeDecreased => "Size decreased",
            Self::Unchanged => "Unchanged",
        };
        f.write_str(label)
    }
}

/// Compares generated-artifact state without reading any paths.
pub fn compare_artifact_snapshots(
    previous: &ArtifactSnapshot,
    current: &ArtifactSnapshot,
) -> Vec<ArtifactSizeChange> {
    let mut previous_by_identity = previous
        .artifacts
        .iter()
        .map(|artifact| (artifact.identity_key(), artifact))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut current_by_identity = current
        .artifacts
        .iter()
        .map(|artifact| (artifact.identity_key(), artifact))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut identities = previous_by_identity
        .keys()
        .chain(current_by_identity.keys())
        .cloned()
        .collect::<Vec<_>>();
    identities.sort();
    identities.dedup();

    identities
        .into_iter()
        .filter_map(|identity| {
            let previous = previous_by_identity.remove(&identity);
            let current = current_by_identity.remove(&identity);
            let artifact = current.or(previous)?;
            let (kind, delta_bytes) = match (previous, current) {
                (None, Some(current)) => (ArtifactChangeKind::New, i128::from(current.size_bytes)),
                (Some(previous), None) => (
                    ArtifactChangeKind::Removed,
                    -i128::from(previous.size_bytes),
                ),
                (Some(previous), Some(current)) => {
                    let delta = i128::from(current.size_bytes) - i128::from(previous.size_bytes);
                    let kind = match delta.cmp(&0) {
                        Ordering::Greater => ArtifactChangeKind::SizeIncreased,
                        Ordering::Less => ArtifactChangeKind::SizeDecreased,
                        Ordering::Equal => ArtifactChangeKind::Unchanged,
                    };
                    (kind, delta)
                }
                (None, None) => return None,
            };

            Some(ArtifactSizeChange {
                path: artifact.path.clone(),
                ecosystem: artifact.ecosystem,
                kind,
                previous_size_bytes: previous.map(|artifact| artifact.size_bytes),
                current_size_bytes: current.map(|artifact| artifact.size_bytes),
                delta_bytes,
            })
        })
        .collect()
}

pub(crate) fn is_scanner_owned_artifact(ecosystem: Ecosystem, path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    matches!(
        (ecosystem, name),
        (Ecosystem::Rust, "target")
            | (Ecosystem::Node, "node_modules")
            | (Ecosystem::Java, "build")
    )
}

fn relative_artifact_path(workspace_path: &Path, artifact_path: &Path) -> PathBuf {
    let workspace = lexical_normalize(workspace_path);
    let artifact = lexical_normalize(artifact_path);

    artifact
        .strip_prefix(&workspace)
        .ok()
        .map(Path::to_path_buf)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from("@external").join(artifact))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    use std::path::Component;

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::models::{Artifact, ArtifactAnalysis, CleanupRecommendation};

    fn timestamp(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn artifact(path: &str, ecosystem: Ecosystem, size_bytes: u64) -> ArtifactAnalysis {
        ArtifactAnalysis {
            artifact: Artifact::new(PathBuf::from(path), ecosystem),
            size_bytes,
            last_modified: None,
            age_days: None,
            recommendation: CleanupRecommendation::Keep,
        }
    }

    fn snapshot(artifacts: Vec<ArtifactAnalysis>) -> ArtifactSnapshot {
        ArtifactSnapshot::from_analysis_at(
            "/workspace/project",
            Path::new("/workspace/project"),
            &AnalysisResult {
                artifacts,
                total_size_bytes: 0,
            },
            timestamp(1),
        )
    }

    #[test]
    fn snapshot_reuses_only_scanner_owned_artifacts() {
        let snapshot = snapshot(vec![
            artifact("/workspace/project/target", Ecosystem::Rust, 10),
            artifact("/workspace/project/Cargo.lock", Ecosystem::Rust, 20),
            artifact("/workspace/project/src/main.rs", Ecosystem::Rust, 30),
        ]);

        assert_eq!(snapshot.artifacts.len(), 1);
        assert_eq!(snapshot.artifacts[0].path, Path::new("target"));
    }

    #[test]
    fn snapshot_path_identity_is_workspace_relative_and_sorted() {
        let snapshot = snapshot(vec![
            artifact("/workspace/project/node_modules", Ecosystem::Node, 20),
            artifact("/workspace/project/target", Ecosystem::Rust, 10),
        ]);

        assert_eq!(
            snapshot
                .artifacts
                .iter()
                .map(|artifact| artifact.identity_key())
                .collect::<Vec<_>>(),
            vec!["Node:node_modules", "Rust:target"]
        );
    }

    #[test]
    fn comparison_distinguishes_all_size_states_and_is_deterministic() {
        let previous = snapshot(vec![
            artifact("/workspace/project/target", Ecosystem::Rust, 10),
            artifact("/workspace/project/node_modules", Ecosystem::Node, 20),
            artifact(
                "/workspace/project/nested/node_modules",
                Ecosystem::Node,
                25,
            ),
            artifact("/workspace/project/build", Ecosystem::Java, 30),
        ]);
        let current = snapshot(vec![
            artifact("/workspace/project/target", Ecosystem::Rust, 15),
            artifact("/workspace/project/node_modules", Ecosystem::Node, 20),
            artifact(
                "/workspace/project/nested/node_modules",
                Ecosystem::Node,
                15,
            ),
            artifact("/workspace/project/nested/target", Ecosystem::Rust, 40),
        ]);

        let changes = compare_artifact_snapshots(&previous, &current);

        assert_eq!(changes.len(), 5);
        assert_eq!(changes[0].kind, ArtifactChangeKind::Removed);
        assert_eq!(changes[0].delta_bytes, -30);
        assert_eq!(changes[1].kind, ArtifactChangeKind::SizeDecreased);
        assert_eq!(changes[1].delta_bytes, -10);
        assert_eq!(changes[2].kind, ArtifactChangeKind::Unchanged);
        assert_eq!(changes[2].delta_bytes, 0);
        assert_eq!(changes[3].kind, ArtifactChangeKind::New);
        assert_eq!(changes[3].delta_bytes, 40);
        assert_eq!(changes[4].kind, ArtifactChangeKind::SizeIncreased);
        assert_eq!(changes[4].delta_bytes, 5);
    }

    #[test]
    fn equal_size_does_not_claim_content_changed() {
        let previous = snapshot(vec![artifact(
            "/workspace/project/target",
            Ecosystem::Rust,
            10,
        )]);
        let current = snapshot(vec![artifact(
            "/workspace/project/target",
            Ecosystem::Rust,
            10,
        )]);

        let changes = compare_artifact_snapshots(&previous, &current);

        assert_eq!(changes[0].kind, ArtifactChangeKind::Unchanged);
        assert!(
            !ArtifactSnapshotResult {
                status: ArtifactSnapshotStatus::Compared,
                snapshot: current,
                previous_snapshot: Some(previous),
                changes,
            }
            .has_changes()
        );
    }
}
