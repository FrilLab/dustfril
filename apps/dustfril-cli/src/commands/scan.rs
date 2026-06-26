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

    if result.artifacts.is_empty() {
        println!("No artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", result.artifacts.len());

    for artifact in result.artifacts {
        println!("  {:?}\n", artifact.path);
    }
}
