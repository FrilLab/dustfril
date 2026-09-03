//! Conservative direct GitHub Actions secret-flow checks.
//!
//! This is deliberately bounded static analysis. It recognizes only GitHub
//! `secrets.NAME` expressions, one-hop workflow/job/step environment aliases,
//! and the command forms listed below. It does not evaluate expressions,
//! execute shell commands, inspect action implementations, or follow values
//! through scripts and files.
//!
//! Supported sinks:
//!
//! - `echo` and `printf` arguments, treated as stdout/logging output;
//! - literal HTTP(S) URLs used with `curl` data or header arguments.
//!
//! For `curl`, the supported request options are `-d`/`--data`,
//! `--data-raw`, `--data-binary`, `--data-urlencode`, `-F`/`--form`,
//! `--form-string`, `-H`/`--header`, and `--proxy-header`. The URL must be a
//! literal HTTP(S) argument in the same command. Other shell and network
//! forms remain unresolved rather than being reported as proven exposure.

use std::collections::{BTreeMap, BTreeSet};

use serde_yaml::Value;

use crate::{
    audit_tool::command::{self, TokenPart, TokenQuote},
    models::{
        RiskLevel, Workflow, WorkflowExposureSink, WorkflowFinding, WorkflowFindingCategory,
        WorkflowJob, WorkflowScanNotice, WorkflowStep,
    },
};

const DIRECT_SECRET_EXPOSURE_RULE: &str = "workflow-direct-secret-exposure";

type SecretReferences = BTreeSet<String>;
type SecretEnvironment = BTreeMap<String, SecretReferences>;

struct StepContext<'a> {
    workflow: &'a Workflow,
    job_id: &'a str,
    step_index: usize,
    step: &'a WorkflowStep,
}

/// Finds direct secret-to-sink flows in parsed workflow steps.
pub fn analyze(
    workflow: &Workflow,
    findings: &mut Vec<WorkflowFinding>,
    notices: &mut Vec<WorkflowScanNotice>,
) {
    for (job_id, job) in &workflow.jobs {
        for (step_index, step) in job.steps.iter().enumerate() {
            analyze_step(workflow, job_id, job, step_index, step, findings, notices);
        }
    }
}

fn analyze_step(
    workflow: &Workflow,
    job_id: &str,
    job: &WorkflowJob,
    step_index: usize,
    step: &WorkflowStep,
    findings: &mut Vec<WorkflowFinding>,
    notices: &mut Vec<WorkflowScanNotice>,
) {
    let Some(run) = step.run.as_deref() else {
        return;
    };

    let environment = effective_secret_environment(workflow, job, step);
    let segments = command::parse_preserving_case(run);
    let mut used_references = SecretReferences::new();
    let mut emitted = BTreeSet::new();
    let context = StepContext {
        workflow,
        job_id,
        step_index,
        step,
    };

    for segment in &segments {
        for parts in &segment.token_parts {
            used_references.extend(references_in_token_parts(parts, &environment));
        }

        let Some(executable) = command::executable_preserving_case(&segment.tokens) else {
            continue;
        };
        let arguments = command::arguments_preserving_case(&segment.tokens);
        let argument_parts =
            command::argument_parts_preserving_case(&segment.tokens, &segment.token_parts);

        if executable.eq_ignore_ascii_case("echo")
            || (executable.eq_ignore_ascii_case("printf") && !printf_assigns_output(arguments))
        {
            let references = references_in_arguments(argument_parts, &environment);
            emit_findings(
                &context,
                WorkflowExposureSink::Stdout,
                references,
                &mut emitted,
                findings,
            );
        } else if executable.eq_ignore_ascii_case("curl") {
            let references = curl_request_references(arguments, argument_parts, &environment);
            emit_findings(
                &context,
                WorkflowExposureSink::NetworkRequest,
                references,
                &mut emitted,
                findings,
            );
        }
    }

    // A known secret used in an unsupported flow is intentionally a notice,
    // not a finding. This keeps the result partial without claiming either
    // that an unknown script is safe or that it definitely leaked a secret.
    let mut unresolved = used_references;
    for (_, _, reference, _) in &emitted {
        unresolved.remove(reference);
    }
    for reference in &unresolved {
        notices.push(WorkflowScanNotice {
            workflow_path: workflow.path.clone(),
            job_id: Some(job_id.to_owned()),
            reason: format!(
                "Secret reference {reference} is used in step {} outside the supported direct stdout/network sink forms; its flow is unresolved.",
                step_index
            ),
        });
    }

    // A secret in the effective environment can be read by an arbitrary
    // script even when the script does not spell out `$NAME`; keep that case
    // partial unless a supported sink was proven.
    for (name, references) in environment {
        for reference in references {
            if !is_emitted(&emitted, &reference) && !unresolved.contains(&reference) {
                notices.push(WorkflowScanNotice {
                    workflow_path: workflow.path.clone(),
                    job_id: Some(job_id.to_owned()),
                    reason: format!(
                        "Secret reference {reference} is available as environment variable {name} in step {step_index}, but no supported direct sink was proven; indirect script flow is unresolved."
                    ),
                });
            }
        }
    }
}

