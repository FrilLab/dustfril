//! Shared models.
// scan
mod artifact_location;
mod artifact_type;
mod scan_result;

// analysis
mod analysis_result;
mod artifact_analysis;
// analysis - recommendation
mod cleanup_recommendation;

// cleanup
mod cleanup_candidate;
mod cleanup_plan;
mod cleanup_result;

// scan
pub use artifact_location::*;
pub use artifact_type::*;
pub use scan_result::*;

// analysis
pub use analysis_result::*;
pub use artifact_analysis::*;
// analysis - recommendation
pub use cleanup_recommendation::*;

// cleanup
pub use cleanup_candidate::*;
pub use cleanup_plan::*;
pub use cleanup_result::*;
