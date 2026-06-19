//! Analyzer module.
mod age;
mod analyze;
mod metadata;
mod recommendation;
mod size;

#[cfg(test)]
mod tests;

pub use age::calculate_age_days;
pub use analyze::analyze;
pub use metadata::find_latest_modified;
pub use recommendation::recommend_cleanup;
pub use size::calculate_directory_size;
