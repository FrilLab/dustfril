use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use chrono::Utc;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    models::{
        ARTIFACT_SNAPSHOT_STATE_VERSION, AnalysisResult, ArtifactSnapshot,
        ArtifactSnapshotArtifact, ArtifactSnapshotResult, ArtifactSnapshotStatus, Ecosystem,
        MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE, compare_artifact_snapshots,
        is_scanner_owned_artifact,
    },
};

static NEXT_TEMP_FILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SNAPSHOT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactSnapshotState {
    version: u32,
    workspaces: BTreeMap<String, Vec<ArtifactSnapshot>>,
}

impl Default for ArtifactSnapshotState {
    fn default() -> Self {
        Self {
            version: ARTIFACT_SNAPSHOT_STATE_VERSION,
            workspaces: BTreeMap::new(),
        }
    }
}

/// Access to the versioned local generated-artifact snapshot collection.
pub struct ArtifactSnapshotStore {
    path: PathBuf,
}

impl ArtifactSnapshotStore {
    /// Creates a store backed by an explicit path, useful for isolated callers/tests.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the path used by this store.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads all retained snapshots in deterministic workspace order.
    pub fn load_all(&self) -> DustResult<Vec<ArtifactSnapshot>> {
        let _guard = snapshot_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let state = load_state(&self.path)?;

        Ok(state.workspaces.into_values().flatten().collect())
    }

