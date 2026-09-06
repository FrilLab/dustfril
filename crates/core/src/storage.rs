use std::path::Path;

use crate::{
    error::{DustError, DustResult},
    models::{
        AnalysisResult, CleanupRecommendation, DeveloperStorageSummary, ScanAccessSummary,
        StorageSummary, VolumeStorage,
    },
};

/// Builds the Overview storage context without traversing the workspace.
///
/// Filesystem capacity is read from the path itself, so the returned volume is
/// the volume containing the selected workspace rather than a hard-coded
/// system volume. Development storage is reused from the supplied completed
/// analysis and therefore does not perform another artifact-size walk.
pub fn summarize(workspace: &Path, analysis: &AnalysisResult) -> DustResult<StorageSummary> {
    summarize_with_access_summary(workspace, analysis, None)
}

/// Reads the capacity of the filesystem containing the selected workspace.
pub fn volume(workspace: &Path) -> DustResult<VolumeStorage> {
    let filesystem = fs2::statvfs(workspace).map_err(|source| DustError::FilesystemStats {
        path: workspace.to_path_buf(),
        source,
    })?;

    Ok(VolumeStorage::from_filesystem_values(
        filesystem.total_space(),
        filesystem.available_space(),
    ))
}

/// Builds the storage context while retaining bounded scan-coverage warnings.
///
/// A partial scan still contributes the bytes DustFril measured, but the
/// caller can present that result as partial instead of implying full
/// workspace coverage.
pub fn summarize_with_access_summary(
    workspace: &Path,
    analysis: &AnalysisResult,
    access_summary: Option<&ScanAccessSummary>,
) -> DustResult<StorageSummary> {
    let volume = volume(workspace)?;

    let recommended_bytes = analysis
        .artifacts
        .iter()
        .filter(|artifact| artifact.recommendation == CleanupRecommendation::SafeToClean)
        .map(|artifact| artifact.size_bytes)
        .fold(0, u64::saturating_add);

    let mut categories = analysis
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact.ecosystem)
        .collect::<Vec<_>>();
    categories.sort_unstable();
    categories.dedup();

    let mut warnings = Vec::new();
    if analysis.measurement_failures > 0 {
        warnings.push(format!(
            "Workspace analysis was partial: {} artifact measurement failure(s) were not measured.",
            analysis.measurement_failures
        ));
    }
    if let Some(summary) = access_summary.filter(|summary| summary.failures > 0) {
        warnings.push(format!(
            "Workspace analysis was partial: {} filesystem access failure(s) were not measured.",
            summary.failures
        ));
    }
    let partial = !warnings.is_empty();

    Ok(StorageSummary {
        volume,
        developer_storage: DeveloperStorageSummary {
            measured_bytes: analysis.total_size_bytes,
            recommended_bytes,
            scope_path: workspace.to_path_buf(),
            categories,
        },
        partial,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::models::{Artifact, ArtifactAnalysis, Ecosystem};

    fn analysis(
        artifacts: Vec<(PathBuf, Ecosystem, u64, CleanupRecommendation)>,
    ) -> AnalysisResult {
        let total_size_bytes = artifacts
            .iter()
            .map(|(_, _, size_bytes, _)| *size_bytes)
            .fold(0, u64::saturating_add);

        AnalysisResult {
            artifacts: artifacts
                .into_iter()
                .map(
                    |(path, ecosystem, size_bytes, recommendation)| ArtifactAnalysis {
                        artifact: Artifact::new(path, ecosystem),
                        size_bytes,
                        last_modified: None,
                        age_days: None,
                        recommendation,
                    },
                )
                .collect(),
            total_size_bytes,
            ..AnalysisResult::default()
        }
    }

    #[test]
    fn volume_values_preserve_the_capacity_relationship() {
        let volume = VolumeStorage::from_filesystem_values(100, 35);

        assert_eq!(volume.used_bytes, 65);
        assert_eq!(
            volume.used_bytes + volume.available_bytes,
            volume.total_bytes
        );
    }

    #[test]
    fn inconsistent_available_value_is_clamped() {
        let volume = VolumeStorage::from_filesystem_values(100, 125);

        assert_eq!(volume.used_bytes, 0);
        assert_eq!(volume.available_bytes, 100);
    }

    #[test]
    fn workspace_path_uses_the_containing_filesystem() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();

        let root_stats = fs2::statvfs(root.path()).unwrap();
        let summary = summarize(&workspace, &AnalysisResult::default()).unwrap();

        assert_eq!(summary.volume.total_bytes, root_stats.total_space());
        assert_eq!(
            summary.volume.available_bytes,
            root_stats.available_space().min(root_stats.total_space())
        );
    }

    #[test]
    fn invalid_workspace_path_reports_filesystem_error() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("missing-workspace");

        let error = summarize(&path, &AnalysisResult::default()).unwrap_err();

        assert!(
            matches!(error, DustError::FilesystemStats { path: ref error_path, .. } if error_path == &path)
        );
        assert!(!error.to_string().contains("0 B"));
    }

    #[test]
    fn volume_reads_capacity_without_an_analysis() {
        let root = tempfile::tempdir().unwrap();

        let volume = volume(root.path()).unwrap();

        assert!(volume.total_bytes > 0);
        assert!(volume.available_bytes <= volume.total_bytes);
        assert_eq!(
            volume.used_bytes + volume.available_bytes,
            volume.total_bytes
        );
    }

    #[test]
    fn measured_storage_includes_all_recommendation_states_and_separates_reclaimable_bytes() {
        let root = tempfile::tempdir().unwrap();
        let analysis = analysis(vec![
            (
                root.path().join("target"),
                Ecosystem::Rust,
                10,
                CleanupRecommendation::Keep,
            ),
            (
                root.path().join("node_modules"),
                Ecosystem::Node,
                20,
                CleanupRecommendation::NeedsReview,
            ),
            (
                root.path().join("build"),
                Ecosystem::Java,
                30,
                CleanupRecommendation::SafeToClean,
            ),
        ]);

        let summary = summarize(root.path(), &analysis).unwrap();

        assert_eq!(summary.developer_storage.measured_bytes, 60);
        assert_eq!(summary.developer_storage.recommended_bytes, 30);
        assert_eq!(summary.developer_storage.scope_path, root.path());
        assert_eq!(
            summary.developer_storage.categories,
            vec![Ecosystem::Rust, Ecosystem::Node, Ecosystem::Java]
        );
    }

    #[test]
    fn share_percentage_is_calculated_against_used_bytes() {
        let summary = StorageSummary {
            volume: VolumeStorage::from_filesystem_values(1_000, 500),
            developer_storage: DeveloperStorageSummary {
                measured_bytes: 50,
                recommended_bytes: 25,
                scope_path: PathBuf::from("/workspace"),
                categories: vec![Ecosystem::Rust],
            },
            partial: false,
            warnings: vec![],
        };

        assert_eq!(summary.detected_share_percent(), Some(10.0));
    }

    #[test]
    fn zero_used_bytes_have_no_share_percentage() {
        let summary = StorageSummary {
            volume: VolumeStorage::from_filesystem_values(0, 0),
            developer_storage: DeveloperStorageSummary {
                measured_bytes: 50,
                recommended_bytes: 0,
                scope_path: PathBuf::from("/workspace"),
                categories: vec![],
            },
            partial: false,
            warnings: vec![],
        };

        assert_eq!(summary.detected_share_percent(), None);
    }

    #[test]
    fn partial_scan_coverage_is_exposed_without_discarding_measured_bytes() {
        let root = tempfile::tempdir().unwrap();
        let analysis = analysis(vec![(
            root.path().join("target"),
            Ecosystem::Rust,
            42,
            CleanupRecommendation::Keep,
        )]);
        let mut access_summary = ScanAccessSummary::new(root.path());
        access_summary.record_failure(root.path(), "permission denied");

        let summary =
            summarize_with_access_summary(root.path(), &analysis, Some(&access_summary)).unwrap();

        assert!(summary.partial);
        assert_eq!(summary.developer_storage.measured_bytes, 42);
        assert_eq!(summary.warnings.len(), 1);
    }

    #[test]
    fn partial_artifact_measurement_is_exposed_without_discarding_measured_bytes() {
        let root = tempfile::tempdir().unwrap();
        let mut analysis = analysis(vec![(
            root.path().join("target"),
            Ecosystem::Rust,
            42,
            CleanupRecommendation::Keep,
        )]);
        analysis.measurement_failures = 2;

        let summary = summarize(root.path(), &analysis).unwrap();

        assert!(summary.partial);
        assert_eq!(summary.developer_storage.measured_bytes, 42);
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.warnings[0].contains("2 artifact measurement failure(s)"));
    }
}
