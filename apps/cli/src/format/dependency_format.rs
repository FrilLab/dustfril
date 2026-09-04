use dustfril_core::models::{
    DependencyChange, DependencyDiff, DependencyEntry, DependencyMetric, DependencyReport,
    DuplicateDependency,
};

/// Prints dependency reports without adding CLI-specific analysis semantics.
pub fn print_dependency_reports(reports: &[DependencyReport]) {
    println!("Dependency exposure report\n");

    if reports.is_empty() {
        println!("No dependency reports available.");
        return;
    }

    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            println!();
        }
        print_dependency_report(report);
    }
}

/// Prints the structured logical diff from an explicit dependency baseline.
pub fn print_dependency_diff(diff: &DependencyDiff) {
    println!("Dependency change report\n");
    println!("Workspace:    {}", diff.workspace_id);
    println!("Baseline:     {}", diff.baseline_status);
    print_changes("Added", &diff.added);
    print_changes("Removed", &diff.removed);
    print_changes("Version changed", &diff.version_changes);
    print_changes("Source changed", &diff.source_changes);
    for warning in &diff.warnings {
        println!("Warning:       {warning}");
    }
    if !diff.has_changes() {
        println!("No dependency changes detected.");
    }
}

fn print_changes(label: &str, changes: &[DependencyChange]) {
    if changes.is_empty() {
        return;
    }
    println!("{label}: {}", changes.len());
    for change in changes {
        let entry = change.current.as_ref().or(change.previous.as_ref());
        if let Some(entry) = entry {
            println!(
                "  {} {} {} ({})",
                entry.ecosystem, entry.name, entry.version, entry.scope
            );
        }
        if let (Some(previous), Some(current)) = (&change.previous, &change.current) {
            println!(
                "    {} -> {}",
                dependency_label(previous),
                dependency_label(current)
            );
        }
    }
}

fn dependency_label(entry: &DependencyEntry) -> String {
    match &entry.source {
        Some(source) => format!("{} {} [{}]", entry.name, entry.version, source),
        None => format!("{} {}", entry.name, entry.version),
    }
}

fn print_dependency_report(report: &DependencyReport) {
    println!("Ecosystem:    {}", report.ecosystem);
    println!("Status:       {}", report.status);
    println!("Manifest:     {}", report.manifest.display());
    if let Some(format) = &report.manifest_format {
        println!("Manifest format: {format}");
    }
    if let Some(lockfile) = &report.lockfile {
        println!(
            "Lockfile:     {} ({})",
            lockfile.path.display(),
            lockfile.status
        );
        if let Some(format) = &lockfile.format {
            println!("Lockfile format: {format}");
        }
    }

    println!("Direct dependencies: {}", report.direct_dependency_total);
    for (category, count) in &report.direct_dependency_counts {
        println!("  {category}: {count}");
    }
    print_metric("Resolved dependencies", &report.resolved_dependency_count);
    print_metric(
        "Transitive dependencies",
        &report.transitive_dependency_count,
    );

    if report.duplicate_versions.is_empty() {
        println!("Duplicate versions: none");
    } else {
        println!("Duplicate versions:");
        for duplicate in &report.duplicate_versions {
            print_duplicate(duplicate);
        }
    }

    for warning in &report.warnings {
        println!("Note:          {warning}");
    }
}

fn print_metric(label: &str, metric: &DependencyMetric) {
    match metric.value {
        Some(value) => println!("{label}: {value}"),
        None => println!(
            "{label}: {} ({})",
            metric.status,
            metric.reason.as_deref().unwrap_or("no value")
        ),
    }
}

fn print_duplicate(duplicate: &DuplicateDependency) {
    println!("  {}: {}", duplicate.name, duplicate.versions.join(", "));
}
