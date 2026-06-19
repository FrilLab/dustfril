use std::fmt;

#[derive(Debug)]
pub enum DustError {
    Io(std::io::Error),
    InvalidPath,
    ScanFailed,
    AnalysisFailed,
    CleanupFailed,
}

pub type DustResult<T> = std::result::Result<T, DustError>;

impl fmt::Display for DustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DustError::Io(error) => {
                write!(f, "I/O error: {error}")
            }
            DustError::InvalidPath => {
                write!(f, "Invalid path")
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
        }
    }
}

impl std::error::Error for DustError {}

impl From<std::io::Error> for DustError {
    fn from(error: std::io::Error) -> Self {
        DustError::Io(error)
    }
}
