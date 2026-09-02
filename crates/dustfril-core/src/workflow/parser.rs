use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use serde_yaml::Value;

use crate::{
    error::{DustError, DustResult},
    models::{Workflow, WorkflowJob, WorkflowPermissionLevel, WorkflowPermissions, WorkflowStep},
};

#[derive(Debug, Deserialize)]
struct RawWorkflow {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    permissions: Option<Value>,
    #[serde(default)]
    env: BTreeMap<String, Value>,
    #[serde(default)]
    jobs: Option<BTreeMap<String, RawJob>>,
}

#[derive(Debug, Deserialize)]
struct RawJob {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    uses: Option<String>,
    #[serde(default)]
    with: BTreeMap<String, Value>,
    #[serde(default)]
    permissions: Option<Value>,
    #[serde(default)]
    env: BTreeMap<String, Value>,
    #[serde(default)]
    steps: Vec<RawStep>,
}

#[derive(Debug, Deserialize)]
struct RawStep {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    uses: Option<String>,
    #[serde(default)]
    with: BTreeMap<String, Value>,
    #[serde(default)]
    env: BTreeMap<String, Value>,
    #[serde(default)]
    run: Option<String>,
}

pub fn discover_and_parse(root: &Path) -> DustResult<Vec<Workflow>> {
    validate_root(root)?;

    let workflows_dir = root.join(".github").join("workflows");
    let metadata = match fs::symlink_metadata(&workflows_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(workflow_io_error(&workflows_dir, error)),
    };

    if metadata.file_type().is_symlink() {
        return Err(DustError::Workflow(format!(
            "{}: symbolic workflow directories are unsupported",
            workflows_dir.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(DustError::Workflow(format!(
            "{}: expected a directory",
            workflows_dir.display()
        )));
    }

    let mut paths = Vec::new();
    for entry in
        fs::read_dir(&workflows_dir).map_err(|error| workflow_io_error(&workflows_dir, error))?
    {
        let entry = entry.map_err(|error| workflow_io_error(&workflows_dir, error))?;
        let path = entry.path();

        if !is_workflow_file(&path) {
            continue;
        }

        let metadata =
            fs::symlink_metadata(&path).map_err(|error| workflow_io_error(&path, error))?;
        if metadata.file_type().is_symlink() {
            return Err(DustError::Workflow(format!(
                "{}: symbolic workflow files are unsupported",
                path.display()
            )));
        }
        if metadata.is_file() {
            paths.push(path);
        }
    }

    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let content =
                fs::read_to_string(&path).map_err(|error| workflow_io_error(&path, error))?;
            parse_workflow(&path, &content)
        })
        .collect()
}

fn is_workflow_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("yml" | "yaml")
    )
}

fn parse_workflow(path: &Path, content: &str) -> DustResult<Workflow> {
    let raw: RawWorkflow =
        serde_yaml::from_str(content).map_err(|error| parse_error(path, error.to_string()))?;
    let jobs = raw.jobs.ok_or_else(|| {
        parse_error(
            path,
            "workflow is missing the required jobs mapping".to_owned(),
        )
    })?;

    let permissions = raw
        .permissions
        .as_ref()
        .map(|value| parse_permissions(value).map_err(|error| error_with_path(path, error)))
        .transpose()?;

    let jobs = jobs
        .into_iter()
        .map(|(id, job)| {
            let permissions = job
                .permissions
                .as_ref()
                .map(|value| parse_permissions(value).map_err(|error| error_with_path(path, error)))
                .transpose()?;
            let workflow_job = WorkflowJob {
                name: job.name,
                uses: job.uses,
                with: job.with,
                permissions,
                env: job.env,
                steps: job.steps.into_iter().map(WorkflowStep::from).collect(),
            };
            Ok::<_, DustError>((id, workflow_job))
        })
        .collect::<DustResult<BTreeMap<_, _>>>()?;

    Ok(Workflow {
        path: path.to_path_buf(),
        name: raw.name,
        permissions,
        env: raw.env,
        jobs,
    })
}

