use crate::{
    developer_cache,
    error::DustResult,
    models::{CleanupResult, DeleteMode, DeveloperCacheDiscovery, DeveloperCacheSelection},
};

/// Discovers supported developer caches from local configuration and defaults.
pub fn discover() -> DustResult<DeveloperCacheDiscovery> {
    developer_cache::discover()
}

/// Revalidates and cleans explicitly selected cache roots.
pub fn cleanup(
    selections: &[DeveloperCacheSelection],
    mode: DeleteMode,
) -> DustResult<CleanupResult> {
    developer_cache::cleanup(selections, mode)
}
