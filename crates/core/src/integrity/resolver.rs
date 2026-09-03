use std::{
    env,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

use crate::models::{IntegrityFailure, IntegrityFailureKind, ToolSpec};

/// PATH-based executable resolver that does not execute candidates.
#[derive(Debug, Clone)]
pub struct ToolResolver {
    search_paths: Vec<PathBuf>,
}

impl ToolResolver {
    /// Builds a resolver from the current process PATH.
    pub fn from_environment() -> io::Result<Self> {
        Ok(Self::from_path(env::var_os("PATH")))
    }

    /// Builds a resolver from a PATH-shaped value.
    pub fn from_path(path: Option<OsString>) -> Self {
        let search_paths = path
            .as_deref()
            .map(env::split_paths)
            .map(Iterator::collect)
            .unwrap_or_default();

        Self { search_paths }
    }

    /// Builds a resolver from explicit search directories, useful for callers
    /// that need deterministic resolution or tests with temporary fixtures.
    pub fn from_paths(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            search_paths: paths.into_iter().collect(),
        }
    }

    /// Resolves a command name or explicit path and inspects only its identity.
    pub fn resolve(&self, tool: &ToolSpec) -> Result<ResolvedExecutable, IntegrityFailure> {
        if tool.name.trim().is_empty() || tool.name.contains('\0') {
            return Err(IntegrityFailure::new(
                IntegrityFailureKind::InvalidToolName,
                "requested tool name is empty or contains a NUL byte",
            ));
        }

        let requested = Path::new(&tool.name);
        if is_explicit_path(requested) {
            return self.inspect_candidate(tool, make_absolute(requested));
        }

        for search_path in &self.search_paths {
            let directory = if search_path.as_os_str().is_empty() {
                Path::new(".")
            } else {
                search_path.as_path()
            };

            for candidate_name in candidate_names(&tool.name) {
                let candidate = make_absolute(&directory.join(candidate_name));
                match fs::symlink_metadata(&candidate) {
                    Ok(_) => match self.inspect_candidate(tool, candidate) {
                        Err(failure) if failure.kind == IntegrityFailureKind::NonExecutable => {
                            continue;
                        }
                        result => return result,
                    },
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => {
                        return Err(io_failure(
                            IntegrityFailureKind::Unreadable,
                            &candidate,
                            error,
                        ));
                    }
                }
            }
        }

        Err(IntegrityFailure::new(
            IntegrityFailureKind::NotFound,
            format!("no PATH candidate found for {}", tool.name),
        ))
    }

    fn inspect_candidate(
        &self,
        tool: &ToolSpec,
        resolved_path: PathBuf,
    ) -> Result<ResolvedExecutable, IntegrityFailure> {
        let link_metadata = fs::symlink_metadata(&resolved_path).map_err(|error| {
            let kind = if error.kind() == io::ErrorKind::NotFound {
                IntegrityFailureKind::NotFound
            } else {
                IntegrityFailureKind::Unreadable
            };
            io_failure(kind, &resolved_path, error)
        })?;
        let is_symlink = link_metadata.file_type().is_symlink();
        let symlink_target = if is_symlink {
            Some(fs::read_link(&resolved_path).map_err(|error| {
                io_failure(IntegrityFailureKind::Unreadable, &resolved_path, error)
            })?)
        } else {
            None
        };

        let canonical_path = fs::canonicalize(&resolved_path).map_err(|error| {
            let kind = if is_symlink && error.kind() == io::ErrorKind::NotFound {
                IntegrityFailureKind::BrokenSymlink
            } else if is_symlink && is_symlink_loop(&error) {
                IntegrityFailureKind::SymlinkLoop
            } else {
                IntegrityFailureKind::Unreadable
            };
            io_failure(kind, &resolved_path, error)
        })?;
        let target_metadata = fs::metadata(&canonical_path).map_err(|error| {
            io_failure(IntegrityFailureKind::Unreadable, &canonical_path, error)
        })?;

        if !target_metadata.is_file() {
            return Err(IntegrityFailure::new(
                IntegrityFailureKind::NonRegularFile,
                format!(
                    "resolved target for {} is not a regular file: {}",
                    tool.name,
                    canonical_path.display()
                ),
            ));
        }

        if !is_executable(&canonical_path, &target_metadata) {
            return Err(IntegrityFailure::new(
                IntegrityFailureKind::NonExecutable,
                format!(
                    "resolved target for {} is not executable: {}",
                    tool.name,
                    canonical_path.display()
                ),
            ));
        }

        Ok(ResolvedExecutable {
            requested_tool: tool.name.clone(),
            resolved_path,
            canonical_path,
            symlink_target,
        })
    }
}

#[cfg(unix)]
fn is_executable(_path: &Path, metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(windows)]
fn is_executable(path: &Path, _metadata: &fs::Metadata) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    let extension = format!(".{extension}");
    let path_extensions =
        env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".COM;.EXE;.BAT;.CMD"));

    path_extensions
        .to_string_lossy()
        .split(';')
        .any(|candidate| candidate.eq_ignore_ascii_case(&extension))
}

#[cfg(not(any(unix, windows)))]
fn is_executable(_path: &Path, _metadata: &fs::Metadata) -> bool {
    true
}

/// The selected PATH identity and the canonical regular file to hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExecutable {
    pub requested_tool: String,
    pub resolved_path: PathBuf,
    pub canonical_path: PathBuf,
    pub symlink_target: Option<PathBuf>,
}

fn is_explicit_path(path: &Path) -> bool {
    path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        || path.to_string_lossy().contains(['/', '\\'])
}

fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    env::current_dir()
        .map(|current| current.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(windows)]
fn candidate_names(name: &str) -> Vec<PathBuf> {
    let path = Path::new(name);
    if path.extension().is_some() {
        return vec![path.to_path_buf()];
    }

    let extensions = env::var_os("PATHEXT")
        .map(|value| {
            value
                .to_string_lossy()
                .split(';')
                .filter(|extension| !extension.is_empty())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .filter(|extensions| !extensions.is_empty())
        .unwrap_or_else(|| {
            [".COM", ".EXE", ".BAT", ".CMD"]
                .into_iter()
                .map(PathBuf::from)
                .collect()
        });

    extensions
        .into_iter()
        .map(|extension| PathBuf::from(format!("{name}{}", extension.display())))
        .collect()
}

#[cfg(not(windows))]
fn candidate_names(name: &str) -> Vec<PathBuf> {
    vec![PathBuf::from(name)]
}

fn io_failure(kind: IntegrityFailureKind, path: &Path, error: io::Error) -> IntegrityFailure {
    IntegrityFailure::new(kind, format!("{}: {error}", path.display()))
}

fn is_symlink_loop(error: &io::Error) -> bool {
    #[cfg(unix)]
    if error.raw_os_error() == Some(40) {
        return true;
    }

    let message = error.to_string().to_ascii_lowercase();
    message.contains("too many levels of symbolic links")
        || message.contains("symbolic link loop")
        || message.contains("symlink loop")
}
