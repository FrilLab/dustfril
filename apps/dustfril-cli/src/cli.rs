use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use dustfril_core::models::Ecosystem;

/// DustFril CLI
#[derive(Parser)]
#[command(
    name = "dfr",
    version,
    about = "Development artifact analyzer and cleaner"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Scan build artifacts
    Scan(PathArgs),

    /// Analyze artifact disk usage
    Analyze(PathArgs),

    /// Clean detected artifacts
    Clean(CleanArgs),

    /// Show the unified local activity history
    History,

    /// Audit lifecycle scripts
    Audit(PathArgs),

    /// Report dependency inventory and explicit baseline changes
    #[command(name = "dependencies", visible_alias = "dependency")]
    Dependencies(DependencyArgs),

    /// Scan lifecycle scripts for suspicious commands
    Security(SecurityArgs),

    /// Compare selected development-tool executables with local baselines
    Integrity(IntegrityArgs),
}

#[derive(Args)]
pub struct SecurityArgs {
    #[command(subcommand)]
    pub command: SecurityCommands,
}

#[derive(Subcommand)]
pub enum SecurityCommands {
    /// Scan Node lifecycle scripts for suspicious commands
    Scan(PathArgs),
    /// Scan local GitHub Actions workflow files without executing them
    #[command(name = "workflows", visible_alias = "workflow")]
    Workflows(WorkflowPathArgs),
}

#[derive(Args)]
pub struct WorkflowPathArgs {
    pub path: Option<PathBuf>,
}

#[derive(Args)]
pub struct IntegrityArgs {
    #[command(subcommand)]
    pub command: IntegrityCommands,
}

#[derive(Subcommand)]
pub enum IntegrityCommands {
    /// Inspect development-tool executables without launching them
    Scan(IntegrityScanArgs),
}

#[derive(Args)]
pub struct IntegrityScanArgs {
    /// Tool name to inspect; repeat for multiple tools. Defaults to the initial tool set.
    #[arg(long = "tool", value_name = "NAME")]
    pub tools: Vec<String>,
}

impl IntegrityScanArgs {
    pub fn tools(&self) -> Vec<dustfril_core::models::ToolSpec> {
        if self.tools.is_empty() {
            return dustfril_core::api::integrity::default_tools();
        }

        self.tools.iter().cloned().map(Into::into).collect()
    }
}

#[derive(Args)]
pub struct PathArgs {
    pub path: Option<PathBuf>,

    #[arg(long)]
    pub rust: bool,

    #[arg(long)]
    pub node: bool,

    #[arg(long)]
    pub java: bool,
}

#[derive(Args)]
pub struct DependencyArgs {
    #[command(flatten)]
    pub path_args: PathArgs,

    /// Compare the current inventory with the stored local baseline.
    #[arg(long)]
    pub compare: bool,

    /// Explicitly accept the current inventory after displaying its diff.
    #[arg(long)]
    pub accept_baseline: bool,
}

impl DependencyArgs {
    pub fn ecosystems(&self) -> Vec<Ecosystem> {
        self.path_args.ecosystems()
    }
}

impl PathArgs {
    /// Returns the selected ecosystem filters in CLI flag order.
    ///
    /// An empty result means all ecosystems should be scanned.
    pub fn ecosystems(&self) -> Vec<Ecosystem> {
        let mut ecosystems = Vec::new();

        if self.rust {
            ecosystems.push(Ecosystem::Rust);
        }

        if self.node {
            ecosystems.push(Ecosystem::Node);
        }

        if self.java {
            ecosystems.push(Ecosystem::Java);
        }

        ecosystems
    }
}

#[derive(Args)]
pub struct CleanArgs {
    #[arg(long)]
    pub permanent: bool,

    #[command(flatten)]
    pub path_args: PathArgs,

    // Preview cleanup operations without deleting files.
    #[arg(long)]
    pub dry_run: bool,
}

impl CleanArgs {
    /// Returns the selected cleanup ecosystem filters.
    pub fn ecosystems(&self) -> Vec<Ecosystem> {
        self.path_args.ecosystems()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_args_ecosystems_returns_empty_when_no_flags_are_set() {
        let args = PathArgs {
            path: None,
            rust: false,
            node: false,
            java: false,
        };

        assert!(args.ecosystems().is_empty());
    }

    #[test]
    fn path_args_ecosystems_preserves_flag_order() {
        let args = PathArgs {
            path: None,
            rust: true,
            node: true,
            java: true,
        };

        assert_eq!(
            args.ecosystems(),
            vec![Ecosystem::Rust, Ecosystem::Node, Ecosystem::Java]
        );
    }

    #[test]
    fn clean_args_ecosystems_delegates_to_path_args() {
        let args = CleanArgs {
            path_args: PathArgs {
                path: None,
                rust: false,
                node: true,
                java: true,
            },
            dry_run: true,
            permanent: false,
        };

        assert_eq!(args.ecosystems(), vec![Ecosystem::Node, Ecosystem::Java]);
    }

    #[test]
    fn cli_parses_security_scan_command() {
        let cli = Cli::try_parse_from(["dfr", "security", "scan", "--node"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Security(SecurityArgs {
                command: SecurityCommands::Scan(PathArgs { node: true, .. })
            })
        ));
    }

    #[test]
    fn cli_parses_workflow_security_scan_command() {
        let cli = Cli::try_parse_from(["dfr", "security", "workflows", "workspace"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Security(SecurityArgs {
                command: SecurityCommands::Workflows(WorkflowPathArgs { path: Some(path) })
            }) if path == std::path::Path::new("workspace")
        ));
    }

    #[test]
    fn cli_parses_dependency_report_command() {
        let cli = Cli::try_parse_from(["dfr", "dependencies", "--rust"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Dependencies(DependencyArgs {
                path_args: PathArgs { rust: true, .. },
                compare: false,
                accept_baseline: false,
            })
        ));
    }

    #[test]
    fn cli_parses_dependency_comparison_and_explicit_acceptance() {
        let cli = Cli::try_parse_from([
            "dfr",
            "dependencies",
            "--compare",
            "--accept-baseline",
            "--node",
        ])
        .unwrap();

        let Commands::Dependencies(args) = cli.command else {
            panic!("expected dependency command");
        };
        assert!(args.compare);
        assert!(args.accept_baseline);
        assert_eq!(args.ecosystems(), vec![Ecosystem::Node]);
    }

    #[test]
    fn cli_parses_history_command() {
        let cli = Cli::try_parse_from(["dfr", "history"]).unwrap();

        assert!(matches!(cli.command, Commands::History));
    }

    #[test]
    fn cli_parses_integrity_scan_and_selected_tools() {
        let cli = Cli::try_parse_from([
            "dfr",
            "integrity",
            "scan",
            "--tool",
            "node",
            "--tool",
            "git",
        ])
        .unwrap();

        let Commands::Integrity(args) = cli.command else {
            panic!("expected integrity command");
        };
        let IntegrityCommands::Scan(args) = args.command;
        assert_eq!(args.tools(), vec!["node".into(), "git".into()]);
    }

    #[test]
    fn integrity_scan_defaults_to_core_tool_selection() {
        let args = IntegrityScanArgs { tools: Vec::new() };

        assert_eq!(args.tools(), dustfril_core::api::integrity::default_tools());
    }
}
