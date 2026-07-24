mod history;

use std::{
    env,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use dustfril_core::{
    api,
    models::{
        CleanupCandidate, CleanupFailureReason, CleanupPlan, DeleteMode, Ecosystem,
        LifecycleScript, RiskLevel, ScriptType,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunOptions {
    root: Option<String>,
    ecosystems: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteCleanupRequest {
    candidates: Vec<CleanupCandidateInput>,
    mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CleanupCandidateInput {
    path: String,
    ecosystem: String,
    size_bytes: u64,
    age_days: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResponse {
    artifacts: Vec<ArtifactDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactDto {
    path: String,
    ecosystem: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisResponse {
    artifacts: Vec<ArtifactAnalysisDto>,
    total_size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactAnalysisDto {
    path: String,
    ecosystem: String,
    size_bytes: u64,
    last_modified_ms: Option<u64>,
    age_days: Option<u64>,
    recommendation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupPlanResponse {
    candidates: Vec<CleanupCandidateDto>,
    reclaimable_size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupCandidateDto {
    path: String,
    ecosystem: String,
    size_bytes: u64,
    age_days: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupResultResponse {
    deleted_paths: Vec<String>,
    failed_paths: Vec<CleanupFailureDto>,
    freed_size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupFailureDto {
    path: String,
    reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LifecycleScriptDto {
    package: String,
    package_manager: String,
    script_type: String,
    command: String,
    risk_level: String,
}

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

fn parse_ecosystems(values: &[String]) -> Result<Vec<Ecosystem>, String> {
    values
        .iter()
        .map(|value| match value.as_str() {
            "Rust" => Ok(Ecosystem::Rust),
            "Node" => Ok(Ecosystem::Node),
            "Java" => Ok(Ecosystem::Java),
            _ => Err(format!("Unsupported ecosystem: {value}")),
        })
        .collect()
}

fn parse_delete_mode(value: &str) -> Result<DeleteMode, String> {
    match value {
        "Trash" => Ok(DeleteMode::Trash),
        "Permanent" => Ok(DeleteMode::Permanent),
        _ => Err(format!("Unsupported delete mode: {value}")),
    }
}

fn system_time_to_ms(value: std::time::SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

fn cleanup_failure_reason_to_string(reason: &CleanupFailureReason) -> String {
    match reason {
        CleanupFailureReason::PermissionDenied => "PermissionDenied".to_string(),
        CleanupFailureReason::NotFound => "NotFound".to_string(),
        CleanupFailureReason::UnsafePath => "UnsafePath".to_string(),
        CleanupFailureReason::SymbolicLink => "SymbolicLink".to_string(),
        CleanupFailureReason::Other(message) => message.clone(),
    }
}

fn script_type_to_string(script_type: ScriptType) -> String {
    script_type.to_string()
}

fn risk_level_to_string(risk_level: RiskLevel) -> String {
    risk_level.to_string()
}

fn artifact_path(path: &Path) -> String {
    path.display().to_string()
}

#[tauri::command]
fn default_root() -> Result<String, String> {
    default_root_path().map(|path| path.display().to_string())
}

#[tauri::command]
fn scan(options: RunOptions) -> Result<ScanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;

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
}

#[tauri::command]
fn analyze(options: RunOptions) -> Result<AnalysisResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
    let analysis = api::analyze(scan_result).map_err(|error| error.to_string())?;

    Ok(AnalysisResponse {
        total_size_bytes: analysis.total_size_bytes,
        artifacts: analysis
            .artifacts
            .into_iter()
            .map(|artifact| ArtifactAnalysisDto {
                path: artifact_path(&artifact.artifact.path),
                ecosystem: artifact.artifact.ecosystem.to_string(),
                size_bytes: artifact.size_bytes,
                last_modified_ms: artifact.last_modified.and_then(system_time_to_ms),
                age_days: artifact.age_days,
                recommendation: artifact.recommendation.to_string(),
            })
            .collect(),
    })
}

#[tauri::command]
fn build_cleanup_plan(options: RunOptions) -> Result<CleanupPlanResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
    let plan = api::clean::build_plan(scan_result).map_err(|error| error.to_string())?;

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
}

#[tauri::command]
fn execute_cleanup(request: ExecuteCleanupRequest) -> Result<CleanupResultResponse, String> {
    let mode = parse_delete_mode(&request.mode)?;
    let candidates = request
        .candidates
        .into_iter()
        .map(|candidate| {
            Ok(CleanupCandidate {
                path: PathBuf::from(candidate.path),
                ecosystem: parse_ecosystems(&[candidate.ecosystem])?
                    .into_iter()
                    .next()
                    .ok_or_else(|| "Missing ecosystem".to_string())?,
                size_bytes: candidate.size_bytes,
                age_days: candidate.age_days,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
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
                reason: cleanup_failure_reason_to_string(&failure.reason),
            })
            .collect(),
        freed_size_bytes: result.freed_size_bytes,
    })
}

#[tauri::command]
fn load_cleanup_history() -> Result<Vec<history::CleanupHistoryEntryDto>, String> {
    history::load_entries().map_err(|error| error.to_string())
}

#[tauri::command]
fn audit(options: RunOptions) -> Result<Vec<LifecycleScriptDto>, String> {
    let root = resolve_root(options.root)?;
    let ecosystems = parse_ecosystems(&options.ecosystems)?;
    let result = api::audit(&root, &ecosystems).map_err(|error| error.to_string())?;

    Ok(result.into_iter().map(lifecycle_script_to_dto).collect())
}

fn lifecycle_script_to_dto(script: LifecycleScript) -> LifecycleScriptDto {
    LifecycleScriptDto {
        package: script.package,
        package_manager: script.package_manager.to_string(),
        script_type: script_type_to_string(script.script_type),
        command: script.command,
        risk_level: risk_level_to_string(script.risk_level),
    }
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
            load_cleanup_history,
            audit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
