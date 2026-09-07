use std::{
    fs,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::models::{
    Artifact, Ecosystem, ProjectIdentity, ProjectTechnology, ScanAccessSummary, TechnologyEvidence,
};

pub(crate) fn metadata_file_exists_with_summary(
    path: &Path,
    summary: &mut ScanAccessSummary,
) -> bool {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {
            summary.record_metadata_file();
            true
        }
        Ok(_) => false,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            summary.record_failure(path, &error.to_string());
            false
        }
    }
}

pub(crate) fn metadata_text_with_summary(
    path: &Path,
    summary: &mut ScanAccessSummary,
) -> Option<String> {
    match fs::read(path) {
        Ok(bytes) => {
            summary.record_metadata_file();
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            summary.record_failure(path, &error.to_string());
            None
        }
    }
}

fn metadata_directory_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
}

fn metadata_file_exists_without_following_symlink(
    path: &Path,
    summary: Option<&mut ScanAccessSummary>,
) -> bool {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => {
            if let Some(summary) = summary {
                summary.record_metadata_file();
            }
            true
        }
        Ok(_) => false,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            if let Some(summary) = summary {
                summary.record_failure(path, &error.to_string());
            }
            false
        }
    }
}

fn metadata_text(path: &Path) -> Option<String> {
    fs::read(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn evidence(path: impl Into<PathBuf>, detail: &str) -> TechnologyEvidence {
    TechnologyEvidence {
        path: path.into(),
        detail: detail.to_owned(),
    }
}

fn identity(root: PathBuf, ecosystem: Ecosystem, technology: ProjectTechnology) -> ProjectIdentity {
    ProjectIdentity::with_technology(root, ecosystem, technology)
}

fn artifact_from_directory(
    path: PathBuf,
    project: &ProjectIdentity,
    summary: Option<&mut ScanAccessSummary>,
) -> Option<Artifact> {
    let is_directory = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata.is_dir(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            if let Some(summary) = summary {
                summary.record_failure(&path, &error.to_string());
            }
            false
        }
    };

    is_directory.then(|| Artifact::for_project(path, project.clone()))
}

/// Registered detectors for all supported ecosystems.
pub static DETECTORS: &[&dyn Detector] = &[
    &RustDetector,
    &NodeDetector,
    &JavaDetector,
    &CMakeDetector,
    &DotNetDetector,
    &PythonDetector,
    &SwiftDetector,
    &DartDetector,
    &PhpDetector,
    &ElixirDetector,
    &ZigDetector,
    &GoDetector,
    &RubyDetector,
];

/// Matches project roots and returns removable artifact directories.
pub trait Detector: Sync {
    #[allow(dead_code)]
    fn matches(&self, root: &Path) -> bool;

    /// Recognized project metadata names checked by this detector.
    fn metadata_paths(&self) -> &[&str];

    /// Artifact directory names managed by this detector.
    fn artifact_paths(&self) -> &[&str];

    /// Ecosystem handled by the detector's default identity.
    fn ecosystem(&self) -> Ecosystem;

    /// Additional identities that share this detector's evidence rules.
    fn supports_ecosystem(&self, ecosystem: Ecosystem) -> bool {
        self.ecosystem() == ecosystem
    }

    fn project_with_summary(
        &self,
        root: &Path,
        _scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        self.matches_with_summary(root, summary).then(|| {
            identity(
                root.to_path_buf(),
                self.ecosystem(),
                self.technology_with_summary(root, summary),
            )
        })
    }

    /// Provides structured technology context after project evidence is found.
    fn technology_with_summary(
        &self,
        root: &Path,
        _summary: &mut ScanAccessSummary,
    ) -> ProjectTechnology {
        let mut technology = ProjectTechnology::for_ecosystem(self.ecosystem());
        technology.evidence = self
            .metadata_paths()
            .iter()
            .filter(|name| root.join(name).is_file())
            .map(|name| evidence(*name, "Project metadata"))
            .collect();
        technology
    }

    fn matches_with_summary(&self, root: &Path, summary: &mut ScanAccessSummary) -> bool {
        self.metadata_paths()
            .iter()
            .any(|name| metadata_file_exists_with_summary(&root.join(name), summary))
    }

    #[allow(dead_code)]
    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        self.artifacts_with_summary(root, None)
    }

    #[allow(dead_code)]
    fn artifacts_with_summary(
        &self,
        root: &Path,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> Vec<Artifact> {
        let Some(project) = summary
            .as_deref_mut()
            .and_then(|summary| self.project_with_summary(root, root, summary))
            .or_else(|| {
                self.matches(root).then(|| {
                    identity(
                        root.to_path_buf(),
                        self.ecosystem(),
                        ProjectTechnology::for_ecosystem(self.ecosystem()),
                    )
                })
            })
        else {
            return Vec::new();
        };

        self.artifacts_for_project_with_summary(root, &project, summary)
    }

    /// Returns whether an artifact directory is owned by a project at root.
    /// The optional summary is omitted during boundary checks to avoid
    /// inspecting the same metadata twice before the project is analyzed.
    fn artifact_path_is_valid(
        &self,
        _root: &Path,
        artifact_name: &str,
        _summary: Option<&mut ScanAccessSummary>,
    ) -> bool {
        self.artifact_paths().contains(&artifact_name)
    }

    /// Returns artifact names that may be owned by the project at `root`.
    /// Detectors can extend this for safe, metadata-validated name patterns.
    fn artifact_names(&self, _root: &Path) -> Vec<String> {
        self.artifact_paths()
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn artifacts_for_project_with_summary(
        &self,
        root: &Path,
        project: &ProjectIdentity,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> Vec<Artifact> {
        let mut artifacts = Vec::new();
        for name in self.artifact_names(root) {
            if !self.artifact_path_is_valid(root, &name, summary.as_deref_mut()) {
                continue;
            }
            let path = root.join(&name);
            if let Some(artifact) = artifact_from_directory(path, project, summary.as_deref_mut()) {
                artifacts.push(artifact);
            }
        }
        artifacts
    }

    /// Used by the directory walker to establish terminal artifact roots.
    fn is_artifact_directory_with_summary(
        &self,
        path: &Path,
        summary: &mut ScanAccessSummary,
    ) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        let Some(parent) = path.parent() else {
            return false;
        };

        self.matches_with_summary(parent, summary)
            && self.artifact_path_is_valid(parent, name, None)
    }
}

pub fn select_detectors(ecosystems: &[Ecosystem]) -> Vec<&'static dyn Detector> {
    if ecosystems.is_empty() {
        return DETECTORS.to_vec();
    }

    DETECTORS
        .iter()
        .copied()
        .filter(|detector| {
            ecosystems
                .iter()
                .any(|ecosystem| detector.supports_ecosystem(*ecosystem))
        })
        .collect()
}

pub fn detector_for(ecosystem: Ecosystem) -> Option<&'static dyn Detector> {
    DETECTORS
        .iter()
        .copied()
        .find(|detector| detector.supports_ecosystem(ecosystem))
}

