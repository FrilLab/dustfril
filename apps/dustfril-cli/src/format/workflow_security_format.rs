use dustfril_core::models::WorkflowScanReport;

/// Prints the structured result of the local GitHub Actions workflow scan.
pub fn print_workflow_security_scan_report(report: &WorkflowScanReport) {
    println!("GitHub Actions workflow security scan\n");
    println!("Workflows inspected: {}", report.workflows.len());

    if report.is_partial() {
        println!("Analysis status: Partial");
    } else {
        println!("Analysis status: Complete");
    }

    if report.findings.is_empty() {
        println!("\nNo workflow security findings detected.");
    } else {
        println!("\nFound {} finding(s)\n", report.findings.len());
        for finding in &report.findings {
            println!("Rule:         {}", finding.rule_id);
            println!("Category:     {}", finding.category);
            println!("Risk Level:   {}", finding.risk_level);
            println!("Workflow:     {}", finding.workflow_path.display());
            if let Some(job_id) = &finding.job_id {
                println!("Job:          {job_id}");
            }
            if let Some(step_index) = finding.step_index {
                println!("Step:         {step_index}");
            }
            if let Some(step_name) = &finding.step_name {
                println!("Step Name:    {step_name}");
            }
            if let Some(secret_reference) = &finding.secret_reference {
                println!("Secret:       {secret_reference}");
            }
            if let Some(exposure_sink) = &finding.exposure_sink {
                println!("Sink:         {exposure_sink}");
            }
            if let Some(evidence) = &finding.evidence {
                println!("Evidence:     {evidence}");
            }
            println!("Reason:       {}\n", finding.reason);
        }
    }

    if !report.notices.is_empty() {
        println!("Partial-analysis notices\n");
        for notice in &report.notices {
            print!("Workflow:     {}", notice.workflow_path.display());
            if let Some(job_id) = &notice.job_id {
                print!(" (job: {job_id})");
            }
            println!("\nReason:       {}\n", notice.reason);
        }
    }
}
