//! Models for local executable-integrity observations and comparisons.

use std::{collections::BTreeMap, fmt, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::SignatureReport;

/// Version of the on-disk executable-integrity baseline format.
pub const INTEGRITY_STATE_VERSION: u32 = 1;

/// A caller-selected development tool to inspect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolSpec {
    /// The command name or explicit filesystem path requested by the caller.
    pub name: String,
}

impl ToolSpec {
    /// Creates a tool selection without resolving or executing it.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl From<&str> for ToolSpec {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for ToolSpec {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

/// The result of comparing a current observation with the previous baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntegrityStatus {
    /// No prior successful observation exists for this tool.
    NewBaseline,
    /// The selected identity and content hash are unchanged.
    Unchanged,
    /// The resolved file content hash differs from the previous observation.
    ContentChanged,
    /// PATH resolution, canonical target, or symlink relationship changed.
    ResolvedPathChanged,
    /// No candidate could be found for the requested tool.
    Missing,
    /// The candidate existed but could not be safely inspected.
    InspectionFailed,
}

impl fmt::Display for IntegrityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::NewBaseline => "New baseline",
            Self::Unchanged => "Unchanged",
            Self::ContentChanged => "Content changed",
            Self::ResolvedPathChanged => "Resolved path changed",
            Self::Missing => "Missing",
            Self::InspectionFailed => "Inspection failed",
        };

        f.write_str(label)
    }
}

/// The reason a requested executable could not be inspected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntegrityFailureKind {
    /// The requested tool name was empty or otherwise invalid.
    InvalidToolName,
    /// No PATH candidate or explicit path exists.
    NotFound,
    /// The resolved target is not a regular file.
    NonRegularFile,
    /// The resolved regular file is not executable for the current platform.
    NonExecutable,
    /// Metadata, opening, or reading the target was denied or unavailable.
    Unreadable,
    /// A symlink target does not exist.
    BrokenSymlink,
    /// Symlink resolution exceeded the platform's loop limit.
    SymlinkLoop,
    /// Reading the target failed after hashing began.
    HashFailed,
}

impl fmt::Display for IntegrityFailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::InvalidToolName => "Invalid tool name",
            Self::NotFound => "Tool not found",
            Self::NonRegularFile => "Resolved target is not a regular file",
            Self::NonExecutable => "Resolved target is not executable",
            Self::Unreadable => "Target is unreadable",
            Self::BrokenSymlink => "Broken symlink",
            Self::SymlinkLoop => "Symlink loop",
            Self::HashFailed => "Hashing failed",
        };

        f.write_str(label)
    }
}

/// Structured details for an inspection failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityFailure {
    pub kind: IntegrityFailureKind,
    pub message: String,
}

impl IntegrityFailure {
    pub(crate) fn new(kind: IntegrityFailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// A successful, non-executing observation of a requested executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableObservation {
    pub requested_tool: String,
    /// The PATH entry or explicit path selected before canonicalization.
    pub resolved_path: PathBuf,
    /// The regular file whose bytes were hashed.
    pub canonical_path: PathBuf,
    /// The direct symlink target, when the selected path is a symlink.
    pub symlink_target: Option<PathBuf>,
    pub size_bytes: u64,
    /// Lowercase hexadecimal SHA-256 of the canonical target's bytes.
    pub sha256: String,
    pub observed_at: DateTime<Utc>,
    /// Reserved for safe metadata sources; inspection never invokes the tool.
    pub version_metadata: Option<String>,
}

/// One tool's current integrity result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityCheck {
    pub requested_tool: String,
    pub status: IntegrityStatus,
    pub observation: Option<ExecutableObservation>,
    pub previous_observation: Option<ExecutableObservation>,
    pub failure: Option<IntegrityFailure>,
    /// Signature evidence for the current canonical target, when the target
    /// was resolved successfully. Signature verification is deliberately not
    /// part of the hash baseline comparison.
    pub signature: Option<SignatureReport>,
}

/// Structured results for an executable-integrity scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IntegrityReport {
    pub checks: Vec<IntegrityCheck>,
}

impl IntegrityReport {
    /// Returns whether any tool differed from its previous successful state.
    pub fn has_changes(&self) -> bool {
        self.checks.iter().any(|check| {
            matches!(
                check.status,
                IntegrityStatus::ContentChanged | IntegrityStatus::ResolvedPathChanged
            ) || (check.status == IntegrityStatus::Missing && check.previous_observation.is_some())
        })
    }
}

/// Versioned local state containing the last successful observation per tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityBaseline {
    pub version: u32,
    pub observations: BTreeMap<String, ExecutableObservation>,
}

impl Default for IntegrityBaseline {
    fn default() -> Self {
        Self {
            version: INTEGRITY_STATE_VERSION,
            observations: BTreeMap::new(),
        }
    }
}
