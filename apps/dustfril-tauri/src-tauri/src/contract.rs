use std::path::Path;

use dustfril_core::models::{
    CleanupFailureReason, CleanupRecommendation, DeleteMode, Ecosystem, LifecycleScript,
    LockfileCheck, LockfileKind, LockfileStatus, PackageManager, RiskLevel, ScriptType,
    SecurityFinding, SecurityReport, SecurityWarning,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RunOptions {
    pub(crate) root: Option<String>,
    pub(crate) ecosystems: Vec<EcosystemDto>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExecuteCleanupRequest {
    pub(crate) candidates: Vec<CleanupCandidateInput>,
    pub(crate) mode: DeleteModeDto,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CleanupCandidateInput {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) size_bytes: u64,
    pub(crate) age_days: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanResponse {
    pub(crate) artifacts: Vec<ArtifactDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_warning: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalysisResponse {
    pub(crate) artifacts: Vec<ArtifactAnalysisDto>,
    pub(crate) total_size_bytes: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactAnalysisDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) size_bytes: u64,
    pub(crate) last_modified_ms: Option<u64>,
    pub(crate) age_days: Option<u64>,
    pub(crate) recommendation: RecommendationDto,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupPlanResponse {
    pub(crate) candidates: Vec<CleanupCandidateDto>,
    pub(crate) reclaimable_size_bytes: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupCandidateDto {
    pub(crate) path: String,
    pub(crate) ecosystem: EcosystemDto,
    pub(crate) size_bytes: u64,
    pub(crate) age_days: Option<u64>,
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
    }

    #[test]
    fn cleanup_request_wire_format_is_stable() {
        let request: ExecuteCleanupRequest = serde_json::from_value(json!({
            "candidates": [{
                "path": "/workspace/target",
                "ecosystem": "Rust",
                "sizeBytes": 42,
                "ageDays": null
            }],
            "mode": "Trash"
        }))
        .unwrap();

        assert_eq!(request.mode, DeleteModeDto::Trash);
        assert_eq!(request.candidates[0].size_bytes, 42);
        assert_eq!(request.candidates[0].age_days, None);
    }

    #[test]
    fn analysis_response_wire_format_is_stable() {
        let response = AnalysisResponse {
            artifacts: vec![ArtifactAnalysisDto {
                path: "/workspace/target".to_string(),
                ecosystem: EcosystemDto::Rust,
                size_bytes: 42,
                last_modified_ms: None,
                age_days: Some(120),
                recommendation: RecommendationDto::SafeToClean,
            }],
            total_size_bytes: 42,
        };

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "artifacts": [{
                    "path": "/workspace/target",
                    "ecosystem": "Rust",
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
                size_bytes: 42,
                age_days: None,
            }],
            reclaimable_size_bytes: 42,
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
                    "sizeBytes": 42,
                    "ageDays": null
                }],
                "reclaimableSizeBytes": 42
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
            }],
            history_warning: None,
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
                    "ecosystem": "Node"
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
        };
        let cleanup = CleanupResultResponse {
            deleted_paths: Vec::new(),
            failed_paths: Vec::new(),
            freed_size_bytes: 0,
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
    }
}