fn emit_findings(
    context: &StepContext<'_>,
    sink: WorkflowExposureSink,
    references: SecretReferences,
    emitted: &mut BTreeSet<(String, usize, String, u8)>,
    findings: &mut Vec<WorkflowFinding>,
) {
    for reference in references {
        let key = (
            context.job_id.to_owned(),
            context.step_index,
            reference.clone(),
            sink_key(sink),
        );
        if !emitted.insert(key) {
            continue;
        }

        findings.push(WorkflowFinding {
            workflow_path: context.workflow.path.clone(),
            job_id: Some(context.job_id.to_owned()),
            step_index: Some(context.step_index),
            step_name: context.step.name.clone(),
            rule_id: DIRECT_SECRET_EXPOSURE_RULE.to_owned(),
            category: WorkflowFindingCategory::SecretExposure,
            risk_level: RiskLevel::High,
            // Keep diagnostics structural and sanitized. In particular, do
            // not persist the raw run command alongside a secret finding.
            evidence: Some(format!(
                "supported {sink} sink; secret reference: {reference}"
            )),
            reason: format!(
                "GitHub secret reference {reference} is passed directly to a supported {sink} sink."
            ),
            secret_reference: Some(reference),
            exposure_sink: Some(sink),
        });
    }
}

fn sink_key(sink: WorkflowExposureSink) -> u8 {
    match sink {
        WorkflowExposureSink::Stdout => 0,
        WorkflowExposureSink::NetworkRequest => 1,
    }
}

fn is_emitted(emitted: &BTreeSet<(String, usize, String, u8)>, reference: &str) -> bool {
    emitted.iter().any(|(_, _, name, _)| name == reference)
}

fn effective_secret_environment(
    workflow: &Workflow,
    job: &WorkflowJob,
    step: &WorkflowStep,
) -> SecretEnvironment {
    let mut environment = SecretEnvironment::new();
    apply_environment(&mut environment, &workflow.env);
    apply_environment(&mut environment, &job.env);
    apply_environment(&mut environment, &step.env);
    environment
}

fn apply_environment(environment: &mut SecretEnvironment, values: &BTreeMap<String, Value>) {
    for (name, value) in values {
        let references = secret_references_in_value(value);
        if references.is_empty() {
            // An env value at a narrower scope replaces the broader value,
            // including when the replacement is non-secret or unsupported.
            environment.remove(name);
        } else {
            environment.insert(name.clone(), references);
        }
    }
}

fn secret_references_in_value(value: &Value) -> SecretReferences {
    value
        .as_str()
        .map(secret_references_in_text)
        .unwrap_or_default()
}

