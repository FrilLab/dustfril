use std::path::Path;

use crate::models::PackageManager;

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

/// Resolves the package manager for a package.json path by checking the package
/// directory and walking up toward the scan root.
pub fn detect_for_package(package_json: &Path, scan_root: &Path) -> PackageManager {
    let mut current = package_json.parent();

    while let Some(dir) = current {
        if let Some(package_manager) = detect_in_dir(dir) {
            return package_manager;
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

    PackageManager::Unknown
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
            detect_for_package(&dependency_dir.join("package.json"), temp_dir.path()),
            PackageManager::Pnpm
        );
    }
}
