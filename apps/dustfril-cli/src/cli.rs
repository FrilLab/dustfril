use clap::{Parser, Subcommand};

/// DustFril CLI
#[derive(Parser, Debug)]
#[command(name = "dfr", version, about = "Rust artifact analyzer and cleaner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan Rust artifacts
    Scan,

    /// Analyze disk usage
    Analyze,

    /// Clean artifacts
    Clean,
}
