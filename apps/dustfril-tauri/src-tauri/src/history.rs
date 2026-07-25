use dustfril_core::{
    api,
    models::{CleanupResult, DeleteMode},
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupHistoryEntryDto {
    pub executed_at_ms: u64,
    pub mode: String,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<String>,
    pub failed_paths: Vec<String>,
}

pub fn record(mode: DeleteMode, result: &CleanupResult) -> Result<(), String> {
    api::history::record(mode, result).map_err(|error| error.to_string())
}

pub fn load_entries() -> Result<Vec<CleanupHistoryEntryDto>, String> {
    api::history::load_all()
        .map_err(|error| error.to_string())?
        .into_iter()
        .rev()
        .map(history_entry_to_dto)
        .collect()
}

fn history_entry_to_dto(
    entry: dustfril_core::models::CleanupHistoryEntry,
) -> Result<CleanupHistoryEntryDto, String> {
    let executed_at_ms = entry
        .executed_at
        .timestamp_millis()
        .try_into()
        .map_err(|error| format!("Invalid timestamp: {error}"))?;

    Ok(CleanupHistoryEntryDto {
        executed_at_ms,
        mode: delete_mode_to_string(entry.mode),
        freed_size_bytes: entry.freed_size_bytes,
        deleted_paths: entry
            .deleted_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        failed_paths: entry
            .failed_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
    })
}

fn delete_mode_to_string(mode: DeleteMode) -> String {
    match mode {
        DeleteMode::Trash => "Trash".to_string(),
        DeleteMode::Permanent => "Permanent".to_string(),
    }
}
