use std::path::Path;

use dustfril_core::detector;

pub fn execute() {
    let result = detector::scan(Path::new("."));

    if result.artifacts.is_empty() {
        println!("No Rust artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", result.artifacts.len());

    for artifact in result.artifacts {
        println!("[{}]", artifact.artifact_type);

        println!("  {}\n", artifact.path.display());
    }
}
