//! Analyzer module.
mod age;
mod analyze;
mod format;
mod metadata;
mod recommendation;
mod size;

#[cfg(test)]
mod tests;

pub use age::calculate_age_days;
pub use analyze::analyze;
pub use format::{format_modified, format_size};
pub use metadata::get_latest_modified;
pub use recommendation::recommend_cleanup;
pub use size::calculate_directory_size;
