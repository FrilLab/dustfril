mod cli;
mod commands;

use clap::Parser;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan => {
            commands::scan::execute();
        }

        Commands::Analyze => {
            commands::analyze::execute();
        }

        Commands::Clean(args) => {
            if args.dry_run {
                commands::clean::dry_run();
            } else {
                commands::clean::execute();
            }
        }
    }
}
