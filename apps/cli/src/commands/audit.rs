use dustfril_core::api;

use crate::{
    cli::PathArgs,
    format,
    shared::path::{resolve_path, validate_path},
};

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

    let ecosystems = args.ecosystems();

    let scripts = match api::audit(&path, &ecosystems) {
        Ok(scripts) => scripts,
        Err(error) => {
            eprintln!("Audit failed: {error}");
            return false;
        }
    };

    if scripts.is_empty() {
        println!("No lifecycle scripts found.");
        return true;
    }

    format::print_audit_report(&scripts);

    true
}
