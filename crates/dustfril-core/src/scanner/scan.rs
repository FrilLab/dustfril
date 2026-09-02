use std::path::Path;

use crate::{
    error::{DustError, DustResult},
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

    let directories = match walk_dirs_with_summary(root, &mut result.access_summary) {
        Ok(directories) => directories,
        Err(source @ DustError::InvalidPath(_)) => return Err(source),
        Err(source) => {
            return Err(DustError::ScanAccess {
                source: Box::new(source),
                access_summary: result.access_summary,
            });
        }
    };

    for dir in directories {
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
