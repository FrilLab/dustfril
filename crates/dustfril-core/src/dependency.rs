//! Deterministic dependency inventory parsing.
//!
//! This module only reads project manifests and lockfiles. It deliberately
//! does not inspect installed dependency trees, Cargo registries, or network
//! metadata, and it does not share security findings with the security scan.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use serde_yaml::Value as YamlValue;

use crate::{
    error::{DustError, DustResult},
    models::{
        DependencyEntry, DependencyLockfile, DependencyLockfileStatus, DependencyMetric,
        DependencyReport, DependencyReportStatus, DependencyScope, DuplicateDependency, Ecosystem,
        LockfileKind,
    },
};

const NODE_DEPENDENCY_CATEGORIES: &[&str] = &[
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];
const CARGO_DEPENDENCY_CATEGORIES: &[&str] =
    &["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Debug)]
struct ParsedManifest {
    path: PathBuf,
    format: String,
    project_names: BTreeSet<String>,
    direct_counts: BTreeMap<String, usize>,
    direct_names: BTreeSet<String>,
    node_manager: Option<NodeManager>,
    is_cargo_workspace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Unsupported,
}

#[derive(Debug)]
enum LockSelection {
    Supported {
        kind: LockfileKind,
        path: PathBuf,
    },
    Missing {
        kind: LockfileKind,
        path: PathBuf,
    },
    Unsupported {
        path: Option<PathBuf>,
        format: Option<String>,
        reason: String,
    },
}

#[derive(Debug)]
struct ResolvedDependency {
    name: String,
    version: String,
    source: Option<String>,
    direct: bool,
    classification_available: bool,
}

/// Produces one report for every selected ecosystem.
pub fn report(root: &Path, selected: &[Ecosystem]) -> DustResult<Vec<DependencyReport>> {
    validate_root(root)?;

    let ecosystems = effective_ecosystems(selected);
    ecosystems
        .into_iter()
        .map(|ecosystem| match ecosystem {
            Ecosystem::Node => report_node(root),
            Ecosystem::Rust => report_rust(root),
            Ecosystem::Java => Ok(DependencyReport::unsupported(
                Ecosystem::Java,
                root.to_path_buf(),
                "Java dependency analysis is not supported; Maven and Gradle formats require a separate design.",
            )),
        })
        .collect()
}

fn effective_ecosystems(selected: &[Ecosystem]) -> Vec<Ecosystem> {
    if selected.is_empty() {
        return vec![Ecosystem::Node, Ecosystem::Rust];
    }

    let mut result = Vec::new();
    for ecosystem in selected.iter().copied() {
        if !result.contains(&ecosystem) {
            result.push(ecosystem);
        }
    }
    result
}

fn report_node(root: &Path) -> DustResult<DependencyReport> {
    let path = root.join("package.json");
    if !path.is_file() {
        return Ok(DependencyReport::unsupported(
            Ecosystem::Node,
            path,
            "Node dependency reporting requires a package.json manifest.",
        ));
    }

    let manifest = parse_node_manifest(&path)?;
    let selection = select_node_lockfile(root, manifest.node_manager);

    match selection {
        LockSelection::Supported { kind, path } => {
            let entries = parse_lockfile(&path, kind, &manifest)?;
            Ok(complete_report(manifest, kind, path, entries))
        }
        LockSelection::Missing { kind, path } => Ok(missing_lockfile_report(manifest, kind, path)),
        LockSelection::Unsupported {
            path,
            format,
            reason,
        } => Ok(unsupported_format_report(manifest, path, format, reason)),
    }
}

fn report_rust(root: &Path) -> DustResult<DependencyReport> {
    let path = root.join("Cargo.toml");
    if !path.is_file() {
        return Ok(DependencyReport::unsupported(
            Ecosystem::Rust,
            path,
            "Rust dependency reporting requires a Cargo.toml manifest.",
        ));
    }

    let manifest = parse_cargo_manifest(&path)?;
    if manifest.is_cargo_workspace {
        return Ok(unsupported_format_report(
            manifest,
            None,
            None,
            "Cargo workspace dependency reporting is not supported yet; member manifests require a separate workspace-aware design."
                .to_owned(),
        ));
    }
    let lockfile_path = root.join(LockfileKind::CargoLock.filename());

    if !lockfile_path.is_file() {
        return Ok(missing_lockfile_report(
            manifest,
            LockfileKind::CargoLock,
            lockfile_path,
        ));
    }

    let entries = parse_lockfile(&lockfile_path, LockfileKind::CargoLock, &manifest)?;
    Ok(complete_report(
        manifest,
        LockfileKind::CargoLock,
        lockfile_path,
        entries,
    ))
}

fn complete_report(
    manifest: ParsedManifest,
    kind: LockfileKind,
    lockfile_path: PathBuf,
    entries: Vec<ResolvedDependency>,
) -> DependencyReport {
    let duplicates = duplicate_versions(&entries);
    let resolved_count = entries.len();
    let transitive_count = entries.iter().filter(|entry| !entry.direct).count();
    let all_classifications_available = entries.iter().all(|entry| entry.classification_available);
    let mut resolved_dependencies = entries
        .into_iter()
        .map(|entry| DependencyEntry {
            ecosystem: ecosystem_for_kind(kind),
            name: entry.name,
            version: entry.version,
            source: entry.source,
            scope: if !entry.classification_available {
                DependencyScope::Unknown
            } else if entry.direct {
                DependencyScope::Direct
            } else {
                DependencyScope::Transitive
            },
        })
        .collect::<Vec<_>>();
    resolved_dependencies.sort();
    let lockfile_format = lockfile_format(kind);
    let transitive_metric = if all_classifications_available {
        DependencyMetric::available(transitive_count)
    } else {
        DependencyMetric::unknown(
            "This lockfile records resolved packages but does not preserve enough graph context for a trustworthy direct/transitive split.",
        )
    };

    DependencyReport {
        ecosystem: ecosystem_for_kind(kind),
        status: DependencyReportStatus::Complete,
        manifest: manifest.path,
        manifest_format: Some(manifest.format),
        lockfile: Some(DependencyLockfile {
            path: lockfile_path,
            kind: Some(kind),
            format: Some(lockfile_format),
            status: DependencyLockfileStatus::Parsed,
            reason: None,
        }),
        direct_dependency_counts: manifest.direct_counts,
        direct_dependency_total: manifest.direct_names.len(),
        resolved_dependency_count: DependencyMetric::available(resolved_count),
        transitive_dependency_count: transitive_metric,
        duplicate_versions: duplicates,
        resolved_dependencies,
        warnings: Vec::new(),
    }
}

fn missing_lockfile_report(
    manifest: ParsedManifest,
    kind: LockfileKind,
    path: PathBuf,
) -> DependencyReport {
    let reason = format!(
        "Expected {} is missing; resolved and transitive dependency counts are unknown.",
        kind.filename()
    );

    DependencyReport {
        ecosystem: ecosystem_for_kind(kind),
        status: DependencyReportStatus::MissingLockfile,
        manifest: manifest.path,
        manifest_format: Some(manifest.format),
        lockfile: Some(DependencyLockfile {
            path,
            kind: Some(kind),
            format: Some(kind.filename().to_owned()),
            status: DependencyLockfileStatus::Missing,
            reason: Some(reason.clone()),
        }),
        direct_dependency_counts: manifest.direct_counts,
        direct_dependency_total: manifest.direct_names.len(),
        resolved_dependency_count: DependencyMetric::unknown(reason.clone()),
        transitive_dependency_count: DependencyMetric::unknown(reason.clone()),
        duplicate_versions: Vec::new(),
        resolved_dependencies: Vec::new(),
        warnings: vec![reason],
    }
}

fn unsupported_format_report(
    manifest: ParsedManifest,
    path: Option<PathBuf>,
    format: Option<String>,
    reason: String,
) -> DependencyReport {
    DependencyReport {
        ecosystem: manifest_ecosystem(&manifest),
        status: DependencyReportStatus::Unsupported,
        manifest: manifest.path,
        manifest_format: Some(manifest.format),
        lockfile: path.map(|path| DependencyLockfile {
            path,
            kind: None,
            format,
            status: DependencyLockfileStatus::Unsupported,
            reason: Some(reason.clone()),
        }),
        direct_dependency_counts: manifest.direct_counts,
        direct_dependency_total: manifest.direct_names.len(),
        resolved_dependency_count: DependencyMetric::unsupported(reason.clone()),
        transitive_dependency_count: DependencyMetric::unsupported(reason.clone()),
        duplicate_versions: Vec::new(),
        resolved_dependencies: Vec::new(),
        warnings: vec![reason],
    }
}

fn ecosystem_for_kind(kind: LockfileKind) -> Ecosystem {
    kind.ecosystem()
}

fn manifest_ecosystem(manifest: &ParsedManifest) -> Ecosystem {
    if manifest.format == "Cargo.toml" {
        Ecosystem::Rust
    } else {
        Ecosystem::Node
    }
}

fn lockfile_format(kind: LockfileKind) -> String {
    // The parser stores normalized package/version records, so the format
    // name remains useful even when a lockfile has no package entries.
    kind.filename().to_owned()
}

fn select_node_lockfile(root: &Path, manager: Option<NodeManager>) -> LockSelection {
    let package_lock = root.join(LockfileKind::PackageLockJson.filename());
    let pnpm_lock = root.join(LockfileKind::PnpmLockYaml.filename());
    let bun_lock = root.join(LockfileKind::BunLock.filename());
    let yarn_lock = root.join("yarn.lock");
    let bun_legacy_lock = root.join("bun.lockb");

    match manager {
        Some(NodeManager::Npm) => supported_or_missing(LockfileKind::PackageLockJson, package_lock),
        Some(NodeManager::Pnpm) => supported_or_missing(LockfileKind::PnpmLockYaml, pnpm_lock),
        Some(NodeManager::Bun) => {
            if bun_lock.is_file() {
                LockSelection::Supported {
                    kind: LockfileKind::BunLock,
                    path: bun_lock,
                }
            } else if bun_legacy_lock.is_file() {
                unsupported_lock(
                    Some(bun_legacy_lock),
                    Some("bun.lockb".to_owned()),
                    "Legacy binary bun.lockb is not supported by the dependency report.",
                )
            } else {
                LockSelection::Missing {
                    kind: LockfileKind::BunLock,
                    path: bun_lock,
                }
            }
        }
        Some(NodeManager::Yarn) => unsupported_lock(
            Some(yarn_lock),
            Some("yarn.lock".to_owned()),
            "Yarn lockfiles are not supported by the dependency report.",
        ),
        Some(NodeManager::Unsupported) => unsupported_lock(
            None,
            None,
            "The package.json packageManager declares an unsupported package manager.",
        ),
        None => {
            if package_lock.is_file() {
                LockSelection::Supported {
                    kind: LockfileKind::PackageLockJson,
                    path: package_lock,
                }
            } else if pnpm_lock.is_file() {
                LockSelection::Supported {
                    kind: LockfileKind::PnpmLockYaml,
                    path: pnpm_lock,
                }
            } else if bun_lock.is_file() {
                LockSelection::Supported {
                    kind: LockfileKind::BunLock,
                    path: bun_lock,
                }
            } else if yarn_lock.is_file() {
                unsupported_lock(
                    Some(yarn_lock),
                    Some("yarn.lock".to_owned()),
                    "Yarn lockfiles are not supported by the dependency report.",
                )
            } else if bun_legacy_lock.is_file() {
                unsupported_lock(
                    Some(bun_legacy_lock),
                    Some("bun.lockb".to_owned()),
                    "Legacy binary bun.lockb is not supported by the dependency report.",
                )
            } else {
                LockSelection::Missing {
                    kind: LockfileKind::PackageLockJson,
                    path: package_lock,
                }
            }
        }
    }
}

fn supported_or_missing(kind: LockfileKind, path: PathBuf) -> LockSelection {
    if path.is_file() {
        LockSelection::Supported { kind, path }
    } else {
        LockSelection::Missing { kind, path }
    }
}

fn unsupported_lock(path: Option<PathBuf>, format: Option<String>, reason: &str) -> LockSelection {
    LockSelection::Unsupported {
        path,
        format,
        reason: reason.to_owned(),
    }
}

fn parse_node_manifest(path: &Path) -> DustResult<ParsedManifest> {
    let content = read_input(path)?;
    let value: Value =
        serde_json::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        manifest_error(path, "package.json must contain a JSON object".to_owned())
    })?;

    let node_manager = object
        .get("packageManager")
        .map(|value| {
            let declaration = value.as_str().ok_or_else(|| {
                manifest_error(path, "packageManager must be a string".to_owned())
            })?;
            Ok::<NodeManager, DustError>(node_manager_from_declaration(declaration))
        })
        .transpose()?;

    let mut direct_counts = BTreeMap::new();
    let mut direct_names = BTreeSet::new();
    for category in NODE_DEPENDENCY_CATEGORIES {
        let Some(value) = object.get(*category) else {
            continue;
        };
        let dependencies = value
            .as_object()
            .ok_or_else(|| manifest_error(path, format!("{category} must be a JSON object")))?;

        direct_counts.insert((*category).to_owned(), dependencies.len());
        for (name, specification) in dependencies {
            if !specification.is_string() {
                return Err(manifest_error(
                    path,
                    format!("{category} entry {name:?} must have a string specification"),
                ));
            }
            direct_names.insert(name.clone());
        }
    }
    for category in NODE_DEPENDENCY_CATEGORIES {
        direct_counts.entry((*category).to_owned()).or_insert(0);
    }

    Ok(ParsedManifest {
        path: path.to_path_buf(),
        format: "package.json".to_owned(),
        project_names: object
            .get("name")
            .and_then(Value::as_str)
            .map(|name| [name.to_owned()].into_iter().collect())
            .unwrap_or_default(),
        direct_counts,
        direct_names,
        node_manager,
        is_cargo_workspace: false,
    })
}

