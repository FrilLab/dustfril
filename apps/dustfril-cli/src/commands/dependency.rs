use dustfril_core::api;

use crate::{
    cli::PathArgs,
    format,
    shared::path::{resolve_path, validate_path},
};

/// Builds and prints the structured Core dependency inventory.
pub fn execute(args: &PathArgs) -> bool {
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

    match api::dependency_report(&path, &args.ecosystems()) {
        Ok(reports) => {
            format::print_dependency_reports(&reports);
            true
        }
        Err(error) => {
            eprintln!("Dependency report failed: {error}");
            false
        }
    }
}
