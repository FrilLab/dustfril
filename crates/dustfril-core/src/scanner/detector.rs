use std::path::Path;

use crate::models::{Artifact, Ecosystem};

/// Registered detectors for all supported ecosystems.
pub static DETECTORS: &[&dyn Detector] = &[&RustDetector, &NodeDetector, &JavaDetector];

/// Matches project roots and returns removable artifact directories.
pub trait Detector: Sync {
    /// Returns true if this directory is a project of this ecosystem.
    fn matches(&self, root: &Path) -> bool;

    /// Artifact directory names managed by this detector.
    fn artifact_paths(&self) -> &[&str];

    /// Ecosystem handled by this detector.
    fn ecosystem(&self) -> Ecosystem;

    /// Finds removable artifacts inside the project.
    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        self.artifact_paths()
            .iter()
            .map(|name| root.join(name))
            .filter(|path| path.is_dir())
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
