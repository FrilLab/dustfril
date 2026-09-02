use core::fmt;
use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{Ecosystem, LockfileKind};

/// Overall state of one ecosystem's dependency inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyReportStatus {
    /// The manifest and supported lockfile were parsed successfully.
    Complete,
    /// The manifest was parsed, but its expected lockfile is not present.
    MissingLockfile,
    /// The ecosystem, package manager, or lockfile format is outside the
    /// dependency report's supported scope.
    Unsupported,
}

impl fmt::Display for DependencyReportStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Complete => "Complete",
            Self::MissingLockfile => "Missing lockfile",
            Self::Unsupported => "Unsupported",
        };

        f.write_str(value)
    }
}

/// Availability of a metric whose value may not be derivable from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyMetricStatus {
    /// The value was calculated from a parsed lockfile.
    Available,
    /// The value is unknown because the expected lockfile is missing.
    Unknown,
    /// The value cannot be calculated for the selected format or ecosystem.
    Unsupported,
}

impl fmt::Display for DependencyMetricStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Available => "Available",
            Self::Unknown => "Unknown",
            Self::Unsupported => "Unsupported",
        };

        f.write_str(value)
    }
}

/// A count with an explicit availability state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyMetric {
    /// The count when `status` is `Available`.
    pub value: Option<usize>,
    /// Whether the count is available, unknown, or unsupported.
    pub status: DependencyMetricStatus,
    /// Explanation for an unknown or unsupported value.
    pub reason: Option<String>,
}

impl DependencyMetric {
    /// Creates an available count.
    pub fn available(value: usize) -> Self {
        Self {
            value: Some(value),
            status: DependencyMetricStatus::Available,
            reason: None,
        }
    }

    /// Creates a metric that is unavailable because a lockfile is missing.
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self {
            value: None,
            status: DependencyMetricStatus::Unknown,
            reason: Some(reason.into()),
        }
    }

    /// Creates a metric that is not supported by the selected input.
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self {
            value: None,
            status: DependencyMetricStatus::Unsupported,
            reason: Some(reason.into()),
        }
    }
}

/// State of the lockfile selected for a dependency report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyLockfileStatus {
    /// The lockfile was parsed and contributed to the inventory.
    Parsed,
    /// The manifest implies a lockfile that is not present.
    Missing,
    /// A lockfile was identified but its format is not supported here.
    Unsupported,
}

impl fmt::Display for DependencyLockfileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Parsed => "Parsed",
            Self::Missing => "Missing",
            Self::Unsupported => "Unsupported",
        };

        f.write_str(value)
    }
}

/// Metadata about the lockfile used by a dependency report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyLockfile {
    /// Expected or observed lockfile path.
    pub path: PathBuf,
    /// Existing DustFril lockfile kind, when recognized.
    pub kind: Option<LockfileKind>,
    /// Format and version description, when known.
    pub format: Option<String>,
    /// Parsing/availability state.
    pub status: DependencyLockfileStatus,
    /// Explanation for a missing or unsupported lockfile.
    pub reason: Option<String>,
}

/// One package that resolves at more than one version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateDependency {
    /// Package or crate name.
    pub name: String,
    /// Distinct resolved versions in deterministic order.
    pub versions: Vec<String>,
}

/// Whether a resolved dependency is directly declared, transitive, or cannot
/// be classified reliably by the selected lockfile format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyScope {
    Direct,
    Transitive,
    Unknown,
}

impl fmt::Display for DependencyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Direct => "Direct",
            Self::Transitive => "Transitive",
            Self::Unknown => "Unknown",
        };

        f.write_str(value)
    }
}

/// A normalized logical dependency entry used by reports and baselines.
///
/// The dependency identity is `(ecosystem, name, version, source)`. A
/// lockfile location is intentionally not included, so ordering and layout
/// changes do not create false changes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEntry {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    /// Registry/source identifier when the parser can obtain it reliably.
    pub source: Option<String>,
    pub scope: DependencyScope,
}

/// State of the explicit baseline used for a dependency comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyBaselineStatus {
    /// No prior baseline existed; the current inventory was stored.
    BaselineCreated,
    /// A prior baseline was compared and remains unchanged until explicitly
    /// accepted by the caller.
    Compared,
}

impl fmt::Display for DependencyBaselineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BaselineCreated => "Baseline created",
            Self::Compared => "Compared",
        };

        f.write_str(value)
    }
}

