use std::path::PathBuf;

#[derive(Debug)]
pub struct CleanupResult {
    pub deleted_paths: Vec<PathBuf>,

    pub failed_paths: Vec<PathBuf>,

    pub freed_size_bytes: u64,
}
