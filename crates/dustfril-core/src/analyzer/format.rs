pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let bytes = bytes as f64;

    if bytes >= TB {
        format!("{:.2} TB", bytes / TB)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes / KB)
    } else {
        format!("{:.0} B", bytes)
    }
}

use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn format_modified(modified: Option<SystemTime>) -> String {
    let Some(modified) = modified else {
        return "Unknown".to_string();
    };

    let datetime: DateTime<Local> = modified.into();

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
