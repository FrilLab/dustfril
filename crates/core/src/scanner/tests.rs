use tempfile::TempDir;

use crate::{
    models::{Ecosystem, MAX_SCAN_FAILURE_SAMPLES},
    scanner::{
        detector::{Detector, NodeDetector, RustDetector},
        scan,
    },
};

fn create_rust_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    let target = root.join("target");
    std::fs::create_dir_all(&target).unwrap();
    target
}

fn create_node_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("package.json"), "{}").unwrap();
    let node_modules = root.join("node_modules");
    std::fs::create_dir_all(&node_modules).unwrap();
    node_modules
}

fn create_java_artifact(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(root.join("pom.xml"), "<project></project>").unwrap();
    let build = root.join("build");
    std::fs::create_dir_all(&build).unwrap();
    build
}

#[test]
fn scan_returns_empty_when_no_projects() {
    let temp_dir = TempDir::new().unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert!(result.artifacts.is_empty());
    assert_eq!(result.access_summary.root, temp_dir.path());
    assert_eq!(result.access_summary.directories_visited, 1);
    assert_eq!(result.access_summary.files_inspected, 0);
    assert_eq!(result.access_summary.metadata_files_inspected, 0);
    assert_eq!(result.access_summary.artifact_candidates, 0);
    assert_eq!(result.access_summary.symlinks_skipped, 0);
    assert_eq!(result.access_summary.failures, 0);
}

#[test]
fn scan_detects_rust_project() {
    let temp_dir = TempDir::new().unwrap();

    let target = create_rust_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);

    let artifact = &result.artifacts[0];

    assert_eq!(artifact.ecosystem, Ecosystem::Rust);
    assert_eq!(artifact.path, target);
    assert_eq!(result.access_summary.directories_visited, 2);
    assert_eq!(result.access_summary.files_inspected, 2);
    assert_eq!(result.access_summary.metadata_files_inspected, 2);
    assert_eq!(result.access_summary.artifact_candidates, 1);
}

#[test]
fn rust_artifact_keeps_its_discovered_project_identity() {
    let temp_dir = TempDir::new().unwrap();
    let project = temp_dir.path().join("dustfril");
    std::fs::create_dir(&project).unwrap();

    let target = create_rust_artifact(&project);
    let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, target);
    assert_eq!(result.artifacts[0].project.root, project);
    assert_eq!(result.artifacts[0].project.display_name, "dustfril");
    assert_eq!(result.artifacts[0].project.ecosystem, Ecosystem::Rust);
}

#[test]
fn rust_projects_with_the_same_artifact_name_keep_distinct_identities() {
    let temp_dir = TempDir::new().unwrap();
    let alpha = temp_dir.path().join("alpha");
    let beta = temp_dir.path().join("beta");
    std::fs::create_dir_all(&alpha).unwrap();
    std::fs::create_dir_all(&beta).unwrap();

    create_rust_artifact(&alpha);
    create_rust_artifact(&beta);

    let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();
    let mut project_names = result
        .artifacts
        .iter()
        .map(|artifact| artifact.project.display_name.as_str())
        .collect::<Vec<_>>();
    project_names.sort_unstable();

    assert_eq!(project_names, ["alpha", "beta"]);
    assert!(result.artifacts.iter().all(|artifact| {
        artifact.path.file_name().and_then(|name| name.to_str()) == Some("target")
    }));
}

#[test]
fn scan_detects_node_project() {
    let temp_dir = TempDir::new().unwrap();

    let node_modules = create_node_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
    assert_eq!(result.artifacts[0].path, node_modules);
    assert_eq!(result.access_summary.directories_visited, 2);
    assert_eq!(result.access_summary.files_inspected, 2);
    assert_eq!(result.access_summary.metadata_files_inspected, 2);
    assert_eq!(result.access_summary.artifact_candidates, 1);
}

