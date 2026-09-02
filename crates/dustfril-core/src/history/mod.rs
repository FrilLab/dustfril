use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    models::{
        ActivityRecord, CleanupHistoryEntry, CleanupResult, DeleteMode, Ecosystem,
        ScanAccessSummary, ScanResult, SecurityReport,
    },
};

const HISTORY_VERSION: u32 = 1;

static NEXT_TEMP_FILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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

/// Records a scan that failed before producing a scan result.
pub fn record_scan_failure(target_path: &Path, reason: &str) -> DustResult<()> {
    record(ActivityRecord::scan_failure(target_path, reason))
}

/// Records a failed scan together with the bounded summary collected before
/// traversal stopped.
pub fn record_scan_failure_with_summary(
    target_path: &Path,
    reason: &str,
    access_summary: &ScanAccessSummary,
) -> DustResult<()> {
    record(ActivityRecord::scan_failure_with_summary(
        target_path,
        reason,
        access_summary,
    ))
}

/// Records a cleanup that failed before producing a cleanup result.
pub fn record_cleanup_failure(mode: DeleteMode, reason: &str) -> DustResult<()> {
    record(ActivityRecord::cleanup_failure(mode, reason))
}

/// Records one explicit security scan in the unified activity history.
pub fn record_security_scan(
    target_path: &Path,
    ecosystems: &[Ecosystem],
    report: &SecurityReport,
) -> DustResult<()> {
    record(ActivityRecord::security(target_path, ecosystems, report))
}

