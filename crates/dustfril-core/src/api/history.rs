use crate::{error::DustResult, history, models::CleanupHistoryEntry};

pub fn record(
    mode: crate::models::DeleteMode,
    result: &crate::models::CleanupResult,
) -> DustResult<()> {
    history::record(mode, result)
}

pub fn load_all() -> DustResult<Vec<CleanupHistoryEntry>> {
    history::load_all()
}
