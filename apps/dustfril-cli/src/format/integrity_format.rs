use dustfril_core::models::{IntegrityReport, IntegrityStatus};

/// Prints non-executing executable-integrity facts and comparison states.
pub fn print_integrity_report(report: &IntegrityReport) {
    println!("Executable integrity scan\n");

    for check in &report.checks {
        println!("{}: {}", check.requested_tool, check.status);

        if let Some(observation) = &check.observation {
            println!("  Path:           {}", observation.resolved_path.display());
            println!("  Canonical:      {}", observation.canonical_path.display());
            println!("  Size:           {} bytes", observation.size_bytes);
            println!("  SHA-256:        {}", observation.sha256);
            println!("  Observed:       {}", observation.observed_at);
            if let Some(target) = &observation.symlink_target {
                println!("  Symlink target:  {}", target.display());
            }
        }

        if let Some(previous) = &check.previous_observation
            && matches!(
                check.status,
                IntegrityStatus::ContentChanged | IntegrityStatus::ResolvedPathChanged
            )
        {
            println!("  Previous SHA-256: {}", previous.sha256);
        }

        if let Some(failure) = &check.failure {
            println!("  Reason:         {} ({})", failure.kind, failure.message);
        }

        if matches!(
            check.status,
            IntegrityStatus::ContentChanged | IntegrityStatus::ResolvedPathChanged
        ) {
            println!("  Note:           A changed executable is not proof of malware.");
        }

        println!();
    }
}
