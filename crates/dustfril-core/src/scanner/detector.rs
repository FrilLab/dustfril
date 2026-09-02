use std::{fs, path::Path};

use crate::models::{Artifact, Ecosystem, ScanAccessSummary};

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

    /// Matches a project while recording only metadata files actually found
    /// and inspected by the detector.
    fn matches_with_summary(&self, root: &Path, summary: &mut ScanAccessSummary) -> bool {
        self.metadata_paths().iter().any(|name| {
            let path = root.join(name);
            match fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => {
                    summary.record_metadata_file();
                    true
                }
                Ok(_) => false,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => {
                    summary.record_failure(&path, &error.to_string());
                    false
                }
            }
        })
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
            .map(|path| Artifact::new(path, self.ecosystem()))
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
        root.join("package.json").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["package.json"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["node_modules"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }
}
/// Detects `build/` directories for Maven and Gradle projects.
pub struct JavaDetector;

impl Detector for JavaDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("pom.xml").is_file()
            || root.join("build.gradle").is_file()
            || root.join("build.gradle.kts").is_file()
    }

    fn metadata_paths(&self) -> &[&str] {
        &["pom.xml", "build.gradle", "build.gradle.kts"]
    }

    fn artifact_paths(&self) -> &[&str] {
        &["build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Java
    }
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
