use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    error::DustResult,
    integrity,
    models::{ExecutableObservation, IntegrityFailure, IntegrityReport, ToolSpec},
};

pub use crate::integrity::{BaselineStore, ResolvedExecutable, ToolResolver};

/// Returns the representative development-tool selection used by the CLI.
pub fn default_tools() -> Vec<ToolSpec> {
    integrity::default_tools()
}

/// Returns the OS-specific local path used for executable-integrity state.
pub fn state_path() -> std::io::Result<PathBuf> {
    integrity::state_path()
}

/// Resolves and compares selected tools against a local baseline using PATH.
pub fn scan(tools: &[ToolSpec], baseline_path: &Path) -> DustResult<IntegrityReport> {
    integrity::scan(tools, baseline_path)
}

/// Resolves and compares selected tools with deterministic caller-provided PATH entries.
pub fn scan_with_paths(
    tools: &[ToolSpec],
    paths: impl IntoIterator<Item = PathBuf>,
    baseline_path: &Path,
) -> DustResult<IntegrityReport> {
    let resolver = integrity::ToolResolver::from_paths(paths);
    integrity::scan_with_resolver(tools, &resolver, baseline_path)
}

/// Resolves and compares selected tools using a PATH-shaped value.
pub fn scan_with_path_value(
    tools: &[ToolSpec],
    path: Option<OsString>,
    baseline_path: &Path,
) -> DustResult<IntegrityReport> {
    let resolver = integrity::ToolResolver::from_path(path);
    integrity::scan_with_resolver(tools, &resolver, baseline_path)
}

/// Inspects one tool without comparing or persisting it.
pub fn inspect_tool(
    tool: &ToolSpec,
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<ExecutableObservation, IntegrityFailure> {
    let resolver = integrity::ToolResolver::from_paths(paths);
    integrity::inspect_tool(tool, &resolver)
}
