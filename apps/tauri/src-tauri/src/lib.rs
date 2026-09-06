mod contract;
mod history;

use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::UNIX_EPOCH,
};

use contract::{
    artifact_path, artifact_snapshot_to_dto, cleanup_failure_reason, dependency_inventory_to_dto,
    project_identity_to_dto, storage_summary_to_dto, volume_storage_to_dto, AnalysisResponse,
    ArtifactAnalysisDto, ArtifactDto, CleanupCandidateDto, CleanupFailureDto,
    CleanupHistoryEntryDto, CleanupPlanResponse, CleanupResultResponse,
    DependencyBaselineAcceptOptions, DependencyInventoryResponse, ExecuteCleanupRequest,
    LifecycleScriptDto, RunOptions, ScanResponse, SecurityScanResponse, StorageSummaryDto,
    VolumeStorageDto, WorkspaceAnalysisResponse,
};
use dustfril_core::{
    api,
    error::DustError,
    models::{AnalysisResult, ArtifactAnalysis, ArtifactSelection, DependencyReport, Ecosystem},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AnalysisCacheKey {
    root: PathBuf,
    ecosystems: Vec<Ecosystem>,
}

#[derive(Debug, Clone)]
struct CachedAnalysis {
    scope: AnalysisCacheKey,
    analysis: AnalysisResult,
}

impl AnalysisCacheKey {
    fn new(root: &Path, ecosystems: &[Ecosystem]) -> Self {
        let mut ecosystems = if ecosystems.is_empty() {
            vec![Ecosystem::Rust, Ecosystem::Node, Ecosystem::Java]
        } else {
            ecosystems.to_vec()
        };
        ecosystems.sort_unstable();
        ecosystems.dedup();

        Self {
            root: root.to_path_buf(),
            ecosystems,
        }
    }
}

static NEXT_ANALYSIS_ID: AtomicU64 = AtomicU64::new(1);
static ANALYSIS_CACHE: OnceLock<Mutex<HashMap<u64, CachedAnalysis>>> = OnceLock::new();

fn analysis_cache() -> &'static Mutex<HashMap<u64, CachedAnalysis>> {
    ANALYSIS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_analysis(root: &Path, ecosystems: &[Ecosystem], analysis: &AnalysisResult) -> String {
    let id = NEXT_ANALYSIS_ID.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut cache) = analysis_cache().lock() {
        cache.insert(
            id,
            CachedAnalysis {
                scope: AnalysisCacheKey::new(root, ecosystems),
                analysis: analysis.clone(),
            },
        );
    }
    id.to_string()
}

fn cached_analysis(
    analysis_id: &str,
    root: &Path,
    ecosystems: &[Ecosystem],
) -> Option<AnalysisResult> {
    let id = analysis_id.parse().ok()?;
    let cache = analysis_cache().lock().ok()?;
    let cached = cache.get(&id)?;

    (cached.scope == AnalysisCacheKey::new(root, ecosystems)).then(|| cached.analysis.clone())
}

fn cleanup_candidate_from_analysis(
    artifact: &ArtifactAnalysis,
    selected_by_default: bool,
) -> CleanupCandidateDto {
    CleanupCandidateDto {
        path: artifact_path(&artifact.artifact.path),
        ecosystem: artifact.artifact.ecosystem.into(),
        project: project_identity_to_dto(&artifact.artifact.project),
        size_bytes: artifact.size_bytes,
        age_days: artifact.age_days,
        recommendation: artifact.recommendation.into(),
        selected_by_default,
    }
}

fn cleanup_candidate_from_core_candidate(
    candidate: dustfril_core::models::CleanupCandidate,
) -> CleanupCandidateDto {
    CleanupCandidateDto {
        path: artifact_path(&candidate.path),
        ecosystem: candidate.ecosystem.into(),
        project: project_identity_to_dto(&candidate.project),
        size_bytes: candidate.size_bytes,
        age_days: candidate.age_days,
        recommendation: candidate.recommendation.into(),
        selected_by_default: candidate.recommendation.selected_by_default(),
    }
}

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