pub struct RustDetector;

impl Detector for RustDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("Cargo.toml").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["Cargo.toml"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["target"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }
}

pub struct NodeDetector;

impl Detector for NodeDetector {
    fn matches(&self, root: &Path) -> bool {
        node_marker(root)
    }

    fn metadata_paths(&self) -> &[&str] {
        &[
            "package.json",
            "pnpm-workspace.yaml",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lock",
        ]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["node_modules"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn technology_with_summary(
        &self,
        root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> ProjectTechnology {
        let typescript = ["tsconfig.json", "jsconfig.json"]
            .iter()
            .find(|name| metadata_file_exists_with_summary(&root.join(name), summary));
        let language = if typescript.is_some() {
            "TypeScript"
        } else {
            "JavaScript"
        };
        let mut evidence_items = self
            .metadata_paths()
            .iter()
            .filter(|name| root.join(name).is_file())
            .map(|name| {
                evidence(
                    *name,
                    if *name == "package.json" {
                        "Node.js project manifest"
                    } else {
                        "Node.js package-manager metadata"
                    },
                )
            })
            .collect::<Vec<_>>();
        if let Some(name) = typescript {
            evidence_items.push(evidence(
                *name,
                "TypeScript/JavaScript project configuration",
            ));
        }

        ProjectTechnology::new(
            vec![language.to_owned()],
            Some("Node.js".to_owned()),
            None,
            evidence_items,
        )
    }
}

fn node_marker(root: &Path) -> bool {
    [
        "package.json",
        "pnpm-workspace.yaml",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "bun.lock",
    ]
    .iter()
    .any(|name| root.join(name).is_file())
}

pub struct JavaDetector;

#[derive(Clone, Copy, PartialEq, Eq)]
enum JavaMarker {
    Maven,
    GradleSettings,
    GradleBuild,
}

impl Detector for JavaDetector {
    fn matches(&self, root: &Path) -> bool {
        java_marker(root).is_some()
    }

    fn metadata_paths(&self) -> &[&str] {
        &[
            "settings.gradle",
            "settings.gradle.kts",
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
        ]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["target", "build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Java
    }

    fn supports_ecosystem(&self, ecosystem: Ecosystem) -> bool {
        matches!(ecosystem, Ecosystem::Java | Ecosystem::Kotlin)
    }

    fn project_with_summary(
        &self,
        root: &Path,
        scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        let (marker, marker_path, marker_contents) = java_marker_with_summary(root, summary)?;
        let project_root = match marker {
            JavaMarker::GradleBuild => {
                find_gradle_root(root, scan_root, summary).unwrap_or_else(|| root.to_path_buf())
            }
            JavaMarker::Maven | JavaMarker::GradleSettings => root.to_path_buf(),
        };
        let is_kotlin = marker_contents
            .as_deref()
            .is_some_and(contains_kotlin_plugin)
            || (marker == JavaMarker::GradleBuild
                && ["build.gradle", "build.gradle.kts"]
                    .iter()
                    .filter_map(|name| {
                        if *name == marker_path {
                            None
                        } else {
                            metadata_text(&root.join(name))
                        }
                    })
                    .any(|contents| contains_kotlin_plugin(&contents)));
        let ecosystem = if is_kotlin {
            Ecosystem::Kotlin
        } else {
            Ecosystem::Java
        };
        let build_system = match marker {
            JavaMarker::Maven => "Maven",
            JavaMarker::GradleSettings | JavaMarker::GradleBuild => "Gradle",
        };
        let language = if is_kotlin { "Kotlin" } else { "Java" };
        Some(identity(
            project_root,
            ecosystem,
            ProjectTechnology::new(
                vec![language.to_owned()],
                None,
                Some(build_system.to_owned()),
                vec![evidence(marker_path, "Java build metadata")],
            ),
        ))
    }

    fn artifact_path_is_valid(
        &self,
        root: &Path,
        artifact_name: &str,
        _summary: Option<&mut ScanAccessSummary>,
    ) -> bool {
        match java_marker(root) {
            Some(JavaMarker::Maven) => artifact_name == "target",
            Some(JavaMarker::GradleSettings | JavaMarker::GradleBuild) => artifact_name == "build",
            None => false,
        }
    }
}

fn java_marker(root: &Path) -> Option<JavaMarker> {
    if root.join("settings.gradle").is_file() || root.join("settings.gradle.kts").is_file() {
        Some(JavaMarker::GradleSettings)
    } else if root.join("pom.xml").is_file() {
        Some(JavaMarker::Maven)
    } else if root.join("build.gradle").is_file() || root.join("build.gradle.kts").is_file() {
        Some(JavaMarker::GradleBuild)
    } else {
        None
    }
}

fn java_marker_with_summary(
    root: &Path,
    summary: &mut ScanAccessSummary,
) -> Option<(JavaMarker, String, Option<String>)> {
    if metadata_file_exists_with_summary(&root.join("settings.gradle"), summary) {
        return Some((
            JavaMarker::GradleSettings,
            "settings.gradle".to_owned(),
            metadata_text_with_summary(&root.join("settings.gradle"), summary),
        ));
    }
    if metadata_file_exists_with_summary(&root.join("settings.gradle.kts"), summary) {
        return Some((
            JavaMarker::GradleSettings,
            "settings.gradle.kts".to_owned(),
            metadata_text_with_summary(&root.join("settings.gradle.kts"), summary),
        ));
    }
    if metadata_file_exists_with_summary(&root.join("pom.xml"), summary) {
        return Some((JavaMarker::Maven, "pom.xml".to_owned(), None));
    }
    if metadata_file_exists_with_summary(&root.join("build.gradle"), summary) {
        return Some((
            JavaMarker::GradleBuild,
            "build.gradle".to_owned(),
            metadata_text_with_summary(&root.join("build.gradle"), summary),
        ));
    }
    if metadata_file_exists_with_summary(&root.join("build.gradle.kts"), summary) {
        return Some((
            JavaMarker::GradleBuild,
            "build.gradle.kts".to_owned(),
            metadata_text_with_summary(&root.join("build.gradle.kts"), summary),
        ));
    }
    None
}

fn contains_kotlin_plugin(contents: &str) -> bool {
    let contents = contents.to_ascii_lowercase();
    contents.contains("kotlin(")
        || contents.contains("org.jetbrains.kotlin")
        || contents.contains("kotlin-jvm")
}

fn find_gradle_root(
    root: &Path,
    scan_root: &Path,
    summary: &mut ScanAccessSummary,
) -> Option<PathBuf> {
    root.ancestors()
        .skip(1)
        .take_while(|ancestor| ancestor.starts_with(scan_root))
        .find(|ancestor| {
            metadata_file_exists_with_summary(&ancestor.join("settings.gradle"), summary)
                || metadata_file_exists_with_summary(&ancestor.join("settings.gradle.kts"), summary)
        })
        .map(Path::to_path_buf)
}

pub struct CMakeDetector;

impl Detector for CMakeDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("CMakeLists.txt").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["CMakeLists.txt"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["build", "cmake-build-debug", "cmake-build-release"]
    }

    fn artifact_names(&self, root: &Path) -> Vec<String> {
        let mut names = self
            .artifact_paths()
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>();
        const PREFIX: &str = "cmake-build-";

        if let Ok(entries) = fs::read_dir(root) {
            names.extend(entries.flatten().filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                (name.starts_with(PREFIX)
                    && name.len() > PREFIX.len()
                    && entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
                .then_some(name)
            }));
        }

        names.sort_unstable();
        names.dedup();
        names
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::CMake
    }

    fn project_with_summary(
        &self,
        root: &Path,
        _scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        if !self.matches_with_summary(root, summary) {
            return None;
        }
        let contents =
            metadata_text_with_summary(&root.join("CMakeLists.txt"), summary).unwrap_or_default();
        let (languages, detail) = cmake_languages(&contents);
        Some(identity(
            root.to_path_buf(),
            Ecosystem::CMake,
            ProjectTechnology::new(
                languages,
                None,
                Some("CMake".to_owned()),
                vec![evidence("CMakeLists.txt", detail)],
            ),
        ))
    }

    fn artifact_path_is_valid(
        &self,
        root: &Path,
        artifact_name: &str,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> bool {
        if artifact_name != "build"
            && !(artifact_name.starts_with("cmake-build-")
                && artifact_name.len() > "cmake-build-".len())
        {
            return false;
        }
        let path = root.join(artifact_name);
        ["CMakeCache.txt", "cmake_install.cmake"]
            .iter()
            .any(|name| match summary.as_deref_mut() {
                Some(summary) => {
                    metadata_file_exists_without_following_symlink(&path.join(name), Some(summary))
                }
                None => metadata_file_exists_without_following_symlink(&path.join(name), None),
            })
            || metadata_directory_exists(&path.join("CMakeFiles"))
    }
}

fn cmake_languages(contents: &str) -> (Vec<String>, &'static str) {
    let upper = contents.to_ascii_uppercase();
    let tokens = upper
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '+'))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let has_c = tokens.contains(&"C");
    let has_cpp = tokens
        .iter()
        .any(|token| matches!(*token, "CXX" | "C++" | "CPP" | "CC"));
    let languages = match (has_c, has_cpp) {
        (true, true) => vec!["C".to_owned(), "C++".to_owned()],
        (true, false) => vec!["C".to_owned()],
        (false, _) => vec!["C++".to_owned()],
    };
    let detail = if has_c && has_cpp {
        "C and C++ language declarations"
    } else if has_c {
        "C language declaration"
    } else {
        "CMake project metadata"
    };
    (languages, detail)
}

pub struct DotNetDetector;

impl Detector for DotNetDetector {
    fn matches(&self, root: &Path) -> bool {
        dotnet_project_files(root).next().is_some()
    }

