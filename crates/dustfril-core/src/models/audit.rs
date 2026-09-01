use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

use super::LockfileCheck;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleScript {
    pub package: String,
    pub package_manager: PackageManager,
    pub script_type: ScriptType,
    pub command: String,
    pub risk_level: RiskLevel,
}

/// A lifecycle script finding produced by the security rule engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityWarning {
    pub package: String,
    pub script_type: String,
    pub command: String,
    pub risk_level: RiskLevel,
    pub reason: String,
}

/// The type of supply-chain issue reported by the security scanner.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityFindingKind {
    SuspiciousScript,
    UntrustedDependency,
    KnownMaliciousPackage,
    MissingLockfile,
    ModifiedLockfile,
    UntrackedLockfile,
}

impl fmt::Display for SecurityFindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::SuspiciousScript => "Suspicious script",
            Self::UntrustedDependency => "Untrusted dependency",
            Self::KnownMaliciousPackage => "Known malicious package",
            Self::MissingLockfile => "Missing lockfile",
            Self::ModifiedLockfile => "Modified lockfile",
            Self::UntrackedLockfile => "Untracked lockfile",
        };

        f.write_str(value)
    }
}

/// A normalized supply-chain security finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityFinding {
    /// Manifest or lockfile that produced the finding.
    pub path: PathBuf,
    /// Broad category of the finding.
    pub kind: SecurityFindingKind,
    /// Package name when the finding concerns a dependency.
    pub package: Option<String>,
    /// Severity assigned by the offline rule set.
    pub risk_level: RiskLevel,
    /// Optional command, dependency source, or lockfile status that supports the finding.
    pub evidence: Option<String>,
    /// Human-readable explanation and remediation context.
    pub reason: String,
}

impl SecurityFinding {
    /// Creates a normalized security finding.
    pub fn new(
        path: PathBuf,
        kind: SecurityFindingKind,
        package: Option<String>,
        risk_level: RiskLevel,
        evidence: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            path,
            kind,
            package,
            risk_level,
            evidence,
            reason: reason.into(),
        }
    }
}

/// Complete result of a read-only supply-chain security scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SecurityReport {
    /// All findings, including lifecycle warnings and lockfile issues.
    pub findings: Vec<SecurityFinding>,
    /// Lifecycle warnings retained for callers using the original audit model.
    pub lifecycle_warnings: Vec<SecurityWarning>,
    /// Lockfiles inspected while building the report.
    pub lockfiles: Vec<LockfileCheck>,
    /// Manifests successfully inspected by the report.
    pub manifests: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Unknown,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
            Self::Unknown => "unknown",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScriptType {
    Preinstall,
    Install,
    Postinstall,
    Prepare,
    Prepublish,
    PrepublishOnly,
}

impl ScriptType {
    pub fn from_script_name(name: &str) -> Option<Self> {
        match name {
            "preinstall" => Some(Self::Preinstall),
            "install" => Some(Self::Install),
            "postinstall" => Some(Self::Postinstall),
            "prepare" => Some(Self::Prepare),
            "prepublish" => Some(Self::Prepublish),
            "prepublishOnly" => Some(Self::PrepublishOnly),
            _ => None,
        }
    }
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Preinstall => "preinstall",
            Self::Install => "install",
            Self::Postinstall => "postinstall",
            Self::Prepare => "prepare",
            Self::Prepublish => "prepublish",
            Self::PrepublishOnly => "prepublishOnly",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        };

        write!(f, "{value}")
    }
}
