use dustfril_core::api;

use crate::{cli::DependencyArgs, format, shared::path::resolve_path};

/// Builds and prints the structured Core dependency inventory or baseline diff.
pub fn execute(args: &DependencyArgs) -> bool {
    let path = match resolve_path(&args.path_args.path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to resolve path: {error}");
            return false;
        }
    };

    if !validate_dependency_path(&path) {
        return false;
    }

    if args.compare || args.accept_baseline {
        return compare(&path, args);
    }

    match api::dependency_report(&path, &args.ecosystems()) {
        Ok(reports) => format::print_dependency_reports(&reports),
        Err(error) => {
            eprintln!("Dependency report failed: {error}");
            return false;
        }
    }

    true
}

fn validate_dependency_path(path: &std::path::Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => true,
        Ok(_) => {
            eprintln!("Path is not a directory: {}", path.display());
            false
        }
        Err(error) => {
            eprintln!("Cannot access path {}: {error}", path.display());
            false
        }
    }
}

fn compare(path: &std::path::Path, args: &DependencyArgs) -> bool {
    let baseline_path = match api::dependency_baseline_path() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to determine dependency baseline path: {error}");
            return false;
        }
    };
    let reports = match api::dependency_report(path, &args.ecosystems()) {
        Ok(reports) => reports,
        Err(error) => {
            eprintln!("Dependency report failed: {error}");
            return false;
        }
    };
    let diff = match api::dependency_diff(path, &reports, &baseline_path) {
        Ok(diff) => diff,
        Err(error) => {
            eprintln!("Dependency comparison failed: {error}");
            return false;
        }
    };
    format::print_dependency_diff(&diff);

    if args.accept_baseline {
        if let Err(error) = api::accept_dependency_baseline(path, &reports, &baseline_path) {
            eprintln!("Failed to accept dependency baseline: {error}");
            return false;
        }
        println!("Baseline accepted.");
    }

    true
}
