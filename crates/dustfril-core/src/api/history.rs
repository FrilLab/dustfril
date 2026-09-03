use std::path::Path;

use crate::{
    error::DustResult,
    history,
    models::{
        ActivityRecord, CleanupCandidate, CleanupHistoryEntry, CleanupResult, DeleteMode,
        Ecosystem, ScanAccessSummary, ScanResult, SecurityReport,
    },
};

/// Backwards-compatible cleanup history entry point.
pub fn record(mode: DeleteMode, result: &CleanupResult) -> DustResult<()> {
    record_cleanup(mode, result)
}

/// Records any activity type in the local, versioned history file.
pub fn record_activity(activity: ActivityRecord) -> DustResult<()> {
    history::record(activity)
}

/// Records a cleanup operation while preserving the original API shape.
pub fn record_cleanup(mode: DeleteMode, result: &CleanupResult) -> DustResult<()> {
    history::record_cleanup(mode, result)
}

/// Records cleanup details with the workspace and analyzed candidates that
/// were available to the caller at execution time.
pub fn record_cleanup_with_context(
    target_path: &Path,
    candidates: &[CleanupCandidate],
    mode: DeleteMode,
    result: &CleanupResult,
) -> DustResult<()> {
    history::record_cleanup_with_context(target_path, candidates, mode, result)
}

/// Records a scan operation and its detected artifact summary.
pub fn record_scan(
    target_path: &Path,
    result: &ScanResult,
    total_size_bytes: u64,
) -> DustResult<()> {
    history::record_scan(target_path, result, total_size_bytes)
}

/// Records a scan that failed before producing a scan result.
pub fn record_scan_failure(target_path: &Path, reason: &str) -> DustResult<()> {
    history::record_scan_failure(target_path, reason)
}

/// Records a failed scan together with its bounded partial access summary.
pub fn record_scan_failure_with_summary(
    target_path: &Path,
    reason: &str,
    access_summary: &ScanAccessSummary,
) -> DustResult<()> {
    history::record_scan_failure_with_summary(target_path, reason, access_summary)
}

/// Records a cleanup that failed before producing a cleanup result.
pub fn record_cleanup_failure(mode: DeleteMode, reason: &str) -> DustResult<()> {
    history::record_cleanup_failure(mode, reason)
}

/// Records a cleanup preparation failure with its workspace target.
pub fn record_cleanup_failure_with_context(
    target_path: &Path,
    mode: DeleteMode,
    reason: &str,
) -> DustResult<()> {
    history::record_cleanup_failure_with_context(target_path, mode, reason)
}

/// Records one explicit security scan in the unified activity history.
pub fn record_security_scan(
    target_path: &Path,
    ecosystems: &[Ecosystem],
    report: &SecurityReport,
) -> DustResult<()> {
    history::record_security_scan(target_path, ecosystems, report)
}

/// Records a failed explicit security scan in the unified activity history.
pub fn record_security_failure(
    target_path: &Path,
    ecosystems: &[Ecosystem],
    reason: &str,
) -> DustResult<()> {
    history::record_security_failure(target_path, ecosystems, reason)
}

pub fn load_all() -> DustResult<Vec<ActivityRecord>> {
    history::load_all()
}

/// Loads only cleanup entries for callers that still use the legacy model.
pub fn load_cleanup_history() -> DustResult<Vec<CleanupHistoryEntry>> {
    Ok(history::load_all()?
        .into_iter()
        .filter_map(cleanup_entry)
        .collect())
}

fn cleanup_entry(activity: ActivityRecord) -> Option<CleanupHistoryEntry> {
    if activity.kind != crate::models::ActivityKind::Cleanup {
        return None;
    }

    let details = activity.result.details;
    let mode = match details.get("mode")?.as_str()? {
        "trash" => DeleteMode::Trash,
        "permanent" => DeleteMode::Permanent,
        _ => return None,
    };
    let deleted_paths = details
        .get("deleted")?
        .as_array()?
        .iter()
        .filter_map(|path| path.as_str().map(Into::into))
        .collect();
    let failed_paths = details
        .get("failed")?
        .as_array()?
        .iter()
        .filter_map(|failure| {
            failure
                .get("path")
                .and_then(|path| path.as_str())
                .map(Into::into)
        })
        .collect();
    let freed_size_bytes = details.get("freed")?.as_u64()?;

    Some(CleanupHistoryEntry {
        executed_at: activity.timestamp,
        mode,
        freed_size_bytes,
        deleted_paths,
        failed_paths,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::models::{ActivityKind, ActivityResult};

    #[test]
    fn cleanup_history_projection_preserves_cleanup_details() {
        let activity = ActivityRecord {
            id: "activity-1".to_owned(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-02T03:04:05Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            kind: ActivityKind::Cleanup,
            result: ActivityResult::new(
                false,
                json!({
                    "mode": "permanent",
                    "deleted": ["target"],
                    "failed": [{"path": "node_modules", "reason": "NotFound"}],
                    "freed": 2048
                }),
            ),
        };

        let entry = cleanup_entry(activity).unwrap();

        assert_eq!(entry.mode, DeleteMode::Permanent);
        assert_eq!(entry.freed_size_bytes, 2048);
        assert_eq!(
            entry.deleted_paths,
            vec![std::path::PathBuf::from("target")]
        );
        assert_eq!(
            entry.failed_paths,
            vec![std::path::PathBuf::from("node_modules")]
        );
    }

    #[test]
    fn cleanup_history_projection_ignores_non_cleanup_or_invalid_records() {
        let scan = ActivityRecord {
            id: "scan-1".to_owned(),
            timestamp: chrono::Utc::now(),
            kind: ActivityKind::Scan,
            result: ActivityResult::new(true, json!({})),
        };
        let invalid_cleanup = ActivityRecord {
            id: "cleanup-1".to_owned(),
            timestamp: chrono::Utc::now(),
            kind: ActivityKind::Cleanup,
            result: ActivityResult::new(true, json!({"mode": "unknown"})),
        };

        assert!(cleanup_entry(scan).is_none());
        assert!(cleanup_entry(invalid_cleanup).is_none());
    }
}