    fn metadata_paths(&self) -> &[&str] {
        &[]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["bin", "obj"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::DotNet
    }

    fn matches_with_summary(&self, root: &Path, summary: &mut ScanAccessSummary) -> bool {
        dotnet_project_files_with_summary(root, summary)
            .next()
            .is_some()
    }

    fn project_with_summary(
        &self,
        root: &Path,
        _scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        let files = dotnet_project_files_with_summary(root, summary).collect::<Vec<_>>();
        let first = files.first()?;
        let language = if first.extension().and_then(|value| value.to_str()) == Some("fsproj") {
            "F#"
        } else {
            "C#"
        };
        Some(identity(
            root.to_path_buf(),
            Ecosystem::DotNet,
            ProjectTechnology::new(
                vec![language.to_owned()],
                Some(".NET".to_owned()),
                None,
                files
                    .into_iter()
                    .map(|path| evidence(path, ".NET project file"))
                    .collect(),
            ),
        ))
    }
}

fn dotnet_project_files(root: &Path) -> impl Iterator<Item = PathBuf> {
    read_dir_files(root).filter(|path| {
        matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("csproj" | "fsproj")
        )
    })
}

fn dotnet_project_files_with_summary(
    root: &Path,
    summary: &mut ScanAccessSummary,
) -> impl Iterator<Item = PathBuf> {
    dotnet_project_files(root).inspect(|_| summary.record_metadata_file())
}

pub struct PythonDetector;

impl Detector for PythonDetector {
    fn matches(&self, root: &Path) -> bool {
        python_metadata(root).next().is_some()
    }