fn node_manager_from_declaration(value: &str) -> NodeManager {
    let manager = value.split_once('@').map_or(value, |(manager, _)| manager);
    match manager {
        "npm" => NodeManager::Npm,
        "pnpm" => NodeManager::Pnpm,
        "yarn" => NodeManager::Yarn,
        "bun" => NodeManager::Bun,
        _ => NodeManager::Unsupported,
    }
}

fn parse_cargo_manifest(path: &Path) -> DustResult<ParsedManifest> {
    let content = read_input(path)?;
    let value: toml::Value =
        toml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let table = value
        .as_table()
        .ok_or_else(|| manifest_error(path, "Cargo.toml must contain a TOML table".to_owned()))?;

    if table.contains_key("workspace") {
        return Ok(ParsedManifest {
            path: path.to_path_buf(),
            format: "Cargo.toml".to_owned(),
            project_names: BTreeSet::new(),
            direct_counts: BTreeMap::new(),
            direct_names: BTreeSet::new(),
            node_manager: None,
            is_cargo_workspace: true,
        });
    }

    let mut direct_counts = BTreeMap::new();
    let mut direct_names = BTreeSet::new();
    collect_cargo_dependencies(table, path, &mut direct_counts, &mut direct_names)?;

    let project_names = if let Some(package) = table.get("package") {
        let package = package
            .as_table()
            .ok_or_else(|| manifest_error(path, "[package] must be a TOML table".to_owned()))?;
        let name = package
            .get("name")
            .ok_or_else(|| manifest_error(path, "[package] is missing required name".to_owned()))?;
        let name = name
            .as_str()
            .ok_or_else(|| manifest_error(path, "[package].name must be a string".to_owned()))?;
        [name.to_owned()].into_iter().collect()
    } else {
        BTreeSet::new()
    };

    Ok(ParsedManifest {
        path: path.to_path_buf(),
        format: "Cargo.toml".to_owned(),
        project_names,
        direct_counts,
        direct_names,
        node_manager: None,
        is_cargo_workspace: false,
    })
}

