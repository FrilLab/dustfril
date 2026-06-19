use std::path::Path;

use crate::{error::DustResult, models::ScanResult};

use super::{
    project,
    rust::{git, registry, target},
};

// Scan a single Rust project for artifacts.
pub fn scan_project(root: &Path) -> DustResult<ScanResult> {
    let mut result = ScanResult::default();

    if !project::is_cargo_project(root) {
        return Ok(result);
    }

    if let Some(target) = target::detect(root) {
        result.artifacts.push(target);
    }

    Ok(result)
}

// Recursively scan for Rust projects and their artifacts.
pub fn scan_workspace(root: &Path) -> DustResult<ScanResult> {
    let mut result = ScanResult::default();

    let projects = project::find_projects(root);

    for project in projects {
        let project_result = scan_project(&project.root)?;

        result.artifacts.extend(project_result.artifacts);
    }

    Ok(result)
}

// Global artifacts that are not tied to a specific project, like Cargo registry and Git repositories.
pub fn scan_global() -> DustResult<ScanResult> {
    let mut result = ScanResult::default();

    if let Some(registry) = registry::detect() {
        result.artifacts.push(registry);
    }

    if let Some(git) = git::detect() {
        result.artifacts.push(git);
    }

    Ok(result)
}

// pub fn scan(root: &Path) -> ScanResult {
//     let mut result = scan_workspace(root);

//     result.artifacts.extend(scan_global().artifacts);

//     result
// }
