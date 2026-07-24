mod lifecycle;
mod package_manager;
mod risk;

use std::path::Path;

use crate::{
    error::DustResult,
    models::{LifecycleScript, RiskLevel},
};

pub fn audit_scan(root: &Path) -> DustResult<Vec<LifecycleScript>> {
    lifecycle::audit_scan(root)
}

pub fn classify(command: &str) -> RiskLevel {
    risk::classify(command)
}

#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;