fn collect_cargo_dependencies(
    table: &toml::map::Map<String, toml::Value>,
    path: &Path,
    counts: &mut BTreeMap<String, usize>,
    names: &mut BTreeSet<String>,
) -> DustResult<()> {
    for (key, value) in table {
        if CARGO_DEPENDENCY_CATEGORIES.contains(&key.as_str()) {
            let dependencies = value
                .as_table()
                .ok_or_else(|| manifest_error(path, format!("[{key}] must be a TOML table")))?;
            *counts.entry(key.clone()).or_default() += dependencies.len();
            for (name, specification) in dependencies {
                let package_name = cargo_dependency_name(path, name, specification)?;
                names.insert(package_name);
            }
            continue;
        }

        if let Some(nested) = value.as_table() {
            collect_cargo_dependencies(nested, path, counts, names)?;
        }
    }

    for category in CARGO_DEPENDENCY_CATEGORIES {
        counts.entry((*category).to_owned()).or_insert(0);
    }

    Ok(())
}

fn cargo_dependency_name(
    path: &Path,
    name: &str,
    specification: &toml::Value,
) -> DustResult<String> {
    match specification {
        toml::Value::String(_) => Ok(name.to_owned()),
        toml::Value::Table(table) => {
            if let Some(package) = table.get("package") {
                package.as_str().map(str::to_owned).ok_or_else(|| {
                    manifest_error(
                        path,
                        format!("dependency {name:?} package must be a string"),
                    )
                })
            } else {
                Ok(name.to_owned())
            }
        }
        _ => Err(manifest_error(
            path,
            format!("dependency {name:?} must be a string or table"),
        )),
    }
}

fn parse_lockfile(
    path: &Path,
    kind: LockfileKind,
    manifest: &ParsedManifest,
) -> DustResult<Vec<ResolvedDependency>> {
    match kind {
        LockfileKind::PackageLockJson => parse_package_lock(path, manifest),
        LockfileKind::PnpmLockYaml => parse_pnpm_lock(path, manifest),
        LockfileKind::BunLock => parse_bun_lock(path, manifest),
        LockfileKind::CargoLock => parse_cargo_lock(path, manifest),
    }
}

fn parse_package_lock(
    path: &Path,
    manifest: &ParsedManifest,
) -> DustResult<Vec<ResolvedDependency>> {
    let content = read_input(path)?;
    let lockfile: Value =
        serde_json::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let object = lockfile.as_object().ok_or_else(|| {
        manifest_error(
            path,
            "package-lock.json must contain a JSON object".to_owned(),
        )
    })?;
    let version = object
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

    if version >= 2 {
        let packages = object.get("packages").ok_or_else(|| {
            manifest_error(
                path,
                format!("package-lock.json lockfileVersion {version} is missing packages"),
            )
        })?;
        let packages = packages.as_object().ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json packages must be an object".to_owned(),
            )
        })?;
        parse_package_lock_packages(path, packages, manifest)
    } else {
        let dependencies = object.get("dependencies").ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json lockfileVersion 1 is missing dependencies".to_owned(),
            )
        })?;
        let dependencies = dependencies.as_object().ok_or_else(|| {
            manifest_error(
                path,
                "package-lock.json dependencies must be an object".to_owned(),
            )
        })?;
        let mut entries = Vec::new();
        parse_npm_dependency_tree(path, dependencies, 0, &mut entries)?;
        Ok(entries)
    }
}

