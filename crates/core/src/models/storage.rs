use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Ecosystem;

/// Capacity statistics for the filesystem containing a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VolumeStorage {
    /// Total capacity reported by the filesystem.
    pub total_bytes: u64,
    /// Bytes considered used, derived as `total_bytes - available_bytes`.
    pub used_bytes: u64,
    /// Bytes available to the current user according to the filesystem.
    pub available_bytes: u64,
}

impl VolumeStorage {
    /// Builds internally consistent volume statistics from filesystem values.
    ///
    /// A filesystem should never report more available bytes than total bytes,
    /// but clamping that value keeps the serialized relationship safe if an
    /// unusual or inconsistent filesystem response is encountered.
    pub(crate) fn from_filesystem_values(total_bytes: u64, available_bytes: u64) -> Self {
        let available_bytes = available_bytes.min(total_bytes);
        let used_bytes = total_bytes.saturating_sub(available_bytes);

        Self {
            total_bytes,
            used_bytes,
            available_bytes,
        }
    }
}

/// Scope and measured categories represented by the developer-storage value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperStorageSummary {
    /// Sum of all normalized, measured scanner-owned artifacts.
    pub measured_bytes: u64,
    /// Sum of artifacts currently recommended for cleanup.
    pub recommended_bytes: u64,
    /// The selected workspace represented by `measured_bytes`.
    pub scope_path: PathBuf,
    /// Ecosystems represented by the analyzed artifact set.
    pub categories: Vec<Ecosystem>,
}

/// Storage context shown by the Overview.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageSummary {
    /// Capacity of the volume containing the selected workspace.
    pub volume: VolumeStorage,
    /// Measured development storage and its explicit scope.
    pub developer_storage: DeveloperStorageSummary,
    /// Whether the supplied workspace analysis had filesystem access failures.
    pub partial: bool,
    /// Human-readable coverage warnings for a partial analysis.
    pub warnings: Vec<String>,
}

impl StorageSummary {
    /// Returns measured development storage as a percentage of used volume
    /// storage. A zero-used volume has no meaningful share percentage.
    pub fn detected_share_percent(&self) -> Option<f64> {
        (self.volume.used_bytes > 0).then(|| {
            self.developer_storage.measured_bytes as f64 / self.volume.used_bytes as f64 * 100.0
        })
    }
}
