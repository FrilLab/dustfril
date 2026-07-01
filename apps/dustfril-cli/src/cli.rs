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

    /// Audit lifecycle scripts
    Audit(PathArgs),
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
}
