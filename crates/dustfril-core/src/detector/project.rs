use std::{
    fs,
    path::{Path, PathBuf},
};

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Ecosystem {
// Rust,
// Node,
// Java,
// }

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    // pub ecosystem: Ecosystem,
}

pub fn find_projects(root: &Path) -> Vec<Project> {
    let mut projects = Vec::new();

    visit(root, &mut projects);

    projects
}

/// Cargo project detection and artifact scanning.
pub fn is_cargo_project(root: &Path) -> bool {
    root.join("Cargo.toml").is_file()
}

fn visit(dir: &Path, projects: &mut Vec<Project>) {
    if is_cargo_project(dir) {
        projects.push(Project {
            root: dir.to_path_buf(),
            // ecosystem: Ecosystem::Rust,
        });

        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            visit(&path, projects);
        }
    }
}
