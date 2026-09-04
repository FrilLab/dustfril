use dustfril_core::api;

use crate::{cli::IntegrityScanArgs, format};

pub fn scan(args: &IntegrityScanArgs) -> bool {
    let state_path = match api::integrity::state_path() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to determine executable-integrity state path: {error}");
            return false;
        }
    };

    let tools = args.tools();
    let report = match api::integrity::scan(&tools, &state_path) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Executable integrity scan failed: {error}");
            return false;
        }
    };

    format::print_integrity_report(&report);
    true
}
