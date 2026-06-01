use std::{fs, path::Path};

pub fn calculate_directory_size(path: &Path) -> u64 {
    let mut total_size = 0;

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_directory_size(&entry.path());
        }
    }

    total_size
}