/// Records a failed explicit security scan in the unified activity history.
pub fn record_security_failure(
    target_path: &Path,
    ecosystems: &[Ecosystem],
    reason: &str,
) -> DustResult<()> {
    record(ActivityRecord::security_failure(
        target_path,
        ecosystems,
        reason,
    ))
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
        // The original history remains untouched when serialization or the
        // replacement fails. Best-effort cleanup avoids leaving stale files
        // beside the history file without hiding the actual write error.
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }

    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("history.json");
    let id = NEXT_TEMP_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
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
        RiskLevel, ScanAccessSummary, SecurityFinding, SecurityFindingKind, SecurityReport,
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
        assert_eq!(history[0].result.details["mode"], "permanent");
        assert_eq!(history[0].result.details["deleted"][0], "build");
        assert_eq!(history[0].result.details["failed"][0]["path"], "target");

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
    fn corrupted_history_is_reported_without_overwriting_the_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let original = "{not valid json";
        std::fs::write(&path, original).unwrap();

        let result = load_unlocked(&path);

        assert!(matches!(result, Err(DustError::Io(_))));
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn unsupported_history_version_is_rejected() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        std::fs::write(&path, r#"{"version":2,"records":[]}"#).unwrap();

        let result = load_unlocked(&path);

        assert!(
            matches!(result, Err(DustError::Io(error)) if error.to_string().contains("Unsupported activity history version: 2"))
        );
    }

    #[test]
    fn save_replaces_history_without_leaving_a_temporary_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let activity = ActivityRecord::new(
            ActivityKind::Scan,
            ActivityResult::new(true, json!({"artifacts": 0})),
        );

        record_to(&path, activity).unwrap();

        let files = std::fs::read_dir(temp.path()).unwrap().collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap().file_name(), "history.json");
    }

    #[test]
    fn scan_activity_contains_target_summary() {
        let scan = ScanResult {
            artifacts: vec![Artifact::new("target".into(), Ecosystem::Rust)],
            ..ScanResult::default()
        };

        let activity = ActivityRecord::scan(Path::new("/workspace"), &scan, 4096);

        assert_eq!(activity.kind, ActivityKind::Scan);
        assert_eq!(activity.result.details["path"], "/workspace");
        assert_eq!(activity.result.details["artifacts"], 1);
        assert_eq!(activity.result.details["size"], 4096);
    }

    #[test]
    fn scan_access_summary_survives_history_reload() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let mut access_summary = ScanAccessSummary::new("/workspace");
        access_summary.directories_visited = 2;
        access_summary.files_inspected = 1;
        access_summary.metadata_files_inspected = 1;
        access_summary.artifact_candidates = 1;

        let scan = ScanResult {
            artifacts: vec![Artifact::new("/workspace/target".into(), Ecosystem::Rust)],
            access_summary,
        };
        record_to(
            &path,
            ActivityRecord::scan(Path::new("/workspace"), &scan, 1024),
        )
        .unwrap();

        let records = load_unlocked(&path).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].result.details["accessSummary"]["directoriesVisited"],
            2
        );
        assert_eq!(
            records[0].result.details["accessSummary"]["artifactCandidates"],
            1
        );
    }

    #[test]
    fn failed_scan_access_summary_survives_history_reload() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let mut access_summary = ScanAccessSummary::new("/workspace");
        access_summary.record_failure(Path::new("/workspace/restricted"), "permission denied");

        record_to(
            &path,
            ActivityRecord::scan_failure_with_summary(
                Path::new("/workspace"),
                "I/O error: permission denied",
                &access_summary,
            ),
        )
        .unwrap();

        let records = load_unlocked(&path).unwrap();

        assert_eq!(records.len(), 1);
        assert!(!records[0].result.success);
        assert_eq!(records[0].result.details["accessSummary"]["failures"], 1);
        assert_eq!(
            records[0].result.details["accessSummary"]["failureSamples"][0]["path"],
            "restricted"
        );
    }

    #[test]
    fn scan_history_does_not_persist_unrelated_source_contents() {
        let workspace = TempDir::new().unwrap();
        let source_contents = "unique source contents must stay out of history";
        std::fs::write(workspace.path().join("source.rs"), source_contents).unwrap();
        let scan = crate::api::scan(workspace.path(), &[]).unwrap();
        let activity = ActivityRecord::scan(workspace.path(), &scan, 0);

        let serialized = serde_json::to_string(&activity).unwrap();

        assert!(!serialized.contains(source_contents));
        assert!(!serialized.contains("source.rs"));
    }

    #[test]
    fn cleanup_activity_marks_partial_failure() {
        let activity = ActivityRecord::cleanup(DeleteMode::Trash, &cleanup_result());

        assert!(!activity.result.success);
        assert_eq!(activity.result.details["deleted"][0], "target");
        assert_eq!(activity.result.details["failed"][0]["path"], "node_modules");
    }

    #[test]
    fn cleanup_activity_records_success_without_items() {
        let activity = ActivityRecord::cleanup(DeleteMode::Trash, &CleanupResult::default());

        assert!(activity.result.success);
        assert!(
            activity.result.details["deleted"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            activity.result.details["failed"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(activity.result.details["freed"], 0);
    }

    #[test]
    fn failed_operations_are_reloadable_activity_records() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");

        record_to(
            &path,
            ActivityRecord::scan_failure(Path::new("/workspace"), "scan failed"),
        )
        .unwrap();
        record_to(
            &path,
            ActivityRecord::cleanup_failure(DeleteMode::Permanent, "cleanup failed"),
        )
        .unwrap();

        let records = load_unlocked(&path).unwrap();

        assert_eq!(records.len(), 2);
        assert!(!records[0].result.success);
        assert!(!records[1].result.success);
        assert_eq!(records[0].kind, ActivityKind::Scan);
        assert_eq!(records[1].kind, ActivityKind::Cleanup);
    }

    #[test]
    fn replacement_failure_keeps_existing_valid_history() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let activity = ActivityRecord::new(
            ActivityKind::Scan,
            ActivityResult::new(true, json!({"artifacts": 0})),
        );
        record_to(&path, activity.clone()).unwrap();
        let original = std::fs::read_to_string(&path).unwrap();

        let replacement_source = temp.path().join("replacement-source");
        std::fs::create_dir(&replacement_source).unwrap();
        let result = replace_file(&replacement_source, &path);

        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn failed_save_cleans_temporary_file_without_touching_destination() {
        let temp = TempDir::new().unwrap();
        let destination = temp.path().join("history-directory");
        std::fs::create_dir(&destination).unwrap();
        let activity = ActivityRecord::new(
            ActivityKind::Scan,
            ActivityResult::new(true, json!({"artifacts": 0})),
        );

        let result = save(&destination, &[activity]);

        assert!(result.is_err());
        assert!(destination.is_dir());
        assert_eq!(
            std::fs::read_dir(temp.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
                .count(),
            0
        );
    }

    #[test]
    fn concurrent_appends_remain_parseable_and_lossless() {
        let temp = TempDir::new().unwrap();
        let path = std::sync::Arc::new(temp.path().join("history.json"));
        let workers = 12;

        let handles = (0..workers)
            .map(|index| {
                let path = std::sync::Arc::clone(&path);
                std::thread::spawn(move || {
                    record_to(
                        &path,
                        ActivityRecord::new(
                            ActivityKind::Scan,
                            ActivityResult::new(true, json!({"worker": index})),
                        ),
                    )
                    .unwrap();
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        let records = load_unlocked(&path).unwrap();
        assert_eq!(records.len(), workers);
        assert!(records.iter().all(|record| record.result.success));
    }

    #[test]
    fn security_activity_is_persisted_as_one_reloadable_unified_record() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");
        let report = SecurityReport {
            findings: vec![SecurityFinding::new(
                temp.path().join("package.json"),
                SecurityFindingKind::MissingLockfile,
                None,
                RiskLevel::High,
                None,
                "Expected lockfile is missing.",
            )],
            ..SecurityReport::default()
        };

        record_to(
            &path,
            ActivityRecord::security(temp.path(), &[Ecosystem::Node], &report),
        )
        .unwrap();

        let records = load_unlocked(&path).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].kind, ActivityKind::Security);
        assert_eq!(records[0].result.details["findingCount"], 1);
        assert_eq!(records[0].result.details["highestRisk"], "High");
        assert_eq!(
            records[0].result.details["findings"][0]["source"],
            "package.json"
        );
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
