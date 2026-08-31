use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::models::{CleanupResult, DeleteMode, ScanResult};

/// The kind of operation represented by an activity record.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    Scan,
    Cleanup,
    Security,
}

/// Compatibility alias for clients that use the issue's original terminology.
pub type ActivityType = ActivityKind;

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Cleanup => "cleanup",
            Self::Security => "security",
        }
    }
}

/// The result of an activity, with extensible JSON details for future event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityResult {
    pub success: bool,
    pub details: Value,
}

impl ActivityResult {
    pub fn new(success: bool, details: Value) -> Self {
        Self { success, details }
    }

    /// Builds the result payload for a completed scan.
    pub fn from_scan(
        target_path: &std::path::Path,
        scan: &ScanResult,
        total_size_bytes: u64,
    ) -> Self {
        Self::new(
            true,
            json!({
                "path": target_path.display().to_string(),
                "artifacts": scan.artifacts.len(),
                "size": total_size_bytes,
            }),
        )
    }

    /// Builds the result payload for a cleanup attempt, including partial failures.
    pub fn from_cleanup(mode: DeleteMode, result: &CleanupResult) -> Self {
        let failed_paths: Vec<Value> = result
            .failed_paths
            .iter()
            .map(|failure| {
                json!({
                    "path": failure.path.display().to_string(),
                    "reason": failure.reason.to_string(),
                })
            })
            .collect();

        Self::new(
            result.failed_paths.is_empty(),
            json!({
                "mode": delete_mode_label(mode),
                "deleted": paths_to_values(&result.deleted_paths),
                "failed": failed_paths,
                "freed": result.freed_size_bytes,
            }),
        )
    }
}

/// A single operation in the local DustFril activity log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: ActivityKind,
    pub result: ActivityResult,
}

impl ActivityRecord {
    pub fn new(kind: ActivityKind, result: ActivityResult) -> Self {
        Self {
            id: next_activity_id(),
            timestamp: Utc::now(),
            kind,
            result,
        }
    }

    pub fn scan(target_path: &std::path::Path, scan: &ScanResult, total_size_bytes: u64) -> Self {
        Self::new(
            ActivityKind::Scan,
            ActivityResult::from_scan(target_path, scan, total_size_bytes),
        )
    }

    pub fn cleanup(mode: DeleteMode, result: &CleanupResult) -> Self {
        Self::new(
            ActivityKind::Cleanup,
            ActivityResult::from_cleanup(mode, result),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupHistoryEntry {
    pub executed_at: DateTime<Utc>,
    pub mode: DeleteMode,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<PathBuf>,
    pub failed_paths: Vec<PathBuf>,
}

impl CleanupHistoryEntry {
    /// Converts a legacy cleanup entry to the unified activity representation.
    pub fn into_activity_record(self, id: String) -> ActivityRecord {
        let failed = self
            .failed_paths
            .iter()
            .map(|path| {
                json!({
                    "path": path.display().to_string(),
                })
            })
            .collect::<Vec<_>>();

        ActivityRecord {
            id,
            timestamp: self.executed_at,
            kind: ActivityKind::Cleanup,
            result: ActivityResult::new(
                self.failed_paths.is_empty(),
                json!({
                    "mode": delete_mode_label(self.mode),
                    "deleted": paths_to_values(&self.deleted_paths),
                    "failed": failed,
                    "freed": self.freed_size_bytes,
                }),
            ),
        }
    }
}

fn delete_mode_label(mode: DeleteMode) -> &'static str {
    match mode {
        DeleteMode::Trash => "trash",
        DeleteMode::Permanent => "permanent",
    }
}

fn paths_to_values(paths: &[PathBuf]) -> Vec<Value> {
    paths
        .iter()
        .map(|path| Value::String(path.display().to_string()))
        .collect()
}

fn next_activity_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    format!(
        "activity-{}-{sequence}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}
