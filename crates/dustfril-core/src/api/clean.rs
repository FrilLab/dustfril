use crate::{
    analyzer, cleaner,
    models::{CleanupPlan, CleanupResult, ScanResult},
};

pub fn build_plan(scan: ScanResult) -> CleanupPlan {
    let analysis = analyzer::analyze(scan);
    cleaner::create_cleanup_plan(analysis)
}

pub fn execute(plan: &CleanupPlan) -> CleanupResult {
    cleaner::execute_cleanup(plan)
}
