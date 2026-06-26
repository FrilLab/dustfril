use std::path::Path;

use crate::models::Ecosystem;

pub static DETECTORS: &[&dyn Detector] = &[&RustDetector, &NodeDetector, &JavaDetector];

pub trait Detector: Sync {
    fn detect(&self, root: &Path) -> bool;

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
    fn detect(&self, root: &Path) -> bool {
        root.join("Cargo.toml").is_file()
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }
}
pub struct NodeDetector;

impl Detector for NodeDetector {
    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").is_file()
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }
}
pub struct JavaDetector;

impl Detector for JavaDetector {
    fn detect(&self, root: &Path) -> bool {
        root.join("pom.xml").is_file()
            || root.join("build.gradle").is_file()
            || root.join("build.gradle.kts").is_file()
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Java
    }
}
