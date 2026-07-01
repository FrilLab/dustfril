use dustfril_core::api;
use dustfril_core::models::{AnalysisResult, CleanupRecommendation};

use crate::cli::PathArgs;
use crate::format;
use crate::shared::path::{resolve_path, validate_path};

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

            CleanupRecommendation::NeedsReview => {
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
        format::format_size(analysis.total_size_bytes)
    );

    println!("Keep: {}", keep);
    println!(
        "Review: {}, Size: {}",
        review,
        format::format_size(review_size)
    );
    println!(
        "Safe To Clean: {}, Size: {}",
        safe_to_clean,
        format::format_size(safe_size)
    );

    println!("\n----------------------------------------\n");
}

pub fn execute(args: PathArgs) {
    let path = resolve_path(&args.path);

    if !validate_path(&path) {
        eprintln!("Invalid path");
        return;
    }

    let ecosystems = args.ecosystems();

    let scan_result = match api::scan(&path, &ecosystems) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return;
        }
    };

    let analysis_result = match api::analyze(scan_result) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Analysis failed: {}", e);
            return;
        }
    };

    if analysis_result.artifacts.is_empty() {
        println!("No artifacts found.");
        return;
    }

    println!("Found {} artifact(s)\n", analysis_result.artifacts.len());

    for artifact in &analysis_result.artifacts {
        let age_display = artifact
            .age_days
            .map(|d| format!("{d} days"))
            .unwrap_or_else(|| "Unknown".to_string());
        println!("----------------------------------------");
        println!("[{}]", artifact.artifact.ecosystem);
        println!("  Path:           {}", artifact.artifact.path.display());
        println!(
            "  Size:           {}",
            format::format_size(artifact.size_bytes)
        );
        println!(
            "  Modified:       {}",
            format::format_modified(artifact.last_modified)
        );
        println!("  Age:            {}", age_display);
        println!("  Recommendation: {}", artifact.recommendation);
    }

    print_summary(&analysis_result);
}
