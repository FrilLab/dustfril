use std::path::Path;

use crate::{detector, error::DustResult, models::ScanResult};

pub fn scan(root: &Path, global: bool) -> DustResult<ScanResult> {
    if global {
        detector::scan_global()
    } else {
        detector::scan_workspace(root)
    }
}