    fn metadata_paths(&self) -> &[&str] {
        &[]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[
            ".venv",
            "__pycache__",
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            "build",
            "dist",
        ]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn matches_with_summary(&self, root: &Path, summary: &mut ScanAccessSummary) -> bool {
        python_metadata_with_summary(root, summary).next().is_some()
    }

    fn project_with_summary(
        &self,
        root: &Path,
        scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        let metadata = python_metadata_with_summary(root, summary).collect::<Vec<_>>();
        if !metadata.is_empty() {
            return Some(python_identity(root.to_path_buf(), metadata));
        }

        if !root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(is_python_cache_name)
        {
            return None;
        }

        let project_root = nearest_python_project_root(root, scan_root, summary)?;
        let metadata = python_metadata_with_summary(&project_root, summary).collect::<Vec<_>>();
        (!metadata.is_empty()).then(|| python_identity(project_root, metadata))
    }

    fn artifacts_for_project_with_summary(
        &self,
        root: &Path,
        project: &ProjectIdentity,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> Vec<Artifact> {
        let is_nested_cache = root != project.root
            && root.starts_with(&project.root)
            && root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_python_cache_name);
        if is_nested_cache {
            return artifact_from_directory(root.to_path_buf(), project, summary)
                .into_iter()
                .collect();
        }

        let mut candidate_paths = self
            .artifact_names(root)
            .into_iter()
            .map(|name| root.join(name))
            .collect::<Vec<_>>();
        candidate_paths.extend(python_nested_cache_paths(root));
        candidate_paths.sort_unstable();
        candidate_paths.dedup();

        let mut artifacts = Vec::new();
        for path in candidate_paths {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !self.artifact_path_is_valid(root, name, summary.as_deref_mut()) {
                continue;
            }
            if let Some(artifact) = artifact_from_directory(path, project, summary.as_deref_mut()) {
                artifacts.push(artifact);
            }
        }
        artifacts
    }

    fn is_artifact_directory_with_summary(
        &self,
        path: &Path,
        summary: &mut ScanAccessSummary,
    ) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        if is_python_cache_name(name) {
            let scan_root = summary.root.clone();
            return nearest_python_project_root(path, &scan_root, summary).is_some();
        }

        path.parent().is_some_and(|parent| {
            self.matches_with_summary(parent, summary)
                && self.artifact_path_is_valid(parent, name, None)
        })
    }

