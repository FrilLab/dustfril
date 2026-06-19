use serde::{Deserialize, Serialize};

use crate::models::Artifact;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanResult {
    pub artifacts: Vec<Artifact>,
}