#[test]
fn node_artifact_keeps_its_discovered_project_identity() {
    let temp_dir = TempDir::new().unwrap();
    let project = temp_dir.path().join("web");
    std::fs::create_dir(&project).unwrap();

    let node_modules = create_node_artifact(&project);
    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, node_modules);
    assert_eq!(result.artifacts[0].project.root, project);
    assert_eq!(result.artifacts[0].project.display_name, "web");
    assert_eq!(result.artifacts[0].project.ecosystem, Ecosystem::Node);
}

#[test]
fn artifact_boundary_metadata_checks_are_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path().join("target");

    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    std::fs::create_dir(&target).unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert!(result.artifacts.is_empty());
    assert_eq!(result.access_summary.files_inspected, 1);
    assert_eq!(result.access_summary.metadata_files_inspected, 1);
    assert_eq!(result.access_summary.failures, 0);
}

#[test]
fn scan_detects_java_project() {
    let temp_dir = TempDir::new().unwrap();

    let build = create_java_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Java);
    assert_eq!(result.artifacts[0].path, build);
    assert_eq!(result.access_summary.directories_visited, 2);
    assert_eq!(result.access_summary.files_inspected, 2);
    assert_eq!(result.access_summary.metadata_files_inspected, 2);
    assert_eq!(result.access_summary.artifact_candidates, 1);
}

#[test]
fn scan_detects_mixed_ecosystems_in_one_project() {
    let temp_dir = TempDir::new().unwrap();

    let rust_target = create_rust_artifact(temp_dir.path());
    let node_modules = create_node_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 2);
    assert!(
        result
            .artifacts
            .iter()
            .any(|artifact| artifact.ecosystem == Ecosystem::Rust && artifact.path == rust_target)
    );
    assert!(result.artifacts.iter().any(|artifact| {
        artifact.ecosystem == Ecosystem::Node && artifact.path == node_modules
    }));
    assert_eq!(result.access_summary.artifact_candidates, 2);
}

#[test]
fn scan_detects_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");
    let java = temp_dir.path().join("java");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();
    std::fs::create_dir_all(&java).unwrap();

    let rust_target = create_rust_artifact(&rust);
    let node_modules = create_node_artifact(&node);
    let java_build = create_java_artifact(&java);

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.artifacts.len(), 3);

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Rust && a.path == rust_target)
    );

    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Node && a.path == node_modules)
    );
    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.ecosystem == Ecosystem::Java && a.path == java_build)
    );
    assert_eq!(result.access_summary.directories_visited, 7);
    assert_eq!(result.access_summary.files_inspected, 6);
    assert_eq!(result.access_summary.metadata_files_inspected, 6);
    assert_eq!(result.access_summary.artifact_candidates, 3);
}

#[test]
fn scan_filters_rust_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    let rust_target = create_rust_artifact(&rust);
    create_node_artifact(&node);

    let result = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Rust);
    assert_eq!(result.artifacts[0].path, rust_target);
}

#[test]
fn scan_filters_node_only() {
    let temp_dir = TempDir::new().unwrap();

    let rust = temp_dir.path().join("rust");
    let node = temp_dir.path().join("node");

    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(&node).unwrap();

    create_rust_artifact(&rust);
    let node_modules = create_node_artifact(&node);

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].ecosystem, Ecosystem::Node);
    assert_eq!(result.artifacts[0].path, node_modules);
}

#[test]
fn node_modules_is_a_terminal_discovery_boundary() {
    let temp_dir = TempDir::new().unwrap();
    let outer_artifact = create_node_artifact(temp_dir.path());
    let nested_package = outer_artifact.join("package-a");
    let deeply_nested_package = nested_package.join("node_modules").join("package-b");

    std::fs::create_dir_all(&deeply_nested_package).unwrap();
    std::fs::write(nested_package.join("package.json"), "{}").unwrap();
    std::fs::write(deeply_nested_package.join("package.json"), "{}").unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, outer_artifact);
}

