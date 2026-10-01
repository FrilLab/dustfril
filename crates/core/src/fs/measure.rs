use std::{fs, path::Path, time::SystemTime};

use walkdir::WalkDir;

/// Maximum number of representative access failures retained for one tree.
pub const MAX_MEASUREMENT_FAILURE_SAMPLES: usize = 8;

/// One-pass, no-follow measurement of a directory tree.
#[derive(Debug, Default)]
pub struct DirectoryMeasurement {
    pub size_bytes: u64,
    pub latest_modified: Option<SystemTime>,
    pub failures: u64,
    pub failure_samples: Vec<String>,
}

/// Measures file bytes and modification time without retaining all paths.
/// Symbolic links are never followed or counted as file contents.
pub fn measure_directory(root: &Path) -> DirectoryMeasurement {
    let mut measurement = DirectoryMeasurement::default();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                record_failure(&mut measurement, root, error.path().unwrap_or(root));
                continue;
            }
        };

        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => {
                record_failure(&mut measurement, root, entry.path());
                continue;
            }
        };

        if metadata.is_file() {
            measurement.size_bytes = measurement.size_bytes.saturating_add(metadata.len());
        }

        match metadata.modified() {
            Ok(modified) => {
                measurement.latest_modified = measurement.latest_modified.max(Some(modified));
            }
            Err(_) => record_failure(&mut measurement, root, entry.path()),
        }
    }

    measurement
}

fn record_failure(measurement: &mut DirectoryMeasurement, root: &Path, path: &Path) {
    measurement.failures = measurement.failures.saturating_add(1);
    if measurement.failure_samples.len() < MAX_MEASUREMENT_FAILURE_SAMPLES {
        let sample = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        measurement.failure_samples.push(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_read_failure_samples_are_bounded() {
        let root = Path::new("/cache/root");
        let mut measurement = DirectoryMeasurement::default();

        for index in 0..100 {
            record_failure(&mut measurement, root, &root.join(format!("entry-{index}")));
        }

        assert_eq!(measurement.failures, 100);
        assert_eq!(
            measurement.failure_samples.len(),
            MAX_MEASUREMENT_FAILURE_SAMPLES
        );
        assert_eq!(measurement.failure_samples[0], "entry-0");
        assert_eq!(measurement.failure_samples[7], "entry-7");
    }
}
