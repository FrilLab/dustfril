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

    let result = match api::scan(&path, args.global) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return;
        }
    };

    if result.artifacts.is_empty() {
        println!("No Rust artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", result.artifacts.len());

    for artifact in result.artifacts {
        println!("[{}]", artifact.artifact_type);

        println!("  {}\n", artifact.path.display());
    }
}