    /// Loads retained snapshots for one canonical workspace.
    pub fn load_workspace(&self, workspace_path: &Path) -> DustResult<Vec<ArtifactSnapshot>> {
        let workspace_id = workspace_id(workspace_path)?;
        let _guard = snapshot_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let state = load_state(&self.path)?;

        Ok(state
            .workspaces
            .get(&workspace_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Creates and persists one snapshot from an existing analysis result.
    ///
    /// Snapshot construction only reads the structured analysis fields. It
    /// does not inspect or recursively walk any artifact path.
    pub fn record(
        &self,
        workspace_path: &Path,
        analysis: &AnalysisResult,
    ) -> DustResult<ArtifactSnapshotResult> {
        self.record_with_ecosystems(workspace_path, analysis, &[])
    }

    /// Creates and persists one snapshot while preserving artifacts from
    /// ecosystems not included in the selected scan.
    pub fn record_with_ecosystems(
        &self,
        workspace_path: &Path,
        analysis: &AnalysisResult,
        selected_ecosystems: &[Ecosystem],
    ) -> DustResult<ArtifactSnapshotResult> {
        let workspace_id = workspace_id(workspace_path)?;
        let snapshot =
            ArtifactSnapshot::from_analysis_at(workspace_id, workspace_path, analysis, Utc::now());
        self.record_snapshot_with_ecosystems(snapshot, selected_ecosystems)
    }

    /// Persists a caller-constructed snapshot and compares it with its previous state.
    pub fn record_snapshot(
        &self,
        snapshot: ArtifactSnapshot,
    ) -> DustResult<ArtifactSnapshotResult> {
        self.record_snapshot_with_ecosystems(snapshot, &[])
    }

    /// Persists a snapshot while treating omitted ecosystems as unobserved.
    ///
    /// The latest state from omitted ecosystems is carried forward so a
    /// filtered scan cannot turn an unobserved artifact into `Removed`, nor
    /// cause a later full scan to report it as spuriously `New`.
    pub fn record_snapshot_with_ecosystems(
        &self,
        snapshot: ArtifactSnapshot,
        selected_ecosystems: &[Ecosystem],
    ) -> DustResult<ArtifactSnapshotResult> {
        validate_snapshot(&snapshot)?;

        let _guard = snapshot_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let state = load_state(&self.path)?;
        let previous_snapshot = state
            .workspaces
            .get(&snapshot.workspace_id)
            .and_then(|snapshots| snapshots.last())
            .cloned();
        let snapshot =
            merge_unselected_artifacts(previous_snapshot.as_ref(), snapshot, selected_ecosystems);
        validate_snapshot(&snapshot)?;
        let status = if previous_snapshot.is_some() {
            ArtifactSnapshotStatus::Compared
        } else {
            ArtifactSnapshotStatus::BaselineCreated
        };
        let changes = previous_snapshot
            .as_ref()
            .map(|previous| compare_artifact_snapshots(previous, &snapshot))
            .unwrap_or_default();

        let mut next_state = state;
        let snapshots = next_state
            .workspaces
            .entry(snapshot.workspace_id.clone())
            .or_default();
        snapshots.push(snapshot.clone());
        if snapshots.len() > MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE {
            let first_retained = snapshots.len() - MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE;
            snapshots.drain(..first_retained);
        }

        save_state(&self.path, &next_state)?;

        Ok(ArtifactSnapshotResult {
            status,
            snapshot,
            previous_snapshot,
            changes,
        })
    }
}

fn merge_unselected_artifacts(
    previous: Option<&ArtifactSnapshot>,
    mut current: ArtifactSnapshot,
    selected_ecosystems: &[Ecosystem],
) -> ArtifactSnapshot {
    if selected_ecosystems.is_empty() {
        return current;
    }

    let mut current_identities = current
        .artifacts
        .iter()
        .map(ArtifactSnapshotArtifact::identity_key)
        .collect::<BTreeSet<_>>();

    if let Some(previous) = previous {
        current.artifacts.extend(
            previous
                .artifacts
                .iter()
                .filter(|artifact| {
                    !selected_ecosystems.contains(&artifact.ecosystem)
                        && current_identities.insert(artifact.identity_key())
                })
                .cloned(),
        );
        current
            .artifacts
            .sort_by(ArtifactSnapshotArtifact::compare_identity);
    }

    current
}

/// Returns the OS-specific local path used for artifact snapshots.
pub fn default_state_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;

    fs::create_dir_all(dirs.data_dir())?;
    Ok(dirs.data_dir().join("artifact-snapshots.json"))
}

fn workspace_id(path: &Path) -> io::Result<String> {
    fs::canonicalize(path).map(|path| path.display().to_string())
}

fn load_state(path: &Path) -> DustResult<ArtifactSnapshotState> {
    if !path.exists() {
        return Ok(ArtifactSnapshotState::default());
    }

    let json = fs::read_to_string(path).map_err(DustError::from)?;
    let value: Value = serde_json::from_str(&json).map_err(state_error)?;
    let state: ArtifactSnapshotState = serde_json::from_value(value).map_err(state_error)?;

    if state.version != ARTIFACT_SNAPSHOT_STATE_VERSION {
        return Err(DustError::ArtifactSnapshotState(format!(
            "unsupported artifact snapshot state version: {}",
            state.version
        )));
    }

    for (stored_workspace_id, snapshots) in &state.workspaces {
        if snapshots.is_empty() {
            return Err(DustError::ArtifactSnapshotState(format!(
                "workspace {stored_workspace_id:?} has no snapshots"
            )));
        }

        if snapshots.len() > MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE {
            return Err(DustError::ArtifactSnapshotState(format!(
                "workspace {stored_workspace_id:?} exceeds snapshot retention limit"
            )));
        }

        for snapshot in snapshots {
            if &snapshot.workspace_id != stored_workspace_id {
                return Err(DustError::ArtifactSnapshotState(
                    "snapshot workspace key does not match workspace_id".to_owned(),
                ));
            }
            validate_snapshot(snapshot)?;
        }
    }

    Ok(state)
}

fn save_state(path: &Path, state: &ArtifactSnapshotState) -> DustResult<()> {
    if state.version != ARTIFACT_SNAPSHOT_STATE_VERSION {
        return Err(DustError::ArtifactSnapshotState(format!(
            "unsupported artifact snapshot state version: {}",
            state.version
        )));
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state).map_err(state_serialize_error)?;
    let temporary_path = temporary_path(path);
    let write_result = (|| -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        replace_file(&temporary_path, path)
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }

    Ok(())
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    // SAFETY: both buffers are null-terminated UTF-16 paths that remain alive
    // for the duration of the call, and the flags request replacement with a
    // write-through move so an existing destination is replaced atomically.
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn validate_snapshot(snapshot: &ArtifactSnapshot) -> DustResult<()> {
    if snapshot.workspace_id.is_empty() {
        return Err(DustError::ArtifactSnapshotState(
            "snapshot workspace_id must not be empty".to_owned(),
        ));
    }

    let mut identities = BTreeSet::new();
    for artifact in &snapshot.artifacts {
        if !is_scanner_owned_artifact(artifact.ecosystem, &artifact.path) {
            return Err(DustError::ArtifactSnapshotState(format!(
                "snapshot contains unsupported artifact path: {}",
                artifact.path.display()
            )));
        }
        if !identities.insert(artifact.identity_key()) {
            return Err(DustError::ArtifactSnapshotState(format!(
                "snapshot contains duplicate artifact identity: {}",
                artifact.identity_key()
            )));
        }
    }

    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact-snapshots.json");
    let id = NEXT_TEMP_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
}

fn state_error(error: serde_json::Error) -> DustError {
    DustError::ArtifactSnapshotState(error.to_string())
}

fn state_serialize_error(error: serde_json::Error) -> DustError {
    DustError::ArtifactSnapshotState(format!(
        "could not serialize artifact snapshot state: {error}"
    ))
}

fn snapshot_lock() -> &'static Mutex<()> {
    SNAPSHOT_LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use chrono::TimeZone;
    use tempfile::TempDir;

    use super::*;
    use crate::models::{
        Artifact, ArtifactAnalysis, ArtifactChangeKind, CleanupRecommendation, Ecosystem,
    };

    fn analyzed_artifact(
        path: impl Into<PathBuf>,
        ecosystem: Ecosystem,
        size_bytes: u64,
    ) -> ArtifactAnalysis {
        ArtifactAnalysis {
            artifact: Artifact::new(path.into(), ecosystem),
            size_bytes,
            last_modified: None,
            age_days: None,
            recommendation: CleanupRecommendation::Keep,
        }
    }

    fn analysis(path: &str, size_bytes: u64) -> AnalysisResult {
        AnalysisResult {
            artifacts: vec![analyzed_artifact(path, Ecosystem::Rust, size_bytes)],
            total_size_bytes: size_bytes,
        }
    }

    fn snapshot(workspace: &Path, path: &str, size_bytes: u64, timestamp: i64) -> ArtifactSnapshot {
        ArtifactSnapshot::from_analysis_at(
            fs::canonicalize(workspace).unwrap().display().to_string(),
            workspace,
            &analysis(path, size_bytes),
            Utc.timestamp_opt(timestamp, 0).single().unwrap(),
        )
    }

    #[test]
    fn first_snapshot_creates_a_baseline_without_growth_changes() {
        let workspace = TempDir::new().unwrap();
        let store = ArtifactSnapshotStore::new(workspace.path().join("snapshots.json"));

        let result = store
            .record_snapshot(snapshot(workspace.path(), "target", 10, 1))
            .unwrap();

        assert_eq!(result.status, ArtifactSnapshotStatus::BaselineCreated);
        assert!(result.previous_snapshot.is_none());
        assert!(result.changes.is_empty());
    }

    #[test]
    fn record_compares_size_changes_and_retains_previous_workspace_state() {
        let workspace = TempDir::new().unwrap();
        let store = ArtifactSnapshotStore::new(workspace.path().join("snapshots.json"));

        store
            .record_snapshot(snapshot(workspace.path(), "target", 10, 1))
            .unwrap();
        let result = store
            .record_snapshot(snapshot(workspace.path(), "target", 15, 2))
            .unwrap();

        assert_eq!(result.status, ArtifactSnapshotStatus::Compared);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].kind, ArtifactChangeKind::SizeIncreased);
        assert_eq!(result.changes[0].previous_size_bytes, Some(10));
        assert_eq!(result.changes[0].current_size_bytes, Some(15));
        assert_eq!(result.changes[0].delta_bytes, 5);
        assert_eq!(store.load_all().unwrap().len(), 2);
    }

