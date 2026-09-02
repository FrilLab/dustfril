mod contract;
mod history;

use std::{
    env,
    fmt::Display,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use contract::{
    artifact_path, artifact_snapshot_to_dto, cleanup_failure_reason, AnalysisResponse,
    ArtifactAnalysisDto, ArtifactDto, CleanupCandidateDto, CleanupFailureDto,
    CleanupHistoryEntryDto, CleanupPlanResponse, CleanupResultResponse, ExecuteCleanupRequest,
    LifecycleScriptDto, RunOptions, ScanResponse, SecurityScanResponse, WorkspaceAnalysisResponse,
};
use dustfril_core::{
    api,
    error::DustError,
    models::{CleanupCandidate, CleanupPlan},
};

fn resolve_root(root: Option<String>) -> Result<PathBuf, String> {
    match root.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => {
            let path = PathBuf::from(value);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
                    "Path must not be a symbolic link: {}",
                    path.display()
                )),
                Ok(metadata) if metadata.is_dir() => Ok(path),
                Ok(_) => Err(format!("Path is not a directory: {}", path.display())),
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    Err(format!("Path does not exist: {}", path.display()))
                }
                Err(error) => Err(format!("Cannot access path {}: {error}", path.display())),
            }
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

fn format_history_warning(operation: &str, error: impl Display) -> String {
    let warning = format!("Failed to record {operation} activity history: {error}");
    eprintln!("{warning}");
    warning
}

fn report_failed_activity_record(operation: &str, error: impl Display) {
    let _ = format_history_warning(operation, error);
}

