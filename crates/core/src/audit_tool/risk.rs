use crate::{
    audit_tool::{command, rule},
    models::RiskLevel,
};

/// Estimates the risk level of a lifecycle script command.
pub fn classify(command: &str) -> RiskLevel {
    if let Some(rule) = rule::find(command) {
        return rule.risk_level;
    }

    if command::parse(command).iter().any(is_known_runtime_command) {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

fn is_known_runtime_command(segment: &command::Segment) -> bool {
    matches!(
        command::executable(&segment.tokens),
        Some("node" | "tsx" | "ts-node" | "python" | "python3" | "npm" | "pnpm" | "yarn" | "bun")
    )
}
