use dustfril_core::api;

pub fn record_scan_failure(target_path: &std::path::Path, reason: &str) -> std::io::Result<()> {
    api::history::record_scan_failure(target_path, reason)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_for_workspace(
    target_path: &std::path::Path,
    plan: &dustfril_core::models::CleanupPlan,
    mode: dustfril_core::models::DeleteMode,
    result: &dustfril_core::models::CleanupResult,
) -> std::io::Result<()> {
    api::history::record_cleanup_with_context(target_path, &plan.candidates, mode, result)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_failure_for_workspace(
    target_path: &std::path::Path,
    mode: dustfril_core::models::DeleteMode,
    reason: &str,
) -> std::io::Result<()> {
    api::history::record_cleanup_failure_with_context(target_path, mode, reason)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_scan_failure_with_summary(
    target_path: &std::path::Path,
    reason: &str,
    access_summary: &dustfril_core::models::ScanAccessSummary,
) -> std::io::Result<()> {
    api::history::record_scan_failure_with_summary(target_path, reason, access_summary)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_cleanup_failure(
    mode: dustfril_core::models::DeleteMode,
    reason: &str,
) -> std::io::Result<()> {
    api::history::record_cleanup_failure(mode, reason)
        .map_err(|error| std::io::Error::other(error.to_string()))
}
