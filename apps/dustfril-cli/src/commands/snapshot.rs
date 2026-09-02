use dustfril_core::api;

use crate::{
    cli::PathArgs,
    format,
    shared::path::{resolve_path, validate_path},
};

/// Creates one explicit artifact-size snapshot from a scan/analyze workflow.
pub fn execute(args: PathArgs) -> bool {
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

    let scan = match api::scan(&path, &args.ecosystems()) {
        Ok(scan) => scan,
        Err(error) => {
            eprintln!("Snapshot scan failed: {error}");
            return false;
        }
    };
    let analysis = match api::analyze(scan) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("Snapshot analysis failed: {error}");
            return false;
        }
    };

    let result = match api::artifact_snapshot::record_artifact_snapshot(&path, &analysis) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Failed to persist artifact snapshot: {error}");
            return false;
        }
    };

    format::print_artifact_snapshot_result(&result);
    true
}
