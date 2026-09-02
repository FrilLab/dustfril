use dustfril_core::models::{ArtifactSizeChange, ArtifactSnapshotResult};

/// Prints factual generated-artifact size changes without content/security interpretations.
pub fn print_artifact_snapshot_result(result: &ArtifactSnapshotResult) {
    println!("Artifact size snapshot\n");
    println!("Workspace: {}", result.snapshot.workspace_id);
    println!("Status:    {}", result.status);
    println!("Artifacts: {}", result.snapshot.artifacts.len());

    if result.status == dustfril_core::models::ArtifactSnapshotStatus::BaselineCreated {
        println!("\nBaseline created; no growth changes reported.");
        return;
    }

    if result.changes.is_empty() {
        println!("\nNo artifact changes detected.");
        return;
    }

    println!("\nChanges: {}", result.changes.len());
    for change in &result.changes {
        print_change(change);
    }
}

fn print_change(change: &ArtifactSizeChange) {
    println!("  [{}] {}", change.ecosystem, change.path.display());
    println!("    State:    {}", change.kind);
    println!(
        "    Previous: {}",
        change
            .previous_size_bytes
            .map(format_size)
            .unwrap_or_else(|| "None".to_owned())
    );
    println!(
        "    Current:  {}",
        change
            .current_size_bytes
            .map(format_size)
            .unwrap_or_else(|| "None".to_owned())
    );
    println!("    Change:   {}", format_delta(change.delta_bytes));
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        "0 B".to_owned()
    } else {
        format!("{bytes} bytes")
    }
}

fn format_delta(delta: i128) -> String {
    if delta >= 0 {
        format!("+{} bytes", delta)
    } else {
        format!("{} bytes", delta)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Utc;
    use dustfril_core::models::{
        ArtifactChangeKind, ArtifactSizeChange, ArtifactSnapshot, ArtifactSnapshotResult,
        ArtifactSnapshotStatus,
    };

    use super::*;

    #[test]
    fn formatting_helpers_preserve_exact_byte_values() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1024 bytes");
        assert_eq!(format_delta(42), "+42 bytes");
        assert_eq!(format_delta(-42), "-42 bytes");
    }

    #[test]
    fn snapshot_result_can_be_rendered_with_optional_sizes() {
        let result = ArtifactSnapshotResult {
            status: ArtifactSnapshotStatus::Compared,
            snapshot: ArtifactSnapshot {
                workspace_id: "/workspace".to_owned(),
                timestamp: Utc::now(),
                artifacts: Vec::new(),
            },
            previous_snapshot: None,
            changes: vec![ArtifactSizeChange {
                path: PathBuf::from("target"),
                ecosystem: dustfril_core::models::Ecosystem::Rust,
                kind: ArtifactChangeKind::Removed,
                previous_size_bytes: Some(42),
                current_size_bytes: None,
                delta_bytes: -42,
            }],
        };

        print_artifact_snapshot_result(&result);
    }
}