fn references_in_arguments(
    arguments: &[Vec<TokenPart>],
    environment: &SecretEnvironment,
) -> SecretReferences {
    arguments
        .iter()
        .flat_map(|parts| references_in_token_parts(parts, environment))
        .collect()
}

fn references_in_token_parts(
    parts: &[TokenPart],
    environment: &SecretEnvironment,
) -> SecretReferences {
    let mut references = secret_references_in_text(
        &parts
            .iter()
            .map(|part| part.text.as_str())
            .collect::<String>(),
    );
    for part in parts {
        if part.quote != TokenQuote::Single {
            for variable in shell_variables_in_text(&part.text) {
                if let Some(variable_references) = environment.get(&variable) {
                    references.extend(variable_references.iter().cloned());
                }
            }
        }
    }
    references
}

fn secret_references_in_text(text: &str) -> SecretReferences {
    let mut references = SecretReferences::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("${{") {
        let tail = &remaining[start + 3..];
        let Some(end) = tail.find("}}") else {
            break;
        };
        let expression = tail[..end].trim();
        if let Some(name) = expression.strip_prefix("secrets.")
            && is_secret_name(name)
        {
            references.insert(name.to_owned());
        }
        remaining = &tail[end + 2..];
    }

    references
}

fn shell_variables_in_text(text: &str) -> SecretReferences {
    let mut variables = SecretReferences::new();
    let characters: Vec<_> = text.char_indices().collect();
    let mut index = 0;

    while index < characters.len() {
        let (position, character) = characters[index];
        if character != '$' || (position > 0 && text.as_bytes().get(position - 1) == Some(&b'\\')) {
            index += 1;
            continue;
        }

        if characters
            .get(index + 1)
            .is_some_and(|(_, next)| *next == '{')
        {
            let Some(closing_index) =
                (index + 2..characters.len()).find(|candidate| characters[*candidate].1 == '}')
            else {
                index += 1;
                continue;
            };
            let name_start = index + 2;
            let name: String = characters[name_start..closing_index]
                .iter()
                .map(|(_, character)| *character)
                .collect();
            if is_shell_variable_name(&name) {
                variables.insert(name);
            }
            index = closing_index + 1;
            continue;
        }

        let Some((_, next)) = characters.get(index + 1) else {
            index += 1;
            continue;
        };
        if !is_shell_variable_start(*next) {
            index += 1;
            continue;
        }

        let start = index + 1;
        let mut end = start + 1;
        while end < characters.len() && is_shell_variable_part(characters[end].1) {
            end += 1;
        }
        variables.insert(
            characters[start..end]
                .iter()
                .map(|(_, character)| *character)
                .collect(),
        );
        index = end;
    }

    variables
}

fn is_secret_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters.next().is_some_and(is_shell_variable_start) && characters.all(is_shell_variable_part)
}

fn is_shell_variable_name(name: &str) -> bool {
    is_secret_name(name)
}

fn is_shell_variable_start(character: char) -> bool {
    character == '_' || character.is_ascii_alphabetic()
}

fn is_shell_variable_part(character: char) -> bool {
    is_shell_variable_start(character) || character.is_ascii_digit()
}

fn curl_request_references(
    arguments: &[String],
    argument_parts: &[Vec<TokenPart>],
    environment: &SecretEnvironment,
) -> SecretReferences {
    let mut references = SecretReferences::new();
    let mut has_literal_url = false;
    let mut index = 0;

    while index < arguments.len() {
        let argument = &arguments[index];

        if is_http_url(argument) {
            has_literal_url = true;
            index += 1;
            continue;
        }

        if let Some((value_index, consumed)) = curl_option_value(arguments, index) {
            let value = &arguments[value_index];
            if is_url_option(argument) && is_http_url(value) {
                has_literal_url = true;
            } else if is_secret_sink_option(argument) {
                references.extend(references_in_token_parts(
                    &argument_parts[value_index],
                    environment,
                ));
            }
            index += consumed;
            continue;
        }

        if let Some(consumed) = curl_option_consumed(argument) {
            index += consumed;
            continue;
        }

        index += 1;
    }

    if has_literal_url {
        references
    } else {
        SecretReferences::new()
    }
}

