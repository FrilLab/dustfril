use dustfril_core::{
    api,
    error::DustError,
    models::{CleanupPlan, CleanupResult},
};

use crate::{cli::CleanArgs, format, shared::path::resolve_path};

pub fn dry_run(args: &CleanArgs) {
    let plan = match build_cleanup_plan(args) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
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

use std::io::{self, Write};

fn build_cleanup_plan(args: &CleanArgs) -> Result<CleanupPlan, DustError> {
    let path = resolve_path(&args.path_args.path);

    let scan = api::scan(&path, args.path_args.global)?;
    let plan = api::clean::build_plan(scan)?;

    Ok(plan)
}
fn confirm_cleanup() -> bool {
    print!("Continue? (y/N): ");

    // Flush stdout to ensure the prompt is displayed before reading input
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
        println!("[{}]", candidate.artifact_type);

        println!("  Path: {}", candidate.path.display());

        println!("  Size: {}\n", format::format_size(candidate.size_bytes));
    }

    println!("Total Reclaimable Space\n");

    println!("  {}\n", format::format_size(plan.reclaimable_size_bytes()));
}

fn print_cleanup_result(result: &CleanupResult) {
    println!("Cleanup completed.");

    println!("Deleted: {}", result.deleted_paths.len());

    println!("Failed: {}", result.failed_paths.len());

    println!("Freed: {}", format::format_size(result.freed_size_bytes));

    if !result.deleted_paths.is_empty() {
        println!("Deleted\n");

        for path in &result.deleted_paths {
            println!("  {}", path.display());
        }

        println!();
    }
}
pub fn execute(args: &CleanArgs) {
    let plan = match build_cleanup_plan(args) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
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

    let result = match api::clean::execute(&plan) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Cleanup failed: {}", e);
            return;
        }
    };

    print_cleanup_result(&result);
}