fn parse_package_lock_packages(
    path: &Path,
    packages: &serde_json::Map<String, Value>,
    manifest: &ParsedManifest,
) -> DustResult<Vec<ResolvedDependency>> {
    let mut entries = Vec::new();
    for (key, value) in packages {
        let object = value.as_object().ok_or_else(|| {
            manifest_error(
                path,
                format!("package-lock.json package entry {key:?} must be an object"),
            )
        })?;
        if key.is_empty() {
            continue;
        }
        if package_lock_path_is_workspace(key) {
            // npm workspace package records use project-relative paths such
            // as packages/app. The corresponding node_modules entry is a
            // link, so neither record is a resolved external package node.
            continue;
        }
        if object.get("link").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let selector_name = package_name_from_selector(key);
        let name = optional_json_string(object, "name", path)?
            .map(str::to_owned)
            .or_else(|| selector_name.as_ref().map(|name| (*name).to_owned()));
        let Some(name) = name else {
            continue;
        };
        let Some(version) = optional_json_string(object, "version", path)? else {
            return Err(manifest_error(
                path,
                format!("package-lock.json package entry {key:?} is missing version"),
            ));
        };
        let direct = package_lock_path_is_direct(key)
            && (manifest.direct_names.contains(&name)
                || selector_name.is_some_and(|selector| manifest.direct_names.contains(selector)));
        entries.push(ResolvedDependency {
            name,
            version: version.to_owned(),
            source: optional_json_string(object, "resolved", path)?.map(str::to_owned),
            direct,
            classification_available: true,
        });
    }

    Ok(entries)
}

fn parse_npm_dependency_tree(
    path: &Path,
    dependencies: &serde_json::Map<String, Value>,
    depth: usize,
    entries: &mut Vec<ResolvedDependency>,
) -> DustResult<()> {
    for (name, value) in dependencies {
        let object = value.as_object().ok_or_else(|| {
            manifest_error(
                path,
                format!("package-lock.json dependency entry {name:?} must be an object"),
            )
        })?;
        let version = required_json_string(object, "version", path)?;
        entries.push(ResolvedDependency {
            name: name.clone(),
            version: version.to_owned(),
            source: optional_json_string(object, "resolved", path)?.map(str::to_owned),
            direct: depth == 0,
            classification_available: true,
        });

        if let Some(nested) = object.get("dependencies") {
            let nested = nested.as_object().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("package-lock.json nested dependencies for {name:?} must be an object"),
                )
            })?;
            parse_npm_dependency_tree(path, nested, depth + 1, entries)?;
        }
    }

    Ok(())
}

fn package_lock_path_is_direct(path: &str) -> bool {
    path.starts_with("node_modules/") && path.matches("node_modules/").count() == 1
}

fn package_lock_path_is_workspace(path: &str) -> bool {
    !path.is_empty() && !path.starts_with("node_modules/") && !path.contains("/node_modules/")
}

fn parse_pnpm_lock(path: &Path, manifest: &ParsedManifest) -> DustResult<Vec<ResolvedDependency>> {
    let content = read_input(path)?;
    let lockfile: YamlValue =
        serde_yaml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let object = lockfile.as_mapping().ok_or_else(|| {
        manifest_error(
            path,
            "pnpm-lock.yaml must contain a YAML mapping".to_owned(),
        )
    })?;
    let version = yaml_scalar_field(object, "lockfileVersion", path)?.ok_or_else(|| {
        manifest_error(
            path,
            "pnpm-lock.yaml must declare lockfileVersion".to_owned(),
        )
    })?;
    let major = version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u64>().ok())
        .ok_or_else(|| {
            manifest_error(
                path,
                format!("unsupported pnpm lockfileVersion {version:?}"),
            )
        })?;
    if !(5..=9).contains(&major) {
        return Err(manifest_error(
            path,
            format!("unsupported pnpm lockfileVersion {version:?}"),
        ));
    }

    let snapshots_present = yaml_value_field(object, "snapshots").is_some();
    let packages = parse_pnpm_section(object, "packages", path, manifest, !snapshots_present)?;
    let snapshots = parse_pnpm_section(object, "snapshots", path, manifest, snapshots_present)?;

    if snapshots_present {
        Ok(snapshots)
    } else {
        Ok(packages)
    }
}

fn parse_pnpm_section(
    object: &serde_yaml::Mapping,
    section: &str,
    path: &Path,
    manifest: &ParsedManifest,
    collect: bool,
) -> DustResult<Vec<ResolvedDependency>> {
    let Some(value) = yaml_value_field(object, section) else {
        return Ok(Vec::new());
    };
    let entries = value.as_mapping().ok_or_else(|| {
        manifest_error(path, format!("pnpm-lock.yaml {section} must be a mapping"))
    })?;
    let mut resolved = Vec::new();

    for (selector, package) in entries {
        let selector = selector.as_str().ok_or_else(|| {
            manifest_error(
                path,
                format!("pnpm-lock.yaml {section} keys must be strings"),
            )
        })?;
        let package = package.as_mapping().ok_or_else(|| {
            manifest_error(
                path,
                format!("pnpm-lock.yaml entry {selector:?} must be a mapping"),
            )
        })?;
        validate_pnpm_resolution(package, selector, path)?;
        let parsed_selector = package_selector(selector);
        if parsed_selector.is_none() {
            return Err(manifest_error(
                path,
                format!("pnpm-lock.yaml {section} entry {selector:?} is not a package selector"),
            ));
        }

        if !collect {
            continue;
        }
        let (name, version) = parsed_selector.expect("selector was validated above");
        let source = yaml_value_field(package, "resolution")
            .and_then(YamlValue::as_mapping)
            .and_then(|resolution| {
                ["tarball", "repo", "url"]
                    .into_iter()
                    .find_map(|key| yaml_value_field(resolution, key).and_then(YamlValue::as_str))
            })
            .map(str::to_owned);
        resolved.push(ResolvedDependency {
            direct: manifest.direct_names.contains(&name),
            name,
            version,
            source,
            classification_available: false,
        });
    }

    Ok(resolved)
}

fn validate_pnpm_resolution(
    package: &serde_yaml::Mapping,
    selector: &str,
    path: &Path,
) -> DustResult<()> {
    let Some(resolution) = yaml_value_field(package, "resolution") else {
        return Ok(());
    };
    let resolution = resolution.as_mapping().ok_or_else(|| {
        manifest_error(
            path,
            format!("pnpm-lock.yaml resolution for {selector:?} must be a mapping"),
        )
    })?;
    for key in ["tarball", "repo", "url"] {
        if let Some(value) = yaml_value_field(resolution, key)
            && value.as_str().is_none()
        {
            return Err(manifest_error(
                path,
                format!("pnpm-lock.yaml resolution {key} for {selector:?} must be a string"),
            ));
        }
    }
    Ok(())
}

