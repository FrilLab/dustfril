use std::io::{self, Write};

use dustfril_core::{
    api,
    error::DustError,
    models::{CleanupPlan, CleanupResult, DeleteMode},
};

use crate::{
    cli::CleanArgs,
    format, history,
    shared::path::{resolve_path, validate_path},
};

pub fn dry_run(args: &CleanArgs) {
    let plan = match build_cleanup_plan(args) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("Cleanup preview failed: {}", e);
            return;
        }
    };

    if plan.candidates.is_empty() {
        println!("No cleanup candidates found.");
        return;
    }

    print_cleanup_plan(&plan);
    println!("No files were deleted.");
}

pub fn execute(args: &CleanArgs) {
    let plan = match build_cleanup_plan(args) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("Cleanup preparation failed: {}", e);
            return;
        }
    };

    if plan.candidates.is_empty() {
        println!("No cleanup candidates found.");
        return;
    }

    print_cleanup_plan(&plan);

    if !confirm_cleanup() {
        println!("Cleanup cancelled.");
        return;
    }

    let mode = if args.permanent {
        DeleteMode::Permanent
    } else {
        DeleteMode::default()
    };

    let result = match api::clean::execute(&plan, mode) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Cleanup failed: {}", e);
            return;
        }
    };
    history::record(mode, &result)
        .unwrap_or_else(|e| eprintln!("Failed to record cleanup history: {}", e));

    print_cleanup_result(&result);
}

fn build_cleanup_plan(args: &CleanArgs) -> Result<CleanupPlan, DustError> {
    let path = resolve_path(&args.path_args.path);

    if !validate_path(&path) {
        return Err(DustError::InvalidPath(path));
    }

    let ecosystems = args.ecosystems();

    let scan = api::scan(&path, &ecosystems)?;
    let total_size_bytes = api::analyze(scan.clone())?.total_size_bytes;
    api::history::record_scan(&path, &scan, total_size_bytes)?;
    let plan = api::clean::build_plan(scan)?;

    Ok(plan)
}

fn confirm_cleanup() -> bool {
    print!("Continue? (y/N): ");

    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    matches!(input.trim(), "y" | "Y")
}

fn print_cleanup_plan(plan: &CleanupPlan) {
    println!("Cleanup Preview\n");

    for candidate in &plan.candidates {
        println!("[{}]", candidate.ecosystem);
        println!("  Path: {}", candidate.path.display());
        println!("  Size: {}", format::format_size(candidate.size_bytes));

        if let Some(age_days) = candidate.age_days {
            println!("  Age: {} day(s)", age_days);
        }

        println!();
    }

    println!("Total Reclaimable Space");
    println!("  {}\n", format::format_size(plan.reclaimable_size_bytes()));
}

fn print_cleanup_result(result: &CleanupResult) {
    println!("Cleanup completed.");
    println!("Deleted: {}", result.deleted_paths.len());
    println!("Failed: {}", result.failed_paths.len());
    println!("Freed: {}", format::format_size(result.freed_size_bytes));

    if !result.deleted_paths.is_empty() {
        println!("\nDeleted");

        for path in &result.deleted_paths {
            println!("  {}", path.display());
        }
    }

    if !result.failed_paths.is_empty() {
        println!("\nFailed");

        for failure in &result.failed_paths {
            println!("  {} ({})", failure.path.display(), failure.reason);
        }
    }
}
