use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::DeleteMode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupHistoryEntry {
    pub executed_at: DateTime<Utc>,
    pub mode: DeleteMode,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<PathBuf>,
    pub failed_paths: Vec<PathBuf>,
}
