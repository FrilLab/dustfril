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

pub fn dry_run(args: &CleanArgs) -> bool {
    let (_, plan) = match build_cleanup_plan(args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Cleanup preview failed: {}", e);
            return false;
        }
    };

    if plan.candidates.is_empty() {
        println!("No cleanup candidates found.");
        return true;
    }

    print_cleanup_plan(&plan);
    println!("No files were deleted.");

    true
}

pub fn execute(args: &CleanArgs) -> bool {
    let mode = if args.permanent {
        DeleteMode::Permanent
    } else {
        DeleteMode::default()
    };

    let (target_path, plan) = match build_cleanup_plan(args) {
        Ok(result) => result,
        Err(e) => {
            if let Err(history_error) = history::record_cleanup_failure(mode, &e.to_string()) {
                eprintln!("Failed to record cleanup failure history: {history_error}");
            }
            eprintln!("Cleanup preparation failed: {}", e);
            return false;
        }
    };

    if plan.candidates.is_empty() {
        let result = CleanupResult::default();
        history::record_for_workspace(&target_path, &plan, mode, &result)
            .unwrap_or_else(|e| eprintln!("Failed to record cleanup history: {e}"));
        println!("No cleanup candidates found.");
        return true;
    }

    print_cleanup_plan(&plan);

    match confirm_cleanup() {
        Ok(true) => {}
        Ok(false) => {
            println!("Cleanup cancelled.");
            return true;
        }
        Err(error) => {
            eprintln!("Could not read cleanup confirmation: {error}");
            return false;
        }
    }

    let result = match api::clean::execute(&plan, mode) {
        Ok(res) => res,
        Err(e) => {
            if let Err(history_error) =
                history::record_failure_for_workspace(&target_path, mode, &e.to_string())
            {
                eprintln!("Failed to record cleanup failure history: {history_error}");
            }
            eprintln!("Cleanup failed: {}", e);
            return false;
        }
    };
    history::record_for_workspace(&target_path, &plan, mode, &result)
        .unwrap_or_else(|e| eprintln!("Failed to record cleanup history: {}", e));

    print_cleanup_result(&result);

    cleanup_succeeded(&result)
}

fn cleanup_succeeded(result: &CleanupResult) -> bool {
    result.failed_paths.is_empty()
}

fn build_cleanup_plan(args: &CleanArgs) -> Result<(std::path::PathBuf, CleanupPlan), DustError> {
    let path = resolve_path(&args.path_args.path)?;

    if !validate_path(&path) {
        return Err(DustError::InvalidPath(path));
    }

    let ecosystems = args.ecosystems();

    let scan = api::scan(&path, &ecosystems)?;
    let analysis = api::analyze(scan)?;
    let plan = api::clean::build_plan_from_analysis(analysis)?;

    Ok((path, plan))
}

fn confirm_cleanup() -> io::Result<bool> {
    print!("Continue? (y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    Ok(matches!(input.trim(), "y" | "Y"))
}

fn print_cleanup_plan(plan: &CleanupPlan) {
    println!("Cleanup Preview\n");

    for candidate in &plan.candidates {
        println!("[{}]", candidate.ecosystem);
        println!("  Project: {}", candidate.project.display_name);
        println!("  Root:    {}", candidate.project.root.display());
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

#[cfg(test)]
mod tests {
    use super::*;
    use dustfril_core::models::{CleanupFailure, CleanupFailureReason};

    #[test]
    fn incomplete_cleanup_is_reported_as_a_command_failure() {
        let result = CleanupResult {
            failed_paths: vec![CleanupFailure {
                path: "target".into(),
                reason: CleanupFailureReason::PermissionDenied,
            }],
            ..CleanupResult::default()
        };

        assert!(!cleanup_succeeded(&result));
    }

    #[test]
    fn complete_cleanup_is_reported_as_a_command_success() {
        assert!(cleanup_succeeded(&CleanupResult::default()));
    }
}
