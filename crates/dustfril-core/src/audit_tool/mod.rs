mod command;
mod lifecycle;
mod package_manager;
mod risk;
mod rule;

use std::path::Path;

use crate::{
    error::DustResult,
    models::{LifecycleScript, RiskLevel, SecurityWarning},
};

pub fn audit_scan(root: &Path) -> DustResult<Vec<LifecycleScript>> {
    lifecycle::audit_scan(root)
}

pub fn classify(command: &str) -> RiskLevel {
    risk::classify(command)
}

pub fn security_scan(root: &Path) -> DustResult<Vec<SecurityWarning>> {
    lifecycle::audit_scan(root).map(|scripts| {
        scripts
            .into_iter()
            .filter_map(|script| {
                let rule = rule::find(&script.command)?;

                Some(SecurityWarning {
                    package: script.package,
                    script_type: script.script_type.to_string(),
                    command: script.command,
                    risk_level: rule.risk_level,
                    reason: rule.reason.to_string(),
                })
            })
            .collect()
    })
}

#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;