    #[test]
    fn filtered_snapshot_preserves_unselected_ecosystems() {
        let workspace = TempDir::new().unwrap();
        let store = ArtifactSnapshotStore::new(workspace.path().join("snapshots.json"));
        let workspace_id = fs::canonicalize(workspace.path())
            .unwrap()
            .display()
            .to_string();
        let complete_analysis = AnalysisResult {
            artifacts: vec![
                analyzed_artifact(workspace.path().join("target"), Ecosystem::Rust, 10),
                analyzed_artifact(workspace.path().join("node_modules"), Ecosystem::Node, 20),
                analyzed_artifact(workspace.path().join("build"), Ecosystem::Java, 30),
            ],
            total_size_bytes: 60,
        };
        store
            .record_snapshot(ArtifactSnapshot::from_analysis_at(
                workspace_id,
                workspace.path(),
                &complete_analysis,
                Utc.timestamp_opt(1, 0).single().unwrap(),
            ))
            .unwrap();

        let filtered_analysis = AnalysisResult {
            artifacts: vec![analyzed_artifact(
                workspace.path().join("target"),
                Ecosystem::Rust,
                15,
            )],
            total_size_bytes: 15,
        };
        let filtered_result = store
            .record_with_ecosystems(workspace.path(), &filtered_analysis, &[Ecosystem::Rust])
            .unwrap();

        assert_eq!(filtered_result.snapshot.artifacts.len(), 3);
        assert!(filtered_result.changes.iter().all(|change| !matches!(
            change.kind,
            ArtifactChangeKind::New | ArtifactChangeKind::Removed
        )));
        assert!(
            filtered_result
                .changes
                .iter()
                .any(|change| change.kind == ArtifactChangeKind::SizeIncreased)
        );

        let full_analysis = AnalysisResult {
            artifacts: vec![
                analyzed_artifact(workspace.path().join("target"), Ecosystem::Rust, 15),
                analyzed_artifact(workspace.path().join("node_modules"), Ecosystem::Node, 20),
                analyzed_artifact(workspace.path().join("build"), Ecosystem::Java, 30),
            ],
            total_size_bytes: 65,
        };
        let full_result = store
            .record_with_ecosystems(workspace.path(), &full_analysis, &[])
            .unwrap();

        assert!(full_result.changes.iter().all(|change| !matches!(
            change.kind,
            ArtifactChangeKind::New | ArtifactChangeKind::Removed
        )));
    }

