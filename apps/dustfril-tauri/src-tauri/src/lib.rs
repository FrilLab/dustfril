mod contract;
mod history;

use std::{
    env,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use contract::{
    artifact_path, cleanup_failure_reason, AnalysisResponse, ArtifactAnalysisDto, ArtifactDto,
    CleanupCandidateDto, CleanupFailureDto, CleanupHistoryEntryDto, CleanupPlanResponse,
    CleanupResultResponse, ExecuteCleanupRequest, LifecycleScriptDto, RunOptions, ScanResponse,
};
use dustfril_core::{
    api,
    models::{CleanupCandidate, CleanupPlan},
};

fn resolve_root(root: Option<String>) -> Result<PathBuf, String> {
    match root.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => {
            let path = PathBuf::from(value);
            if !path.exists() {
                return Err(format!("Path does not exist: {}", path.display()));
            }
            Ok(path)
        }
        _ => default_root_path(),
    }
}

fn discover_workspace_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join(".git").exists())
        .map(Path::to_path_buf)
}

fn default_root_path() -> Result<PathBuf, String> {
    let current_dir = env::current_dir().map_err(|error| error.to_string())?;
    Ok(discover_workspace_root(&current_dir).unwrap_or(current_dir))
}

fn system_time_to_ms(value: std::time::SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

#[tauri::command]
async fn default_root() -> Result<String, String> {
    tokio::task::spawn_blocking(|| default_root_path().map(|path| path.display().to_string()))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn scan(options: RunOptions) -> Result<ScanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
    let total_size_bytes = api::analyze(result.clone())
        .map_err(|error| error.to_string())?
        .total_size_bytes;
    if let Err(error) = history::record_scan(&root, &result, total_size_bytes) {
        eprintln!("Failed to record scan history: {error}");
    }

    Ok(ScanResponse {
        artifacts: result
            .artifacts
            .into_iter()
            .map(|artifact| ArtifactDto {
                path: artifact_path(&artifact.path),
                ecosystem: artifact.ecosystem.to_string(),
            })
            .collect(),
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn analyze(options: RunOptions) -> Result<AnalysisResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
        let analysis = api::analyze(scan_result).map_err(|error| error.to_string())?;

        Ok(AnalysisResponse {
            total_size_bytes: analysis.total_size_bytes,
            artifacts: analysis
                .artifacts
                .into_iter()
                .map(|artifact| ArtifactAnalysisDto {
                    path: artifact_path(&artifact.artifact.path),
                    ecosystem: artifact.artifact.ecosystem.into(),
                    size_bytes: artifact.size_bytes,
                    last_modified_ms: artifact.last_modified.and_then(system_time_to_ms),
                    age_days: artifact.age_days,
                    recommendation: artifact.recommendation.into(),
                })
                .collect(),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn build_cleanup_plan(options: RunOptions) -> Result<CleanupPlanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
    let analysis = api::analyze(scan_result.clone()).map_err(|error| error.to_string())?;
    let plan = api::clean::build_plan_from_analysis(analysis).map_err(|error| error.to_string())?;

    Ok(CleanupPlanResponse {
        reclaimable_size_bytes: plan.reclaimable_size_bytes(),
        candidates: plan
            .candidates
            .into_iter()
            .map(|candidate| CleanupCandidateDto {
                path: artifact_path(&candidate.path),
                ecosystem: candidate.ecosystem.to_string(),
                size_bytes: candidate.size_bytes,
                age_days: candidate.age_days,
            })
            .collect(),
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn execute_cleanup(request: ExecuteCleanupRequest) -> Result<CleanupResultResponse, String> {
    let mode = request.mode.into();
    let candidates: Vec<_> = request
        .candidates
        .into_iter()
        .map(|candidate| CleanupCandidate {
            path: PathBuf::from(candidate.path),
            ecosystem: candidate.ecosystem.into(),
            size_bytes: candidate.size_bytes,
            age_days: candidate.age_days,
        })
        .collect();

    tokio::task::spawn_blocking(move || {
        let plan = CleanupPlan { candidates };
        let result = api::clean::execute(&plan, mode).map_err(|error| error.to_string())?;
        history::record(mode, &result).map_err(|error| error.to_string())?;

        Ok(CleanupResultResponse {
            deleted_paths: result
                .deleted_paths
                .into_iter()
                .map(|path| artifact_path(&path))
                .collect(),
            failed_paths: result
                .failed_paths
                .into_iter()
                .map(|failure| CleanupFailureDto {
                    path: artifact_path(&failure.path),
                    reason: cleanup_failure_reason(&failure.reason),
                })
                .collect(),
            freed_size_bytes: result.freed_size_bytes,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn load_activity_history() -> Result<Vec<history::ActivityRecordDto>, String> {
    history::load_entries().map_err(|error| error.to_string())
}

#[tauri::command]
fn load_cleanup_history() -> Result<Vec<history::CleanupHistoryEntryDto>, String> {
    history::load_cleanup_entries().map_err(|error| error.to_string())
}

#[tauri::command]
fn audit(options: RunOptions) -> Result<Vec<LifecycleScriptDto>, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let result = api::audit(&root, &ecosystems).map_err(|error| error.to_string())?;

        Ok(result.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            default_root,
            scan,
            analyze,
            build_cleanup_plan,
            execute_cleanup,
            load_activity_history,
            load_cleanup_history,
            audit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