fn parse_bun_lock(path: &Path, manifest: &ParsedManifest) -> DustResult<Vec<ResolvedDependency>> {
    let content = read_input(path)?;
    let lockfile = parse_jsonc(&content).map_err(|error| manifest_error(path, error))?;
    let object = lockfile
        .as_object()
        .ok_or_else(|| manifest_error(path, "bun.lock must contain a JSON object".to_owned()))?;
    let version = object
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

    let mut entries = Vec::new();
    if let Some(packages) = object.get("packages") {
        let packages = packages.as_object().ok_or_else(|| {
            manifest_error(path, "bun.lock packages must be an object".to_owned())
        })?;
        for (selector, package) in packages {
            match package {
                Value::Array(values) => {
                    let (name, version) = values
                        .first()
                        .and_then(Value::as_str)
                        .and_then(package_selector)
                        .or_else(|| package_selector(selector))
                        .ok_or_else(|| {
                            manifest_error(
                                path,
                                format!("bun.lock package entry {selector:?} is missing a version"),
                            )
                        })?;
                    entries.push(ResolvedDependency {
                        direct: manifest.direct_names.contains(&name),
                        name,
                        version,
                        source: values.get(1).and_then(Value::as_str).map(str::to_owned),
                        classification_available: false,
                    });
                }
                Value::Object(_) => {
                    let fallback_name = package_name_from_selector(selector).ok_or_else(|| {
                        manifest_error(
                            path,
                            format!("bun.lock package key {selector:?} is not a package selector"),
                        )
                    })?;
                    parse_bun_package_value(path, fallback_name, package, manifest, &mut entries)?
                }
                _ => {
                    return Err(manifest_error(
                        path,
                        format!("bun.lock package entry {selector:?} must be an array or object"),
                    ));
                }
            }
        }
    }

    if entries.is_empty()
        && let Some(workspaces) = object.get("workspaces")
    {
        parse_bun_workspaces(path, workspaces, manifest, &mut entries)?;
    }

    Ok(entries)
}

fn parse_bun_workspaces(
    path: &Path,
    workspaces: &Value,
    manifest: &ParsedManifest,
    entries: &mut Vec<ResolvedDependency>,
) -> DustResult<()> {
    let workspaces = workspaces
        .as_object()
        .ok_or_else(|| manifest_error(path, "bun.lock workspaces must be an object".to_owned()))?;
    for (workspace_name, value) in workspaces {
        let workspace = value.as_object().ok_or_else(|| {
            manifest_error(
                path,
                format!("bun.lock workspace {workspace_name:?} must be an object"),
            )
        })?;
        for category in NODE_DEPENDENCY_CATEGORIES {
            let Some(dependencies) = workspace.get(*category) else {
                continue;
            };
            let dependencies = dependencies.as_object().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("bun.lock workspace {workspace_name:?} {category} must be an object"),
                )
            })?;
            for (name, dependency) in dependencies {
                parse_bun_workspace_dependency(path, name, dependency, manifest, entries, true)?;
            }
        }
    }
    Ok(())
}

fn parse_bun_workspace_dependency(
    path: &Path,
    fallback_name: &str,
    dependency: &Value,
    manifest: &ParsedManifest,
    entries: &mut Vec<ResolvedDependency>,
    direct: bool,
) -> DustResult<()> {
    match dependency {
        Value::Array(values) => {
            let (name, version) = values
                .first()
                .and_then(Value::as_str)
                .and_then(package_selector)
                .ok_or_else(|| {
                    manifest_error(
                        path,
                        format!("bun.lock dependency {fallback_name:?} is missing a version"),
                    )
                })?;
            entries.push(ResolvedDependency {
                direct,
                name,
                version,
                source: values.get(1).and_then(Value::as_str).map(str::to_owned),
                classification_available: false,
            });
            for value in values.iter().skip(1) {
                if let Value::Object(object) = value {
                    parse_bun_nested_dependencies(path, object, manifest, entries)?;
                }
            }
            Ok(())
        }
        Value::Object(object) => {
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or(fallback_name);
            let version = object
                .get("version")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    manifest_error(
                        path,
                        format!("bun.lock dependency {name:?} is missing a version"),
                    )
                })?;
            entries.push(ResolvedDependency {
                direct,
                name: name.to_owned(),
                version: version.to_owned(),
                source: object
                    .get("resolved")
                    .or_else(|| object.get("url"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                classification_available: false,
            });
            parse_bun_nested_dependencies(path, object, manifest, entries)
        }
        _ => Err(manifest_error(
            path,
            format!("bun.lock dependency {fallback_name:?} must be an array or object"),
        )),
    }
}

fn parse_bun_nested_dependencies(
    path: &Path,
    object: &serde_json::Map<String, Value>,
    manifest: &ParsedManifest,
    entries: &mut Vec<ResolvedDependency>,
) -> DustResult<()> {
    for category in NODE_DEPENDENCY_CATEGORIES {
        let Some(dependencies) = object.get(*category) else {
            continue;
        };
        let dependencies = dependencies.as_object().ok_or_else(|| {
            manifest_error(path, format!("bun.lock {category} must be an object"))
        })?;
        for (name, dependency) in dependencies {
            parse_bun_workspace_dependency(path, name, dependency, manifest, entries, false)?;
        }
    }
    Ok(())
}

