use std::path::Path;

use dustfril_core::{
    analyzer, cleaner, detector,
    models::{CleanupPlan, CleanupResult},
};

// dry-run
pub fn dry_run() {
    let plan = build_cleanup_plan();

    if plan.candidates.is_empty() {
        println!("No cleanup candidates found.");

        return;
    }

    print_cleanup_plan(&plan);

    println!("No files were deleted.");
}

use std::io::{self, Write};

fn build_cleanup_plan() -> CleanupPlan {
    let scan_result = detector::scan(Path::new("."));

    let analysis = analyzer::analyze(scan_result);

    cleaner::create_cleanup_plan(analysis)
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

        println!("  Size: {}\n", analyzer::format_size(candidate.size_bytes));
    }

    println!("Total Reclaimable Space\n");

    println!("  {}\n", analyzer::format_size(plan.reclaimable_size_bytes));
}

fn print_cleanup_result(result: &CleanupResult) {
    println!("Cleanup completed.");

    println!("Deleted: {}", result.deleted_paths.len());

    println!("Failed: {}", result.failed_paths.len());

    println!("Freed: {}", analyzer::format_size(result.freed_size_bytes,));

    if !result.deleted_paths.is_empty() {
        println!("Deleted\n");

        for path in &result.deleted_paths {
            println!("  {}", path.display());
        }

        println!();
    }
}

pub fn execute() {
    let plan = build_cleanup_plan();

    if plan.candidates.is_empty() {
        println!("No cleanup candidates found.");
        return;
    }

    print_cleanup_plan(&plan);

    if !confirm_cleanup() {
        println!("Cleanup cancelled.");
        return;
    }

    let result = cleaner::execute_cleanup(&plan);

    print_cleanup_result(&result);
}