/// The kind of logical dependency change found between two baselines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyChangeKind {
    Added,
    Removed,
    VersionChanged,
    SourceChanged,
}

impl fmt::Display for DependencyChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Added => "Added",
            Self::Removed => "Removed",
            Self::VersionChanged => "Version changed",
            Self::SourceChanged => "Source changed",
        };

        f.write_str(value)
    }
}

/// One deterministic dependency change. Added entries have only `current`,
/// removed entries only `previous`, and replacement changes have both.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyChange {
    pub kind: DependencyChangeKind,
    pub previous: Option<DependencyEntry>,
    pub current: Option<DependencyEntry>,
}

/// Structured result of comparing a current inventory with an explicit local
/// baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyDiff {
    /// Stable identity derived from the canonical workspace root.
    pub workspace_id: String,
    pub baseline_status: DependencyBaselineStatus,
    pub added: Vec<DependencyChange>,
    pub removed: Vec<DependencyChange>,
    pub version_changes: Vec<DependencyChange>,
    pub source_changes: Vec<DependencyChange>,
    pub warnings: Vec<String>,
}

impl DependencyDiff {
    pub fn empty(
        workspace_id: impl Into<String>,
        baseline_status: DependencyBaselineStatus,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            baseline_status,
            added: Vec::new(),
            removed: Vec::new(),
            version_changes: Vec::new(),
            source_changes: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns whether the comparison found any logical dependency changes.
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.version_changes.is_empty()
            || !self.source_changes.is_empty()
    }
}

/// Versioned local state containing one inventory per observed ecosystem and
/// workspace. Only normalized dependency data is persisted.
pub const DEPENDENCY_BASELINE_STATE_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyBaseline {
    pub workspace_id: String,
    pub inventories: BTreeMap<Ecosystem, Vec<DependencyEntry>>,
}

/// On-disk dependency baseline collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyBaselineState {
    pub version: u32,
    pub projects: BTreeMap<String, DependencyBaseline>,
}

impl Default for DependencyBaselineState {
    fn default() -> Self {
        Self {
            version: DEPENDENCY_BASELINE_STATE_VERSION,
            projects: BTreeMap::new(),
        }
    }
}

/// Structured dependency inventory for one supported ecosystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyReport {
    /// Ecosystem represented by this report.
    pub ecosystem: Ecosystem,
    /// Whether the inventory is complete, missing a lockfile, or unsupported.
    pub status: DependencyReportStatus,
    /// Manifest inspected, or the expected manifest path for an unsupported
    /// or missing project input.
    pub manifest: PathBuf,
    /// Manifest format, when known.
    pub manifest_format: Option<String>,
    /// Lockfile metadata and explicit missing/unsupported state.
    pub lockfile: Option<DependencyLockfile>,
    /// Direct dependency counts by ecosystem-specific category. Node uses
    /// `dependencies`, `devDependencies`, `optionalDependencies`, and
    /// `peerDependencies`; Rust uses `dependencies`, `dev-dependencies`, and
    /// `build-dependencies`.
    pub direct_dependency_counts: BTreeMap<String, usize>,
    /// Number of unique direct package names across all categories.
    pub direct_dependency_total: usize,
    /// Number of logical package/version nodes represented by the lockfile.
    pub resolved_dependency_count: DependencyMetric,
    /// Number of resolved nodes classified as non-direct.
    pub transitive_dependency_count: DependencyMetric,
    /// Packages with more than one distinct resolved version.
    pub duplicate_versions: Vec<DuplicateDependency>,
    /// Normalized resolved dependency entries reused by baseline comparison.
    pub resolved_dependencies: Vec<DependencyEntry>,
    /// Non-fatal scope or availability notes.
    pub warnings: Vec<String>,
}

impl DependencyReport {
    /// Creates an unsupported report for an ecosystem or format outside the
    /// inventory implementation.
    pub fn unsupported(ecosystem: Ecosystem, manifest: PathBuf, reason: impl Into<String>) -> Self {
        let reason = reason.into();

        Self {
            ecosystem,
            status: DependencyReportStatus::Unsupported,
            manifest,
            manifest_format: None,
            lockfile: None,
            direct_dependency_counts: BTreeMap::new(),
            direct_dependency_total: 0,
            resolved_dependency_count: DependencyMetric::unsupported(reason.clone()),
            transitive_dependency_count: DependencyMetric::unsupported(reason.clone()),
            duplicate_versions: Vec::new(),
            resolved_dependencies: Vec::new(),
            warnings: vec![reason],
        }
    }
}
