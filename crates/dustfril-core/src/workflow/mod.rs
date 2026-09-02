//! Local, read-only GitHub Actions workflow security analysis.
//!
//! Parsing is kept separate from the command and permission rules so later
//! workflow checks, including secret-flow analysis, can consume the same
//! parsed model without reparsing YAML.

mod commands;
mod parser;
mod permissions;

use crate::{error::DustResult, models::WorkflowScanReport};

/// Discovers and parses only .github/workflows/*.yml and *.yaml files.
pub fn parse(root: &std::path::Path) -> DustResult<Vec<crate::models::Workflow>> {
    parser::discover_and_parse(root)
}

/// Runs all workflow checks supported by this offline analyzer.
pub fn scan(root: &std::path::Path) -> DustResult<WorkflowScanReport> {
    let workflows = parse(root)?;
    let mut report = WorkflowScanReport {
        workflows,
        ..WorkflowScanReport::default()
    };

    for workflow in &report.workflows {
        commands::analyze(workflow, &mut report.findings);
        permissions::analyze(workflow, &mut report.findings, &mut report.notices);
    }

    Ok(report)
}
