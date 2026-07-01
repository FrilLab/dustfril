use crate::models::RiskLevel;

/// Estimates the risk level of a lifecycle script command.
pub fn classify(command: &str) -> RiskLevel {
    let command = command.to_ascii_lowercase();

    if contains_any(
        &command,
        &[
            "curl",
            "wget",
            "invoke-webrequest",
            "powershell",
            "bash",
            "sh ",
            "chmod",
            "sudo",
            "rm -rf",
        ],
    ) {
        return RiskLevel::High;
    }

    if contains_any(
        &command,
        &[
            "node", "tsx", "ts-node", "python", "python3", "npm", "pnpm", "yarn", "bun",
        ],
    ) {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

fn contains_any(command: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| command.contains(pattern))
}
