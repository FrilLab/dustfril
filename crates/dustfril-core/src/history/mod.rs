use std::{
    fs, io,
    path::{Path, PathBuf},
};

use chrono::Utc;
use directories::ProjectDirs;

use crate::{
    error::{DustError, DustResult},
    models::{CleanupHistoryEntry, CleanupResult, DeleteMode},
};

pub fn record(mode: DeleteMode, result: &CleanupResult) -> DustResult<()> {
    let path = history_path()?;

    record_to(&path, mode, result)
}

pub fn load_all() -> DustResult<Vec<CleanupHistoryEntry>> {
    let path = history_path()?;

    load(&path)
}

pub fn history_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;

    let data_dir = dirs.data_dir();

    fs::create_dir_all(data_dir)?;

    Ok(data_dir.join("history.json"))
}

fn record_to(path: &Path, mode: DeleteMode, result: &CleanupResult) -> DustResult<()> {
    let mut histories = load(path)?;

    histories.push(CleanupHistoryEntry {
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

fn load(path: &Path) -> DustResult<Vec<CleanupHistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(path)?;

    serde_json::from_str(&json).map_err(|error| DustError::Io(io::Error::other(error)))
}

fn save(path: &Path, histories: &[CleanupHistoryEntry]) -> DustResult<()> {
    let json = serde_json::to_string_pretty(histories)
        .map_err(|error| DustError::Io(io::Error::other(error)))?;

    fs::write(path, json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::models::{CleanupFailure, CleanupFailureReason};

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
        assert_eq!(history[0].deleted_paths.len(), 1);
        assert_eq!(history[0].failed_paths.len(), 1);
    }
}
