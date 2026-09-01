use dustfril_core::api;

use crate::{
    cli::PathArgs,
    format,
    shared::path::{resolve_path, validate_path},
};

pub fn scan(args: &PathArgs) {
    let path = resolve_path(&args.path);

    if !validate_path(&path) {
        eprintln!("Invalid path");
        return;
    }

    let ecosystems = args.ecosystems();

    let warnings = match api::security_scan(&path, &ecosystems) {
        Ok(warnings) => warnings,
        Err(error) => {
            eprintln!("Security scan failed: {error}");
            return;
        }
    };

    if warnings.is_empty() {
        println!("No suspicious lifecycle scripts detected.");
        return;
    }

    format::print_security_report(&warnings);
}