    fn artifact_path_is_valid(
        &self,
        root: &Path,
        artifact_name: &str,
        _summary: Option<&mut ScanAccessSummary>,
    ) -> bool {
        if matches!(artifact_name, "build" | "dist") {
            python_packaging_metadata(root)
        } else {
            true
        }
    }
}

fn python_identity(root: PathBuf, metadata: Vec<PathBuf>) -> ProjectIdentity {
    identity(
        root,
        Ecosystem::Python,
        ProjectTechnology::new(
            vec!["Python".to_owned()],
            None,
            None,
            metadata
                .into_iter()
                .map(|path| evidence(path, "Python project metadata"))
                .collect(),
        ),
    )
}

fn is_python_cache_name(name: &str) -> bool {
    matches!(
        name,
        "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache"
    )
}

fn nearest_python_project_root(
    path: &Path,
    scan_root: &Path,
    summary: &mut ScanAccessSummary,
) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .take_while(|ancestor| ancestor.starts_with(scan_root))
        .find(|ancestor| {
            python_metadata_with_summary(ancestor, summary)
                .next()
                .is_some()
        })
        .map(Path::to_path_buf)
}

fn python_nested_cache_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut entries = WalkDir::new(root).min_depth(1).into_iter();

    while let Some(entry) = entries.next() {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if is_python_cache_name(name) {
            paths.push(path.to_path_buf());
            entries.skip_current_dir();
            continue;
        }

        let is_known_artifact = DETECTORS
            .iter()
            .any(|detector| detector.artifact_paths().contains(&name))
            || (name.starts_with("cmake-build-") && name.len() > "cmake-build-".len());
        let is_nested_project = DETECTORS.iter().any(|detector| detector.matches(path));
        if is_known_artifact || is_nested_project {
            entries.skip_current_dir();
        }
    }

    paths
}

