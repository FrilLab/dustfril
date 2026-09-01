mod cli;
mod commands;
mod format;
mod history;
mod shared;

use clap::Parser;
use std::process::ExitCode;

use cli::{Cli, Commands};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let succeeded = match cli.command {
        Commands::Scan(args) => commands::scan::execute(args),

        Commands::Analyze(args) => commands::analyze::execute(args),

        Commands::Clean(args) => {
            if args.dry_run {
                commands::clean::dry_run(&args)
            } else {
                commands::clean::execute(&args)
            }
        }

        Commands::History => commands::history::execute(),

        Commands::Audit(args) => commands::audit::execute(&args),

        Commands::Security(args) => match args.command {
            cli::SecurityCommands::Scan(args) => commands::security::scan(&args),
        },

        Commands::Integrity(args) => match args.command {
            cli::IntegrityCommands::Scan(args) => commands::integrity::scan(&args),
        },
    };

    if succeeded {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