impl From<RawStep> for WorkflowStep {
    fn from(step: RawStep) -> Self {
        Self {
            name: step.name,
            id: step.id,
            uses: step.uses,
            with: step.with,
            env: step.env,
            run: step.run,
        }
    }
}

fn parse_permissions(value: &Value) -> DustResult<WorkflowPermissions> {
    match value {
        Value::String(value) => Ok(match value.as_str() {
            "read-all" => WorkflowPermissions::ReadAll,
            "write-all" => WorkflowPermissions::WriteAll,
            other => WorkflowPermissions::Unknown(other.to_owned()),
        }),
        Value::Mapping(mapping) if mapping.is_empty() => Ok(WorkflowPermissions::Empty),
        Value::Mapping(mapping) => {
            let mut permissions = BTreeMap::new();
            for (scope, level) in mapping {
                let scope = scope.as_str().ok_or_else(|| {
                    DustError::Workflow(
                        "permissions mapping contains a non-string scope".to_owned(),
                    )
                })?;
                permissions.insert(scope.to_owned(), parse_permission_level(level));
            }
            Ok(WorkflowPermissions::Map(permissions))
        }
        other => Ok(WorkflowPermissions::Unknown(value_summary(other))),
    }
}

fn parse_permission_level(value: &Value) -> WorkflowPermissionLevel {
    match value.as_str() {
        Some("none") => WorkflowPermissionLevel::None,
        Some("read") => WorkflowPermissionLevel::Read,
        Some("write") => WorkflowPermissionLevel::Write,
        _ => WorkflowPermissionLevel::Unknown(value_summary(value)),
    }
}

fn value_summary(value: &Value) -> String {
    serde_yaml::to_string(value)
        .map(|value| value.trim().to_owned())
        .unwrap_or_else(|_| "<unrepresentable YAML value>".to_owned())
}

fn validate_root(root: &Path) -> DustResult<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(DustError::InvalidPath(root.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(DustError::InvalidPath(root.to_path_buf()))
        }
        Err(error) => Err(DustError::Io(error)),
    }
}

fn parse_error(path: &Path, message: String) -> DustError {
    DustError::Workflow(format!("{}: {message}", path.display()))
}

fn error_with_path(path: &Path, error: DustError) -> DustError {
    match error {
        DustError::Workflow(message) => parse_error(path, message),
        other => other,
    }
}

