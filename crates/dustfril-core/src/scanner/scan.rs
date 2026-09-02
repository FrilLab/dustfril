use std::path::Path;

use crate::{
    error::DustResult,
    fs::walk_dirs_with_summary,
    models::{Ecosystem, ScanAccessSummary, ScanResult},
    scanner::detector::{self},
};

pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    let detectors = detector::select_detectors(ecosystems);

    let mut result = ScanResult {
        access_summary: ScanAccessSummary::new(root),
        ..ScanResult::default()
    };

    for dir in walk_dirs_with_summary(root, &mut result.access_summary)? {
        for detector in &detectors {
            if !detector.matches_with_summary(&dir, &mut result.access_summary) {
                continue;
            }

            let artifacts = detector.artifacts_with_summary(&dir, Some(&mut result.access_summary));
            for _ in &artifacts {
                result.access_summary.record_artifact_candidate();
            }
            result.artifacts.extend(artifacts);
        }
    }

    Ok(result)
}
