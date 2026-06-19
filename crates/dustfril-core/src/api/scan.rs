use std::path::Path;

use crate::detector;

pub fn scan(root: &Path, global: bool) -> crate::models::ScanResult {
    if global {
        detector::scan_global()
    } else {
        detector::scan_workspace(root)
    }
}
