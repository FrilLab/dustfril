use dustfril_core::models::LifecycleScript;

pub fn print_audit_report(scripts: &[LifecycleScript]) {
    println!("Found {} lifecycle script(s)\n", scripts.len());

    for script in scripts {
        println!("----------------------------------------");
        println!("Package:      {}", script.package);
        println!("Manager:      {}", script.package_manager);
        println!("Script Type:  {}", script.script_type);
        println!("Risk Level:   {}", script.risk_level);
        println!("Command:      {}", script.command);
    }

    println!("\n----------------------------------------");
}
