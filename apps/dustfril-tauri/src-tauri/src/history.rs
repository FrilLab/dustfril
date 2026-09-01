use std::path::Path;

use dustfril_core::{
    api,
    models::{ActivityKind, ActivityRecord, CleanupResult, DeleteMode, ScanResult},
};
use serde::Serialize;

use crate::contract::CleanupHistoryEntryDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecordDto {
    pub id: String,
    pub timestamp_ms: u64,
    pub kind: String,
    pub result: dustfril_core::models::ActivityResult,
}

/// Records a cleanup operation for the desktop activity log.
pub fn record(mode: DeleteMode, result: &CleanupResult) -> Result<(), String> {
    api::history::record_cleanup(mode, result).map_err(|error| error.to_string())
}

/// Records a completed scan for the desktop activity log.
pub fn record_scan(
    target_path: &Path,
    result: &ScanResult,
    total_size_bytes: u64,
) -> Result<(), String> {
    api::history::record_scan(target_path, result, total_size_bytes)
        .map_err(|error| error.to_string())
}

pub fn load_entries() -> Result<Vec<ActivityRecordDto>, String> {
    api::history::load_all()
        .map_err(|error| error.to_string())?
        .into_iter()
        .rev()
        .map(activity_record_to_dto)
        .collect()
}

pub fn load_cleanup_entries() -> Result<Vec<CleanupHistoryEntryDto>, String> {
    api::history::load_cleanup_history()
        .map_err(|error| error.to_string())?
        .into_iter()
        .rev()
        .map(cleanup_entry_to_dto)
        .collect()
}

fn activity_record_to_dto(entry: ActivityRecord) -> Result<ActivityRecordDto, String> {
    let timestamp_ms = entry
        .timestamp
        .timestamp_millis()
        .try_into()
        .map_err(|error| format!("Invalid timestamp: {error}"))?;

    Ok(ActivityRecordDto {
        id: entry.id,
        timestamp_ms,
        kind: activity_kind_to_string(entry.kind),
        result: entry.result,
    })
}

fn activity_kind_to_string(kind: ActivityKind) -> String {
    match kind {
        ActivityKind::Scan => "Scan".to_string(),
        ActivityKind::Cleanup => "Cleanup".to_string(),
        ActivityKind::Security => "Security".to_string(),
    }
}

fn cleanup_entry_to_dto(
    entry: dustfril_core::models::CleanupHistoryEntry,
) -> Result<CleanupHistoryEntryDto, String> {
    let executed_at_ms = entry
        .executed_at
        .timestamp_millis()
        .try_into()
        .map_err(|error| format!("Invalid timestamp: {error}"))?;

    Ok(CleanupHistoryEntryDto {
        executed_at_ms,
        mode: entry.mode.into(),
        freed_size_bytes: entry.freed_size_bytes,
        deleted_paths: entry
            .deleted_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        failed_paths: entry
            .failed_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
    })
}
