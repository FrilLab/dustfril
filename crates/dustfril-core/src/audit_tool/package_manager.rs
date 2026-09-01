use std::{fs, path::Path};

use crate::{
    error::{DustError, DustResult},
    models::PackageManager,
};

/// Detects the package manager used by a Node project from lockfiles in a directory.
pub fn detect_in_dir(dir: &Path) -> Option<PackageManager> {
    if dir.join("pnpm-lock.yaml").is_file() {
        return Some(PackageManager::Pnpm);
    }

    if dir.join("yarn.lock").is_file() {
        return Some(PackageManager::Yarn);
    }

    if dir.join("bun.lockb").is_file() || dir.join("bun.lock").is_file() {
        return Some(PackageManager::Bun);
    }

    if dir.join("package-lock.json").is_file() {
        return Some(PackageManager::Npm);
    }

    None
}

/// Resolves the package manager for a package.json path.
///
/// An explicit `packageManager` declaration takes precedence over lockfiles.
/// This keeps lifecycle audit results aligned with the project's own package
/// manager selection instead of inventing npm when a different manager is
/// declared.
pub fn detect_for_package(package_json: &Path, scan_root: &Path) -> DustResult<PackageManager> {
    if let Some(package_manager) = declared_manager(package_json)? {
        return Ok(package_manager);
    }

    let mut current = package_json.parent();

    while let Some(dir) = current {
        if Some(dir) != package_json.parent() {
            let parent_manifest = dir.join("package.json");
            if parent_manifest.is_file()
                && let Some(package_manager) = declared_manager(&parent_manifest)?
            {
                return Ok(package_manager);
            }
        }

        if let Some(package_manager) = detect_in_dir(dir) {
            return Ok(package_manager);
        }

        if dir == scan_root {
            break;
        }

        let parent = match dir.parent() {
            Some(parent) => parent,
            None => break,
        };

        current = if parent
            .file_name()
            .is_some_and(|name| name == "node_modules")
        {
            parent.parent()
        } else {
            Some(parent)
        };
    }

    Ok(PackageManager::Unknown)
}

fn declared_manager(package_json: &Path) -> DustResult<Option<PackageManager>> {
    let content = fs::read_to_string(package_json)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| DustError::Manifest(format!("{}: {error}", package_json.display())))?;

    let Some(value) = manifest.get("packageManager") else {
        return Ok(None);
    };

    let value = value.as_str().ok_or_else(|| {
        DustError::Manifest(format!(
            "{}: packageManager must be a string",
            package_json.display()
        ))
    })?;

    Ok(Some(manager_from_declaration(value)))
}

fn manager_from_declaration(value: &str) -> PackageManager {
    let manager = value.split_once('@').map_or(value, |(manager, _)| manager);

    match manager {
        "npm" => PackageManager::Npm,
        "pnpm" => PackageManager::Pnpm,
        "yarn" => PackageManager::Yarn,
        "bun" => PackageManager::Bun,
        _ => PackageManager::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn detect_in_dir_identifies_supported_package_managers() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();
        assert_eq!(detect_in_dir(temp_dir.path()), Some(PackageManager::Npm));

        std::fs::remove_file(temp_dir.path().join("package-lock.json")).unwrap();
        std::fs::write(
            temp_dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: 9.0",
        )
        .unwrap();
        assert_eq!(detect_in_dir(temp_dir.path()), Some(PackageManager::Pnpm));

        std::fs::remove_file(temp_dir.path().join("pnpm-lock.yaml")).unwrap();
        std::fs::write(temp_dir.path().join("yarn.lock"), "# yarn lockfile v1").unwrap();
        assert_eq!(detect_in_dir(temp_dir.path()), Some(PackageManager::Yarn));

        std::fs::remove_file(temp_dir.path().join("yarn.lock")).unwrap();
        std::fs::write(temp_dir.path().join("bun.lockb"), "bun").unwrap();
        assert_eq!(detect_in_dir(temp_dir.path()), Some(PackageManager::Bun));
    }

    #[test]
    fn detect_for_package_uses_project_root_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("app");
        let dependency_dir = project_root.join("node_modules").join("left-pad");
        std::fs::create_dir_all(&dependency_dir).unwrap();
        std::fs::write(project_root.join("pnpm-lock.yaml"), "lockfileVersion: 9.0").unwrap();
        std::fs::write(
            dependency_dir.join("package.json"),
            r#"{"name":"left-pad","scripts":{"postinstall":"node setup.js"}}"#,
        )
        .unwrap();

        assert_eq!(
            detect_for_package(&dependency_dir.join("package.json"), temp_dir.path()).unwrap(),
            PackageManager::Pnpm
        );
    }

    #[test]
    fn explicit_package_manager_takes_precedence_over_lockfiles() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name":"demo","packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();
        std::fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

        assert_eq!(
            detect_for_package(&temp_dir.path().join("package.json"), temp_dir.path()).unwrap(),
            PackageManager::Pnpm
        );
    }

    #[test]
    fn malformed_package_manager_manifest_is_not_silently_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");
        std::fs::write(&package_json, "{invalid json").unwrap();

        assert!(matches!(
            detect_for_package(&package_json, temp_dir.path()),
            Err(DustError::Manifest(message)) if message.contains("package.json")
        ));
    }
}