#[test]
fn nested_dependency_manifests_do_not_create_projects() {
    let temp_dir = TempDir::new().unwrap();
    let web = temp_dir.path().join("web");
    std::fs::create_dir(&web).unwrap();
    let outer_artifact = create_node_artifact(&web);
    let package = outer_artifact.join("package-a");
    let nested_artifact = package.join("node_modules");

    std::fs::create_dir_all(&nested_artifact).unwrap();
    std::fs::write(package.join("package.json"), "{}\n").unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, outer_artifact);
    assert_eq!(result.artifacts[0].project.display_name, "web");
}

#[test]
fn gradle_module_artifact_uses_the_gradle_workspace_identity() {
    let temp_dir = TempDir::new().unwrap();
    let backend = temp_dir.path().join("backend");
    let app = backend.join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(backend.join("settings.gradle.kts"), "include(\":app\")\n").unwrap();
    std::fs::write(backend.join("build.gradle.kts"), "\n").unwrap();
    std::fs::write(app.join("build.gradle.kts"), "\n").unwrap();
    let build = app.join("build");
    std::fs::create_dir(&build).unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, build);
    assert_eq!(result.artifacts[0].project.root, backend);
    assert_eq!(result.artifacts[0].project.display_name, "backend");
    assert_eq!(result.artifacts[0].project.ecosystem, Ecosystem::Java);
}

#[test]
fn gradle_discovery_does_not_escape_the_scanned_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let backend = temp_dir.path().join("backend");
    let app = backend.join("app");
    std::fs::create_dir_all(app.join("build")).unwrap();
    std::fs::write(backend.join("settings.gradle"), "include(\":app\")\n").unwrap();
    std::fs::write(app.join("build.gradle"), "\n").unwrap();

    let result = scan(&app, &[Ecosystem::Java]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].project.root, app);
}

#[test]
fn maven_artifact_uses_the_maven_project_root() {
    let temp_dir = TempDir::new().unwrap();
    let api = temp_dir.path().join("api");
    std::fs::create_dir(&api).unwrap();
    let build = create_java_artifact(&api);

    let result = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, build);
    assert_eq!(result.artifacts[0].project.root, api);
    assert_eq!(result.artifacts[0].project.display_name, "api");
}

#[test]
fn independent_projects_and_projects_outside_artifacts_are_still_discovered() {
    let temp_dir = TempDir::new().unwrap();
    let first = temp_dir.path().join("first");
    let second = temp_dir.path().join("workspace").join("second");
    let third = temp_dir.path().join("build").join("legitimate-project");

    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    std::fs::create_dir_all(&third).unwrap();
    let first_artifact = create_node_artifact(&first);
    let second_artifact = create_node_artifact(&second);
    let third_artifact = create_node_artifact(&third);

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert_eq!(result.artifacts.len(), 3);
    assert!(result.artifacts.iter().any(|artifact| {
        artifact.path == first_artifact && artifact.ecosystem == Ecosystem::Node
    }));
    assert!(result.artifacts.iter().any(|artifact| {
        artifact.path == second_artifact && artifact.ecosystem == Ecosystem::Node
    }));
    assert!(result.artifacts.iter().any(|artifact| {
        artifact.path == third_artifact && artifact.ecosystem == Ecosystem::Node
    }));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_application_bundles_are_opaque_to_workspace_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let app = temp_dir
        .path()
        .join(".vscode-test")
        .join("vscode-darwin-arm64-1.96.0")
        .join("Visual Studio Code.app");
    let app_contents = app.join("Contents").join("Resources").join("app");
    let app_node_modules = app_contents.join("node_modules");

    std::fs::create_dir_all(app_node_modules.join("package-a").join("node_modules")).unwrap();
    std::fs::write(app_contents.join("package.json"), "{}").unwrap();
    std::fs::write(
        app_node_modules.join("package-a").join("package.json"),
        "{}",
    )
    .unwrap();

    let result = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

    assert!(result.artifacts.is_empty());
    assert!(
        result
            .artifacts
            .iter()
            .all(|artifact| !artifact.path.starts_with(&app))
    );

    let analysis = crate::api::analyze(result).unwrap();
    let selection = crate::models::ArtifactSelection {
        path: app_contents.join("node_modules"),
        ecosystem: Ecosystem::Node,
    };
    assert!(matches!(
        crate::api::clean::build_plan_from_analysis_with_selection(&analysis, &[selection]),
        Err(crate::error::DustError::InvalidCleanupSelection(_))
    ));
}

