//! Platform-aware executable code-signature verification models.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The operating-system signing mechanism used for a signature inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignaturePlatform {
    MacOs,
    Windows,
    Linux,
    Other,
}

impl fmt::Display for SignaturePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::MacOs => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::Other => "Other",
        };

        f.write_str(label)
    }
}

/// Platform-neutral result of checking an executable's code signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SignatureStatus {
    /// The platform verifier accepted the signature cryptographically.
    Valid,
    /// The target has no signature understood by the platform verifier.
    Unsigned,
    /// The target has a signature, but verification rejected it.
    Invalid,
    /// The current platform has no supported verifier for this check.
    Unsupported,
    /// The target or verifier could not be inspected reliably.
    InspectionFailed,
}

impl fmt::Display for SignatureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Valid => "Valid",
            Self::Unsigned => "Unsigned",
            Self::Invalid => "Invalid",
            Self::Unsupported => "Unsupported",
            Self::InspectionFailed => "Inspection failed",
        };

        f.write_str(label)
    }
}

/// The operational reason a signature check could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SignatureFailureKind {
    PlatformUnsupported,
    VerifierUnavailable,
    VerifierFailed,
    TargetMissing,
    TargetUnreadable,
    TargetNonRegularFile,
    TargetChangedDuringVerification,
}

impl fmt::Display for SignatureFailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::PlatformUnsupported => "Platform unsupported",
            Self::VerifierUnavailable => "Verifier unavailable",
            Self::VerifierFailed => "Verifier failed",
            Self::TargetMissing => "Target missing",
            Self::TargetUnreadable => "Target unreadable",
            Self::TargetNonRegularFile => "Target is not a regular file",
            Self::TargetChangedDuringVerification => "Target changed during verification",
        };

        f.write_str(label)
    }
}

/// Details about an operational failure, separate from an unsigned or
/// cryptographically invalid signature result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureFailure {
    pub kind: SignatureFailureKind,
    pub message: String,
}

impl SignatureFailure {
    pub(crate) fn new(kind: SignatureFailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Structured evidence from one platform-specific signature inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureReport {
    pub platform: SignaturePlatform,
    pub status: SignatureStatus,
    /// Signer/publisher information reported by the operating system, when
    /// available. This is evidence, not a general trust verdict.
    pub signer: Option<String>,
    pub team_identifier: Option<String>,
    pub verification_message: Option<String>,
    pub verification_code: Option<i32>,
    pub failure: Option<SignatureFailure>,
}

impl SignatureReport {
    pub(crate) fn new(platform: SignaturePlatform, status: SignatureStatus) -> Self {
        Self {
            platform,
            status,
            signer: None,
            team_identifier: None,
            verification_message: None,
            verification_code: None,
            failure: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_platform_serialize_as_stable_wire_values() {
        let mut report = SignatureReport::new(SignaturePlatform::MacOs, SignatureStatus::Valid);
        report.signer = Some("Example Developer".to_owned());

        let value = serde_json::to_value(report).unwrap();

        assert_eq!(value["platform"], "macos");
        assert_eq!(value["status"], "valid");
        assert_eq!(value["signer"], "Example Developer");
    }
}
