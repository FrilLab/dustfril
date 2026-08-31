use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleScript {
    pub package: String,
    pub package_manager: PackageManager,
    pub script_type: ScriptType,
    pub command: String,
    pub risk_level: RiskLevel,
}

/// A lifecycle script finding produced by the security rule engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityWarning {
    pub package: String,
    pub script_type: String,
    pub command: String,
    pub risk_level: RiskLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Unknown,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
            Self::Unknown => "unknown",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScriptType {
    Preinstall,
    Install,
    Postinstall,
    Prepare,
    Prepublish,
    PrepublishOnly,
}

impl ScriptType {
    pub fn from_script_name(name: &str) -> Option<Self> {
        match name {
            "preinstall" => Some(Self::Preinstall),
            "install" => Some(Self::Install),
            "postinstall" => Some(Self::Postinstall),
            "prepare" => Some(Self::Prepare),
            "prepublish" => Some(Self::Prepublish),
            "prepublishOnly" => Some(Self::PrepublishOnly),
            _ => None,
        }
    }
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Preinstall => "preinstall",
            Self::Install => "install",
            Self::Postinstall => "postinstall",
            Self::Prepare => "prepare",
            Self::Prepublish => "prepublish",
            Self::PrepublishOnly => "prepublishOnly",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        };

        write!(f, "{value}")
    }
}
