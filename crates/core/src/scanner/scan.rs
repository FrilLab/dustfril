use std::path::Path;

use crate::{
    error::{DustError, DustResult},
    fs::walk_dirs_with_summary_and_boundary,
    models::{Ecosystem, ScanAccessSummary, ScanResult, normalize_artifacts},
    scanner::detector::{self, metadata_file_exists_with_summary},
};

pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    let detectors = detector::select_detectors(ecosystems);

    let mut result = ScanResult {
        access_summary: ScanAccessSummary::new(root),
        ..ScanResult::default()
    };

    let directories = match walk_dirs_with_summary_and_boundary(
        root,
        &mut result.access_summary,
        is_discovery_boundary,
    ) {
        Ok(directories) => directories,
        Err(source @ DustError::InvalidPath(_)) => return Err(source),
        Err(source) => {
            return Err(DustError::ScanAccess {
                source: Box::new(source),
                access_summary: result.access_summary,
            });
        }
    };

    let mut artifacts = Vec::new();
    for dir in directories {
        for detector in &detectors {
            let Some(project) =
                detector.project_with_summary(&dir, root, &mut result.access_summary)
            else {
                continue;
            };

            artifacts.extend(detector.artifacts_for_project_with_summary(
                &dir,
                &project,
                Some(&mut result.access_summary),
            ));
        }
    }

    result.artifacts = normalize_artifacts(artifacts);
    for _ in &result.artifacts {
        result.access_summary.record_artifact_candidate();
    }

    Ok(result)
}

fn is_discovery_boundary(path: &Path, summary: &mut ScanAccessSummary) -> bool {
    let is_artifact_directory = path
        .file_name()
        .and_then(|name| name.to_str())
        .zip(path.parent())
        .is_some_and(|(name, parent)| {
            detector::DETECTORS.iter().any(|detector| {
                detector.artifact_paths().contains(&name)
                    && detector.metadata_paths().iter().any(|metadata| {
                        metadata_file_exists_with_summary(&parent.join(metadata), summary)
                    })
            })
        });

    #[cfg(target_os = "macos")]
    let is_application_bundle = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"));

    #[cfg(not(target_os = "macos"))]
    let is_application_bundle = false;

    is_artifact_directory || is_application_bundle
}
