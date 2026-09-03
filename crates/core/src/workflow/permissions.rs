use crate::models::{
    RiskLevel, Workflow, WorkflowFinding, WorkflowFindingCategory, WorkflowJob,
    WorkflowPermissionLevel, WorkflowPermissions, WorkflowScanNotice,
};

const WRITE_ALL_RULE: &str = "workflow-write-all-permissions";
const BROAD_WRITE_RULE: &str = "workflow-broad-write-permissions";
const NARROW_WRITE_RULE: &str = "workflow-narrow-write-permission";

pub fn analyze(
    workflow: &Workflow,
    findings: &mut Vec<WorkflowFinding>,
    notices: &mut Vec<WorkflowScanNotice>,
) {
    if workflow.jobs.is_empty() {
        analyze_scope(
            workflow,
            None,
            workflow.permissions.as_ref(),
            findings,
            notices,
        );
        return;
    }

    for (job_id, job) in &workflow.jobs {
        analyze_scope(
            workflow,
            Some((job_id, job)),
            job.permissions.as_ref().or(workflow.permissions.as_ref()),
            findings,
            notices,
        );
    }
}

fn analyze_scope(
    workflow: &Workflow,
    job: Option<(&str, &WorkflowJob)>,
    permissions: Option<&WorkflowPermissions>,
    findings: &mut Vec<WorkflowFinding>,
    notices: &mut Vec<WorkflowScanNotice>,
) {
    let Some(permissions) = permissions else {
        notices.push(WorkflowScanNotice {
            workflow_path: workflow.path.clone(),
            job_id: job.map(|(job_id, _)| job_id.to_owned()),
            reason: "Effective token permissions are not declared; repository and event defaults are outside this offline analyzer's scope.".to_owned(),
        });
        return;
    };

    match permissions {
        WorkflowPermissions::WriteAll => findings.push(permission_finding(
            workflow,
            job,
            WRITE_ALL_RULE,
            RiskLevel::High,
            permissions.summary(),
            "The workflow grants write access to every available GitHub token permission, so a compromised step can act broadly with the token.".to_owned(),
        )),
        WorkflowPermissions::Map(permission_map) => {
            if permission_map
                .values()
                .any(|level| matches!(level, WorkflowPermissionLevel::Unknown(_)))
            {
                notices.push(WorkflowScanNotice {
                    workflow_path: workflow.path.clone(),
                    job_id: job.map(|(job_id, _)| job_id.to_owned()),
                    reason: format!(
                        "Permission mapping contains an unsupported level or scope; effective write scope is unresolved ({})",
                        permissions.summary()
                    ),
                });
                return;
            }

            let write_permissions: Vec<_> = permission_map
                .iter()
                .filter(|(_, level)| matches!(level, WorkflowPermissionLevel::Write))
                .map(|(scope, _)| scope.as_str())
                .collect();
            let material_write_permissions: Vec<_> = write_permissions
                .iter()
                .copied()
                .filter(|scope| *scope != "id-token")
                .collect();

            if material_write_permissions.is_empty() {
                return;
            }

            let (rule_id, risk_level, reason) = if material_write_permissions.len() >= 2 {
                (
                    BROAD_WRITE_RULE,
                    RiskLevel::High,
                    format!(
                        "The workflow grants write access to multiple token scopes ({}), broadening what a compromised step can modify.",
                        material_write_permissions.join(", ")
                    ),
                )
            } else {
                (
                    NARROW_WRITE_RULE,
                    RiskLevel::Medium,
                    format!(
                        "The workflow grants write access to the narrow token scope {}; review whether that write capability is required.",
                        material_write_permissions[0]
                    ),
                )
            };

            findings.push(permission_finding(
                workflow,
                job,
                rule_id,
                risk_level,
                permissions.summary(),
                reason,
            ));
        }
        WorkflowPermissions::Unknown(value) => notices.push(WorkflowScanNotice {
            workflow_path: workflow.path.clone(),
            job_id: job.map(|(job_id, _)| job_id.to_owned()),
            reason: format!(
                "Permission declaration {value} is unsupported; effective token scope is unresolved."
            ),
        }),
        WorkflowPermissions::ReadAll | WorkflowPermissions::Empty => {}
    }
}

