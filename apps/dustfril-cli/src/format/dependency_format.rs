use dustfril_core::models::{DependencyMetric, DependencyReport, DuplicateDependency};

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
