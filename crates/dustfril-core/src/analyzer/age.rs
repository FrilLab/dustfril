use std::time::{SystemTime, UNIX_EPOCH};

pub fn calculate_age_days(modified: Option<SystemTime>) -> Option<u64> {
    let modified = modified?;

    let now = SystemTime::now();

    let duration = now.duration_since(modified).ok()?;

    Some(duration.as_secs() / 86_400)
}
