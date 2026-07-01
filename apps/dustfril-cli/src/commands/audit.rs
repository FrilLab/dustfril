use dustfril_core::api;

use crate::{
    cli::PathArgs,
    shared::path::{resolve_path, validate_path},
};

pub fn execute(args: &PathArgs) {
    let path = resolve_path(&args.path);

    if !validate_path(&path) {
        return;
    }

    let ecosystems = args.ecosystems();

    let scripts = match api::audit(&path, &ecosystems) {
        Ok(scripts) => scripts,
        Err(error) => {
            eprintln!("Audit failed: {error}");
            return;
        }
    };

    if scripts.is_empty() {
        println!("No lifecycle scripts found.");
        return;
    }

    // TODO: Replace this temporary plain-text output with a dedicated audit formatter.
    println!("Found {} lifecycle script(s)\n", scripts.len());

    for script in scripts {
        println!(
            "[{}] {} ({})",
            script.risk_level, script.package, script.script_type
        );
        println!("  {}", script.command);
    }
}
