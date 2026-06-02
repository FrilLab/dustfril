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
    Scan,

    /// Analyze disk usage
    Analyze,

    /// Clean artifacts
    Clean(CleanArgs),
}

#[derive(Args)]
pub struct CleanArgs {
    #[arg(long)]
    pub dry_run: bool,
}
