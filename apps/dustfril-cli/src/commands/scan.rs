use dustfril_core::api;

use crate::{
    cli::PathArgs,
    shared::path::{resolve_path, validate_path},
};

pub fn execute(args: PathArgs) {
    let path = resolve_path(&args.path);

    if !validate_path(&path) {
        eprintln!("Invalid path");
        return;
    }

    let ecosystems = args.ecosystems();

    let result = match api::scan(&path, &ecosystems) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return;
        }
    };

    let total_size_bytes = api::analyze(result.clone())
        .map(|analysis| analysis.total_size_bytes)
        .unwrap_or_default();
    if let Err(error) = api::history::record_scan(&path, &result, total_size_bytes) {
        eprintln!("Failed to record scan history: {error}");
    }

    if result.artifacts.is_empty() {
        println!("No artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", result.artifacts.len());

    for artifact in result.artifacts {
        println!("  [{}] {}", artifact.ecosystem, artifact.path.display());
    }
}
