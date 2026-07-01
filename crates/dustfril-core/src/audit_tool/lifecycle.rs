use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;

use crate::{
    audit_tool,
    error::DustResult,
    fs::walk_dirs,
    models::{LifecycleScript, ScriptType},
};

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    scripts: Option<HashMap<String, String>>,
}

/// Scans package.json files and returns lifecycle scripts.
pub fn audit_scan(root: &Path) -> DustResult<Vec<LifecycleScript>> {
    let mut scripts = Vec::new();

    for dir in walk_dirs(root) {
        let package_json = dir.join("package.json");

        if !package_json.is_file() {
            continue;
        }

        scripts.extend(audit_scan_package(&package_json)?);
    }

    Ok(scripts)
}

fn audit_scan_package(path: &Path) -> DustResult<Vec<LifecycleScript>> {
    let json = fs::read_to_string(path)?;

    let package: PackageJson = serde_json::from_str(&json).unwrap_or(PackageJson {
        name: None,
        scripts: None,
    });

    let mut result = Vec::new();

    let Some(scripts) = package.scripts else {
        return Ok(result);
    };

    let package_name = package.name.unwrap_or_else(|| "<unknown>".into());

    for (name, command) in scripts {
        let Some(script_type) = ScriptType::from_script_name(&name) else {
            continue;
        };

        result.push(LifecycleScript {
            package: package_name.clone(),
            script_type,
            command: command.clone(),
            risk_level: audit_tool::classify(&command),
        });
    }

    Ok(result)
}
