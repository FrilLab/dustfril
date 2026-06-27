use std::path::Path;

use crate::models::{Artifact, Ecosystem};

pub static DETECTORS: &[&dyn Detector] = &[&RustDetector, &NodeDetector, &JavaDetector];

pub trait Detector: Sync {
    /// Is this directory a project of this ecosystem?
    fn matches(&self, root: &Path) -> bool;

    /// Find removable artifacts inside this project.
    fn artifacts(&self, root: &Path) -> Vec<Artifact>;

    fn ecosystem(&self) -> Ecosystem;
}

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
