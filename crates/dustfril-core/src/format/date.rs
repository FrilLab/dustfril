use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn format_modified(modified: Option<SystemTime>) -> String {
    let Some(modified) = modified else {
        return "Unknown".to_string();
    };

    let datetime: DateTime<Local> = modified.into();

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
