use dustfril_core::models::SecurityReport;

/// Prints the complete supply-chain security report.
pub fn print_security_scan_report(report: &SecurityReport) {
    println!("Supply-chain security scan\n");
    println!(
        "Inspected {} manifest(s) and {} lockfile(s)",
        report.manifests.len(),
        report.lockfiles.len()
    );

    if report.findings.is_empty() {
        println!("No security findings detected.");
        return;
    }

    println!("Found {} finding(s)\n", report.findings.len());

    for finding in &report.findings {
        println!("----------------------------------------");
        println!("Category:     {}", finding.kind);
        println!("Risk Level:   {}", finding.risk_level);
        println!("Path:         {}", finding.path.display());
        if let Some(package) = &finding.package {
            println!("Package:      {package}");
        }
        if let Some(evidence) = &finding.evidence {
            println!("Evidence:     {evidence}");
        }
        println!("Reason:       {}", finding.reason);
    }

    println!("\n----------------------------------------");
}
