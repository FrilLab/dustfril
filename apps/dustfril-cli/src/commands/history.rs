use dustfril_core::api;

/// Loads and prints the same unified activity records used by the desktop app.
pub fn execute() -> bool {
    let records = match api::history::load_all() {
        Ok(records) => records,
        Err(error) => {
            eprintln!("Failed to load activity history: {error}");
            return false;
        }
    };

    match serde_json::to_string_pretty(&records) {
        Ok(json) => {
            println!("{json}");
            true
        }
        Err(error) => {
            eprintln!("Failed to format activity history: {error}");
            false
        }
    }
}
