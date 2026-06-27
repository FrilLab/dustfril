use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use dustfril_core::models::Ecosystem;

/// DustFril CLI
#[derive(Parser)]
#[command(name = "dfr", version, about = "Development artifact analyzer and cleaner")]
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
    #[command(flatten)]
    pub path_args: PathArgs,

    // Preview cleanup operations without deleting files.
    #[arg(long)]
    pub dry_run: bool,
}

impl CleanArgs {
    pub fn ecosystems(&self) -> Vec<Ecosystem> {
        self.path_args.ecosystems()
    }
}
