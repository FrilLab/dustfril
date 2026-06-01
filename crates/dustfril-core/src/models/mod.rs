//! Shared models.
mod artifact_location;
mod artifact_type;
mod scan_result;

mod analysis_result;
mod artifact_analysis;

mod cleanup_recommendation;

pub use artifact_location::*;
pub use artifact_type::*;
pub use scan_result::*;

pub use analysis_result::*;
pub use artifact_analysis::*;

pub use cleanup_recommendation::*;
