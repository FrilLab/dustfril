use super::ArtifactLocation;

#[derive(Debug, Default)]
pub struct ScanResult {
    pub artifacts: Vec<ArtifactLocation>,
}
