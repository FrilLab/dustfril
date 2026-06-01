use std::path::Path;

use crate::models::ScanResult;

use super::{cargo_project, git, registry, target};

pub fn scan_project(root: &Path) -> ScanResult {
    let mut result = ScanResult::default();

    if !cargo_project::is_cargo_project(root) {
        return result;
    }

    if let Some(target) = target::detect(root) {
        result.artifacts.push(target);
    }

    result
}

pub fn scan_global() -> ScanResult {
    let mut result = ScanResult::default();

    if let Some(registry) = registry::detect() {
        result.artifacts.push(registry);
    }

    if let Some(git) = git::detect() {
        result.artifacts.push(git);
    }

    result
}

pub fn scan(root: &Path) -> ScanResult {
    let mut result = scan_project(root);

    result.artifacts.extend(scan_global().artifacts);

    result
}
