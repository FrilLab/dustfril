use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// DustFril CLI
#[derive(Parser)]
#[command(name = "dfr", version, about = "Rust artifact analyzer and cleaner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Scan Rust artifacts
    Scan(PathArgs),

    /// Analyze disk usage
    Analyze(PathArgs),

    /// Clean artifacts
    Clean(CleanArgs),
}

#[derive(Args)]
pub struct PathArgs {
    pub path: Option<PathBuf>,

    /// Scan the entire system instead of a specific workspace.
    #[arg(long)]
    pub global: bool,
}

#[derive(Args)]
pub struct CleanArgs {
    #[command(flatten)]
    pub path_args: PathArgs,

    // Preview cleanup operations without deleting files.
    #[arg(long)]
    pub dry_run: bool,
}
