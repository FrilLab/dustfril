use std::{fs, path::Path, time::SystemTime};

pub fn get_latest_modified(path: &Path) -> Option<SystemTime> {
    let mut latest = fs::metadata(path).ok()?.modified().ok();

    let Ok(entries) = fs::read_dir(path) else {
        return latest;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let current = if path.is_dir() {
            get_latest_modified(&path)
        } else {
            entry.metadata().ok().and_then(|m| m.modified().ok())
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
