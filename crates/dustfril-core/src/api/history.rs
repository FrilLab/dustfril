use std::path::Path;

use crate::{
    error::DustResult,
    history,
    models::{ActivityRecord, CleanupHistoryEntry, CleanupResult, DeleteMode, ScanResult},
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

/// Records a scan operation and its detected artifact summary.
pub fn record_scan(
    target_path: &Path,
    result: &ScanResult,
    total_size_bytes: u64,
) -> DustResult<()> {
    history::record_scan(target_path, result, total_size_bytes)
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
