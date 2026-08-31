use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    models::{ActivityRecord, CleanupHistoryEntry, CleanupResult, DeleteMode, ScanResult},
};

const HISTORY_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct ActivityHistoryFile {
    version: u32,
    records: Vec<ActivityRecord>,
}

static HISTORY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Appends a unified activity record to the local history.
pub fn record(activity: ActivityRecord) -> DustResult<()> {
    let path = history_path()?;
    let _guard = history_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut records = load_unlocked(&path)?;

    records.push(activity);
    save(&path, &records)
}

/// Records a cleanup operation in the unified activity history.
pub fn record_cleanup(mode: DeleteMode, result: &CleanupResult) -> DustResult<()> {
    record(ActivityRecord::cleanup(mode, result))
}

/// Records a scan operation in the unified activity history.
pub fn record_scan(
    target_path: &Path,
    result: &ScanResult,
    total_size_bytes: u64,
) -> DustResult<()> {
    record(ActivityRecord::scan(target_path, result, total_size_bytes))
}

/// Loads all activity records, migrating a legacy cleanup history when needed.
pub fn load_all() -> DustResult<Vec<ActivityRecord>> {
    let path = history_path()?;
    let _guard = history_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    load_unlocked(&path)
}

pub fn history_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;

    let data_dir = dirs.data_dir();

    fs::create_dir_all(data_dir)?;

    Ok(data_dir.join("history.json"))
}

fn load_unlocked(path: &Path) -> DustResult<Vec<ActivityRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(path)?;
    let (records, was_legacy) = decode(&json)?;

    if was_legacy {
        save(path, &records)?;
    }

    Ok(records)
}

fn decode(json: &str) -> DustResult<(Vec<ActivityRecord>, bool)> {
    let value: Value = serde_json::from_str(json).map_err(json_error)?;

    if value.is_array() {
        let legacy_entries: Vec<CleanupHistoryEntry> =
            serde_json::from_value(value).map_err(json_error)?;
        let records = legacy_entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| entry.into_activity_record(format!("legacy-cleanup-{index}")))
            .collect();

        return Ok((records, true));
    }

    let history: ActivityHistoryFile = serde_json::from_value(value).map_err(json_error)?;

    if history.version != HISTORY_VERSION {
        return Err(DustError::Io(io::Error::other(format!(
            "Unsupported activity history version: {}",
            history.version
        ))));
    }

    Ok((history.records, false))
}

fn save(path: &Path, records: &[ActivityRecord]) -> DustResult<()> {
    let history = ActivityHistoryFile {
        version: HISTORY_VERSION,
        records: records.to_vec(),
    };
    let json = serde_json::to_string_pretty(&history).map_err(json_error)?;

    fs::write(path, json)?;

    Ok(())
}

fn json_error(error: serde_json::Error) -> DustError {
    DustError::Io(io::Error::other(error))
}

fn history_lock() -> &'static Mutex<()> {
    HISTORY_LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::models::{
        ActivityKind, ActivityResult, Artifact, CleanupFailure, CleanupFailureReason, Ecosystem,
    };

    fn cleanup_result() -> CleanupResult {
        CleanupResult {
            deleted_paths: vec!["target".into()],
            failed_paths: vec![CleanupFailure {
                path: "node_modules".into(),
                reason: CleanupFailureReason::NotFound,
            }],
            freed_size_bytes: 1024,
        }
    }

    #[test]
    fn record_appends_versioned_activity_history() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let activity = ActivityRecord::cleanup(DeleteMode::Trash, &cleanup_result());

        record_to(&path, activity.clone()).unwrap();

        let history = load_unlocked(&path).unwrap();

        assert_eq!(history, vec![activity]);
        let file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(file["version"], 1);
        assert_eq!(file["records"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn legacy_cleanup_history_is_loaded_and_migrated() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let executed_at: DateTime<Utc> = "2026-01-02T03:04:05Z".parse().unwrap();
        let legacy = vec![CleanupHistoryEntry {
            executed_at,
            mode: DeleteMode::Permanent,
            freed_size_bytes: 2048,
            deleted_paths: vec!["build".into()],
            failed_paths: vec!["target".into()],
        }];
        std::fs::write(&path, serde_json::to_string(&legacy).unwrap()).unwrap();

        let history = load_unlocked(&path).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "legacy-cleanup-0");
        assert_eq!(history[0].timestamp, executed_at);
        assert_eq!(history[0].kind, crate::models::ActivityKind::Cleanup);
        assert!(!history[0].result.success);
        assert_eq!(history[0].result.details["freed"], 2048);

        let migrated: Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(migrated["version"], 1);
        assert!(migrated["records"].is_array());
    }

    #[test]
    fn recording_after_migration_preserves_legacy_entries() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let legacy = vec![CleanupHistoryEntry {
            executed_at: Utc::now(),
            mode: DeleteMode::Trash,
            freed_size_bytes: 1,
            deleted_paths: vec![],
            failed_paths: vec![],
        }];
        std::fs::write(&path, serde_json::to_string(&legacy).unwrap()).unwrap();

        record_to(
            &path,
            ActivityRecord::new(
                ActivityKind::Scan,
                ActivityResult::new(
                    true,
                    json!({"path": "/workspace", "artifacts": 0, "size": 0}),
                ),
            ),
        )
        .unwrap();

        assert_eq!(load_unlocked(&path).unwrap().len(), 2);
    }

    #[test]
    fn scan_activity_contains_target_summary() {
        let scan = ScanResult {
            artifacts: vec![Artifact::new("target".into(), Ecosystem::Rust)],
        };

        let activity = ActivityRecord::scan(Path::new("/workspace"), &scan, 4096);

        assert_eq!(activity.kind, ActivityKind::Scan);
        assert_eq!(activity.result.details["path"], "/workspace");
        assert_eq!(activity.result.details["artifacts"], 1);
        assert_eq!(activity.result.details["size"], 4096);
    }

    #[test]
    fn cleanup_activity_marks_partial_failure() {
        let activity = ActivityRecord::cleanup(DeleteMode::Trash, &cleanup_result());

        assert!(!activity.result.success);
        assert_eq!(activity.result.details["deleted"][0], "target");
        assert_eq!(activity.result.details["failed"][0]["path"], "node_modules");
    }

    fn record_to(path: &Path, activity: ActivityRecord) -> DustResult<()> {
        let _guard = history_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut records = load_unlocked(path)?;
        records.push(activity);
        save(path, &records)
    }
}
