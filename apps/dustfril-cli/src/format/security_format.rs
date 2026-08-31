use dustfril_core::models::SecurityWarning;

pub fn print_security_report(warnings: &[SecurityWarning]) {
    println!("Suspicious lifecycle scripts detected\n");
    println!("Found {} warning(s)\n", warnings.len());

    for warning in warnings {
        println!("----------------------------------------");
        println!("Package:      {}", warning.package);
        println!("Script Type:  {}", warning.script_type);
        println!("Risk Level:   {}", warning.risk_level);
        println!("Command:      {}", warning.command);
        println!("Reason:       {}", warning.reason);
    }

    println!("\n----------------------------------------");
}
