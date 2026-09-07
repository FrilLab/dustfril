// macOS code-signature verification through the system `codesign` verifier.
use std::{io, path::Path, process::Command};

use crate::models::{
    SignatureFailure, SignatureFailureKind, SignaturePlatform, SignatureReport, SignatureStatus,
};

const CODESIGN_PATH: &str = "/usr/bin/codesign";

pub fn verify(path: &Path) -> SignatureReport {
    let output = Command::new(CODESIGN_PATH)
        .arg("--verify")
        .arg("--verbose=4")
        // `path` is an argument to the verifier, never shell-interpolated.
        .arg(path.as_os_str())
        .output();

    match output {
        Ok(output) => report_from_output(
            path,
            output.status.code(),
            output.status.success(),
            &output,
        ),
        Err(error) => report_for_command_error(error),
    }
}

fn report_from_output(
    path: &Path,
    exit_code: Option<i32>,
    successful: bool,
    output: &std::process::Output,
) -> SignatureReport {
    let details = parse_codesign_details(&combined_output(output));

    if successful {
        let mut report = SignatureReport::new(SignaturePlatform::MacOs, SignatureStatus::Valid);
        report.verification_code = exit_code;
        report.verification_message = Some("codesign accepted the executable signature".to_owned());
        if let Some(metadata) = display_metadata(path) {
            report.signer = metadata.signer;
            report.team_identifier = metadata.team_identifier;
        } else {
            report.verification_message = Some(
                "codesign accepted the executable signature; signer metadata is unavailable"
                    .to_owned(),
            );
        }
        return report;
    }

    let status = classify_failure(&details.output);
    let mut report = SignatureReport::new(SignaturePlatform::MacOs, status);
    report.signer = details.signer;
    report.team_identifier = details.team_identifier;
    report.verification_code = exit_code;
    report.verification_message = Some(match status {
        SignatureStatus::Unsigned => "codesign found no supported signature".to_owned(),
        SignatureStatus::Invalid => "codesign rejected the executable signature".to_owned(),
        SignatureStatus::InspectionFailed => {
            "codesign could not classify the verification result".to_owned()
        }
        SignatureStatus::Valid | SignatureStatus::Unsupported => {
            "codesign returned an unexpected verification result".to_owned()
        }
    });

    if status == SignatureStatus::InspectionFailed {
        report.failure = Some(SignatureFailure::new(
            SignatureFailureKind::VerifierFailed,
            verifier_failure_message(exit_code),
        ));
    }

    report
}

fn display_metadata(path: &Path) -> Option<CodesignDetails> {
    let output = Command::new(CODESIGN_PATH)
        .arg("-d")
        .arg("--verbose=4")
        // `path` is an argument to the verifier, never shell-interpolated.
        .arg(path.as_os_str())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(parse_codesign_details(&combined_output(&output)))
}

fn report_for_command_error(error: io::Error) -> SignatureReport {
    let kind = if error.kind() == io::ErrorKind::NotFound {
        SignatureFailureKind::VerifierUnavailable
    } else {
        SignatureFailureKind::VerifierFailed
    };
    let mut report =
        SignatureReport::new(SignaturePlatform::MacOs, SignatureStatus::InspectionFailed);
    report.verification_message = Some("macOS signature verifier could not be started".to_owned());
    report.failure = Some(SignatureFailure::new(
        kind,
        format!("{CODESIGN_PATH}: {error}"),
    ));
    report
}

fn verifier_failure_message(exit_code: Option<i32>) -> String {
    match exit_code {
        Some(code) => {
            format!("codesign returned an unclassified verification failure (exit code {code})")
        }
        None => "codesign terminated without an exit code".to_owned(),
    }
}

struct CodesignDetails {
    output: String,
    signer: Option<String>,
    team_identifier: Option<String>,
}

fn parse_codesign_details(output: &str) -> CodesignDetails {
    let mut signer = None;
    let mut team_identifier = None;

    for line in output.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("Authority=")
            && !value.is_empty()
            && signer.is_none()
        {
            signer = Some(value.to_owned());
        }
        if let Some(value) = line.strip_prefix("TeamIdentifier=")
            && !value.is_empty()
            && !value.eq_ignore_ascii_case("not set")
        {
            team_identifier = Some(value.to_owned());
        }
    }

    CodesignDetails {
        output: output.to_owned(),
        signer,
        team_identifier,
    }
}

fn classify_failure(output: &str) -> SignatureStatus {
    let output = output.to_ascii_lowercase();

    if output.contains("not signed")
        || output.contains("no code signature")
        || output.contains("code object is unsigned")
    {
        SignatureStatus::Unsigned
    } else if output.contains("invalid")
        || output.contains("sealed resource")
        || output.contains("code object is unsealed")
        || output.contains("code or signature modified")
        || output.contains("invalid signature")
        || output.contains("signature invalid")
        || output.contains("hash mismatch")
        || output.contains("a valid code signature")
    {
        SignatureStatus::Invalid
    } else {
        SignatureStatus::InspectionFailed
    }
}

fn combined_output(output: &std::process::Output) -> String {
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    if !combined.is_empty() {
        combined.push('\n');
    }
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_codesign_unsigned_output_without_treating_it_as_verifier_failure() {
        assert_eq!(
            classify_failure("/tmp/tool: code object is not signed at all"),
            SignatureStatus::Unsigned
        );
    }

    #[test]
    fn maps_codesign_tampering_output_to_invalid() {
        assert_eq!(
            classify_failure("a sealed resource is missing or invalid"),
            SignatureStatus::Invalid
        );
        assert_eq!(
            classify_failure("code or signature modified"),
            SignatureStatus::Invalid
        );
    }

    #[test]
    fn leaves_unknown_codesign_failures_as_inspection_failures() {
        assert_eq!(
            classify_failure("codesign: operation not permitted"),
            SignatureStatus::InspectionFailed
        );
    }

    #[test]
    fn extracts_signer_and_team_metadata() {
        let details = parse_codesign_details(
            "Authority=Developer ID Application: Example (TEAM123)\nTeamIdentifier=TEAM123\n",
        );

        assert_eq!(
            details.signer.as_deref(),
            Some("Developer ID Application: Example (TEAM123)")
        );
        assert_eq!(details.team_identifier.as_deref(), Some("TEAM123"));
    }

    #[test]
    fn unsigned_fixture_is_reported_as_unsigned() {
        let temp = tempfile::TempDir::new().unwrap();
        let target = temp.path().join("unsigned-tool");
        std::fs::write(&target, b"unsigned fixture").unwrap();

        let report = verify(&target);

        assert_eq!(report.status, SignatureStatus::Unsigned);
        assert!(report.failure.is_none());
    }

    #[test]
    fn signed_system_fixture_is_reported_as_valid() {
        let report = verify(Path::new("/usr/bin/true"));

        assert_eq!(report.status, SignatureStatus::Valid);
        // The system authority is named "macOS Software Signing" on newer
        // macOS releases and "Software Signing" on older releases.
        assert!(matches!(
            report.signer.as_deref(),
            Some("Software Signing" | "macOS Software Signing")
        ));
        assert!(report.team_identifier.is_none());
        assert!(report.failure.is_none());
    }

    #[test]
    fn special_path_is_passed_without_shell_interpolation() {
        let temp = tempfile::TempDir::new().unwrap();
        let target = temp.path().join("tool with spaces;$(touch marker)");
        std::fs::write(&target, b"unsigned fixture").unwrap();

        let report = verify(&target);

        assert_eq!(report.status, SignatureStatus::Unsigned);
        assert!(target.exists());
    }
}
