use std::path::Path;

use crate::{
    dependency,
    error::DustResult,
    models::{DependencyReport, Ecosystem},
};

/// Builds deterministic dependency inventory reports without traversing
/// installed dependency trees or contacting package registries.
pub fn dependency_report(
    root: &Path,
    ecosystems: &[Ecosystem],
) -> DustResult<Vec<DependencyReport>> {
    dependency::report(root, ecosystems)
}

/// Alias emphasizing the exposure/inventory meaning of the report.
pub fn dependency_exposure_report(
    root: &Path,
    ecosystems: &[Ecosystem],
) -> DustResult<Vec<DependencyReport>> {
    dependency_report(root, ecosystems)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn api_exposes_node_dependency_report() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"left-pad":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{"lockfileVersion":3,"packages":{"":{"name":"demo","version":"1.0.0"},"node_modules/left-pad":{"version":"1.0.0"}}}"#,
        )
        .unwrap();

        let reports = dependency_report(temp_dir.path(), &[Ecosystem::Node]).unwrap();

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].resolved_dependency_count.value, Some(1));
    }
}
