use dustfril_core::api;

pub fn record(
    mode: dustfril_core::models::DeleteMode,
    result: &dustfril_core::models::CleanupResult,
) -> std::io::Result<()> {
    api::history::record(mode, result).map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_scan_failure(target_path: &std::path::Path, reason: &str) -> std::io::Result<()> {
    api::history::record_scan_failure(target_path, reason)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_cleanup_failure(
    mode: dustfril_core::models::DeleteMode,
    reason: &str,
) -> std::io::Result<()> {
    api::history::record_cleanup_failure(mode, reason)
        .map_err(|error| std::io::Error::other(error.to_string()))
}
