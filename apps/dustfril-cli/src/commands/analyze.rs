use std::path::Path;

use dustfril_core::{analyzer, detector};

use dustfril_core::models::{AnalysisResult, CleanupRecommendation};

fn print_summary(analysis: &AnalysisResult) {
    let mut keep = 0;
    let mut review = 0;
    let mut safe_to_clean = 0;
    let mut review_size = 0_u64;
    let mut safe_size = 0_u64;

    for artifact in &analysis.artifacts {
        match artifact.recommendation {
            CleanupRecommendation::Keep => {
                keep += 1;
            }

            CleanupRecommendation::Review => {
                review += 1;
                review_size += artifact.size_bytes;
            }

            CleanupRecommendation::SafeToClean => {
                safe_to_clean += 1;
                safe_size += artifact.size_bytes;
            }
        }
    }

    println!("----------------------------------------\n");

    println!("DustFril Analysis Summary\n");

    println!("Artifacts: {}", analysis.artifacts.len());

    println!(
        "Total Size: {}\n",
        analyzer::format_size(analysis.total_size_bytes)
    );

    println!("Keep: {}", keep);
    println!(
        "Review: {}, Size: {}",
        review,
        analyzer::format_size(review_size)
    );
    println!(
        "Safe To Clean: {}, Size: {}",
        safe_to_clean,
        analyzer::format_size(safe_size)
    );

    println!("\n----------------------------------------\n");
}

pub fn execute() {
    let scan_result = detector::scan(Path::new("."));

    let analysis_result = analyzer::analyze(scan_result);

    if analysis_result.artifacts.is_empty() {
        println!("No Rust artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", analysis_result.artifacts.len());

    for artifact in &analysis_result.artifacts {
        println!("[{}]", artifact.artifact.artifact_type);

        println!("  Path: {}", artifact.artifact.path.display());

        println!("  Size: {}", analyzer::format_size(artifact.size_bytes));

        println!(
            "  Modified: {}",
            analyzer::format_modified(artifact.last_modified)
        );

        let age_display = artifact
            .age_days
            .map(|d| format!("{d} days"))
            .unwrap_or_else(|| "Unknown".to_string());

        println!("  Age: {}", age_display);

        println!("  Recommendation: {}\n", artifact.recommendation);
    }

    print_summary(&analysis_result);
}
