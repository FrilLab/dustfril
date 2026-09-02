use std::path::{Path, PathBuf};

use crate::{
    dependency, dependency_baseline,
    error::DustResult,
    models::{DependencyDiff, DependencyReport, Ecosystem},
};

pub use crate::dependency_baseline::DependencyBaselineStore;

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

/// Returns the OS-specific local path used for dependency baselines.
pub fn dependency_baseline_path() -> std::io::Result<PathBuf> {
    dependency_baseline::default_state_path()
}

/// Compares an already parsed inventory with the explicit local baseline.
///
/// The reports are accepted as input so callers can reuse one parsed
/// inventory for both reporting and comparison.
pub fn dependency_diff(
    root: &Path,
    reports: &[DependencyReport],
    baseline_path: &Path,
) -> DustResult<DependencyDiff> {
    DependencyBaselineStore::new(baseline_path).compare(root, reports)
}

/// Explicitly replaces the selected workspace inventories in the local
/// baseline after the caller has inspected a diff.
pub fn accept_dependency_baseline(
    root: &Path,
    reports: &[DependencyReport],
    baseline_path: &Path,
) -> DustResult<()> {
    DependencyBaselineStore::new(baseline_path).accept(root, reports)
}

/// Parses the current inventory once and compares it with the local baseline.
pub fn dependency_changes(
    root: &Path,
    ecosystems: &[Ecosystem],
    baseline_path: &Path,
) -> DustResult<DependencyDiff> {
    let reports = dependency_report(root, ecosystems)?;
    dependency_diff(root, &reports, baseline_path)
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
        assert_eq!(reports[0].resolved_dependencies[0].name, "left-pad");
        assert_eq!(
            reports[0].resolved_dependencies[0].scope,
            crate::models::DependencyScope::Direct
        );
    }

    #[test]
    fn api_compares_the_same_parsed_inventory_and_accepts_explicitly() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"left-pad":"1.0.0"}}"#,
        )
        .unwrap();
        let lockfile = temp_dir.path().join("package-lock.json");
        fs::write(
            &lockfile,
            r#"{"lockfileVersion":3,"packages":{"":{"name":"demo"},"node_modules/left-pad":{"version":"1.0.0"}}}"#,
        )
        .unwrap();
        let baseline_path = temp_dir.path().join("dependency-baseline.json");

        let first_reports = dependency_report(temp_dir.path(), &[Ecosystem::Node]).unwrap();
        let first = dependency_diff(temp_dir.path(), &first_reports, &baseline_path).unwrap();
        assert_eq!(
            first.baseline_status,
            crate::models::DependencyBaselineStatus::BaselineCreated
        );
        assert!(!first.has_changes());

        fs::write(
            &lockfile,
            r#"{"lockfileVersion":3,"packages":{"node_modules/new-package":{"version":"1.0.0"},"":{"name":"demo"},"node_modules/left-pad":{"version":"1.0.0"}}}"#,
        )
        .unwrap();
        let current_reports = dependency_report(temp_dir.path(), &[Ecosystem::Node]).unwrap();
        let diff = dependency_diff(temp_dir.path(), &current_reports, &baseline_path).unwrap();
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].current.as_ref().unwrap().name, "new-package");
        assert_eq!(
            diff.added[0].current.as_ref().unwrap().scope,
            crate::models::DependencyScope::Transitive
        );

        accept_dependency_baseline(temp_dir.path(), &current_reports, &baseline_path).unwrap();
        let accepted = dependency_diff(temp_dir.path(), &current_reports, &baseline_path).unwrap();
        assert!(!accepted.has_changes());
    }
}