fn python_metadata(root: &Path) -> impl Iterator<Item = PathBuf> {
    read_dir_files(root).filter(|path| is_python_metadata(path))
}

fn python_metadata_with_summary(
    root: &Path,
    summary: &mut ScanAccessSummary,
) -> impl Iterator<Item = PathBuf> {
    python_metadata(root).inspect(|_| summary.record_metadata_file())
}

fn is_python_metadata(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name == "pyproject.toml"
                || name == "setup.py"
                || name == "setup.cfg"
                || (name.starts_with("requirements") && name.ends_with(".txt"))
        })
}

fn python_packaging_metadata(root: &Path) -> bool {
    ["pyproject.toml", "setup.py", "setup.cfg"]
        .iter()
        .any(|name| {
            root.join(name).is_file()
                && (name != &"pyproject.toml"
                    || metadata_text(&root.join(name)).is_some_and(|contents| {
                        contents.contains("[build-system]") || contents.contains("[project]")
                    }))
        })
}

pub struct SwiftDetector;

impl Detector for SwiftDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("Package.swift").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["Package.swift"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[".build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Swift
    }
}

pub struct DartDetector;

impl Detector for DartDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("pubspec.yaml").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["pubspec.yaml"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[".dart_tool", "build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Dart
    }

    fn supports_ecosystem(&self, ecosystem: Ecosystem) -> bool {
        matches!(ecosystem, Ecosystem::Dart | Ecosystem::Flutter)
    }

    fn project_with_summary(
        &self,
        root: &Path,
        _scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        if !self.matches_with_summary(root, summary) {
            return None;
        }
        let contents =
            metadata_text_with_summary(&root.join("pubspec.yaml"), summary).unwrap_or_default();
        let flutter = is_flutter_pubspec(&contents);
        let ecosystem = if flutter {
            Ecosystem::Flutter
        } else {
            Ecosystem::Dart
        };
        Some(identity(
            root.to_path_buf(),
            ecosystem,
            ProjectTechnology::new(
                vec!["Dart".to_owned()],
                flutter.then_some("Dart".to_owned()),
                flutter.then_some("Flutter".to_owned()),
                vec![evidence("pubspec.yaml", "Dart package metadata")],
            ),
        ))
    }

    fn artifact_path_is_valid(
        &self,
        root: &Path,
        artifact_name: &str,
        _summary: Option<&mut ScanAccessSummary>,
    ) -> bool {
        artifact_name == ".dart_tool"
            || (artifact_name == "build"
                && root.join("pubspec.yaml").is_file()
                && metadata_text(&root.join("pubspec.yaml"))
                    .is_some_and(|contents| is_flutter_pubspec(&contents)))
    }
}

