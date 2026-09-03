/// Formats a byte count using binary size units for human-readable CLI output.
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

#[test]
fn format_size_bytes() {
    assert_eq!(format_size(512), "512 B");
}

#[test]
fn format_size_kilobytes() {
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(2048), "2.00 KB");
}

#[test]
fn format_size_megabytes() {
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
}

#[test]
fn format_size_gigabytes() {
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn format_size_terabytes() {
    assert_eq!(format_size(1024_u64.pow(4)), "1.00 TB");
}
