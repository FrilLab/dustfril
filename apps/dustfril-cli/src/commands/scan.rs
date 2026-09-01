use dustfril_core::api;

use crate::{
    cli::PathArgs,
    history,
    shared::path::{resolve_path, validate_path},
};

pub fn execute(args: PathArgs) -> bool {
    let path = match resolve_path(&args.path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to resolve path: {error}");
            return false;
        }
    };

    if !validate_path(&path) {
        return false;
    }

    let ecosystems = args.ecosystems();

    let result = match api::scan(&path, &ecosystems) {
        Ok(res) => res,
        Err(e) => {
            if let Err(history_error) = history::record_scan_failure(&path, &e.to_string()) {
                eprintln!("Failed to record scan failure history: {history_error}");
            }
            eprintln!("Scan failed: {}", e);
            return false;
        }
    };

    match api::analyze(result.clone()) {
        Ok(analysis) => {
            if let Err(error) = api::history::record_scan(&path, &result, analysis.total_size_bytes)
            {
                eprintln!("Failed to record scan history: {error}");
            }
        }
        Err(error) => {
            eprintln!("Failed to calculate scan size; history was not recorded: {error}");
        }
    }

    if result.artifacts.is_empty() {
        println!("No artifacts found.");
        return true;
    }

    println!("Found {} artifact(s)\n", result.artifacts.len());

    for artifact in result.artifacts {
        println!("  [{}] {}", artifact.ecosystem, artifact.path.display());
    }

    true
}
