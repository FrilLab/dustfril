use dustfril_core::api;

pub fn record(
    mode: dustfril_core::models::DeleteMode,
    result: &dustfril_core::models::CleanupResult,
) -> std::io::Result<()> {
    api::history::record(mode, result).map_err(|error| std::io::Error::other(error.to_string()))
}
