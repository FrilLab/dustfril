use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use dustfril_core::models::{
    ArtifactChangeKind, ArtifactSizeChange, ArtifactSnapshot, ArtifactSnapshotArtifact,
    ArtifactSnapshotResult, ArtifactSnapshotStatus, CleanupFailureReason, CleanupRecommendation,
    DeleteMode, DeveloperStorageSummary, Ecosystem, LifecycleScript, LockfileCheck, LockfileKind,
    LockfileStatus, PackageManager, ProjectIdentity, RecommendationPolicy, RiskLevel, ScriptType,
    SecurityFinding, SecurityReport, SecurityWarning, StorageSummary, DEFAULT_CLEANUP_AGE_DAYS,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RunOptions {
    pub(crate) root: Option<String>,
    pub(crate) ecosystems: Vec<EcosystemDto>,
    #[serde(default)]
    pub(crate) record_history: Option<bool>,
    #[serde(default)]
    pub(crate) cleanup_age_days: Option<u64>,
    #[serde(default)]
    pub(crate) record_artifact_snapshot: Option<bool>,
}

impl RunOptions {
    pub(crate) fn recommendation_policy(&self) -> Result<RecommendationPolicy, String> {
        let cleanup_age_days = self.cleanup_age_days.unwrap_or(DEFAULT_CLEANUP_AGE_DAYS);

        RecommendationPolicy::new(cleanup_age_days)
            .ok_or_else(|| "Cleanup age must be greater than zero days.".to_owned())
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExecuteCleanupRequest {
    pub(crate) root: String,
    pub(crate) ecosystems: Vec<EcosystemDto>,
    pub(crate) analysis_id: String,
    pub(crate) selected_artifacts: Vec<ArtifactSelectionInput>,
    pub(crate) mode: DeleteModeDto,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ArtifactSelectionInput {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanResponse {
    pub(crate) artifacts: Vec<ArtifactDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifact_snapshot: Option<ArtifactSnapshotResultDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifact_snapshot_warning: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactSnapshotResultDto {
    pub(crate) status: ArtifactSnapshotStatus,
    pub(crate) snapshot: ArtifactSnapshotDto,
    pub(crate) previous_snapshot: Option<ArtifactSnapshotDto>,
    pub(crate) changes: Vec<ArtifactSizeChangeDto>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactSnapshotDto {
    pub(crate) workspace_id: String,
    pub(crate) timestamp: String,
    pub(crate) artifacts: Vec<ArtifactSnapshotArtifactDto>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactSnapshotArtifactDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) size_bytes: u64,
    pub(crate) last_modified_ms: Option<u64>,
    pub(crate) age_days: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactSizeChangeDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) kind: ArtifactChangeKind,
    pub(crate) previous_size_bytes: Option<u64>,
    pub(crate) current_size_bytes: Option<u64>,
    pub(crate) delta_bytes: i128,
}

pub(crate) fn artifact_snapshot_to_dto(
    result: ArtifactSnapshotResult,
) -> ArtifactSnapshotResultDto {
    ArtifactSnapshotResultDto {
        status: result.status,
        snapshot: snapshot_to_dto(result.snapshot),
        previous_snapshot: result.previous_snapshot.map(snapshot_to_dto),
        changes: result
            .changes
            .into_iter()
            .map(artifact_size_change_to_dto)
            .collect(),
    }
}

fn snapshot_to_dto(snapshot: ArtifactSnapshot) -> ArtifactSnapshotDto {
    ArtifactSnapshotDto {
        workspace_id: snapshot.workspace_id,
        timestamp: snapshot.timestamp.to_rfc3339(),
        artifacts: snapshot
            .artifacts
            .into_iter()
            .map(artifact_snapshot_artifact_to_dto)
            .collect(),
    }
}

fn artifact_snapshot_artifact_to_dto(
    artifact: ArtifactSnapshotArtifact,
) -> ArtifactSnapshotArtifactDto {
    ArtifactSnapshotArtifactDto {
        path: artifact.path.display().to_string(),
        ecosystem: artifact.ecosystem.into(),
        size_bytes: artifact.size_bytes,
        last_modified_ms: artifact.last_modified.and_then(system_time_to_ms),
        age_days: artifact.age_days,
    }
}

fn artifact_size_change_to_dto(change: ArtifactSizeChange) -> ArtifactSizeChangeDto {
    ArtifactSizeChangeDto {
        path: change.path.display().to_string(),
        ecosystem: change.ecosystem.into(),
        kind: change.kind,
        previous_size_bytes: change.previous_size_bytes,
        current_size_bytes: change.current_size_bytes,
        delta_bytes: change.delta_bytes,
    }
}

fn system_time_to_ms(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) project: ProjectIdentityDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectIdentityDto {
    pub(crate) root: String,
    pub(crate) display_name: String,
    pub(crate) ecosystem: EcosystemDto,
}

pub(crate) fn project_identity_to_dto(project: &ProjectIdentity) -> ProjectIdentityDto {
    ProjectIdentityDto {
        root: project.root.display().to_string(),
        display_name: project.display_name.clone(),
        ecosystem: project.ecosystem.into(),
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalysisResponse {
    pub(crate) artifacts: Vec<ArtifactAnalysisDto>,
    pub(crate) total_size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_warning: Option<String>,
}

/// Result of the single user-facing Workspace analysis workflow. It carries
/// both the analyzed artifacts and the cleanup plan derived from that same
/// scan, so the desktop does not need to repeat filesystem traversal.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceAnalysisResponse {
    pub(crate) analysis: AnalysisResponse,
    pub(crate) cleanup_plan: CleanupPlanResponse,
    pub(crate) storage_summary: StorageSummaryDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifact_snapshot: Option<ArtifactSnapshotResultDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifact_snapshot_warning: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactAnalysisDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) project: ProjectIdentityDto,
    pub(crate) size_bytes: u64,
    pub(crate) last_modified_ms: Option<u64>,
    pub(crate) age_days: Option<u64>,
    pub(crate) recommendation: RecommendationDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupPlanResponse {
    /// All scanner-owned analyzed artifacts available for user selection.
    pub(crate) candidates: Vec<CleanupCandidateDto>,
    /// Bytes selected by the recommendation-driven default selection.
    pub(crate) reclaimable_size_bytes: u64,
    /// Opaque Core-owned analysis identity required to execute this selection.
    pub(crate) analysis_id: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupCandidateDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) project: ProjectIdentityDto,
    pub(crate) size_bytes: u64,
    pub(crate) age_days: Option<u64>,
    /// Advisory status retained even after a user manually selects the item.
    pub(crate) recommendation: RecommendationDto,
    /// Whether the recommendation selects this item in a fresh review.
    pub(crate) selected_by_default: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupResultResponse {
    pub(crate) deleted_paths: Vec<String>,
    pub(crate) failed_paths: Vec<CleanupFailureDto>,
    pub(crate) freed_size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_warning: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub(crate) enum StorageSummaryDto {
    Available {
        #[serde(rename = "totalBytes")]
        total_bytes: u64,
        #[serde(rename = "usedBytes")]
        used_bytes: u64,
        #[serde(rename = "availableBytes")]
        available_bytes: u64,
        #[serde(rename = "detectedDevelopmentBytes")]
        detected_development_bytes: u64,
        #[serde(rename = "detectedSharePercent")]
        detected_share_percent: Option<f64>,
        partial: bool,
        warnings: Vec<String>,
        #[serde(rename = "recommendedBytes")]
        recommended_bytes: u64,
        #[serde(rename = "scopePath")]
        scope_path: String,
        categories: Vec<EcosystemDto>,
    },
    Unavailable {
        reason: String,
    },
}

pub(crate) fn storage_summary_to_dto(summary: StorageSummary) -> StorageSummaryDto {
    let detected_share_percent = summary.detected_share_percent();
    let StorageSummary {
        volume,
        developer_storage:
            DeveloperStorageSummary {
                measured_bytes,
                recommended_bytes,
                scope_path,
                categories,
            },
        partial,
        warnings,
    } = summary;

    StorageSummaryDto::Available {
        total_bytes: volume.total_bytes,
        used_bytes: volume.used_bytes,
        available_bytes: volume.available_bytes,
        detected_development_bytes: measured_bytes,
        detected_share_percent,
        partial,
        warnings,
        recommended_bytes,
        scope_path: scope_path.display().to_string(),
        categories: categories.into_iter().map(Into::into).collect(),
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupFailureDto {
    pub(crate) path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupHistoryEntryDto {
    pub(crate) executed_at_ms: u64,
    pub(crate) mode: DeleteModeDto,
    pub(crate) freed_size_bytes: u64,
    pub(crate) deleted_paths: Vec<String>,
    pub(crate) failed_paths: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LifecycleScriptDto {
    pub(crate) package: String,
    pub(crate) package_manager: PackageManagerDto,
    pub(crate) script_type: ScriptTypeDto,
    pub(crate) command: String,
    pub(crate) risk_level: RiskLevelDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityScanResponse {
    pub(crate) findings: Vec<SecurityFindingDto>,
    pub(crate) lifecycle_warnings: Vec<SecurityWarningDto>,
    pub(crate) lockfiles: Vec<LockfileCheckDto>,
    pub(crate) manifests: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_warning: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityFindingDto {
    pub(crate) path: String,
    pub(crate) rule: String,
    pub(crate) package: Option<String>,
    pub(crate) risk_level: RiskLevelDto,
    pub(crate) evidence: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityWarningDto {
    pub(crate) package: String,
    pub(crate) script_type: String,
    pub(crate) command: String,
    pub(crate) risk_level: RiskLevelDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LockfileCheckDto {
    pub(crate) path: String,
    pub(crate) kind: LockfileKindDto,
    pub(crate) status: LockfileStatusDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) enum LockfileKindDto {
    PackageLockJson,
    PnpmLockYaml,
    BunLock,
    CargoLock,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) enum LockfileStatusDto {
    Missing,
    Modified,
    Untracked,
    Clean,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EcosystemDto {
    Rust,
    Node,
    Java,
}

impl From<EcosystemDto> for Ecosystem {
    fn from(value: EcosystemDto) -> Self {
        match value {
            EcosystemDto::Rust => Self::Rust,
            EcosystemDto::Node => Self::Node,
            EcosystemDto::Java => Self::Java,
        }
    }
}

impl From<Ecosystem> for EcosystemDto {
    fn from(value: Ecosystem) -> Self {
        match value {
            Ecosystem::Rust => Self::Rust,
            Ecosystem::Node => Self::Node,
            Ecosystem::Java => Self::Java,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeleteModeDto {
    Trash,
    Permanent,
}

impl From<DeleteModeDto> for DeleteMode {
    fn from(value: DeleteModeDto) -> Self {
        match value {
            DeleteModeDto::Trash => Self::Trash,
            DeleteModeDto::Permanent => Self::Permanent,
        }
    }
}

impl From<DeleteMode> for DeleteModeDto {
    fn from(value: DeleteMode) -> Self {
        match value {
            DeleteMode::Trash => Self::Trash,
            DeleteMode::Permanent => Self::Permanent,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecommendationDto {
    Keep,
    NeedsReview,
    SafeToClean,
}

impl From<CleanupRecommendation> for RecommendationDto {
    fn from(value: CleanupRecommendation) -> Self {
        match value {
            CleanupRecommendation::Keep => Self::Keep,
            CleanupRecommendation::NeedsReview => Self::NeedsReview,
            CleanupRecommendation::SafeToClean => Self::SafeToClean,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RiskLevelDto {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl From<RiskLevel> for RiskLevelDto {
    fn from(value: RiskLevel) -> Self {
        match value {
            RiskLevel::None => Self::None,
            RiskLevel::Low => Self::Low,
            RiskLevel::Medium => Self::Medium,
            RiskLevel::High => Self::High,
            RiskLevel::Critical => Self::Critical,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PackageManagerDto {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Unknown,
}

impl From<PackageManager> for PackageManagerDto {
    fn from(value: PackageManager) -> Self {
        match value {
            PackageManager::Npm => Self::Npm,
            PackageManager::Pnpm => Self::Pnpm,
            PackageManager::Yarn => Self::Yarn,
            PackageManager::Bun => Self::Bun,
            PackageManager::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ScriptTypeDto {
    Preinstall,
    Install,
    Postinstall,
    Prepare,
    Prepublish,
    PrepublishOnly,
}

impl From<ScriptType> for ScriptTypeDto {
    fn from(value: ScriptType) -> Self {
        match value {
            ScriptType::Preinstall => Self::Preinstall,
            ScriptType::Install => Self::Install,
            ScriptType::Postinstall => Self::Postinstall,
            ScriptType::Prepare => Self::Prepare,
            ScriptType::Prepublish => Self::Prepublish,
            ScriptType::PrepublishOnly => Self::PrepublishOnly,
        }
    }
}

pub(crate) fn artifact_path(path: &Path) -> String {
    path.display().to_string()
}

pub(crate) fn cleanup_failure_reason(reason: &CleanupFailureReason) -> String {
    match reason {
        CleanupFailureReason::PermissionDenied => "PermissionDenied".to_string(),
        CleanupFailureReason::NotFound => "NotFound".to_string(),
        CleanupFailureReason::UnsafePath => "UnsafePath".to_string(),
        CleanupFailureReason::SymbolicLink => "SymbolicLink".to_string(),
        CleanupFailureReason::Other(message) => message.clone(),
    }
}

impl From<LifecycleScript> for LifecycleScriptDto {
    fn from(script: LifecycleScript) -> Self {
        Self {
            package: script.package,
            package_manager: script.package_manager.into(),
            script_type: script.script_type.into(),
            command: script.command,
            risk_level: script.risk_level.into(),
        }
    }
}

impl From<SecurityReport> for SecurityScanResponse {
    fn from(report: SecurityReport) -> Self {
        Self {
            findings: report.findings.into_iter().map(Into::into).collect(),
            lifecycle_warnings: report
                .lifecycle_warnings
                .into_iter()
                .map(Into::into)
                .collect(),
            lockfiles: report.lockfiles.into_iter().map(Into::into).collect(),
            manifests: report
                .manifests
                .into_iter()
                .map(|path| path.display().to_string())
                .collect(),
            history_warning: None,
        }
    }
}

impl From<SecurityFinding> for SecurityFindingDto {
    fn from(finding: SecurityFinding) -> Self {
        Self {
            path: finding.path.display().to_string(),
            rule: finding.kind.rule_id().to_owned(),
            package: finding.package,
            risk_level: finding.risk_level.into(),
            evidence: finding.evidence,
            reason: finding.reason,
        }
    }
}

impl From<SecurityWarning> for SecurityWarningDto {
    fn from(warning: SecurityWarning) -> Self {
        Self {
            package: warning.package,
            script_type: warning.script_type,
            command: warning.command,
            risk_level: warning.risk_level.into(),
        }
    }
}

impl From<LockfileCheck> for LockfileCheckDto {
    fn from(check: LockfileCheck) -> Self {
        Self {
            path: check.path.display().to_string(),
            kind: check.kind.into(),
            status: check.status.into(),
        }
    }
}

impl From<LockfileKind> for LockfileKindDto {
    fn from(kind: LockfileKind) -> Self {
        match kind {
            LockfileKind::PackageLockJson => Self::PackageLockJson,
            LockfileKind::PnpmLockYaml => Self::PnpmLockYaml,
            LockfileKind::BunLock => Self::BunLock,
            LockfileKind::CargoLock => Self::CargoLock,
        }
    }
}

impl From<LockfileStatus> for LockfileStatusDto {
    fn from(status: LockfileStatus) -> Self {
        match status {
            LockfileStatus::Missing => Self::Missing,
            LockfileStatus::Modified => Self::Modified,
            LockfileStatus::Untracked => Self::Untracked,
            LockfileStatus::Clean => Self::Clean,
        }
    }
}

#[cfg(test)]
mod tests {
    use dustfril_core::models::AnalysisResult;
    use serde_json::json;

    use super::*;

    #[test]
    fn run_options_wire_format_is_stable() {
        let options: RunOptions = serde_json::from_value(json!({
            "root": "/workspace",
            "ecosystems": ["Rust", "Node", "Java"]
        }))
        .unwrap();

        assert_eq!(options.root.as_deref(), Some("/workspace"));
        assert_eq!(
            options.ecosystems,
            vec![EcosystemDto::Rust, EcosystemDto::Node, EcosystemDto::Java]
        );
        assert_eq!(options.record_history, None);
        assert_eq!(options.cleanup_age_days, None);
        assert_eq!(options.record_artifact_snapshot, None);

        let explicit_analysis_options: RunOptions = serde_json::from_value(json!({
            "root": "/workspace",
            "ecosystems": ["Rust"],
            "recordHistory": true
        }))
        .unwrap();
        assert_eq!(explicit_analysis_options.record_history, Some(true));
        assert_eq!(explicit_analysis_options.cleanup_age_days, None);
        assert_eq!(explicit_analysis_options.record_artifact_snapshot, None);

        let configured_analysis_options: RunOptions = serde_json::from_value(json!({
            "root": "/workspace",
            "ecosystems": ["Rust"],
            "cleanupAgeDays": 60
        }))
        .unwrap();
        assert_eq!(configured_analysis_options.cleanup_age_days, Some(60));
        assert_eq!(configured_analysis_options.record_artifact_snapshot, None);
        assert_eq!(
            configured_analysis_options
                .recommendation_policy()
                .unwrap()
                .cleanup_age_days(),
            60
        );

        let refresh_options: RunOptions = serde_json::from_value(json!({
            "root": "/workspace",
            "ecosystems": ["Rust"],
            "recordHistory": false,
            "recordArtifactSnapshot": false
        }))
        .unwrap();
        assert_eq!(refresh_options.record_history, Some(false));
        assert_eq!(refresh_options.record_artifact_snapshot, Some(false));
    }

    #[test]
    fn cleanup_request_wire_format_is_stable() {
        let request: ExecuteCleanupRequest = serde_json::from_value(json!({
            "root": "/workspace",
            "ecosystems": ["Rust", "Node"],
            "analysisId": "analysis-1",
            "selectedArtifacts": [{
                "path": "/workspace/target",
                "ecosystem": "Rust"
            }],
            "mode": "Trash"
        }))
        .unwrap();

        assert_eq!(request.mode, DeleteModeDto::Trash);
        assert_eq!(request.root, "/workspace");
        assert_eq!(
            request.ecosystems,
            vec![EcosystemDto::Rust, EcosystemDto::Node]
        );
        assert_eq!(request.analysis_id, "analysis-1");
        assert_eq!(request.selected_artifacts[0].path, "/workspace/target");
    }

    #[test]
    fn analysis_response_wire_format_is_stable() {
        let response = AnalysisResponse {
            artifacts: vec![ArtifactAnalysisDto {
                path: "/workspace/target".to_string(),
                ecosystem: EcosystemDto::Rust,
                project: ProjectIdentityDto {
                    root: "/workspace".to_string(),
                    display_name: "workspace".to_string(),
                    ecosystem: EcosystemDto::Rust,
                },
                size_bytes: 42,
                last_modified_ms: None,
                age_days: Some(120),
                recommendation: RecommendationDto::SafeToClean,
            }],
            total_size_bytes: 42,
            history_warning: None,
        };

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "artifacts": [{
                    "path": "/workspace/target",
                    "ecosystem": "Rust",
                    "project": {
                        "root": "/workspace",
                        "displayName": "workspace",
                        "ecosystem": "Rust"
                    },
                    "sizeBytes": 42,
                    "lastModifiedMs": null,
                    "ageDays": 120,
                    "recommendation": "SafeToClean"
                }],
                "totalSizeBytes": 42
            })
        );
    }

    #[test]
    fn workspace_analysis_response_wire_format_keeps_analysis_and_plan_together() {
        let response = WorkspaceAnalysisResponse {
            analysis: AnalysisResponse {
                artifacts: Vec::new(),
                total_size_bytes: 42,
                history_warning: None,
            },
            cleanup_plan: CleanupPlanResponse {
                candidates: vec![CleanupCandidateDto {
                    path: "/workspace/target".to_string(),
                    ecosystem: EcosystemDto::Rust,
                    project: ProjectIdentityDto {
                        root: "/workspace".to_string(),
                        display_name: "workspace".to_string(),
                        ecosystem: EcosystemDto::Rust,
                    },
                    size_bytes: 42,
                    age_days: Some(120),
                    recommendation: RecommendationDto::SafeToClean,
                    selected_by_default: true,
                }],
                reclaimable_size_bytes: 42,
                analysis_id: "analysis-1".to_string(),
            },
            storage_summary: StorageSummaryDto::Available {
                total_bytes: 100,
                used_bytes: 60,
                available_bytes: 40,
                detected_development_bytes: 12,
                detected_share_percent: Some(20.0),
                partial: false,
                warnings: vec![],
                recommended_bytes: 4,
                scope_path: "/workspace".to_string(),
                categories: vec![EcosystemDto::Rust, EcosystemDto::Node],
            },
            artifact_snapshot: None,
            artifact_snapshot_warning: None,
        };

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "analysis": {
                    "artifacts": [],
                    "totalSizeBytes": 42
                },
                "cleanupPlan": {
                    "candidates": [{
                        "path": "/workspace/target",
                        "ecosystem": "Rust",
                        "project": {
                            "root": "/workspace",
                            "displayName": "workspace",
                            "ecosystem": "Rust"
                        },
                    "sizeBytes": 42,
                    "ageDays": 120,
                    "recommendation": "SafeToClean",
                    "selectedByDefault": true
                    }],
                    "reclaimableSizeBytes": 42,
                    "analysisId": "analysis-1"
                },
                "storageSummary": {
                    "status": "available",
                    "totalBytes": 100,
                    "usedBytes": 60,
                    "availableBytes": 40,
                    "detectedDevelopmentBytes": 12,
                    "detectedSharePercent": 20.0,
                    "partial": false,
                    "warnings": [],
                    "recommendedBytes": 4,
                    "scopePath": "/workspace",
                    "categories": ["Rust", "Node"]
                }
            })
        );
    }

    #[test]
    fn lifecycle_script_wire_values_are_stable() {
        let response = LifecycleScriptDto {
            package: "demo".to_string(),
            package_manager: PackageManagerDto::Pnpm,
            script_type: ScriptTypeDto::PrepublishOnly,
            command: "node publish.js".to_string(),
            risk_level: RiskLevelDto::High,
        };

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "package": "demo",
                "packageManager": "pnpm",
                "scriptType": "prepublishOnly",
                "command": "node publish.js",
                "riskLevel": "High"
            })
        );
    }

    #[test]
    fn critical_lifecycle_risk_is_preserved_in_wire_contract() {
        let response = LifecycleScriptDto {
            package: "demo".to_string(),
            package_manager: PackageManagerDto::Npm,
            script_type: ScriptTypeDto::Postinstall,
            command: "curl payload && ./payload".to_string(),
            risk_level: RiskLevelDto::Critical,
        };

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "package": "demo",
                "packageManager": "npm",
                "scriptType": "postinstall",
                "command": "curl payload && ./payload",
                "riskLevel": "Critical"
            })
        );
    }

    #[test]
    fn cleanup_response_wire_formats_are_stable() {
        let plan = CleanupPlanResponse {
            candidates: vec![CleanupCandidateDto {
                path: "/workspace/target".to_string(),
                ecosystem: EcosystemDto::Rust,
                project: ProjectIdentityDto {
                    root: "/workspace".to_string(),
                    display_name: "workspace".to_string(),
                    ecosystem: EcosystemDto::Rust,
                },
                size_bytes: 42,
                age_days: None,
                recommendation: RecommendationDto::SafeToClean,
                selected_by_default: true,
            }],
            reclaimable_size_bytes: 42,
            analysis_id: "analysis-1".to_string(),
        };
        let result = CleanupResultResponse {
            deleted_paths: vec!["/workspace/target".to_string()],
            failed_paths: vec![CleanupFailureDto {
                path: "/workspace/node_modules".to_string(),
                reason: "PermissionDenied".to_string(),
            }],
            freed_size_bytes: 42,
            history_warning: None,
        };

        assert_eq!(
            serde_json::to_value(plan).unwrap(),
            json!({
                "candidates": [{
                    "path": "/workspace/target",
                    "ecosystem": "Rust",
                    "project": {
                        "root": "/workspace",
                        "displayName": "workspace",
                        "ecosystem": "Rust"
                    },
                    "sizeBytes": 42,
                    "ageDays": null,
                    "recommendation": "SafeToClean",
                    "selectedByDefault": true
                }],
                "reclaimableSizeBytes": 42,
                "analysisId": "analysis-1"
            })
        );
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({
                "deletedPaths": ["/workspace/target"],
                "failedPaths": [{
                    "path": "/workspace/node_modules",
                    "reason": "PermissionDenied"
                }],
                "freedSizeBytes": 42
            })
        );
    }

    #[test]
    fn scan_and_history_wire_formats_are_stable() {
        let scan = ScanResponse {
            artifacts: vec![ArtifactDto {
                path: "/workspace/node_modules".to_string(),
                ecosystem: EcosystemDto::Node,
                project: ProjectIdentityDto {
                    root: "/workspace".to_string(),
                    display_name: "workspace".to_string(),
                    ecosystem: EcosystemDto::Node,
                },
            }],
            history_warning: None,
            artifact_snapshot: None,
            artifact_snapshot_warning: None,
        };
        let history = CleanupHistoryEntryDto {
            executed_at_ms: 1_750_000_000_000,
            mode: DeleteModeDto::Trash,
            freed_size_bytes: 42,
            deleted_paths: vec!["/workspace/target".to_string()],
            failed_paths: Vec::new(),
        };

        assert_eq!(
            serde_json::to_value(scan).unwrap(),
            json!({
                "artifacts": [{
                    "path": "/workspace/node_modules",
                    "ecosystem": "Node",
                    "project": {
                        "root": "/workspace",
                        "displayName": "workspace",
                        "ecosystem": "Node"
                    }
                }]
            })
        );
        assert_eq!(
            serde_json::to_value(history).unwrap(),
            json!({
                "executedAtMs": 1_750_000_000_000_u64,
                "mode": "Trash",
                "freedSizeBytes": 42,
                "deletedPaths": ["/workspace/target"],
                "failedPaths": []
            })
        );
    }

    #[test]
    fn artifact_snapshot_wire_format_uses_desktop_timestamp_fields() {
        let timestamp =
            ArtifactSnapshot::from_analysis(Path::new("/workspace"), &AnalysisResult::default())
                .timestamp;
        let result = ArtifactSnapshotResult {
            status: ArtifactSnapshotStatus::Compared,
            snapshot: ArtifactSnapshot {
                workspace_id: "/workspace".to_owned(),
                timestamp,
                artifacts: vec![ArtifactSnapshotArtifact {
                    path: "target".into(),
                    ecosystem: Ecosystem::Rust,
                    size_bytes: 15,
                    last_modified: Some(SystemTime::UNIX_EPOCH),
                    age_days: Some(1),
                }],
            },
            previous_snapshot: None,
            changes: vec![ArtifactSizeChange {
                path: "target".into(),
                ecosystem: Ecosystem::Rust,
                kind: ArtifactChangeKind::SizeIncreased,
                previous_size_bytes: Some(10),
                current_size_bytes: Some(15),
                delta_bytes: 5,
            }],
        };

        let value = serde_json::to_value(artifact_snapshot_to_dto(result)).unwrap();
        assert_eq!(value["status"], "compared");
        assert_eq!(value["snapshot"]["workspaceId"], "/workspace");
        assert!(value["snapshot"]["timestamp"].is_string());
        assert_eq!(value["snapshot"]["artifacts"][0]["lastModifiedMs"], 0);
        assert_eq!(value["changes"][0]["kind"], "sizeIncreased");
        assert_eq!(value["changes"][0]["deltaBytes"], 5);
    }

    #[test]
    fn security_scan_wire_format_preserves_structured_findings() {
        let report = SecurityReport {
            findings: vec![SecurityFinding::new(
                "/workspace/package.json".into(),
                dustfril_core::models::SecurityFindingKind::SuspiciousScript,
                Some("demo".to_owned()),
                RiskLevel::High,
                Some("curl payload | bash".to_owned()),
                "Remote script is piped to a shell.",
            )],
            ..SecurityReport::default()
        };
        let response: SecurityScanResponse = report.into();

        assert_eq!(
            serde_json::to_value(response).unwrap()["findings"][0],
            json!({
                "path": "/workspace/package.json",
                "rule": "suspicious-script",
                "package": "demo",
                "riskLevel": "High",
                "evidence": "curl payload | bash",
                "reason": "Remote script is piped to a shell."
            })
        );
    }

    #[test]
    fn history_warning_is_additive_and_omitted_for_healthy_operations() {
        let scan = ScanResponse {
            artifacts: Vec::new(),
            history_warning: None,
            artifact_snapshot: None,
            artifact_snapshot_warning: None,
        };
        let cleanup = CleanupResultResponse {
            deleted_paths: Vec::new(),
            failed_paths: Vec::new(),
            freed_size_bytes: 0,
            history_warning: Some("history is unavailable".to_owned()),
        };
        let analysis = AnalysisResponse {
            artifacts: Vec::new(),
            total_size_bytes: 0,
            history_warning: Some("history is unavailable".to_owned()),
        };

        assert_eq!(
            serde_json::to_value(scan).unwrap(),
            json!({"artifacts": []})
        );
        assert_eq!(
            serde_json::to_value(cleanup).unwrap(),
            json!({
                "deletedPaths": [],
                "failedPaths": [],
                "freedSizeBytes": 0,
                "historyWarning": "history is unavailable"
            })
        );
        assert_eq!(
            serde_json::to_value(analysis).unwrap(),
            json!({
                "artifacts": [],
                "totalSizeBytes": 0,
                "historyWarning": "history is unavailable"
            })
        );
    }

    #[test]
    fn request_rejects_unknown_fields_and_enum_values() {
        assert!(serde_json::from_value::<RunOptions>(json!({
            "root": null,
            "ecosystems": ["Python"]
        }))
        .is_err());
        assert!(serde_json::from_value::<RunOptions>(json!({
            "root": null,
            "ecosystems": [],
            "global": true
        }))
        .is_err());
        assert!(serde_json::from_value::<RunOptions>(json!({
            "root": null,
            "ecosystems": [],
            "cleanupAgeDays": 0
        }))
        .unwrap()
        .recommendation_policy()
        .is_err());
    }
}
