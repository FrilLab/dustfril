//! Cleaner module.
mod executor;
mod plan;

#[cfg(test)]
mod tests;

pub use executor::execute_cleanup;
pub use plan::{create_cleanup_plan, create_cleanup_plan_from_selection};
