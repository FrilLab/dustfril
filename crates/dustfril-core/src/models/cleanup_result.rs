use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_paths: Vec<PathBuf>,

    pub failed_paths: Vec<PathBuf>,

    pub freed_size_bytes: u64,
}
