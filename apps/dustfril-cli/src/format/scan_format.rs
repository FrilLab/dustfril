use dustfril_core::models::ScanAccessSummary;

/// Prints the bounded access scope collected during an explicit artifact scan.
pub fn print_scan_access_summary(summary: &ScanAccessSummary) {
    println!("\nScan Access Summary");
    println!("  Workspace:                    {}", summary.root.display());
    println!(
        "  Directories visited:          {}",
        summary.directories_visited
    );
    println!(
        "  Files inspected:              {}",
        summary.files_inspected
    );
    println!(
        "  Manifest/metadata files:      {}",
        summary.metadata_files_inspected
    );
    println!(
        "  Artifact candidates:          {}",
        summary.artifact_candidates
    );
    println!(
        "  Symlinks skipped:             {}",
        summary.symlinks_skipped
    );
    println!("  Traversal/read failures:      {}", summary.failures);

    if !summary.failure_samples.is_empty() {
        println!("  Representative failures:");
        for failure in &summary.failure_samples {
            println!("    {}: {}", failure.path.display(), failure.reason);
        }
    }
}