#[test]
fn scan_with_unknown_filter_returns_empty() {
    let temp_dir = TempDir::new().unwrap();

    create_rust_artifact(temp_dir.path());

    let result = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

    assert!(result.artifacts.is_empty());
}

#[test]
fn rust_detector_reports_target_as_safe_artifact() {
    let detector = RustDetector;

    assert_eq!(detector.artifact_paths(), &["target"]);
}

#[test]
fn node_detector_accepts_workspace_metadata_as_a_project_marker() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\n",
    )
    .unwrap();

    let detector = NodeDetector;

    assert!(detector.matches(temp_dir.path()));
}

#[test]
fn nested_unsupported_files_are_not_counted_as_inspected_content() {
    let temp_dir = TempDir::new().unwrap();
    let nested = temp_dir.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(
        nested.join("source.rs"),
        "source contents must not be persisted",
    )
    .unwrap();
    std::fs::write(nested.join("notes.txt"), "unrelated contents").unwrap();

    let result = scan(temp_dir.path(), &[]).unwrap();

    assert_eq!(result.access_summary.directories_visited, 2);
    assert_eq!(result.access_summary.files_inspected, 0);
    assert_eq!(result.access_summary.metadata_files_inspected, 0);
    assert_eq!(result.access_summary.artifact_candidates, 0);
}

#[test]
fn access_summary_failure_samples_are_bounded() {
    let root = TempDir::new().unwrap();
    let mut summary = crate::models::ScanAccessSummary::new(root.path());

    for index in 0..(MAX_SCAN_FAILURE_SAMPLES + 3) {
        summary.record_failure(
            &root.path().join(format!("failure-{index}")),
            "permission denied",
        );
    }

    assert_eq!(summary.failures, (MAX_SCAN_FAILURE_SAMPLES + 3) as u64);
    assert_eq!(summary.failure_samples.len(), MAX_SCAN_FAILURE_SAMPLES);
    assert_eq!(
        summary.failure_samples[0].path,
        std::path::Path::new("failure-0")
    );
}

#[cfg(unix)]
#[test]
fn scanner_does_not_return_symbolic_link_artifacts() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let real_target = TempDir::new().unwrap();
    std::fs::write(root.path().join("Cargo.toml"), "[package]").unwrap();
    symlink(real_target.path(), root.path().join("target")).unwrap();

    let result = scan(root.path(), &[Ecosystem::Rust]).unwrap();

    assert!(result.artifacts.is_empty());
    assert_eq!(result.access_summary.symlinks_skipped, 1);
    assert_eq!(result.access_summary.artifact_candidates, 0);
}

#[cfg(unix)]
#[test]
fn scanner_follows_symbolic_linked_project_manifests() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let manifest_root = TempDir::new().unwrap();
    let manifest = manifest_root.path().join("Cargo.toml");
    std::fs::write(&manifest, "[package]").unwrap();
    let target = root.path().join("target");
    std::fs::create_dir(&target).unwrap();
    symlink(&manifest, root.path().join("Cargo.toml")).unwrap();

    let result = scan(root.path(), &[Ecosystem::Rust]).unwrap();

    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].path, target);
    assert_eq!(result.access_summary.files_inspected, 2);
    assert_eq!(result.access_summary.metadata_files_inspected, 2);
    assert_eq!(result.access_summary.symlinks_skipped, 1);
}
