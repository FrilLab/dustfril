use std::path::Path;

use crate::{
    audit_tool,
    error::DustResult,
    models::{Ecosystem, LifecycleScript},
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
}
