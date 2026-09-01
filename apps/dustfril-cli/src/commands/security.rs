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
        Ok(report) => {
            if let Err(error) = api::history::record_security_scan(&path, &ecosystems, &report) {
                eprintln!("Failed to record security scan history: {error}");
            }

            report
        }
        Err(error) => {
            if let Err(history_error) =
                api::history::record_security_failure(&path, &ecosystems, &error.to_string())
            {
                eprintln!("Failed to record security scan history: {history_error}");
            }
            eprintln!("Security scan failed: {error}");
            return false;
        }
    };

    format::print_security_scan_report(&report);

    true
}
