use crate::{analyzer, models::ScanResult};

pub fn analyze(scan_result: ScanResult) -> crate::models::AnalysisResult {
    analyzer::analyze(scan_result)
}
