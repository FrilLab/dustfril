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

    let report = match api::security_scan_report(&path, &ecosystems) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Security scan failed: {error}");
            return false;
        }
    };

    format::print_security_scan_report(&report);

    true
}
