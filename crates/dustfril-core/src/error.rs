use std::fmt;

/// Errors returned by the core scanning, analysis, and cleanup APIs.
#[derive(Debug)]
pub enum DustError {
    /// Wraps an underlying filesystem I/O error.
    Io(std::io::Error),
    /// Indicates that a user-supplied path could not be used.
    InvalidPath(std::path::PathBuf),
    /// Indicates an unrecoverable scan failure.
    ScanFailed,
    /// Indicates an unrecoverable analysis failure.
    AnalysisFailed,
    /// Indicates an unrecoverable cleanup failure.
    CleanupFailed,
    /// Indicates that Git status could not be read for an integrity check.
    Git(String),
    /// Indicates that a project manifest could not be parsed for an integrity check.
    Manifest(String),
}

/// Standard result type used by the core crate.
pub type DustResult<T> = std::result::Result<T, DustError>;

impl fmt::Display for DustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DustError::Io(error) => {
                write!(f, "I/O error: {error}")
            }
            DustError::InvalidPath(path) => {
                write!(f, "Invalid path: {}", path.display())
            }
            DustError::ScanFailed => {
                write!(f, "Scan failed")
            }
            DustError::AnalysisFailed => {
                write!(f, "Analysis failed")
            }
            DustError::CleanupFailed => {
                write!(f, "Cleanup failed")
            }
            DustError::Git(message) => {
                write!(f, "Git error: {message}")
            }
            DustError::Manifest(message) => {
                write!(f, "Manifest error: {message}")
            }
        }
    }
}

impl std::error::Error for DustError {}

impl From<std::io::Error> for DustError {
    fn from(error: std::io::Error) -> Self {
        DustError::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn from_io_error_creates_io_variant() {
        let error = DustError::from(io::Error::other("boom"));

        assert!(matches!(error, DustError::Io(_)));
    }

    #[test]
    fn display_formats_path_error() {
        let error = DustError::InvalidPath(PathBuf::from("/tmp/missing"));

        assert_eq!(error.to_string(), "Invalid path: /tmp/missing");
    }

    #[test]
    fn display_formats_cleanup_error() {
        assert_eq!(DustError::CleanupFailed.to_string(), "Cleanup failed");
    }

    #[test]
    fn display_formats_manifest_error() {
        let error = DustError::Manifest("package.json is invalid".to_owned());

        assert_eq!(error.to_string(), "Manifest error: package.json is invalid");
    }
}
