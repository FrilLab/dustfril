use crate::{
    analyzer,
    error::DustResult,
    models::{AnalysisResult, ScanResult},
};

pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
    analyzer::Analyzer::analyze(scan_result)
}
