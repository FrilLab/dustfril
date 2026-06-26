use std::path::Path;

use crate::{
    error::DustResult,
    fs::walk_dirs,
    models::{Artifact, Ecosystem, ScanResult},
    scanner::detector::{self},
};

pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    let detectors = detector::detectors(ecosystems);

    let mut result = ScanResult::default();

    for dir in walk_dirs(root) {
        for detector in &detectors {
            if detector.detect(&dir) {
                result
                    .artifacts
                    .push(Artifact::new(dir.clone(), detector.ecosystem()));
                break;
            }
        }
    }

    Ok(result)
}