fn workflow_io_error(path: &Path, error: std::io::Error) -> DustError {
    DustError::Workflow(format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn workflow_dir(temp_dir: &TempDir) -> std::path::PathBuf {
        let directory = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn discovers_only_direct_yaml_workflow_files_in_sorted_order() {
        let temp_dir = TempDir::new().unwrap();
        let directory = workflow_dir(&temp_dir);
        std::fs::write(
            directory.join("z.yaml"),
            "name: z\njobs:\n  z:\n    steps: []\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("a.yml"),
            "name: a\njobs:\n  a:\n    steps: []\n",
        )
        .unwrap();
        std::fs::write(directory.join("not-a-workflow.txt"), "jobs: {}").unwrap();
        std::fs::create_dir(directory.join("nested.yml")).unwrap();
        std::fs::write(directory.join("nested.yml/ignored.yml"), "jobs: {}\n").unwrap();

        let workflows = discover_and_parse(temp_dir.path()).unwrap();

        assert_eq!(workflows.len(), 2);
        assert!(workflows[0].path.ends_with("a.yml"));
        assert!(workflows[1].path.ends_with("z.yaml"));
    }

    #[test]
    fn parses_structure_and_multiline_run_without_flat_text_matching() {
        let temp_dir = TempDir::new().unwrap();
        let directory = workflow_dir(&temp_dir);
        let path = directory.join("build.yml");
        std::fs::write(
            &path,
            r#"name: Build
env:
  WORKFLOW_VALUE: value
jobs:
  build:
    name: Build job
    env:
      JOB_VALUE: job
    steps:
      - name: Run build
        id: build
        run: |
          echo "$WORKFLOW_VALUE"
          cargo build
        env:
          STEP_VALUE: step
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0
"#,
        )
        .unwrap();

        let workflow = discover_and_parse(temp_dir.path()).unwrap().remove(0);
        let job = &workflow.jobs["build"];

        assert_eq!(workflow.name.as_deref(), Some("Build"));
        assert_eq!(
            workflow.env["WORKFLOW_VALUE"],
            Value::String("value".into())
        );
        assert_eq!(job.env["JOB_VALUE"], Value::String("job".into()));
        assert_eq!(
            job.steps[0].run.as_deref(),
            Some("echo \"$WORKFLOW_VALUE\"\ncargo build\n")
        );
        assert_eq!(job.steps[0].env["STEP_VALUE"], Value::String("step".into()));
        assert_eq!(job.steps[1].uses.as_deref(), Some("actions/checkout@v7"));
        assert_eq!(
            job.steps[1].with["fetch-depth"],
            serde_yaml::from_str::<Value>("0").unwrap()
        );
        assert_eq!(job.steps[0].id.as_deref(), Some("build"));
    }

    #[test]
    fn supports_yaml_anchors_and_aliases_in_retained_environment_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let directory = workflow_dir(&temp_dir);
        std::fs::write(
            directory.join("anchors.yml"),
            "env: &shared\n  SHARED_VALUE: shared\njobs:\n  build:\n    env: *shared\n    steps: []\n",
        )
        .unwrap();

        let workflow = discover_and_parse(temp_dir.path()).unwrap().remove(0);

        assert_eq!(
            workflow.jobs["build"].env["SHARED_VALUE"],
            Value::String("shared".into())
        );
    }

    #[test]
    fn parses_supported_permission_forms_and_retains_unknown_values() {
        assert_eq!(
            parse_permissions(&Value::String("read-all".into())).unwrap(),
            WorkflowPermissions::ReadAll
        );
        assert_eq!(
            parse_permissions(&Value::String("write-all".into())).unwrap(),
            WorkflowPermissions::WriteAll
        );
        assert_eq!(
            parse_permissions(&Value::Mapping(Default::default())).unwrap(),
            WorkflowPermissions::Empty
        );

        let value: Value = serde_yaml::from_str("contents: read\npull-requests: write\n").unwrap();
        assert!(matches!(
            parse_permissions(&value).unwrap(),
            WorkflowPermissions::Map(_)
        ));
        assert!(matches!(
            parse_permissions(&Value::String("future-all".into())).unwrap(),
            WorkflowPermissions::Unknown(_)
        ));
    }

    #[test]
    fn malformed_yaml_and_missing_jobs_are_explicit_errors() {
        let temp_dir = TempDir::new().unwrap();
        let directory = workflow_dir(&temp_dir);
        std::fs::write(directory.join("bad.yml"), "jobs: [").unwrap();

        let result = discover_and_parse(temp_dir.path());

        assert!(matches!(
            result,
            Err(DustError::Workflow(message)) if message.contains("bad.yml")
        ));

        std::fs::write(directory.join("bad.yml"), "name: missing jobs\n").unwrap();
        let result = discover_and_parse(temp_dir.path());
        assert!(matches!(
            result,
            Err(DustError::Workflow(message)) if message.contains("required jobs")
        ));
    }

    #[test]
    fn unsupported_permission_mapping_errors_identify_the_workflow_path() {
        let temp_dir = TempDir::new().unwrap();
        let directory = workflow_dir(&temp_dir);
        std::fs::write(
            directory.join("unsupported.yml"),
            "permissions:\n  1: write\njobs: {}\n",
        )
        .unwrap();

        let result = discover_and_parse(temp_dir.path());

        assert!(matches!(
            result,
            Err(DustError::Workflow(message))
                if message.contains("unsupported.yml") && message.contains("non-string scope")
        ));
    }
}