fn curl_option_value(arguments: &[String], index: usize) -> Option<(usize, usize)> {
    let argument = arguments.get(index)?.as_str();

    for option in [
        "--data",
        "--data-raw",
        "--data-binary",
        "--data-urlencode",
        "--form",
        "--form-string",
        "--header",
        "--proxy-header",
        "--url",
    ] {
        if argument == option {
            return arguments.get(index + 1).map(|_| (index + 1, 2));
        }
        if argument.starts_with(&format!("{option}=")) {
            return Some((index, 1));
        }
    }

    for option in ["-d", "-F", "-H"] {
        if argument == option {
            return arguments.get(index + 1).map(|_| (index + 1, 2));
        }
        if argument
            .strip_prefix(option)
            .is_some_and(|value| !value.is_empty())
        {
            return Some((index, 1));
        }
    }

    None
}

fn is_secret_sink_option(argument: &str) -> bool {
    argument == "-d"
        || argument == "-F"
        || argument == "-H"
        || argument.starts_with("-d")
        || argument.starts_with("-F")
        || argument.starts_with("-H")
        || [
            "--data",
            "--data-raw",
            "--data-binary",
            "--data-urlencode",
            "--form",
            "--form-string",
            "--header",
            "--proxy-header",
        ]
        .iter()
        .any(|option| argument == *option || argument.starts_with(&format!("{option}=")))
}

fn is_url_option(argument: &str) -> bool {
    argument == "--url" || argument.starts_with("--url=")
}

fn curl_option_consumed(argument: &str) -> Option<usize> {
    let long_options = [
        "--cacert",
        "--cert",
        "--config",
        "--connect-to",
        "--cookie",
        "--cookie-jar",
        "--interface",
        "--key",
        "--max-time",
        "--output",
        "--proxy",
        "--referer",
        "--request",
        "--resolve",
        "--retry",
        "--upload-file",
        "--user",
    ];
    if long_options
        .iter()
        .any(|option| argument == *option || argument.starts_with(&format!("{option}=")))
    {
        return Some(if argument.contains('=') { 1 } else { 2 });
    }

    let short_options = ["-b", "-c", "-e", "-K", "-m", "-o", "-T", "-u", "-x", "-X"];
    short_options.iter().find_map(|option| {
        if argument == *option {
            Some(2)
        } else if argument
            .strip_prefix(option)
            .is_some_and(|value| !value.is_empty())
        {
            Some(1)
        } else {
            None
        }
    })
}

