//! Explicit, local dependency baseline storage and logical comparison.
//!
//! A comparison reads an existing baseline without replacing it. The only
//! implicit write is creating a missing first-observation baseline; callers
//! must explicitly call `accept` to replace an existing comparison point.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use directories::ProjectDirs;
use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    models::{
        DEPENDENCY_BASELINE_STATE_VERSION, DependencyBaseline, DependencyBaselineState,
        DependencyBaselineStatus, DependencyChange, DependencyChangeKind, DependencyDiff,
        DependencyEntry, DependencyReport, DependencyReportStatus, DependencyScope, Ecosystem,
    },
};

static BASELINE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static NEXT_TEMP_FILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

type Inventories = BTreeMap<Ecosystem, Vec<DependencyEntry>>;

/// Access to the versioned local dependency-baseline collection.
#[derive(Debug, Clone)]
pub struct DependencyBaselineStore {
    path: PathBuf,
}

impl DependencyBaselineStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads all project baselines. Missing state is an empty collection;
    /// malformed or unsupported state is returned as an explicit error.
    pub fn load(&self) -> DustResult<DependencyBaselineState> {
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        load_unlocked(&self.path)
    }

    /// Atomically replaces the versioned baseline collection.
    pub fn save(&self, state: &DependencyBaselineState) -> DustResult<()> {
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        save_unlocked(&self.path, state)
    }

    /// Compares reports with the stored baseline. A missing project (or
    /// ecosystem within an existing project) is initialized without emitting
    /// false Added findings. An existing baseline is never replaced here.
    pub fn compare(
        &self,
        workspace_root: &Path,
        reports: &[DependencyReport],
    ) -> DustResult<DependencyDiff> {
        let workspace_id = workspace_id(workspace_root)?;
        let (current, warnings) = current_inventories(reports)?;
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut state = load_unlocked(&self.path)?;

        let Some(previous) = state.projects.get(&workspace_id).cloned() else {
            state
                .projects
                .insert(workspace_id.clone(), baseline_for(&workspace_id, &current));
            save_unlocked(&self.path, &state)?;

            let mut diff =
                DependencyDiff::empty(workspace_id, DependencyBaselineStatus::BaselineCreated);
            diff.warnings = warnings;
            return Ok(diff);
        };

        let mut diff = compare_inventories(&workspace_id, &previous.inventories, &current);
        diff.warnings = warnings;

        let mut missing_ecosystem = false;
        let mut merged = previous.inventories.clone();
        for (ecosystem, entries) in &current {
            if !merged.contains_key(ecosystem) {
                merged.insert(*ecosystem, entries.clone());
                missing_ecosystem = true;
            }
        }
        if missing_ecosystem {
            state.projects.insert(
                workspace_id.clone(),
                DependencyBaseline {
                    workspace_id: workspace_id.clone(),
                    inventories: merged,
                },
            );
            save_unlocked(&self.path, &state)?;
            diff.baseline_status = DependencyBaselineStatus::BaselineCreated;
        }

        Ok(diff)
    }

    /// Explicitly accepts the supplied complete inventories as the next
    /// baseline. Existing baselines for other workspaces or ecosystems remain
    /// untouched.
    pub fn accept(&self, workspace_root: &Path, reports: &[DependencyReport]) -> DustResult<()> {
        let workspace_id = workspace_id(workspace_root)?;
        let (current, _) = current_inventories(reports)?;
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut state = load_unlocked(&self.path)?;
        let mut inventories = state
            .projects
            .get(&workspace_id)
            .map(|baseline| baseline.inventories.clone())
            .unwrap_or_default();
        inventories.extend(current);
        state.projects.insert(
            workspace_id.clone(),
            DependencyBaseline {
                workspace_id,
                inventories,
            },
        );
        save_unlocked(&self.path, &state)
    }
}

/// Returns the OS-specific local dependency-baseline path.
pub fn default_state_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("dependency-baseline.json"))
}

