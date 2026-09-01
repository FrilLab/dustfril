use std::path::Path;

use crate::{
    error::DustResult,
    fs::walk_dirs,
    models::{Ecosystem, ScanResult},
    scanner::detector::{self},
};

pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    let detectors = detector::select_detectors(ecosystems);

    let mut result = ScanResult::default();

    for dir in walk_dirs(root)? {
        for detector in &detectors {
            if !detector.matches(&dir) {
                continue;
            }

            result.artifacts.extend(detector.artifacts(&dir));
        }
    }

    Ok(result)
}
