use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A developer cache whose location can be resolved from local evidence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum DeveloperCacheKind {
    CargoRegistry,
    CargoGit,
}

impl DeveloperCacheKind {
    pub fn tool_name(self) -> &'static str {
        "Cargo"
    }

    pub fn cache_name(self) -> &'static str {
        match self {
            Self::CargoRegistry => "Registry cache",
            Self::CargoGit => "Git cache",
        }
    }

    pub(crate) fn directory_name(self) -> &'static str {
        match self {
            Self::CargoRegistry => "registry",
            Self::CargoGit => "git",
        }
    }
}

/// Whether Core can safely resolve and clean this provider.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeveloperCacheSupportState {
    Supported,
}

/// Scope of a cache relative to projects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeveloperCacheScope {
    Global,
}

/// A discovered developer cache and the evidence used to identify it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperCache {
    pub kind: DeveloperCacheKind,
    pub tool: String,
    pub name: String,
    pub path: PathBuf,
    pub scope: DeveloperCacheScope,
    pub size_bytes: u64,
    pub evidence: String,
    pub support_state: DeveloperCacheSupportState,
    pub cleanup_impact: String,
    pub measurement_failures: u64,
    pub failure_samples: Vec<String>,
}

/// Local cache roots discovered by Core. Warnings are bounded diagnostics for
/// providers or roots that could not be inspected safely.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperCacheDiscovery {
    pub caches: Vec<DeveloperCache>,
    pub warnings: Vec<String>,
}

/// Identity supplied when a user selects one discovered cache root.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeveloperCacheSelection {
    pub kind: DeveloperCacheKind,
    pub path: PathBuf,
}
