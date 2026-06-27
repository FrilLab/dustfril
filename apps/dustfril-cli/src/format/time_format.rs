use chrono::{DateTime, Local};
use std::time::SystemTime;

/// Formats a filesystem timestamp for display in the local timezone.
pub fn format_modified(modified: Option<SystemTime>) -> String {
    let Some(modified) = modified else {
        return "Unknown".to_string();
    };

    let datetime: DateTime<Local> = modified.into();

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_modified_returns_unknown_for_missing_timestamp() {
        assert_eq!(format_modified(None), "Unknown");
    }

    #[test]
    fn format_modified_returns_timestamp_string() {
        let formatted = format_modified(Some(SystemTime::UNIX_EPOCH));

        assert_eq!(formatted.len(), 19);
        assert_eq!(&formatted[4..5], "-");
        assert_eq!(&formatted[7..8], "-");
        assert_eq!(&formatted[10..11], " ");
        assert_eq!(&formatted[13..14], ":");
        assert_eq!(&formatted[16..17], ":");
    }
}
