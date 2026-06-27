use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ecosystem {
    Rust,
    Node,
    Java,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),

            Self::Node => write!(f, "Node"),

            Self::Java => write!(f, "Java"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanResult {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub ecosystem: Ecosystem,
}

impl Artifact {
    /// Creates a scanned artifact entry for the given path and ecosystem.
    pub fn new(path: PathBuf, ecosystem: Ecosystem) -> Self {
        Self { path, ecosystem }
    }
}
