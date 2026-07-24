use std::{
    fs, io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use dustfril_core::models::{CleanupResult, DeleteMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupHistoryRecord {
    pub executed_at: DateTime<Utc>,
    pub mode: DeleteMode,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<PathBuf>,
    pub failed_paths: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupHistoryEntryDto {
    pub executed_at_ms: u64,
    pub mode: String,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<String>,
    pub failed_paths: Vec<String>,
}

pub fn record(mode: DeleteMode, result: &CleanupResult) -> io::Result<()> {
    let path = history_path()?;

    record_to(&path, mode, result)
}

pub fn load_entries() -> io::Result<Vec<CleanupHistoryEntryDto>> {
    let path = history_path()?;

    load(&path)?
        .into_iter()
        .rev()
        .map(history_record_to_dto)
        .collect()
}

fn record_to(path: &Path, mode: DeleteMode, result: &CleanupResult) -> io::Result<()> {
    let mut histories = load(path)?;

    histories.push(CleanupHistoryRecord {
        executed_at: Utc::now(),
        mode,
        freed_size_bytes: result.freed_size_bytes,
        deleted_paths: result.deleted_paths.clone(),
        failed_paths: result
            .failed_paths
            .iter()
            .map(|failure| failure.path.clone())
            .collect(),
    });

    save(path, &histories)
}

fn history_record_to_dto(record: CleanupHistoryRecord) -> io::Result<CleanupHistoryEntryDto> {
    let executed_at_ms = record
        .executed_at
        .timestamp_millis()
        .try_into()
        .map_err(|error| io::Error::other(format!("Invalid timestamp: {error}")))?;

    Ok(CleanupHistoryEntryDto {
        executed_at_ms,
        mode: delete_mode_to_string(record.mode),
        freed_size_bytes: record.freed_size_bytes,
        deleted_paths: record
            .deleted_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        failed_paths: record
            .failed_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
    })
}

fn delete_mode_to_string(mode: DeleteMode) -> String {
    match mode {
        DeleteMode::Trash => "Trash".to_string(),
        DeleteMode::Permanent => "Permanent".to_string(),
    }
}

fn history_path() -> io::Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;

    let data_dir = dirs.data_dir();

    fs::create_dir_all(data_dir)?;

    Ok(data_dir.join("history.json"))
}

fn load(path: &Path) -> io::Result<Vec<CleanupHistoryRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(path)?;

    serde_json::from_str(&json).map_err(io::Error::other)
}

fn save(path: &Path, histories: &[CleanupHistoryRecord]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(histories).map_err(io::Error::other)?;

    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use dustfril_core::models::{CleanupFailure, CleanupFailureReason, CleanupResult};

    #[test]
    fn record_appends_history() {
        let temp = TempDir::new().unwrap();

        let result = CleanupResult {
            deleted_paths: vec!["target".into()],
            failed_paths: vec![CleanupFailure {
                path: "node_modules".into(),
                reason: CleanupFailureReason::NotFound,
            }],
            freed_size_bytes: 1024,
        };

        let path = temp.path().join("history.json");

        record_to(&path, DeleteMode::Trash, &result).unwrap();

        let history = load(&path).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].mode, DeleteMode::Trash);
        assert_eq!(history[0].freed_size_bytes, 1024);
    }
}