fn verify_dependency_inventory_fingerprint(
    reports: &[DependencyReport],
    expected: &str,
) -> Result<String, String> {
    let actual =
        api::dependency_inventory_fingerprint(reports).map_err(|error| error.to_string())?;
    if actual != expected {
        return Err(
            "Dependency inventory changed since it was reviewed. Compare again before accepting the baseline."
                .to_owned(),
        );
    }

    Ok(actual)
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

fn record_artifact_snapshot_if_enabled(
    enabled: bool,
    root: &Path,
    analysis: &AnalysisResult,
    ecosystems: &[Ecosystem],
) -> (Option<contract::ArtifactSnapshotResultDto>, Option<String>) {
    if !enabled {
        return (None, None);
    }

    match api::artifact_snapshot::record_artifact_snapshot_with_ecosystems(
        root, analysis, ecosystems,
    ) {
        Ok(snapshot) => (Some(artifact_snapshot_to_dto(snapshot)), None),
        Err(error) => {
            let warning = format!("Failed to record artifact snapshot: {error}");
            eprintln!("{warning}");
            (None, Some(warning))
        }
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
                    project: project_identity_to_dto(&artifact.project),
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
    let policy = options.recommendation_policy()?;
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
        let analysis = api::analyze_with_policy(scan_result.clone(), policy)
            .map_err(|error| error.to_string())?;
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
                    project: project_identity_to_dto(&artifact.artifact.project),
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
    let policy = options.recommendation_policy()?;
    let record_history = options.record_history.unwrap_or(true);
    let record_artifact_snapshot = options.record_artifact_snapshot.unwrap_or(true);
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
        let analysis = api::analyze_with_policy(scan_result.clone(), policy)
            .map_err(|error| error.to_string())?;
        let analysis_id = cache_analysis(&root, &ecosystems, &analysis);
        let plan = api::clean::build_plan_from_analysis(analysis.clone())
            .map_err(|error| error.to_string())?;
        let storage_summary = match api::storage::summarize_with_access_summary(
            &root,
            &analysis,
            Some(&scan_result.access_summary),
        ) {
            Ok(summary) => storage_summary_to_dto(summary),
            Err(error) => StorageSummaryDto::Unavailable {
                reason: error.to_string(),
            },
        };

        let (artifact_snapshot, artifact_snapshot_warning) = record_artifact_snapshot_if_enabled(
            record_artifact_snapshot,
            &root,
            &analysis,
            &ecosystems,
        );

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
                .iter()
                .map(|artifact| ArtifactAnalysisDto {
                    path: artifact_path(&artifact.artifact.path),
                    ecosystem: artifact.artifact.ecosystem.into(),
                    project: project_identity_to_dto(&artifact.artifact.project),
                    size_bytes: artifact.size_bytes,
                    last_modified_ms: artifact.last_modified.and_then(system_time_to_ms),
                    age_days: artifact.age_days,
                    recommendation: artifact.recommendation.into(),
                })
                .collect(),
        };
        let cleanup_plan = CleanupPlanResponse {
            reclaimable_size_bytes: plan.reclaimable_size_bytes(),
            analysis_id,
            candidates: analysis
                .artifacts
                .iter()
                .map(|artifact| {
                    cleanup_candidate_from_analysis(
                        artifact,
                        artifact.recommendation.selected_by_default(),
                    )
                })
                .collect(),
        };

        Ok(WorkspaceAnalysisResponse {
            analysis: analysis_response,
            cleanup_plan,
            storage_summary,
            artifact_snapshot,
            artifact_snapshot_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Refreshes only filesystem capacity after a permanent cleanup. This does
/// not re-scan or re-analyze the workspace.
#[tauri::command]
async fn refresh_storage_volume(root: String) -> Result<VolumeStorageDto, String> {
    let root = resolve_root(Some(root))?;

    tokio::task::spawn_blocking(move || {
        api::storage::volume(&root)
            .map(volume_storage_to_dto)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn build_cleanup_plan(options: RunOptions) -> Result<CleanupPlanResponse, String> {
    let policy = options.recommendation_policy()?;
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let scan_result = api::scan(&root, &ecosystems).map_err(|error| error.to_string())?;
        let analysis =
            api::analyze_with_policy(scan_result, policy).map_err(|error| error.to_string())?;
        let analysis_id = cache_analysis(&root, &ecosystems, &analysis);
        let plan =
            api::clean::build_plan_from_analysis(analysis).map_err(|error| error.to_string())?;

        Ok(CleanupPlanResponse {
            reclaimable_size_bytes: plan.reclaimable_size_bytes(),
            analysis_id,
            candidates: plan
                .candidates
                .into_iter()
                .map(cleanup_candidate_from_core_candidate)
                .collect(),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn execute_cleanup(request: ExecuteCleanupRequest) -> Result<CleanupResultResponse, String> {
    let mode = request.mode.into();
    let root = resolve_root(Some(request.root))?;
    let ecosystems: Vec<_> = request.ecosystems.into_iter().map(Into::into).collect();
    let analysis_id = request.analysis_id;
    let selected: Vec<_> = request
        .selected_artifacts
        .into_iter()
        .map(|selection| ArtifactSelection {
            path: PathBuf::from(selection.path),
            ecosystem: selection.ecosystem.into(),
        })
        .collect();

    tokio::task::spawn_blocking(move || {
        let analysis = cached_analysis(&analysis_id, &root, &ecosystems)
            .ok_or_else(|| "The selected workspace analysis is no longer available".to_string())?;
        let plan = api::clean::build_plan_from_analysis_with_selection(&analysis, &selected)
            .map_err(|error| error.to_string())?;
        let result = match api::clean::execute(&plan, mode) {
            Ok(result) => result,
            Err(error) => {
                if let Err(history_error) =
                    history::record_failure_with_context(&root, mode, &error.to_string())
                {
                    report_failed_activity_record("cleanup", history_error);
                }
                return Err(error.to_string());
            }
        };
        let history_warning =
            match history::record_with_context(&root, &plan.candidates, mode, &result) {
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
async fn clear_activity_history() -> Result<(), String> {
    tokio::task::spawn_blocking(history::clear)
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
async fn load_dependency_inventory(
    options: RunOptions,
) -> Result<DependencyInventoryResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();

    tokio::task::spawn_blocking(move || {
        let reports =
            api::dependency_report(&root, &ecosystems).map_err(|error| error.to_string())?;
        let inventory_fingerprint =
            api::dependency_inventory_fingerprint(&reports).map_err(|error| error.to_string())?;
        Ok(dependency_inventory_to_dto(
            &root,
            reports,
            None,
            inventory_fingerprint,
        ))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn compare_dependency_baseline(
    options: RunOptions,
) -> Result<DependencyInventoryResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();
    let baseline_path = api::dependency_baseline_path().map_err(|error| error.to_string())?;

    tokio::task::spawn_blocking(move || {
        let reports =
            api::dependency_report(&root, &ecosystems).map_err(|error| error.to_string())?;
        let diff = api::dependency_diff(&root, &reports, &baseline_path)
            .map_err(|error| error.to_string())?;
        let inventory_fingerprint =
            api::dependency_inventory_fingerprint(&reports).map_err(|error| error.to_string())?;
        Ok(dependency_inventory_to_dto(
            &root,
            reports,
            Some(diff),
            inventory_fingerprint,
        ))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn accept_dependency_baseline(
    options: DependencyBaselineAcceptOptions,
) -> Result<DependencyInventoryResponse, String> {
    let root = resolve_root(options.root)?;
    let ecosystems: Vec<_> = options.ecosystems.into_iter().map(Into::into).collect();
    let expected_inventory_fingerprint = options.expected_inventory_fingerprint;
    let baseline_path = api::dependency_baseline_path().map_err(|error| error.to_string())?;

    tokio::task::spawn_blocking(move || {
        let reports =
            api::dependency_report(&root, &ecosystems).map_err(|error| error.to_string())?;
        let inventory_fingerprint =
            verify_dependency_inventory_fingerprint(&reports, &expected_inventory_fingerprint)?;
        api::accept_dependency_baseline(&root, &reports, &baseline_path)
            .map_err(|error| error.to_string())?;
        let diff = api::dependency_diff(&root, &reports, &baseline_path)
            .map_err(|error| error.to_string())?;
        Ok(dependency_inventory_to_dto(
            &root,
            reports,
            Some(diff),
            inventory_fingerprint,
        ))
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
            refresh_storage_volume,
            load_activity_history,
            clear_activity_history,
            load_cleanup_history,
            load_dependency_inventory,
            compare_dependency_baseline,
            accept_dependency_baseline,
            audit,
            security_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_cache_key_separates_scan_scopes() {
        let root = Path::new("/workspace");

        assert_ne!(
            AnalysisCacheKey::new(root, &[Ecosystem::Node]),
            AnalysisCacheKey::new(root, &[Ecosystem::Java])
        );
    }

    #[test]
    fn analysis_cache_key_canonicalizes_equivalent_scopes() {
        let root = Path::new("/workspace");

        assert_eq!(
            AnalysisCacheKey::new(root, &[]),
            AnalysisCacheKey::new(
                root,
                &[
                    Ecosystem::Java,
                    Ecosystem::Node,
                    Ecosystem::Rust,
                    Ecosystem::Node
                ]
            )
        );
    }

    #[test]
    fn analysis_cache_stores_and_refreshes_entries_per_scan_scope() {
        let root = Path::new("/workspace/cache-scope-test");

        let node_id = cache_analysis(
            root,
            &[Ecosystem::Node],
            &AnalysisResult {
                artifacts: Vec::new(),
                total_size_bytes: 1,
                ..AnalysisResult::default()
            },
        );
        let java_id = cache_analysis(
            root,
            &[Ecosystem::Java],
            &AnalysisResult {
                artifacts: Vec::new(),
                total_size_bytes: 2,
                ..AnalysisResult::default()
            },
        );

        assert_eq!(
            cached_analysis(&node_id, root, &[Ecosystem::Node])
                .expect("node analysis should be cached")
                .total_size_bytes,
            1
        );
        assert_eq!(
            cached_analysis(&java_id, root, &[Ecosystem::Java])
                .expect("java analysis should be cached")
                .total_size_bytes,
            2
        );
        assert!(cached_analysis(&node_id, root, &[Ecosystem::Java]).is_none());

        let refreshed_node_id = cache_analysis(
            root,
            &[Ecosystem::Node],
            &AnalysisResult {
                artifacts: Vec::new(),
                total_size_bytes: 3,
                ..AnalysisResult::default()
            },
        );

        assert_eq!(
            cached_analysis(&node_id, root, &[Ecosystem::Node])
                .expect("original node analysis should remain cached")
                .total_size_bytes,
            1
        );
        assert_eq!(
            cached_analysis(&refreshed_node_id, root, &[Ecosystem::Node])
                .expect("refreshed node analysis should be cached")
                .total_size_bytes,
            3
        );
    }

    #[test]
    fn disabled_artifact_snapshot_recording_does_not_touch_snapshot_state() {
        let (snapshot, warning) = record_artifact_snapshot_if_enabled(
            false,
            Path::new("/workspace/snapshot-refresh-test"),
            &AnalysisResult::default(),
            &[Ecosystem::Rust],
        );

        assert!(snapshot.is_none());
        assert!(warning.is_none());
    }

    #[test]
    fn dependency_acceptance_rejects_an_inventory_that_changed_after_review() {
        let original = DependencyReport::unsupported(
            Ecosystem::Node,
            PathBuf::from("/workspace/package.json"),
            "unsupported package manager",
        );
        let expected =
            api::dependency_inventory_fingerprint(std::slice::from_ref(&original)).unwrap();
        let mut changed = original;
        changed.warnings.push("changed after review".to_owned());

        assert!(
            verify_dependency_inventory_fingerprint(std::slice::from_ref(&changed), &expected,)
                .is_err()
        );
        assert_eq!(
            verify_dependency_inventory_fingerprint(std::slice::from_ref(&changed), &expected)
                .unwrap_err(),
            "Dependency inventory changed since it was reviewed. Compare again before accepting the baseline."
        );
    }
}
