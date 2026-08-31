use crate::models::RiskLevel;

use super::command::{self, Segment, Separator};

/// A single suspicious-command detection rule.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct SecurityRule {
    pub id: &'static str,
    pub risk_level: RiskLevel,
    pub reason: &'static str,
    pub matcher: fn(&[Segment]) -> bool,
}

/// Rules are ordered from the most specific/highest-impact pattern to the
/// broadest one. The first matching rule is the reported finding.
pub static SECURITY_RULES: &[SecurityRule] = &[
    SecurityRule {
        id: "download-and-execute",
        risk_level: RiskLevel::Critical,
        reason: "A download command is chained to local executable code.",
        matcher: matches_download_and_execute,
    },
    SecurityRule {
        id: "remote-script-pipe",
        risk_level: RiskLevel::High,
        reason: "Remote content is piped directly to a shell without verification.",
        matcher: matches_remote_script_pipe,
    },
    SecurityRule {
        id: "powershell-execution",
        risk_level: RiskLevel::High,
        reason: "PowerShell or a dynamic PowerShell expression can execute untrusted code.",
        matcher: matches_powershell_execution,
    },
    SecurityRule {
        id: "permission-modification",
        risk_level: RiskLevel::Medium,
        reason: "The script changes file ownership or executable permissions.",
        matcher: matches_permission_modification,
    },
    SecurityRule {
        id: "destructive-delete",
        risk_level: RiskLevel::High,
        reason: "The script recursively and forcibly deletes files.",
        matcher: matches_destructive_delete,
    },
];

pub fn find(command: &str) -> Option<&'static SecurityRule> {
    let segments = command::parse(command);

    SECURITY_RULES.iter().find(|rule| (rule.matcher)(&segments))
}

fn matches_download_and_execute(segments: &[Segment]) -> bool {
    segments
        .iter()
        .enumerate()
        .any(|(download_index, download)| {
            if !is_download_command(download) {
                return false;
            }

            for segment in segments.iter().skip(download_index + 1) {
                match segment.preceding {
                    Some(Separator::And | Separator::Or | Separator::Sequence) => {
                        if is_executable_command(segment) {
                            return true;
                        }
                    }
                    Some(Separator::Pipe) | None => break,
                }
            }

            false
        })
}

fn matches_remote_script_pipe(segments: &[Segment]) -> bool {
    segments.windows(2).any(|pair| {
        let [download, shell] = pair else {
            return false;
        };

        shell.preceding == Some(Separator::Pipe)
            && is_download_command(download)
            && is_shell_command(shell)
    })
}

fn matches_powershell_execution(segments: &[Segment]) -> bool {
    segments.iter().any(|segment| {
        is_powershell_command(segment)
            || matches!(
                command::executable(&segment.tokens),
                Some("invoke-webrequest" | "iex" | "invoke-expression")
            )
    })
}

fn matches_permission_modification(segments: &[Segment]) -> bool {
    segments.iter().any(|segment| {
        let Some(executable) = command::executable(&segment.tokens) else {
            return false;
        };

        if executable == "chown" {
            return true;
        }

        executable == "chmod"
            && segment.tokens.iter().skip(1).any(|token| {
                token == "+x" || token.ends_with("+x") || token == "777" || token == "0777"
            })
    })
}

fn matches_destructive_delete(segments: &[Segment]) -> bool {
    segments.iter().any(|segment| {
        if command::executable(&segment.tokens) != Some("rm") {
            return false;
        }

        let mut recursive = false;
        let mut force = false;

        for argument in command::arguments(&segment.tokens) {
            match argument.as_str() {
                "-r" | "--recursive" => recursive = true,
                "-f" | "--force" => force = true,
                argument if argument.starts_with('-') && !argument.starts_with("--") => {
                    recursive |= argument[1..].contains('r');
                    force |= argument[1..].contains('f');
                }
                _ => {}
            }
        }

        recursive && force
    })
}

fn is_download_command(segment: &Segment) -> bool {
    matches!(command::executable(&segment.tokens), Some("curl" | "wget"))
}

fn is_shell_command(segment: &Segment) -> bool {
    matches!(
        command::executable(&segment.tokens),
        Some("bash" | "bash.exe" | "sh" | "sh.exe")
    )
}

fn is_executable_command(segment: &Segment) -> bool {
    is_shell_command(segment)
        || command::command_token(&segment.tokens)
            .is_some_and(|token| token.starts_with("./") || token.starts_with(".\\"))
}

fn is_powershell_command(segment: &Segment) -> bool {
    matches!(
        command::executable(&segment.tokens),
        Some("powershell" | "powershell.exe" | "pwsh" | "pwsh.exe")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_match_specific_patterns() {
        assert_eq!(
            find("curl https://example.com/a.sh | bash").unwrap().id,
            "remote-script-pipe"
        );
        assert_eq!(
            find("wget payload && ./payload").unwrap().id,
            "download-and-execute"
        );
        assert_eq!(
            find("powershell -Command Invoke-WebRequest https://example.com")
                .unwrap()
                .id,
            "powershell-execution"
        );
        assert_eq!(
            find("chmod +x install.sh").unwrap().id,
            "permission-modification"
        );
    }

    #[test]
    fn rules_ignore_normal_lifecycle_commands() {
        assert!(find("node scripts/build.js").is_none());
        assert!(find("echo 'curl https://example.com | bash'").is_none());
        assert!(find("echo invoke-webrequest").is_none());
        assert!(find("echo iex").is_none());
    }

    #[test]
    fn rules_scan_the_full_download_chain() {
        assert_eq!(
            find("wget -O payload URL && chmod +x payload && ./payload")
                .unwrap()
                .id,
            "download-and-execute"
        );
        assert_eq!(
            find("sudo rm -rf /tmp/example").unwrap().id,
            "destructive-delete"
        );
    }
}