fn parse_bun_package_value(
    path: &Path,
    fallback_name: &str,
    package: &Value,
    manifest: &ParsedManifest,
    entries: &mut Vec<ResolvedDependency>,
) -> DustResult<()> {
    let object = package.as_object().ok_or_else(|| {
        manifest_error(path, "bun.lock package value must be an object".to_owned())
    })?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(fallback_name);
    let Some(version) = object.get("version").and_then(Value::as_str) else {
        return Err(manifest_error(
            path,
            format!("bun.lock package {name:?} is missing a version"),
        ));
    };
    entries.push(ResolvedDependency {
        direct: manifest.direct_names.contains(name),
        name: name.to_owned(),
        version: version.to_owned(),
        source: object
            .get("resolved")
            .or_else(|| object.get("url"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        classification_available: false,
    });
    Ok(())
}

fn parse_cargo_lock(path: &Path, manifest: &ParsedManifest) -> DustResult<Vec<ResolvedDependency>> {
    let content = read_input(path)?;
    let lockfile: toml::Value =
        toml::from_str(&content).map_err(|error| manifest_error(path, error.to_string()))?;
    let table = lockfile
        .as_table()
        .ok_or_else(|| manifest_error(path, "Cargo.lock must contain a TOML table".to_owned()))?;
    let version = match table.get("version") {
        Some(value) => value.as_integer().ok_or_else(|| {
            manifest_error(path, "Cargo.lock version must be an integer".to_owned())
        })?,
        None => 1,
    };
    if !(1..=4).contains(&version) {
        return Err(manifest_error(
            path,
            format!("unsupported Cargo.lock version {version}"),
        ));
    }

    let Some(packages) = table.get("package") else {
        return Ok(Vec::new());
    };
    let packages = packages
        .as_array()
        .ok_or_else(|| manifest_error(path, "Cargo.lock package must be an array".to_owned()))?;
    let mut root_dependency_refs = BTreeMap::<String, BTreeSet<Option<String>>>::new();

    for package in packages {
        let package = package.as_table().ok_or_else(|| {
            manifest_error(path, "Cargo.lock package entries must be tables".to_owned())
        })?;
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| manifest_error(path, "Cargo.lock package is missing name".to_owned()))?;
        if !manifest.project_names.contains(name) {
            continue;
        }
        let Some(dependencies) = package.get("dependencies") else {
            continue;
        };
        let dependencies = dependencies.as_array().ok_or_else(|| {
            manifest_error(
                path,
                format!("Cargo.lock dependencies for {name:?} must be an array"),
            )
        })?;
        for dependency in dependencies {
            let dependency = dependency.as_str().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("Cargo.lock dependencies for {name:?} must contain strings"),
                )
            })?;
            let mut parts = dependency.split_whitespace();
            let dependency_name = parts.next().unwrap_or_default();
            let dependency_version = parts
                .next()
                .filter(|version| version.chars().next().is_some_and(|c| c.is_ascii_digit()))
                .map(str::to_owned);
            if !dependency_name.is_empty() {
                root_dependency_refs
                    .entry(dependency_name.to_owned())
                    .or_default()
                    .insert(dependency_version);
            }
        }
    }

    let mut entries = Vec::new();

    for package in packages {
        let package = package.as_table().ok_or_else(|| {
            manifest_error(path, "Cargo.lock package entries must be tables".to_owned())
        })?;
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| manifest_error(path, "Cargo.lock package is missing name".to_owned()))?;
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                manifest_error(
                    path,
                    format!("Cargo.lock package {name:?} is missing version"),
                )
            })?;
        if manifest.project_names.contains(name) {
            continue;
        }
        if let Some(source) = package.get("source")
            && !source.is_str()
        {
            return Err(manifest_error(
                path,
                format!("Cargo.lock source for {name:?} must be a string"),
            ));
        }
        if let Some(dependencies) = package.get("dependencies") {
            let dependencies = dependencies.as_array().ok_or_else(|| {
                manifest_error(
                    path,
                    format!("Cargo.lock dependencies for {name:?} must be an array"),
                )
            })?;
            if dependencies.iter().any(|dependency| !dependency.is_str()) {
                return Err(manifest_error(
                    path,
                    format!("Cargo.lock dependencies for {name:?} must contain strings"),
                ));
            }
        }
        let direct = match root_dependency_refs.get(name) {
            Some(references) => {
                references.contains(&Some(version.to_owned())) || references.contains(&None)
            }
            None => manifest.direct_names.contains(name),
        };
        entries.push(ResolvedDependency {
            direct,
            name: name.to_owned(),
            version: version.to_owned(),
            source: package
                .get("source")
                .and_then(toml::Value::as_str)
                .map(str::to_owned),
            classification_available: true,
        });
    }

    let mut package_counts = BTreeMap::<String, usize>::new();
    for entry in &entries {
        *package_counts.entry(entry.name.clone()).or_default() += 1;
    }
    for entry in &mut entries {
        let ambiguous_unversioned_reference =
            root_dependency_refs
                .get(&entry.name)
                .is_some_and(|references| {
                    references.contains(&None) && package_counts.get(&entry.name) > Some(&1)
                });
        entry.classification_available = !ambiguous_unversioned_reference
            && (!entry.direct
                || root_dependency_refs.contains_key(&entry.name)
                || package_counts.get(&entry.name) == Some(&1));
    }

    Ok(entries)
}

fn duplicate_versions(entries: &[ResolvedDependency]) -> Vec<DuplicateDependency> {
    let mut versions = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in entries {
        versions
            .entry(entry.name.clone())
            .or_default()
            .insert(entry.version.clone());
    }

    versions
        .into_iter()
        .filter_map(|(name, versions)| {
            (versions.len() > 1).then(|| DuplicateDependency {
                name,
                versions: versions.into_iter().collect(),
            })
        })
        .collect()
}

fn package_selector(selector: &str) -> Option<(String, String)> {
    let selector = selector
        .rsplit("/node_modules/")
        .next()
        .unwrap_or(selector)
        .trim()
        .trim_start_matches("node_modules/")
        .trim_matches(['\'', '"'])
        .trim_start_matches('/')
        .split('(')
        .next()
        .unwrap_or_default();
    if selector.is_empty() {
        return None;
    }

    if selector.starts_with('@') {
        let slash = selector.find('/')?;
        let after_scope = &selector[slash + 1..];
        if let Some(at) = after_scope.find('@') {
            let name_end = slash + 1 + at;
            let name = &selector[..name_end];
            let version = &selector[name_end + 1..];
            return (!version.is_empty()).then(|| (name.to_owned(), version.to_owned()));
        }
        if let Some(slash_after_name) = after_scope.find('/') {
            let name_end = slash + 1 + slash_after_name;
            let name = &selector[..name_end];
            let version = &selector[name_end + 1..];
            return (!version.is_empty()).then(|| (name.to_owned(), version.to_owned()));
        }
        return None;
    }

    if let Some(at) = selector.find('@') {
        let name = &selector[..at];
        let version = &selector[at + 1..];
        return (!name.is_empty() && !version.is_empty())
            .then(|| (name.to_owned(), version.to_owned()));
    }
    if let Some(slash) = selector.rfind('/') {
        let name = &selector[..slash];
        let version = &selector[slash + 1..];
        return (!name.is_empty() && !version.is_empty())
            .then(|| (name.to_owned(), version.to_owned()));
    }

    None
}

fn package_name_from_selector(selector: &str) -> Option<&str> {
    let selector = selector
        .rsplit("/node_modules/")
        .next()
        .unwrap_or(selector)
        .trim()
        .trim_start_matches("node_modules/")
        .trim_matches(['\'', '"'])
        .trim_start_matches('/')
        .split('(')
        .next()
        .unwrap_or_default();
    if selector.is_empty() || selector.contains("://") {
        return None;
    }

    if selector.starts_with('@') {
        let slash = selector.find('/')?;
        let after_scope = &selector[slash + 1..];
        if let Some(at) = after_scope.find('@') {
            return Some(&selector[..slash + 1 + at]);
        }
        if let Some(version_slash) = after_scope.find('/') {
            return Some(&selector[..slash + 1 + version_slash]);
        }
        return Some(selector);
    }

    if let Some(at) = selector.find('@') {
        return Some(&selector[..at]);
    }
    if let Some(slash) = selector.rfind('/') {
        return Some(&selector[..slash]);
    }
    Some(selector)
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

fn read_input(path: &Path) -> DustResult<String> {
    fs::read_to_string(path).map_err(|error| manifest_error(path, error.to_string()))
}

fn validate_root(root: &Path) -> DustResult<()> {
    // Dependency reporting is read-only and keys baselines by the canonical
    // target, so a directory reached through a symlink is a valid project
    // input. Artifact scanning keeps its stricter symlink-root policy.
    match fs::metadata(root) {
        Ok(metadata) if !metadata.is_dir() => Err(DustError::InvalidPath(root.to_path_buf())),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(DustError::InvalidPath(root.to_path_buf()))
        }
        Err(error) => Err(DustError::Io(error)),
    }
}

