mod cli;
mod commands;
mod format;
mod shared;

use clap::Parser;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => {
            commands::scan::execute(args);
        }

        Commands::Analyze(args) => {
            commands::analyze::execute(args);
        }

        Commands::Clean(args) => {
            if args.dry_run {
                commands::clean::dry_run(&args);
            } else {
                commands::clean::execute(&args);
            }
        }
    }
}