    #[test]
    fn workspaces_with_the_same_artifact_name_are_isolated() {
        let root = TempDir::new().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let store = ArtifactSnapshotStore::new(root.path().join("snapshots.json"));

        store
            .record_snapshot(snapshot(&first, "target", 10, 1))
            .unwrap();
        let second_result = store
            .record_snapshot(snapshot(&second, "target", 20, 2))
            .unwrap();

        assert_eq!(
            second_result.status,
            ArtifactSnapshotStatus::BaselineCreated
        );
        assert_eq!(store.load_workspace(&first).unwrap().len(), 1);
        assert_eq!(store.load_workspace(&second).unwrap().len(), 1);
    }

    #[test]
    fn snapshots_survive_reload_and_preserve_other_workspaces() {
        let root = TempDir::new().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let path = root.path().join("snapshots.json");
        let store = ArtifactSnapshotStore::new(&path);

        store
            .record_snapshot(snapshot(&first, "target", 10, 1))
            .unwrap();
        store
            .record_snapshot(snapshot(&second, "target", 20, 2))
            .unwrap();

        let reloaded = ArtifactSnapshotStore::new(path).load_all().unwrap();

        assert_eq!(reloaded.len(), 2);
        assert_eq!(reloaded[0].artifacts[0].size_bytes, 10);
        assert_eq!(reloaded[1].artifacts[0].size_bytes, 20);
    }

    #[test]
    fn malformed_and_unsupported_state_are_explicit_errors() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("snapshots.json");
        let store = ArtifactSnapshotStore::new(&path);

        fs::write(&path, "not json").unwrap();
        assert!(matches!(
            store.load_all(),
            Err(DustError::ArtifactSnapshotState(_))
        ));

        fs::write(&path, r#"{"version":2,"workspaces":{}}"#).unwrap();
        assert!(matches!(
            store.load_all(),
            Err(DustError::ArtifactSnapshotState(message)) if message.contains("unsupported")
        ));
    }

    #[test]
    fn persistence_failure_does_not_mutate_the_completed_analysis() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("unwritable");
        fs::create_dir(&path).unwrap();
        let store = ArtifactSnapshotStore::new(&path);
        let analysis = analysis("target", 10);

        assert!(store.record(temp.path(), &analysis).is_err());
        assert_eq!(analysis.total_size_bytes, 10);
        assert_eq!(analysis.artifacts[0].size_bytes, 10);
    }

    #[test]
    fn retention_is_bounded_to_the_documented_limit() {
        let workspace = TempDir::new().unwrap();
        let store = ArtifactSnapshotStore::new(workspace.path().join("snapshots.json"));

        for timestamp in 0..(MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE + 3) as i64 {
            store
                .record_snapshot(snapshot(
                    workspace.path(),
                    "target",
                    timestamp as u64,
                    timestamp,
                ))
                .unwrap();
        }

        let snapshots = store.load_all().unwrap();
        assert_eq!(snapshots.len(), MAX_ARTIFACT_SNAPSHOTS_PER_WORKSPACE);
        assert_eq!(snapshots[0].artifacts[0].size_bytes, 3);
    }
}
