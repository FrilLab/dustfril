use crate::{
    analyzer,
    error::DustResult,
    models::{AnalysisResult, ScanResult},
};

pub fn analyze(scan_result: ScanResult) -> DustResult<AnalysisResult> {
    analyzer::analyze(scan_result)
}