fn permission_finding(
    workflow: &Workflow,
    job: Option<(&str, &WorkflowJob)>,
    rule_id: &str,
    risk_level: RiskLevel,
    evidence: String,
    reason: String,
) -> WorkflowFinding {
    WorkflowFinding {
        workflow_path: workflow.path.clone(),
        job_id: job.map(|(job_id, _)| job_id.to_owned()),
        step_index: None,
        step_name: None,
        rule_id: rule_id.to_owned(),
        category: WorkflowFindingCategory::TokenPermissions,
        risk_level,
        evidence: Some(evidence),
        reason,
        secret_reference: None,
        exposure_sink: None,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::*;

    fn workflow(
        permissions: Option<WorkflowPermissions>,
        job_permissions: Option<WorkflowPermissions>,
    ) -> Workflow {
        Workflow {
            path: PathBuf::from(".github/workflows/test.yml"),
            name: None,
            permissions,
            env: BTreeMap::new(),
            jobs: BTreeMap::from([(
                "build".to_owned(),
                WorkflowJob {
                    name: None,
                    uses: None,
                    with: BTreeMap::new(),
                    permissions: job_permissions,
                    env: BTreeMap::new(),
                    steps: Vec::new(),
                },
            )]),
        }
    }

    fn map(entries: &[(&str, WorkflowPermissionLevel)]) -> WorkflowPermissions {
        WorkflowPermissions::Map(
            entries
                .iter()
                .map(|(scope, level)| ((*scope).to_owned(), level.clone()))
                .collect(),
        )
    }

    fn findings_for(workflow: &Workflow) -> (Vec<WorkflowFinding>, Vec<WorkflowScanNotice>) {
        let mut findings = Vec::new();
        let mut notices = Vec::new();
        analyze(workflow, &mut findings, &mut notices);
        (findings, notices)
    }

    #[test]
    fn reports_write_all_with_job_context() {
        let (findings, notices) =
            findings_for(&workflow(Some(WorkflowPermissions::WriteAll), None));

        assert!(notices.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, WRITE_ALL_RULE);
        assert_eq!(findings[0].risk_level, RiskLevel::High);
        assert_eq!(findings[0].job_id.as_deref(), Some("build"));
        assert!(findings[0].reason.contains("every available"));
    }

    #[test]
    fn read_all_and_empty_permissions_are_clean() {
        for permissions in [WorkflowPermissions::ReadAll, WorkflowPermissions::Empty] {
            let (findings, notices) = findings_for(&workflow(Some(permissions), None));
            assert!(findings.is_empty());
            assert!(notices.is_empty());
        }
    }

    #[test]
    fn job_permissions_replace_workflow_permissions() {
        let (findings, notices) = findings_for(&workflow(
            Some(WorkflowPermissions::WriteAll),
            Some(WorkflowPermissions::Empty),
        ));

        assert!(findings.is_empty());
        assert!(notices.is_empty());
    }

    #[test]
    fn reports_broad_and_narrow_writes_without_calling_them_malware() {
        let (broad_findings, _) = findings_for(&workflow(
            None,
            Some(map(&[
                ("contents", WorkflowPermissionLevel::Write),
                ("pull-requests", WorkflowPermissionLevel::Write),
            ])),
        ));
        assert_eq!(broad_findings[0].rule_id, BROAD_WRITE_RULE);
        assert_eq!(broad_findings[0].risk_level, RiskLevel::High);
        assert!(
            broad_findings[0]
                .evidence
                .as_deref()
                .unwrap()
                .contains("contents: write")
        );

        let (narrow_findings, _) = findings_for(&workflow(
            None,
            Some(map(&[("contents", WorkflowPermissionLevel::Write)])),
        ));
        assert_eq!(narrow_findings[0].rule_id, NARROW_WRITE_RULE);
        assert_eq!(narrow_findings[0].risk_level, RiskLevel::Medium);
    }

    #[test]
    fn id_token_write_alone_is_not_reported_as_malicious() {
        let (findings, notices) = findings_for(&workflow(
            None,
            Some(map(&[("id-token", WorkflowPermissionLevel::Write)])),
        ));

        assert!(findings.is_empty());
        assert!(notices.is_empty());
    }

    #[test]
    fn undeclared_or_unknown_permissions_are_explicitly_partial() {
        let (findings, notices) = findings_for(&workflow(None, None));
        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("not declared"));

        let (findings, notices) = findings_for(&workflow(
            None,
            Some(WorkflowPermissions::Unknown("future-all".to_owned())),
        ));
        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("unsupported"));
    }

    #[test]
    fn unsupported_permission_scopes_are_partial_and_not_counted_as_writes() {
        let (findings, notices) = findings_for(&workflow(
            None,
            Some(map(&[(
                "custom-scope",
                WorkflowPermissionLevel::Unknown(
                    "unsupported permission scope: custom-scope".to_owned(),
                ),
            )])),
        ));

        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("unsupported level or scope"));
        assert!(notices[0].reason.contains("custom-scope"));
    }
}