fn manifest_error(path: &Path, message: String) -> DustError {
    DustError::Manifest(format!("{}: {message}", path.display()))
}

fn optional_json_string<'a>(
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

fn required_json_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> DustResult<&'a str> {
    optional_json_string(object, key, path)?
        .ok_or_else(|| manifest_error(path, format!("JSON object is missing required {key} field")))
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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::*;

    fn write_node_project(root: &Path, manifest: &str, lockfile: &str) {
        fs::write(root.join("package.json"), manifest).unwrap();
        fs::write(root.join("package-lock.json"), lockfile).unwrap();
    }

    #[test]
    fn node_counts_categories_and_unique_direct_names() {
        let temp_dir = TempDir::new().unwrap();
        write_node_project(
            temp_dir.path(),
            r#"{
                "name":"demo",
                "dependencies":{"runtime":"1.0.0","shared":"1.0.0"},
                "devDependencies":{"tools":"1.0.0","shared":"2.0.0"},
                "optionalDependencies":{"optional":"1.0.0"},
                "peerDependencies":{"peer":"^1.0.0"}
            }"#,
            r#"{
                "lockfileVersion":3,
                "packages":{
                    "":{"name":"demo","version":"1.0.0"},
                    "node_modules/runtime":{"version":"1.0.0"},
                    "node_modules/shared":{"version":"1.0.0"},
                    "node_modules/tools":{"version":"1.0.0"},
                    "node_modules/optional":{"version":"1.0.0"},
                    "node_modules/peer":{"version":"1.0.0"},
                    "node_modules/tools/node_modules/shared":{"version":"2.0.0"}
                }
            }"#,
        );

        let report = report(temp_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);

        assert_eq!(report.status, DependencyReportStatus::Complete);
        assert_eq!(report.direct_dependency_counts["dependencies"], 2);
        assert_eq!(report.direct_dependency_counts["devDependencies"], 2);
        assert_eq!(report.direct_dependency_counts["optionalDependencies"], 1);
        assert_eq!(report.direct_dependency_counts["peerDependencies"], 1);
        assert_eq!(report.direct_dependency_total, 5);
        assert_eq!(report.resolved_dependency_count.value, Some(6));
        assert_eq!(report.transitive_dependency_count.value, Some(1));
        assert_eq!(report.duplicate_versions[0].name, "shared");
        assert_eq!(report.duplicate_versions[0].versions, ["1.0.0", "2.0.0"]);
        assert_eq!(
            report
                .resolved_dependencies
                .iter()
                .filter(|entry| entry.scope == DependencyScope::Direct)
                .count(),
            5
        );
        assert_eq!(
            report
                .resolved_dependencies
                .iter()
                .filter(|entry| entry.scope == DependencyScope::Transitive)
                .count(),
            1
        );
    }

    #[test]
    fn npm_workspace_records_are_not_resolved_nodes() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{
                "name":"root",
                "workspaces":["packages/*"],
                "dependencies":{"external":"1.0.0"}
            }"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{
                "lockfileVersion":3,
                "packages":{
                    "":{"name":"root"},
                    "packages/a":{},
                    "packages/versioned":{"name":"versioned","version":"1.0.0"},
                    "node_modules/a":{"resolved":"packages/a","link":true},
                    "node_modules/versioned":{"resolved":"packages/versioned","link":true},
                    "node_modules/external":{"version":"1.0.0"}
                }
            }"#,
        )
        .unwrap();

        let report = report(temp_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);

        assert_eq!(report.resolved_dependency_count.value, Some(1));
        assert_eq!(report.transitive_dependency_count.value, Some(0));
        assert!(report.duplicate_versions.is_empty());
    }

    #[test]
    fn package_lock_v1_uses_tree_depth_for_transitive_count() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"root":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{"lockfileVersion":1,"dependencies":{"root":{"version":"1.0.0","dependencies":{"nested":{"version":"2.0.0"}}}}}"#,
        )
        .unwrap();

        let report = report(temp_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);

        assert_eq!(report.resolved_dependency_count.value, Some(2));
        assert_eq!(report.transitive_dependency_count.value, Some(1));
    }

    #[test]
    fn lockfile_order_does_not_change_logical_metrics() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"alpha":"1.0.0","beta":"1.0.0"}}"#,
        )
        .unwrap();
        let lockfile = temp_dir.path().join("package-lock.json");
        fs::write(
            &lockfile,
            r#"{"lockfileVersion":3,"packages":{"node_modules/alpha":{"version":"1.0.0"},"":{"name":"demo","version":"1.0.0"},"node_modules/beta":{"version":"1.0.0"}}}"#,
        )
        .unwrap();
        let first = report(temp_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);

        fs::write(
            &lockfile,
            r#"{"packages":{"node_modules/beta":{"version":"1.0.0"},"node_modules/alpha":{"version":"1.0.0"},"":{"version":"1.0.0","name":"demo"}},"lockfileVersion":3}"#,
        )
        .unwrap();
        let second = report(temp_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);

        assert_eq!(
            first.direct_dependency_counts,
            second.direct_dependency_counts
        );
        assert_eq!(
            first.resolved_dependency_count,
            second.resolved_dependency_count
        );
        assert_eq!(
            first.transitive_dependency_count,
            second.transitive_dependency_count
        );
        assert_eq!(first.duplicate_versions, second.duplicate_versions);
    }

    #[test]
    fn pnpm_and_bun_lockfiles_are_supported_without_tree_traversal() {
        let pnpm_dir = TempDir::new().unwrap();
        fs::write(
            pnpm_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0","dependencies":{"foo":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            pnpm_dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: '9.0'\npackages:\n  foo@1.0.0:\n    resolution: {integrity: sha512-test}\n  foo@2.0.0:\n    resolution: {integrity: sha512-test}\n",
        )
        .unwrap();

        let pnpm_report = report(pnpm_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(pnpm_report.resolved_dependency_count.value, Some(2));
        assert_eq!(
            pnpm_report.transitive_dependency_count.status,
            crate::models::DependencyMetricStatus::Unknown
        );
        assert_eq!(
            pnpm_report.duplicate_versions[0].versions,
            ["1.0.0", "2.0.0"]
        );

        let bun_dir = TempDir::new().unwrap();
        fs::write(
            bun_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"bun@1.0.0","dependencies":{"foo":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            bun_dir.path().join("bun.lock"),
            r#"{"lockfileVersion":1,"packages":{"foo":["foo@1.0.0","https://registry.npmjs.org/foo/-/foo-1.0.0.tgz",{},"sha512-test"]}}"#,
        )
        .unwrap();

        let bun_report = report(bun_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(bun_report.resolved_dependency_count.value, Some(1));
        assert_eq!(
            bun_report.transitive_dependency_count.status,
            crate::models::DependencyMetricStatus::Unknown
        );

        let workspace_dir = TempDir::new().unwrap();
        fs::write(
            workspace_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"bun@1.0.0","dependencies":{"foo":"1.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            workspace_dir.path().join("bun.lock"),
            r#"{"lockfileVersion":1,"workspaces":{"":{"dependencies":{"foo":["foo@1.0.0","",{},"sha512-test"]}}}}"#,
        )
        .unwrap();
        let workspace_report = report(workspace_dir.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(workspace_report.resolved_dependency_count.value, Some(1));
    }

    #[test]
    fn rust_counts_dependency_categories_and_excludes_project_package() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "demo"
version = "0.1.0"
[dependencies]
serde = "1"
alias = { package = "real-name", version = "1" }
[dev-dependencies]
criterion = "0.5"
[build-dependencies]
cc = "1"
"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Cargo.lock"),
            r#"version = 3

[[package]]
name = "demo"
version = "0.1.0"
dependencies = ["serde 1.0.0"]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://example.invalid/index"

[[package]]
name = "serde"
version = "2.0.0"

[[package]]
name = "real-name"
version = "1.0.0"

[[package]]
name = "criterion"
version = "0.5.0"

[[package]]
name = "cc"
version = "1.0.0"
"#,
        )
        .unwrap();

        let report = report(temp_dir.path(), &[Ecosystem::Rust])
            .unwrap()
            .remove(0);

        assert_eq!(report.direct_dependency_counts["dependencies"], 2);
        assert_eq!(report.direct_dependency_counts["dev-dependencies"], 1);
        assert_eq!(report.direct_dependency_counts["build-dependencies"], 1);
        assert_eq!(report.direct_dependency_total, 4);
        assert_eq!(report.resolved_dependency_count.value, Some(5));
        assert_eq!(report.transitive_dependency_count.value, Some(1));
        assert_eq!(report.duplicate_versions[0].name, "serde");
        assert_eq!(
            report
                .resolved_dependencies
                .iter()
                .find(|entry| entry.name == "serde")
                .unwrap()
                .source,
            Some("registry+https://example.invalid/index".to_owned())
        );
    }

    #[test]
    fn cargo_workspaces_are_explicitly_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["crates/*"]

[workspace.dependencies]
serde = "1"
"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Cargo.lock"),
            r#"version = 4

[[package]]
name = "workspace-member"
version = "0.1.0"
"#,
        )
        .unwrap();

        let report = report(temp_dir.path(), &[Ecosystem::Rust])
            .unwrap()
            .remove(0);

        assert_eq!(report.status, DependencyReportStatus::Unsupported);
        assert!(report.direct_dependency_counts.is_empty());
        assert_eq!(
            report.resolved_dependency_count.status,
            crate::models::DependencyMetricStatus::Unsupported
        );
        assert!(report.warnings[0].contains("workspace"));
    }

    #[test]
    fn missing_and_unsupported_lockfiles_are_explicit() {
        let missing = TempDir::new().unwrap();
        fs::write(
            missing.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"foo":"1.0.0"}}"#,
        )
        .unwrap();
        let missing_report = report(missing.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(
            missing_report.status,
            DependencyReportStatus::MissingLockfile
        );
        assert_eq!(
            missing_report.resolved_dependency_count.status.to_string(),
            "Unknown"
        );

        let unsupported = TempDir::new().unwrap();
        fs::write(
            unsupported.path().join("package.json"),
            r#"{"name":"demo","packageManager":"yarn@4.0.0","dependencies":{}}"#,
        )
        .unwrap();
        fs::write(unsupported.path().join("yarn.lock"), "__metadata:\n").unwrap();
        let unsupported_report = report(unsupported.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(
            unsupported_report.status,
            DependencyReportStatus::Unsupported
        );
        assert_eq!(
            unsupported_report.lockfile.unwrap().format,
            Some("yarn.lock".to_owned())
        );

        let unknown_manager = TempDir::new().unwrap();
        fs::write(
            unknown_manager.path().join("package.json"),
            r#"{"name":"demo","packageManager":"deno@2.0.0","dependencies":{}}"#,
        )
        .unwrap();
        let unknown_report = report(unknown_manager.path(), &[Ecosystem::Node])
            .unwrap()
            .remove(0);
        assert_eq!(unknown_report.status, DependencyReportStatus::Unsupported);
        assert!(unknown_report.lockfile.is_none());
        assert!(unknown_report.warnings[0].contains("unsupported package manager"));
    }

    #[test]
    fn malformed_inputs_fail_with_the_relevant_path() {
        let node = TempDir::new().unwrap();
        fs::write(node.path().join("package.json"), "{invalid").unwrap();
        assert!(matches!(
            report(node.path(), &[Ecosystem::Node]),
            Err(DustError::Manifest(message)) if message.contains("package.json")
        ));

        let rust = TempDir::new().unwrap();
        fs::write(rust.path().join("Cargo.toml"), "[package").unwrap();
        assert!(matches!(
            report(rust.path(), &[Ecosystem::Rust]),
            Err(DustError::Manifest(message)) if message.contains("Cargo.toml")
        ));

        let lockfile = TempDir::new().unwrap();
        fs::write(
            lockfile.path().join("package.json"),
            r#"{"name":"demo","packageManager":"npm@10.0.0"}"#,
        )
        .unwrap();
        fs::write(
            lockfile.path().join("package-lock.json"),
            r#"{"lockfileVersion":3,"packages":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            report(lockfile.path(), &[Ecosystem::Node]),
            Err(DustError::Manifest(message)) if message.contains("package-lock.json")
        ));
    }

    #[test]
    fn java_is_explicitly_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        let report = report(temp_dir.path(), &[Ecosystem::Java])
            .unwrap()
            .remove(0);

        assert_eq!(report.status, DependencyReportStatus::Unsupported);
        assert!(report.warnings[0].contains("Java"));
    }
}
