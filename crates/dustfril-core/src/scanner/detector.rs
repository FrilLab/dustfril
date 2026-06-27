use std::path::Path;

use crate::models::{Artifact, Ecosystem};

/// Registered detectors for all supported ecosystems.
pub static DETECTORS: &[&dyn Detector] = &[&RustDetector, &NodeDetector, &JavaDetector];

/// Matches project roots and returns removable artifact directories.
pub trait Detector: Sync {
    /// Is this directory a project of this ecosystem?
    fn matches(&self, root: &Path) -> bool;

    /// Find removable artifacts inside this project.
    fn artifacts(&self, root: &Path) -> Vec<Artifact>;

    fn ecosystem(&self) -> Ecosystem;
}

/// Returns the detector set matching the requested ecosystem filters.
pub fn detectors(ecosystems: &[Ecosystem]) -> Vec<&'static dyn Detector> {
    if ecosystems.is_empty() {
        return DETECTORS.to_vec();
    }

    DETECTORS
        .iter()
        .copied()
        .filter(|detector| ecosystems.contains(&detector.ecosystem()))
        .collect()
}

/// Detects Cargo `target/` directories.
pub struct RustDetector;

impl Detector for RustDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("Cargo.toml").is_file() && root.join("target").is_dir()
    }

    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        let target = root.join("target");

        if target.is_dir() {
            artifacts.push(Artifact::new(target, Ecosystem::Rust));
        }

        artifacts
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }
}

/// Detects `node_modules/` directories for Node projects.
pub struct NodeDetector;

impl Detector for NodeDetector {
    fn matches(&self, root: &Path) -> bool {
        root.join("package.json").is_file() && root.join("node_modules").is_dir()
    }

    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        let modules = root.join("node_modules");

        if modules.is_dir() {
            artifacts.push(Artifact::new(modules, Ecosystem::Node));
        }

        artifacts
    }
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }
}

/// Detects `build/` directories for Maven and Gradle projects.
pub struct JavaDetector;

impl Detector for JavaDetector {
    fn matches(&self, root: &Path) -> bool {
        (root.join("pom.xml").is_file()
            || root.join("build.gradle").is_file()
            || root.join("build.gradle.kts").is_file())
            && root.join("build").is_dir()
    }

    fn artifacts(&self, root: &Path) -> Vec<Artifact> {
        let mut artifacts = Vec::new();
        let build = root.join("build");

        if build.is_dir()
            && (root.join("pom.xml").is_file()
                || root.join("build.gradle").is_file()
                || root.join("build.gradle.kts").is_file())
        {
            artifacts.push(Artifact::new(build, Ecosystem::Java));
        }

        artifacts
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
        let detectors = detectors(&[]);

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
        let detectors = detectors(&[Ecosystem::Node]);

        assert_eq!(detectors.len(), 1);
        assert_eq!(detectors[0].ecosystem(), Ecosystem::Node);
    }
}