fn printf_assigns_output(arguments: &[String]) -> bool {
    arguments.first().is_some_and(|argument| argument == "-v")
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::*;

    fn workflow_with_steps(
        workflow_env: &[(&str, &str)],
        job_env: &[(&str, &str)],
        steps: &[(&str, &[(&str, &str)])],
    ) -> Workflow {
        Workflow {
            path: PathBuf::from(".github/workflows/test.yml"),
            name: Some("Test".to_owned()),
            permissions: None,
            env: yaml_map(workflow_env),
            jobs: BTreeMap::from([(
                "build".to_owned(),
                WorkflowJob {
                    name: None,
                    uses: None,
                    with: BTreeMap::new(),
                    permissions: None,
                    env: yaml_map(job_env),
                    steps: steps
                        .iter()
                        .map(|(run, env)| WorkflowStep {
                            name: Some("Test step".to_owned()),
                            id: None,
                            uses: None,
                            with: BTreeMap::new(),
                            env: yaml_map(env),
                            run: Some((*run).to_owned()),
                        })
                        .collect(),
                },
            )]),
        }
    }

    fn yaml_map(entries: &[(&str, &str)]) -> BTreeMap<String, Value> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), Value::String((*value).to_owned())))
            .collect()
    }

    fn findings_for(workflow: &Workflow) -> (Vec<WorkflowFinding>, Vec<WorkflowScanNotice>) {
        let mut findings = Vec::new();
        let mut notices = Vec::new();
        analyze(workflow, &mut findings, &mut notices);
        (findings, notices)
    }

    fn finding_names(findings: &[WorkflowFinding]) -> Vec<&str> {
        findings
            .iter()
            .map(|finding| finding.secret_reference.as_deref().unwrap())
            .collect()
    }

    #[test]
    fn detects_echo_and_printf_secret_references_with_context() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[
                (
                    "echo '${{ secrets.MY_TOKEN }}' '${{ secrets.SECOND_TOKEN }}'",
                    &[],
                ),
                ("printf '%s\\n' ${{ secrets.API_KEY }}", &[]),
            ],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(notices.is_empty());
        assert_eq!(findings.len(), 3);
        assert_eq!(
            finding_names(&findings),
            ["MY_TOKEN", "SECOND_TOKEN", "API_KEY"]
        );
        assert!(findings.iter().all(|finding| {
            let serialized = serde_json::to_string(finding).unwrap();
            finding.rule_id == DIRECT_SECRET_EXPOSURE_RULE
                && finding.category == WorkflowFindingCategory::SecretExposure
                && finding.risk_level == RiskLevel::High
                && finding.exposure_sink == Some(WorkflowExposureSink::Stdout)
                && !serialized.contains("${{")
                && finding
                    .evidence
                    .as_deref()
                    .unwrap()
                    .contains("secret reference")
        }));
    }

    #[test]
    fn detects_supported_curl_request_arguments_but_not_unrelated_secret_text() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[(
                "curl --data '${{ secrets.MY_TOKEN }}' https://example.invalid/upload\ncurl --header \"Authorization: ${{ secrets.HEADER_TOKEN }}\" https://example.invalid/upload\ncurl --output '${{ secrets.NOT_A_BODY }}' https://example.invalid/upload",
                &[],
            )],
        );

        let (findings, notices) = findings_for(&workflow);

        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("NOT_A_BODY"));
        assert_eq!(findings.len(), 2);
        assert_eq!(finding_names(&findings), ["MY_TOKEN", "HEADER_TOKEN"]);
        assert!(
            findings
                .iter()
                .all(|finding| finding.exposure_sink == Some(WorkflowExposureSink::NetworkRequest))
        );
    }

    #[test]
    fn consumes_attached_curl_options_without_skipping_following_sink_options() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[(
                "curl -XPOST -H \"$TOKEN\" https://example.invalid/upload",
                &[("TOKEN", "${{ secrets.API_TOKEN }}")],
            )],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(notices.is_empty());
        assert_eq!(finding_names(&findings), ["API_TOKEN"]);
        assert_eq!(
            findings[0].exposure_sink,
            Some(WorkflowExposureSink::NetworkRequest)
        );
    }

    #[test]
    fn does_not_report_single_quoted_shell_variable_as_exposure() {
        let workflow = workflow_with_steps(
            &[("TOKEN", "${{ secrets.API_TOKEN }}")],
            &[],
            &[("echo '$TOKEN'", &[])],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("API_TOKEN"));
    }

    #[test]
    fn does_not_report_printf_assignment_as_stdout_exposure() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[(
                "printf -v copy '%s' \"$TOKEN\"",
                &[("TOKEN", "${{ secrets.API_TOKEN }}")],
            )],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("API_TOKEN"));
    }

    #[test]
    fn propagates_workflow_job_and_step_environment_with_narrower_precedence() {
        let workflow = workflow_with_steps(
            &[("TOKEN", "${{ secrets.WORKFLOW_TOKEN }}")],
            &[("TOKEN", "${{ secrets.JOB_TOKEN }}")],
            &[
                ("echo \"$TOKEN\"", &[]),
                ("echo \"$TOKEN\"", &[("TOKEN", "${{ secrets.STEP_TOKEN }}")]),
                ("echo \"$TOKEN\"", &[("TOKEN", "safe-value")]),
            ],
        );

        let (findings, notices) = findings_for(&workflow);

        assert_eq!(findings.len(), 2);
        assert_eq!(finding_names(&findings), ["JOB_TOKEN", "STEP_TOKEN"]);
        assert!(notices.is_empty());
    }

    #[test]
    fn propagates_workflow_environment_when_job_and_step_do_not_override_it() {
        let workflow = workflow_with_steps(
            &[("TOKEN", "${{ secrets.WORKFLOW_TOKEN }}")],
            &[],
            &[("echo \"${TOKEN}\"", &[])],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(notices.is_empty());
        assert_eq!(finding_names(&findings), ["WORKFLOW_TOKEN"]);
    }

    #[test]
    fn does_not_report_action_inputs_or_unresolved_script_flows_as_direct_exposure() {
        let workflow = Workflow {
            path: PathBuf::from(".github/workflows/test.yml"),
            name: None,
            permissions: None,
            env: yaml_map(&[("DEPLOY_TOKEN", "${{ secrets.DEPLOY_TOKEN }}")]),
            jobs: BTreeMap::from([(
                "deploy".to_owned(),
                WorkflowJob {
                    name: None,
                    uses: None,
                    with: BTreeMap::new(),
                    permissions: None,
                    env: BTreeMap::new(),
                    steps: vec![
                        WorkflowStep {
                            name: None,
                            id: None,
                            uses: Some("actions/checkout@v4".to_owned()),
                            with: BTreeMap::from([(
                                "token".to_owned(),
                                Value::String("${{ secrets.GITHUB_TOKEN }}".to_owned()),
                            )]),
                            env: BTreeMap::new(),
                            run: None,
                        },
                        WorkflowStep {
                            name: None,
                            id: None,
                            uses: None,
                            with: BTreeMap::new(),
                            env: BTreeMap::new(),
                            run: Some("./deploy.sh".to_owned()),
                        },
                    ],
                },
            )]),
        };

        let (findings, notices) = findings_for(&workflow);

        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("DEPLOY_TOKEN"));
    }

    #[test]
    fn ignores_non_secret_contexts_comments_invalid_expressions_and_safe_commands() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[
                ("echo '${{ vars.TOKEN }}'", &[]),
                ("echo '${{ github.token }}'", &[]),
                ("echo 'secrets.MY_TOKEN'", &[]),
                ("# echo '${{ secrets.COMMENTED }}'\necho safe", &[]),
                ("echo '${{ secrets.INCOMPLETE }'", &[]),
                ("cargo build", &[]),
            ],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(findings.is_empty());
        assert!(notices.is_empty());
    }

    #[test]
    fn deduplicates_repeated_secret_references_per_step_and_sink() {
        let workflow = workflow_with_steps(
            &[],
            &[],
            &[(
                "echo '${{ secrets.MY_TOKEN }}' '${{ secrets.MY_TOKEN }}' && printf '%s' '${{ secrets.MY_TOKEN }}' && curl -d '${{ secrets.MY_TOKEN }}' https://example.invalid/upload",
                &[],
            )],
        );

        let (findings, notices) = findings_for(&workflow);

        assert!(notices.is_empty());
        assert_eq!(findings.len(), 2);
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.exposure_sink)
                .collect::<Vec<_>>(),
            vec![
                Some(WorkflowExposureSink::Stdout),
                Some(WorkflowExposureSink::NetworkRequest)
            ]
        );
    }

    #[test]
    fn reports_unsupported_direct_secret_flow_as_a_partial_notice() {
        let workflow =
            workflow_with_steps(&[], &[], &[("./send.sh '${{ secrets.MY_TOKEN }}'", &[])]);

        let (findings, notices) = findings_for(&workflow);

        assert!(findings.is_empty());
        assert_eq!(notices.len(), 1);
        assert!(notices[0].reason.contains("MY_TOKEN"));
        assert!(notices[0].reason.contains("unresolved"));
    }
}
