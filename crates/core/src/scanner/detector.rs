use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::{Artifact, Ecosystem, ProjectIdentity, ScanAccessSummary};

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

/// Registered detectors for all supported ecosystems.
pub static DETECTORS: &[&dyn Detector] = &[&RustDetector, &NodeDetector, &JavaDetector];

/// Matches project roots and returns removable artifact directories.
pub trait Detector: Sync {
    /// Returns true if this directory is a project of this ecosystem.
    #[allow(dead_code)]
    fn matches(&self, root: &Path) -> bool;

    /// Recognized project metadata names checked by this detector.
    fn metadata_paths(&self) -> &[&str];

    /// Artifact directory names managed by this detector.
    fn artifact_paths(&self) -> &[&str];

    /// Ecosystem handled by this detector.
    fn ecosystem(&self) -> Ecosystem;

    /// Discovers the project that owns artifacts in this directory.
    ///
    /// Detectors may return a different root when an ecosystem has a
    /// workspace/module hierarchy. Analysis and cleanup only consume this
    /// identity; they do not need ecosystem-specific project logic.
    fn project_with_summary(
        &self,
        root: &Path,
        _scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        self.matches_with_summary(root, summary)
            .then(|| ProjectIdentity::new(root.to_path_buf(), self.ecosystem()))
    }

    /// Matches a project while recording only metadata files actually found
    /// and inspected by the detector.
    fn matches_with_summary(&self, root: &Path, summary: &mut ScanAccessSummary) -> bool {
        self.metadata_paths()
            .iter()
            .any(|name| metadata_file_exists_with_summary(&root.join(name), summary))
    }

    /// Finds removable artifacts inside the project.
    #[allow(dead_code)]
    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        self.artifacts_with_summary(root, None)
    }

    /// Finds artifacts and records detector-access failures in the scan
    /// summary when one is supplied.
    fn artifacts_with_summary(
        &self,
        root: &Path,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> Vec<Artifact> {
        let Some(project) = summary
            .as_deref_mut()
            .and_then(|summary| self.project_with_summary(root, root, summary))
            .or_else(|| {
                self.matches(root)
                    .then(|| ProjectIdentity::new(root.to_path_buf(), self.ecosystem()))
            })
        else {
            return Vec::new();
        };

        self.artifacts_for_project_with_summary(root, &project, summary)
    }

    /// Finds artifacts under a discovered project while retaining its
    /// identity on every artifact.
    fn artifacts_for_project_with_summary(
        &self,
        root: &Path,
        project: &ProjectIdentity,
        mut summary: Option<&mut ScanAccessSummary>,
    ) -> Vec<Artifact> {
        self.artifact_paths()
            .iter()
            .map(|name| root.join(name))
            .filter(|path| match fs::symlink_metadata(path) {
                Ok(metadata) => metadata.is_dir(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => {
                    if let Some(summary) = summary.as_deref_mut() {
                        summary.record_failure(path, &error.to_string());
                    }
                    false
                }
            })
            .map(|path| Artifact::for_project(path, project.clone()))
            .collect()
    }
}

/// Returns the detector set matching the requested ecosystem filters.
pub fn select_detectors(ecosystems: &[Ecosystem]) -> Vec<&'static dyn Detector> {
    if ecosystems.is_empty() {
        return DETECTORS.to_vec();
    }

    DETECTORS
        .iter()
        .copied()
        .filter(|detector| ecosystems.contains(&detector.ecosystem()))
        .collect()
}
pub fn detector_for(ecosystem: Ecosystem) -> Option<&'static dyn Detector> {
    DETECTORS
        .iter()
        .copied()
        .find(|detector| detector.ecosystem() == ecosystem)
}

/// Detects Cargo `target/` directories.
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

/// Detects `node_modules/` directories for Node projects.
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
        ]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["node_modules"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }
}

fn node_marker(root: &Path) -> bool {
    [
        "package.json",
        "pnpm-workspace.yaml",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
    ]
    .iter()
    .any(|name| root.join(name).is_file())
}
/// Detects `build/` directories for Maven and Gradle projects.
pub struct JavaDetector;

#[derive(Clone, Copy)]
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
        &["build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Java
    }

    fn project_with_summary(
        &self,
        root: &Path,
        scan_root: &Path,
        summary: &mut ScanAccessSummary,
    ) -> Option<ProjectIdentity> {
        let marker = java_marker_with_summary(root, summary)?;
        let project_root = match marker {
            JavaMarker::GradleBuild => {
                find_gradle_root(root, scan_root, summary).unwrap_or_else(|| root.to_path_buf())
            }
            JavaMarker::Maven | JavaMarker::GradleSettings => root.to_path_buf(),
        };

        Some(ProjectIdentity::new(project_root, self.ecosystem()))
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

fn java_marker_with_summary(root: &Path, summary: &mut ScanAccessSummary) -> Option<JavaMarker> {
    if metadata_file_exists_with_summary(&root.join("settings.gradle"), summary)
        || metadata_file_exists_with_summary(&root.join("settings.gradle.kts"), summary)
    {
        Some(JavaMarker::GradleSettings)
    } else if metadata_file_exists_with_summary(&root.join("pom.xml"), summary) {
        Some(JavaMarker::Maven)
    } else if metadata_file_exists_with_summary(&root.join("build.gradle"), summary)
        || metadata_file_exists_with_summary(&root.join("build.gradle.kts"), summary)
    {
        Some(JavaMarker::GradleBuild)
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detectors_returns_all_when_filter_is_empty() {
        let detectors = select_detectors(&[]);

        assert_eq!(detectors.len(), 3);
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
    fn detectors_filters_to_requested_ecosystem() {
        let detectors = select_detectors(&[Ecosystem::Node]);

        assert_eq!(detectors.len(), 1);
        assert_eq!(detectors[0].ecosystem(), Ecosystem::Node);
    }
}
