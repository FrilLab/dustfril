use dustfril_core::api;

use crate::{
    cli::PathArgs,
    format,
    shared::path::{resolve_path, validate_path},
};

pub fn scan(args: &PathArgs) -> bool {
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

    let warnings = match api::security_scan(&path, &ecosystems) {
        Ok(warnings) => warnings,
        Err(error) => {
            eprintln!("Security scan failed: {error}");
            return false;
        }
    };

    if warnings.is_empty() {
        println!("No suspicious lifecycle scripts detected.");
        return true;
    }

    format::print_security_report(&warnings);

    true
}