fn current_inventories(reports: &[DependencyReport]) -> DustResult<(Inventories, Vec<String>)> {
    let mut inventories = BTreeMap::new();
    let mut warnings = Vec::new();

    for report in reports {
        if report.status != DependencyReportStatus::Complete {
            let reason = report
                .warnings
                .first()
                .cloned()
                .unwrap_or_else(|| "the inventory is not complete".to_owned());
            warnings.push(format!(
                "{} dependency baseline comparison skipped: {reason}",
                report.ecosystem
            ));
            continue;
        }

        let mut entries = BTreeMap::<IdentityKey, DependencyEntry>::new();
        for entry in &report.resolved_dependencies {
            if entry.ecosystem != report.ecosystem {
                return Err(DustError::DependencyState(format!(
                    "report ecosystem {} does not match dependency entry {}",
                    report.ecosystem, entry.name
                )));
            }
            let key = IdentityKey::from(entry);
            entries
                .entry(key)
                .and_modify(|existing| existing.scope = merge_scope(existing.scope, entry.scope))
                .or_insert_with(|| entry.clone());
        }
        inventories.insert(report.ecosystem, entries.into_values().collect());
    }

    if inventories.is_empty() {
        return Err(DustError::DependencyState(
            "at least one complete dependency inventory is required to compare or accept a baseline"
                .to_owned(),
        ));
    }

    warnings.sort();
    warnings.dedup();
    Ok((inventories, warnings))
}

fn baseline_for(workspace_id: &str, inventories: &Inventories) -> DependencyBaseline {
    DependencyBaseline {
        workspace_id: workspace_id.to_owned(),
        inventories: inventories.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IdentityKey {
    ecosystem: Ecosystem,
    name: String,
    version: String,
    source: Option<String>,
}

impl From<&DependencyEntry> for IdentityKey {
    fn from(entry: &DependencyEntry) -> Self {
        Self {
            ecosystem: entry.ecosystem,
            name: entry.name.clone(),
            version: entry.version.clone(),
            source: entry.source.clone(),
        }
    }
}

fn merge_scope(left: DependencyScope, right: DependencyScope) -> DependencyScope {
    if left == right {
        return left;
    }
    if left == DependencyScope::Direct || right == DependencyScope::Direct {
        return DependencyScope::Direct;
    }
    DependencyScope::Unknown
}

fn compare_inventories(
    workspace_id: &str,
    previous: &Inventories,
    current: &Inventories,
) -> DependencyDiff {
    let mut diff = DependencyDiff::empty(workspace_id, DependencyBaselineStatus::Compared);
    // Compare only ecosystems present in this explicit scan. A filtered scan
    // must not interpret an unselected ecosystem as removed.
    for ecosystem in current.keys() {
        if !previous.contains_key(ecosystem) {
            continue;
        }
        let before = previous.get(ecosystem).map(Vec::as_slice).unwrap_or(&[]);
        let after = current.get(ecosystem).map(Vec::as_slice).unwrap_or(&[]);
        compare_entries(before, after, &mut diff);
    }

    diff.added.sort();
    diff.removed.sort();
    diff.version_changes.sort();
    diff.source_changes.sort();
    diff
}

fn compare_entries(
    previous: &[DependencyEntry],
    current: &[DependencyEntry],
    diff: &mut DependencyDiff,
) {
    let mut previous_by_identity = previous
        .iter()
        .cloned()
        .map(|entry| (IdentityKey::from(&entry), entry))
        .collect::<BTreeMap<_, _>>();
    let mut current_by_identity = current
        .iter()
        .cloned()
        .map(|entry| (IdentityKey::from(&entry), entry))
        .collect::<BTreeMap<_, _>>();

    let exact = previous_by_identity
        .keys()
        .filter(|key| current_by_identity.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    for key in exact {
        previous_by_identity.remove(&key);
        current_by_identity.remove(&key);
    }

    let names = previous_by_identity
        .values()
        .chain(current_by_identity.values())
        .map(|entry| (entry.ecosystem, entry.name.clone()))
        .collect::<BTreeSet<_>>();

    for (ecosystem, name) in names {
        let mut before = previous_by_identity
            .values()
            .filter(|entry| entry.ecosystem == ecosystem && entry.name == name)
            .cloned()
            .collect::<Vec<_>>();
        let mut after = current_by_identity
            .values()
            .filter(|entry| entry.ecosystem == ecosystem && entry.name == name)
            .cloned()
            .collect::<Vec<_>>();
        before.sort();
        after.sort();

        // A source change is trustworthy only when both snapshots contain
        // source identifiers for the same name/version.
        let mut index = 0;
        while index < before.len() {
            let Some(current_index) = after.iter().position(|candidate| {
                candidate.version == before[index].version
                    && (candidate.source == before[index].source
                        || (candidate.source.is_some() && before[index].source.is_some()))
            }) else {
                index += 1;
                continue;
            };
            let old = before.remove(index);
            let new = after.remove(current_index);
            if old.source != new.source {
                diff.source_changes.push(DependencyChange {
                    kind: DependencyChangeKind::SourceChanged,
                    previous: Some(old),
                    current: Some(new),
                });
            }
        }

        // If only one side has source metadata, retain the logical
        // name/version match and avoid inventing a source change.
        let mut index = 0;
        while index < before.len() {
            let Some(current_index) = after
                .iter()
                .position(|candidate| candidate.version == before[index].version)
            else {
                index += 1;
                continue;
            };
            before.remove(index);
            after.remove(current_index);
        }

        let version_pairs = before.len().min(after.len());
        for _ in 0..version_pairs {
            diff.version_changes.push(DependencyChange {
                kind: DependencyChangeKind::VersionChanged,
                previous: Some(before.remove(0)),
                current: Some(after.remove(0)),
            });
        }
        for entry in after {
            diff.added.push(DependencyChange {
                kind: DependencyChangeKind::Added,
                previous: None,
                current: Some(entry),
            });
        }
        for entry in before {
            diff.removed.push(DependencyChange {
                kind: DependencyChangeKind::Removed,
                previous: Some(entry),
                current: None,
            });
        }
    }
}

fn workspace_id(root: &Path) -> DustResult<String> {
    let canonical = fs::canonicalize(root).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            DustError::InvalidPath(root.to_path_buf())
        } else {
            DustError::Io(error)
        }
    })?;
    if !fs::metadata(&canonical)?.is_dir() {
        return Err(DustError::InvalidPath(root.to_path_buf()));
    }
    Ok(format!("v1:{}", canonical.to_string_lossy()))
}