fn record_failed_scan_activity(root: &Path, error: &DustError) {
    let result = match error.scan_access_summary() {
        Some(summary) => {
            history::record_scan_failure_with_summary(root, &error.to_string(), summary)
        }
        None => history::record_scan_failure(root, &error.to_string()),
    };

    if let Err(history_error) = result {
        report_failed_activity_record("scan", history_error);
    }
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
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let result = match api::scan(&root, &ecosystems) {
            Ok(result) => result,
            Err(error) => {
                record_failed_scan_activity(&root, &error);
                return Err(error.to_string());
            }
        };
        let mut history_warning = None;
        let mut artifact_snapshot = None;
        let mut artifact_snapshot_warning = None;
        let total_size_bytes = match api::analyze(result.clone()) {
            Ok(analysis) => {
                match api::artifact_snapshot::record_artifact_snapshot_with_ecosystems(
                    &root,
                    &analysis,
                    &ecosystems,
                ) {
                    Ok(snapshot) => artifact_snapshot = Some(artifact_snapshot_to_dto(snapshot)),
                    Err(error) => {
                        let warning = format!("Failed to record artifact snapshot: {error}");
                        eprintln!("{warning}");
                        artifact_snapshot_warning = Some(warning);
                    }
                }
                Some(analysis.total_size_bytes)
            }
            Err(error) => {
                let warning = format!(
                    "Failed to calculate scan size; scan activity history was not recorded: {error}"
                );
                eprintln!("{warning}");
                history_warning = Some(warning);
                None
            }
        };

        if let Some(total_size_bytes) = total_size_bytes {
            if let Err(error) = history::record_scan(&root, &result, total_size_bytes) {
                let warning = format_history_warning("scan", error);
                history_warning = Some(match history_warning {
                    Some(existing) => format!("{existing} {warning}"),
                    None => warning,
                });
            }
        }

        Ok(ScanResponse {
            artifacts: result
                .artifacts
                .into_iter()
                .map(|artifact| ArtifactDto {
                    path: artifact_path(&artifact.path),
                    ecosystem: artifact.ecosystem.into(),
                })
                .collect(),
            history_warning,
            artifact_snapshot,
            artifact_snapshot_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn analyze(options: RunOptions) -> Result<AnalysisResponse, String> {
    let record_history = options.record_history.unwrap_or(false);
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let scan_result = match api::scan(&root, &ecosystems) {
            Ok(result) => result,
            Err(error) => {
                if record_history {
                    record_failed_scan_activity(&root, &error);
                }
                return Err(error.to_string());
            }
        };
        let analysis = api::analyze(scan_result.clone()).map_err(|error| error.to_string())?;
        let history_warning = if record_history {
            match history::record_scan(&root, &scan_result, analysis.total_size_bytes) {
                Ok(()) => None,
                Err(error) => Some(format_history_warning("scan", error)),
            }
        } else {
            None
        };

        Ok(AnalysisResponse {
            total_size_bytes: analysis.total_size_bytes,
            history_warning,
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

/// Runs the complete user-facing Workspace analysis workflow from one scan.
/// The cleanup plan is derived from that analysis rather than scanning the
/// selected folder again.
#[tauri::command]
async fn analyze_workspace(options: RunOptions) -> Result<WorkspaceAnalysisResponse, String> {
    let record_history = options.record_history.unwrap_or(true);
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let scan_result = match api::scan(&root, &ecosystems) {
            Ok(result) => result,
            Err(error) => {
                if record_history {
                    record_failed_scan_activity(&root, &error);
                }
                return Err(error.to_string());
            }
        };
        let analysis = api::analyze(scan_result.clone()).map_err(|error| error.to_string())?;
        let plan = api::clean::build_plan_from_analysis(analysis.clone())
            .map_err(|error| error.to_string())?;

        let mut artifact_snapshot = None;
        let mut artifact_snapshot_warning = None;
        match api::artifact_snapshot::record_artifact_snapshot_with_ecosystems(
            &root,
            &analysis,
            &ecosystems,
        ) {
            Ok(snapshot) => artifact_snapshot = Some(artifact_snapshot_to_dto(snapshot)),
            Err(error) => {
                let warning = format!("Failed to record artifact snapshot: {error}");
                eprintln!("{warning}");
                artifact_snapshot_warning = Some(warning);
            }
        }

        let history_warning = if record_history {
            match history::record_scan(&root, &scan_result, analysis.total_size_bytes) {
                Ok(()) => None,
                Err(error) => Some(format_history_warning("scan", error)),
            }
        } else {
            None
        };

        let analysis_response = AnalysisResponse {
            total_size_bytes: analysis.total_size_bytes,
            history_warning,
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
        };
        let cleanup_plan = CleanupPlanResponse {
            reclaimable_size_bytes: plan.reclaimable_size_bytes(),
            candidates: plan
                .candidates
                .into_iter()
                .map(|candidate| CleanupCandidateDto {
                    path: artifact_path(&candidate.path),
                    ecosystem: candidate.ecosystem.into(),
                    size_bytes: candidate.size_bytes,
                    age_days: candidate.age_days,
                })
                .collect(),
        };

        Ok(WorkspaceAnalysisResponse {
            analysis: analysis_response,
            cleanup_plan,
            artifact_snapshot,
            artifact_snapshot_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn build_cleanup_plan(options: RunOptions) -> Result<CleanupPlanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
        let analysis = api::analyze(scan_result).map_err(|error| error.to_string())?;
        let plan =
            api::clean::build_plan_from_analysis(analysis).map_err(|error| error.to_string())?;

        Ok(CleanupPlanResponse {
            reclaimable_size_bytes: plan.reclaimable_size_bytes(),
            candidates: plan
                .candidates
                .into_iter()
                .map(|candidate| CleanupCandidateDto {
                    path: artifact_path(&candidate.path),
                    ecosystem: candidate.ecosystem.into(),
                    size_bytes: candidate.size_bytes,
                    age_days: candidate.age_days,
                })
                .collect(),
        })
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
        let result = match api::clean::execute(&plan, mode) {
            Ok(result) => result,
            Err(error) => {
                if let Err(history_error) =
                    history::record_cleanup_failure(mode, &error.to_string())
                {
                    report_failed_activity_record("cleanup", history_error);
                }
                return Err(error.to_string());
            }
        };
        let history_warning = match history::record(mode, &result) {
            Ok(()) => None,
            Err(error) => Some(format_history_warning("cleanup", error)),
        };

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
            history_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn load_activity_history() -> Result<Vec<history::ActivityRecordDto>, String> {
    tokio::task::spawn_blocking(|| history::load_entries().map_err(|error| error.to_string()))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn load_cleanup_history() -> Result<Vec<CleanupHistoryEntryDto>, String> {
    tokio::task::spawn_blocking(|| {
        history::load_cleanup_entries().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn audit(options: RunOptions) -> Result<Vec<LifecycleScriptDto>, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let result = api::audit(&root, &ecosystems).map_err(|error| error.to_string())?;

        Ok(result.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn security_scan(options: RunOptions) -> Result<SecurityScanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let report = match api::security_scan_report(&root, &ecosystems) {
            Ok(report) => report,
            Err(error) => {
                if let Err(history_error) =
                    history::record_security_failure(&root, &ecosystems, &error.to_string())
                {
                    eprintln!("Failed to record security scan history: {history_error}");
                }
                return Err(error.to_string());
            }
        };

        let mut response: SecurityScanResponse = report.clone().into();
        response.history_warning = match history::record_security_scan(&root, &ecosystems, &report)
        {
            Ok(()) => None,
            Err(error) => Some(format_history_warning("security scan", error)),
        };

        Ok(response)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            default_root,
            scan,
            analyze,
            analyze_workspace,
            build_cleanup_plan,
            execute_cleanup,
            load_activity_history,
            load_cleanup_history,
            audit,
            security_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
