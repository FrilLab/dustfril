use crate::{
    analyzer,
    error::DustResult,
    models::{AnalysisResult, ScanResult},
};

/// Analyzes scanned artifacts and returns size, age, and cleanup hints.
pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
    analyzer::Analyzer::analyze(scan_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Artifact, Ecosystem};

    #[test]
    fn analyze_aggregates_size_from_scan_result() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("artifact.bin"), b"hello").unwrap();

        let scan_result = ScanResult {
            artifacts: vec![Artifact::new(temp_dir.path().to_path_buf(), Ecosystem::Rust)],
        };

        let result = analyze(scan_result).unwrap();

        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.total_size_bytes, 5);
        assert_eq!(result.artifacts[0].artifact.path, temp_dir.path());
    }
}