fn load_unlocked(path: &Path) -> DustResult<DependencyBaselineState> {
    if !path.exists() {
        return Ok(DependencyBaselineState::default());
    }

    let json = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&json).map_err(state_error)?;
    let state: DependencyBaselineState = serde_json::from_value(value).map_err(state_error)?;

    if state.version != DEPENDENCY_BASELINE_STATE_VERSION {
        return Err(DustError::DependencyState(format!(
            "unsupported dependency baseline state version: {}",
            state.version
        )));
    }
    for (key, baseline) in &state.projects {
        if key != &baseline.workspace_id {
            return Err(DustError::DependencyState(
                "baseline project key does not match workspace_id".to_owned(),
            ));
        }
        for (ecosystem, entries) in &baseline.inventories {
            if entries.iter().any(|entry| entry.ecosystem != *ecosystem) {
                return Err(DustError::DependencyState(format!(
                    "baseline inventory for {ecosystem} contains an entry from another ecosystem"
                )));
            }
        }
    }

    Ok(state)
}

fn save_unlocked(path: &Path, state: &DependencyBaselineState) -> DustResult<()> {
    if state.version != DEPENDENCY_BASELINE_STATE_VERSION {
        return Err(DustError::DependencyState(format!(
            "unsupported dependency baseline state version: {}",
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
        fs::rename(&temporary_path, path)
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }

    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dependency-baseline.json");
    let id = NEXT_TEMP_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
}

fn state_error(error: serde_json::Error) -> DustError {
    DustError::DependencyState(error.to_string())
}

fn state_serialize_error(error: serde_json::Error) -> DustError {
    DustError::DependencyState(format!("could not serialize dependency baseline: {error}"))
}

fn baseline_lock() -> &'static Mutex<()> {
    BASELINE_LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::*;
    use crate::models::{DependencyBaselineStatus, DependencyChangeKind};

    fn entry(name: &str, version: &str, scope: DependencyScope) -> DependencyEntry {
        DependencyEntry {
            ecosystem: Ecosystem::Node,
            name: name.to_owned(),
            version: version.to_owned(),
            source: None,
            scope,
        }
    }

    fn report(entries: Vec<DependencyEntry>) -> DependencyReport {
        DependencyReport {
            ecosystem: Ecosystem::Node,
            status: DependencyReportStatus::Complete,
            manifest: Path::new("package.json").to_owned(),
            manifest_format: Some("package.json".to_owned()),
            lockfile: None,
            direct_dependency_counts: BTreeMap::new(),
            direct_dependency_total: 0,
            resolved_dependency_count: crate::models::DependencyMetric::available(entries.len()),
            transitive_dependency_count: crate::models::DependencyMetric::available(0),
            duplicate_versions: Vec::new(),
            resolved_dependencies: entries,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn first_observation_creates_baseline_without_added_findings() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let reports = vec![report(vec![entry(
            "serde",
            "1.0.0",
            DependencyScope::Direct,
        )])];

        let diff = store.compare(project.path(), &reports).unwrap();

        assert_eq!(
            diff.baseline_status,
            DependencyBaselineStatus::BaselineCreated
        );
        assert!(!diff.has_changes());
        assert!(state.is_file());
    }

    #[test]
    fn repeated_compare_is_empty_and_does_not_replace_baseline() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let first = vec![report(vec![
            entry("serde", "1.0.0", DependencyScope::Direct),
            entry("syn", "2.0.0", DependencyScope::Transitive),
        ])];
        store.compare(project.path(), &first).unwrap();
        let original = fs::read_to_string(&state).unwrap();

        // Reversing the caller's order models lockfile key/format changes.
        let second = vec![report(vec![
            entry("syn", "2.0.0", DependencyScope::Transitive),
            entry("serde", "1.0.0", DependencyScope::Direct),
        ])];
        let restarted_store = DependencyBaselineStore::new(&state);
        let diff = restarted_store.compare(project.path(), &second).unwrap();

        assert_eq!(diff.baseline_status, DependencyBaselineStatus::Compared);
        assert!(!diff.has_changes());
        assert_eq!(fs::read_to_string(state).unwrap(), original);
    }

    #[test]
    fn added_removed_and_version_changes_are_distinguished() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        store
            .compare(
                project.path(),
                &[report(vec![
                    entry("removed", "1.0.0", DependencyScope::Direct),
                    entry("changed", "1.0.0", DependencyScope::Direct),
                ])],
            )
            .unwrap();

        let diff = store
            .compare(
                project.path(),
                &[report(vec![
                    entry("added", "1.0.0", DependencyScope::Direct),
                    entry("changed", "2.0.0", DependencyScope::Direct),
                ])],
            )
            .unwrap();

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.version_changes.len(), 1);
        assert_eq!(
            diff.version_changes[0].kind,
            DependencyChangeKind::VersionChanged
        );
    }

    #[test]
    fn source_changes_require_source_data_on_both_sides() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let mut old = entry("serde", "1.0.0", DependencyScope::Transitive);
        old.source = Some("registry+old".to_owned());
        store.compare(project.path(), &[report(vec![old])]).unwrap();

        let mut new = entry("serde", "1.0.0", DependencyScope::Transitive);
        new.source = Some("registry+new".to_owned());
        let diff = store.compare(project.path(), &[report(vec![new])]).unwrap();

        assert_eq!(diff.source_changes.len(), 1);
        assert!(diff.version_changes.is_empty());
    }

    #[test]
    fn accepting_updates_only_when_explicitly_requested() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let old = vec![report(vec![entry("old", "1.0.0", DependencyScope::Direct)])];
        store.compare(project.path(), &old).unwrap();
        let new = vec![report(vec![entry("new", "1.0.0", DependencyScope::Direct)])];

        assert!(store.compare(project.path(), &new).unwrap().has_changes());
        store.accept(project.path(), &new).unwrap();
        assert!(!store.compare(project.path(), &new).unwrap().has_changes());
    }

    #[test]
    fn two_projects_with_the_same_package_name_are_isolated() {
        let root = TempDir::new().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let state = root.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let first_report = vec![report(vec![entry(
            "same",
            "1.0.0",
            DependencyScope::Direct,
        )])];
        let second_report = vec![report(vec![entry(
            "same",
            "2.0.0",
            DependencyScope::Direct,
        )])];

        store.compare(&first, &first_report).unwrap();
        let diff = store.compare(&second, &second_report).unwrap();

        assert_eq!(
            diff.baseline_status,
            DependencyBaselineStatus::BaselineCreated
        );
        assert!(!diff.has_changes());
        assert_eq!(store.load().unwrap().projects.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_and_canonical_paths_share_a_baseline() {
        let project = TempDir::new().unwrap();
        let link_parent = TempDir::new().unwrap();
        let link = link_parent.path().join("project");
        std::os::unix::fs::symlink(project.path(), &link).unwrap();
        let state = link_parent.path().join("state.json");
        let store = DependencyBaselineStore::new(&state);
        let reports = vec![report(vec![entry(
            "same",
            "1.0.0",
            DependencyScope::Direct,
        )])];

        store.compare(project.path(), &reports).unwrap();
        let diff = store.compare(&link, &reports).unwrap();

        assert_eq!(diff.baseline_status, DependencyBaselineStatus::Compared);
        assert!(!diff.has_changes());
    }

    #[test]
    fn malformed_or_unsupported_state_is_rejected_without_replacement() {
        let project = TempDir::new().unwrap();
        let state = project.path().join("state.json");
        fs::write(&state, r#"{"version":2,"projects":{}}"#).unwrap();
        let store = DependencyBaselineStore::new(&state);

        assert!(matches!(
            store.load(),
            Err(DustError::DependencyState(message)) if message.contains("version")
        ));
        assert_eq!(
            fs::read_to_string(state).unwrap(),
            r#"{"version":2,"projects":{}}"#
        );
    }
}
