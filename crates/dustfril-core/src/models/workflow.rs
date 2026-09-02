use std::{collections::BTreeMap, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::RiskLevel;

/// YAML values retained from workflow environment and action input fields.
pub type WorkflowValue = Value;

/// A parsed GitHub Actions workflow file.
///
/// The model intentionally contains only local workflow structure. It does
/// not resolve expressions, invoke actions, or evaluate runner state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workflow {
    /// The local path of the workflow file.
    pub path: PathBuf,
    /// The optional display name declared by the workflow.
    pub name: Option<String>,
    /// The workflow-level token permission declaration, if present.
    pub permissions: Option<WorkflowPermissions>,
    /// Workflow-level environment variables.
    pub env: BTreeMap<String, WorkflowValue>,
    /// Jobs keyed by their stable workflow job identifiers.
    pub jobs: BTreeMap<String, WorkflowJob>,
}

/// A parsed GitHub Actions job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowJob {
    /// The optional display name declared by the job.
    pub name: Option<String>,
    /// A reusable workflow reference from jobs.<id>.uses, if present.
    pub uses: Option<String>,
    /// Inputs for a reusable workflow invocation.
    pub with: BTreeMap<String, WorkflowValue>,
    /// A job-level token permission declaration, if present.
    pub permissions: Option<WorkflowPermissions>,
    /// Job-level environment variables.
    pub env: BTreeMap<String, WorkflowValue>,
    /// Steps in their source order.
    pub steps: Vec<WorkflowStep>,
}

/// A parsed GitHub Actions step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStep {
    /// The optional display name declared by the step.
    pub name: Option<String>,
    /// The optional stable step identifier declared by id.
    pub id: Option<String>,
    /// An action reference from uses, if this is an action step.
    pub uses: Option<String>,
    /// Structured action inputs from with.
    pub with: BTreeMap<String, WorkflowValue>,
    /// Step-level environment variables.
    pub env: BTreeMap<String, WorkflowValue>,
    /// Shell script content from run, if this is a shell step.
    pub run: Option<String>,
}

/// The supported forms of a GitHub Actions permissions declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowPermissions {
    /// Every available permission is granted read access.
    ReadAll,
    /// Every available permission is granted write access.
    WriteAll,
    /// All token permissions are disabled.
    Empty,
    /// Explicitly declared individual permission scopes.
    Map(BTreeMap<String, WorkflowPermissionLevel>),
    /// A syntactically valid YAML value whose permission semantics are not
    /// supported by this analyzer.
    Unknown(String),
}

/// An individual GitHub Actions token permission level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowPermissionLevel {
    None,
    Read,
    Write,
    /// A value outside the supported GitHub permission levels.
    Unknown(String),
}

impl WorkflowPermissions {
    /// Returns a deterministic human-readable summary suitable for finding
    /// evidence and CLI output.
    pub fn summary(&self) -> String {
        match self {
            Self::ReadAll => "read-all".to_owned(),
            Self::WriteAll => "write-all".to_owned(),
            Self::Empty => "{}".to_owned(),
            Self::Map(permissions) => permissions
                .iter()
                .map(|(scope, level)| format!("{scope}: {}", level.as_str()))
                .collect::<Vec<_>>()
                .join(", "),
            Self::Unknown(value) => value.clone(),
        }
    }
}

impl WorkflowPermissionLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Read => "read",
            Self::Write => "write",
            Self::Unknown(value) => value,
        }
    }
}

/// The security category attached to a workflow finding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowFindingCategory {
    SuspiciousCommand,
    TokenPermissions,
}

impl fmt::Display for WorkflowFindingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SuspiciousCommand => f.write_str("Suspicious command"),
            Self::TokenPermissions => f.write_str("Token permissions"),
        }
    }
}

/// A structured finding produced by workflow security analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowFinding {
    /// The workflow file containing the finding.
    pub workflow_path: PathBuf,
    /// The job identifier, when the finding is scoped to a job.
    pub job_id: Option<String>,
    /// The zero-based step index, when the finding concerns a run step.
    pub step_index: Option<usize>,
    /// The optional display name of the affected step.
    pub step_name: Option<String>,
    /// Stable rule identifier.
    pub rule_id: String,
    /// Broad finding category.
    pub category: WorkflowFindingCategory,
    /// Severity assigned by the conservative offline rule set.
    pub risk_level: RiskLevel,
    /// Relevant command or permission summary.
    pub evidence: Option<String>,
    /// Explanation of the exposure and why it was reported.
    pub reason: String,
}

/// A partial-analysis notice that does not claim a workflow is safe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowScanNotice {
    pub workflow_path: PathBuf,
    pub job_id: Option<String>,
    pub reason: String,
}

/// Complete result of the local, read-only workflow security scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkflowScanReport {
    /// Workflows successfully discovered and parsed.
    pub workflows: Vec<Workflow>,
    /// Security findings produced from the parsed workflow model.
    pub findings: Vec<WorkflowFinding>,
    /// Unsupported or unresolved semantics that make the result partial.
    pub notices: Vec<WorkflowScanNotice>,
}

impl WorkflowScanReport {
    /// Returns whether any part of the report requires a follow-up review.
    pub fn is_partial(&self) -> bool {
        !self.notices.is_empty()
    }
}
