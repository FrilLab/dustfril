//! Platform-specific, read-only executable signature verification.

use std::{fs, io, path::Path};

use crate::models::{
    SignatureFailure, SignatureFailureKind, SignaturePlatform, SignatureReport, SignatureStatus,
};

#[cfg(target_os = "macos")]
mod macos {
    include!("macos.rs");
}

/// Verifies an already resolved executable without executing it.
pub fn verify(path: &Path) -> SignatureReport {
    let platform = current_platform();

    if let Err(failure) = validate_target(path) {
        let mut report = SignatureReport::new(platform, SignatureStatus::InspectionFailed);
        report.verification_message = Some("The executable could not be inspected".to_owned());
        report.failure = Some(failure);
        return report;
    }

    #[cfg(target_os = "macos")]
    {
        macos::verify(path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        unsupported_report(platform)
    }
}

pub(crate) fn target_changed_report(
    previous: &SignatureReport,
    message: impl Into<String>,
) -> SignatureReport {
    let mut report = SignatureReport::new(previous.platform, SignatureStatus::InspectionFailed);
    report.verification_code = previous.verification_code;
    report.verification_message =
        Some("The executable changed while signature verification was in progress".to_owned());
    report.failure = Some(SignatureFailure::new(
        SignatureFailureKind::TargetChangedDuringVerification,
        message,
    ));
    report
}

fn validate_target(path: &Path) -> Result<(), SignatureFailure> {
    let metadata = fs::metadata(path).map_err(|error| {
        let kind = if error.kind() == io::ErrorKind::NotFound {
            SignatureFailureKind::TargetMissing
        } else {
            SignatureFailureKind::TargetUnreadable
        };

        SignatureFailure::new(kind, format!("{}: {error}", path.display()))
    })?;

    if !metadata.is_file() {
        return Err(SignatureFailure::new(
            SignatureFailureKind::TargetNonRegularFile,
            format!("target is not a regular file: {}", path.display()),
        ));
    }

    fs::File::open(path).map_err(|error| {
        SignatureFailure::new(
            SignatureFailureKind::TargetUnreadable,
            format!("{}: {error}", path.display()),
        )
    })?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn unsupported_report(platform: SignaturePlatform) -> SignatureReport {
    let mut report = SignatureReport::new(platform, SignatureStatus::Unsupported);
    report.verification_message = Some(match platform {
        SignaturePlatform::Linux => {
            "Linux does not provide a universal executable code-signature verifier".to_owned()
        }
        SignaturePlatform::Windows => {
            "Windows Authenticode verification is not implemented in this build".to_owned()
        }
        SignaturePlatform::Other => {
            "No executable code-signature verifier is implemented for this platform".to_owned()
        }
        SignaturePlatform::MacOs => unreachable!("macOS uses its platform verifier"),
    });
    report.failure = Some(SignatureFailure::new(
        SignatureFailureKind::PlatformUnsupported,
        "no DustFril signature verifier is configured for this platform",
    ));
    report
}

fn current_platform() -> SignaturePlatform {
    #[cfg(target_os = "macos")]
    {
        SignaturePlatform::MacOs
    }

    #[cfg(target_os = "windows")]
    {
        SignaturePlatform::Windows
    }

    #[cfg(target_os = "linux")]
    {
        SignaturePlatform::Linux
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        SignaturePlatform::Other
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn missing_target_is_not_mislabeled_as_unsigned() {
        let temp = TempDir::new().unwrap();
        let report = verify(&temp.path().join("missing-tool"));

        assert_eq!(report.status, SignatureStatus::InspectionFailed);
        assert_eq!(
            report.failure.unwrap().kind,
            SignatureFailureKind::TargetMissing
        );
    }

    #[test]
    fn directory_target_is_not_mislabeled_as_unsigned() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("tool");
        fs::create_dir(&target).unwrap();

        let report = verify(&target);

        assert_eq!(report.status, SignatureStatus::InspectionFailed);
        assert_eq!(
            report.failure.unwrap().kind,
            SignatureFailureKind::TargetNonRegularFile
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn supported_file_on_non_macos_is_explicitly_unsupported() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("tool");
        fs::write(&target, b"not an executable for a signature verifier").unwrap();

        let report = verify(&target);

        assert_eq!(report.status, SignatureStatus::Unsupported);
        assert_eq!(
            report.failure.unwrap().kind,
            SignatureFailureKind::PlatformUnsupported
        );
    }

    #[test]
    fn inspecting_a_fake_executable_never_launches_it() {
        let temp = TempDir::new().unwrap();
        let marker = temp.path().join("launched");
        let target = temp.path().join("fake executable");
        let script = format!("#!/bin/sh\ntouch {}\n", marker.display());
        fs::write(&target, script).unwrap();

        let _report = verify(&target);

        assert!(!marker.exists());
    }

    #[test]
    fn a_changed_target_invalidates_signature_evidence() {
        let previous = SignatureReport::new(SignaturePlatform::MacOs, SignatureStatus::Valid);
        let report = target_changed_report(&previous, "hash changed after verification");

        assert_eq!(report.status, SignatureStatus::InspectionFailed);
        assert_eq!(
            report.failure.unwrap().kind,
            SignatureFailureKind::TargetChangedDuringVerification
        );
    }
}
