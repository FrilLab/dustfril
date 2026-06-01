use std::{fs, path::Path, time::SystemTime};

pub fn find_latest_modified(path: &Path) -> Option<SystemTime> {
    let mut latest = fs::metadata(path).ok()?.modified().ok();

    let Ok(entries) = fs::read_dir(path) else {
        return latest;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let metadata = entry.metadata().ok();

        let current = match metadata {
            Some(metadata) if metadata.is_dir() => find_latest_modified(&path),

            Some(metadata) => metadata.modified().ok(),

            None => None,
        };

        if let Some(current) = current {
            match latest {
                Some(existing) if current <= existing => {}

                _ => {
                    latest = Some(current);
                }
            }
        }
    }

    latest
}
