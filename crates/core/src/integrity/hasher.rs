use std::{fs::File, io::Read};

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::models::{ExecutableObservation, IntegrityFailure, IntegrityFailureKind};

use super::resolver::ResolvedExecutable;

const HASH_BUFFER_SIZE: usize = 64 * 1024;

/// Hashes a resolved executable and builds an observation without executing it.
pub fn observe(resolved: ResolvedExecutable) -> Result<ExecutableObservation, IntegrityFailure> {
    let mut file = File::open(&resolved.canonical_path).map_err(|error| {
        IntegrityFailure::new(
            IntegrityFailureKind::Unreadable,
            format!("{}: {error}", resolved.canonical_path.display()),
        )
    })?;
    let metadata = file.metadata().map_err(|error| {
        IntegrityFailure::new(
            IntegrityFailureKind::Unreadable,
            format!("{}: {error}", resolved.canonical_path.display()),
        )
    })?;

    let sha256 = hash_reader(&mut file, &resolved.canonical_path)?;
    let observed_at = Utc::now();

    Ok(ExecutableObservation {
        requested_tool: resolved.requested_tool,
        resolved_path: resolved.resolved_path,
        canonical_path: resolved.canonical_path,
        symlink_target: resolved.symlink_target,
        size_bytes: metadata.len(),
        sha256,
        observed_at,
        version_metadata: None,
    })
}

fn hash_reader(reader: &mut impl Read, path: &std::path::Path) -> Result<String, IntegrityFailure> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|error| {
            IntegrityFailure::new(
                IntegrityFailureKind::HashFailed,
                format!("{}: {error}", path.display()),
            )
        })?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("fixture read failure"))
        }
    }

    #[test]
    fn hash_reader_returns_a_structured_hash_failure() {
        let failure = hash_reader(&mut FailingReader, std::path::Path::new("fixture")).unwrap_err();

        assert_eq!(failure.kind, IntegrityFailureKind::HashFailed);
        assert!(failure.message.contains("fixture read failure"));
    }
}