fn is_flutter_pubspec(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "flutter:" || trimmed.starts_with("flutter:") || trimmed.contains("sdk: flutter")
    })
}

pub struct PhpDetector;

impl Detector for PhpDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("composer.json").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["composer.json"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["vendor"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Php
    }
}

pub struct ElixirDetector;

impl Detector for ElixirDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("mix.exs").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["mix.exs"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["_build", "deps"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Elixir
    }
}

pub struct ZigDetector;

impl Detector for ZigDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("build.zig").is_file() || root.join("build.zig.zon").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["build.zig", "build.zig.zon"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[".zig-cache", "zig-out"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Zig
    }
}

/// Go and Ruby are intentionally detection-only in this wave: their global
/// or user-defined build/cache ownership is not safe to infer as cleanup.
pub struct GoDetector;

impl Detector for GoDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("go.mod").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["go.mod"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Go
    }
}

pub struct RubyDetector;

impl Detector for RubyDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("Gemfile").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["Gemfile"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Ruby
    }
}

fn read_dir_files(root: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|kind| kind.is_file())
                .map(|_| entry.path())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detectors_returns_all_when_filter_is_empty() {
        let detectors = select_detectors(&[]);

        assert!(detectors.len() >= 13);
        assert!(
            detectors
                .iter()
                .any(|detector| detector.ecosystem() == Ecosystem::Rust)
        );
        assert!(
            detectors
                .iter()
                .any(|detector| detector.ecosystem() == Ecosystem::Node)
        );
        assert!(
            detectors
                .iter()
                .any(|detector| detector.ecosystem() == Ecosystem::Java)
        );
    }

    #[test]
    fn detectors_filters_kotlin_to_the_gradle_detector() {
        let detectors = select_detectors(&[Ecosystem::Kotlin]);

        assert_eq!(detectors.len(), 1);
        assert_eq!(detectors[0].ecosystem(), Ecosystem::Java);
    }
}
