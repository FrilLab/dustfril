use super::ArtifactAnalysis;

/// Total Analysis Results
#[derive(Debug, Default)]
pub struct AnalysisResult {
    pub artifacts: Vec<ArtifactAnalysis>,
    pub total_size_bytes: u64,
}
