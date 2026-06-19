use crate::{
    analyzer, cleaner,
    error::DustResult,
    models::{CleanupPlan, CleanupResult, ScanResult},
};

pub fn build_plan(scan: ScanResult) -> DustResult<CleanupPlan> {
    let analysis = analyzer::analyze(scan)?;
    cleaner::create_cleanup_plan(analysis)
}

pub fn execute(plan: &CleanupPlan) -> DustResult<CleanupResult> {
    cleaner::execute_cleanup(plan)
}
