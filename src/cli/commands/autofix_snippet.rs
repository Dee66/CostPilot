// Autofix snippet command implementation - Generate fix snippets (MVP)

use crate::engines::autofix::{AutofixEngine, AutofixMode};
use crate::engines::detection::DetectionEngine;
use crate::engines::prediction::PredictionEngine;
use colored::Colorize;
use std::path::PathBuf;

pub struct AutofixSnippetArgs {
    pub plan: PathBuf,
    pub verbose: bool,
}

pub fn execute(
    args: &AutofixSnippetArgs,
    edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Require Premium for autofix
    crate::edition::require_premium(edition, "Autofix")?;

    println!("{}", "🔧 CostPilot Autofix - Snippet Mode".bold().cyan());
    println!();

    // Load and parse plan
    println!("{}", "Loading Terraform plan...".dimmed());
    let plan_content = std::fs::read_to_string(&args.plan)?;
    let plan: serde_json::Value = serde_json::from_str(&plan_content)?;

    // Extract resource changes
    let changes = crate::cli::utils::extract_resource_changes(&plan)?;
    println!("   Found {} resource changes", changes.len());
    println!();

    // Detect cost regressions
    println!("{}", "Detecting cost regressions...".dimmed());
    let detection_engine = DetectionEngine::new();
    let detections = detection_engine.detect(&changes)?;

    if detections.is_empty() {
        println!("   {} No cost issues detected", "✓".green());
        return Ok(());
    }

    println!("   Found {} cost issues", detections.len());
    println!();

    // Generate predictions
    println!("{}", "Estimating costs...".dimmed());
    let prediction_engine = PredictionEngine::new_with_edition(edition)?;
    let mut detections_with_estimates = detections;

    for detection in &mut detections_with_estimates {
        if let Some(change) = changes
            .iter()
            .find(|c| c.resource_id == detection.resource_id)
        {
            if let Ok(estimate) = prediction_engine.predict_resource_cost(change) {
                detection.estimated_cost = Some(estimate.monthly_cost);
            }
        }
    }
    println!("   Estimated {} resources", detections_with_estimates.len());
    println!();

    // Generate snippets
    println!("{}", "Generating fix snippets...".dimmed());
    let autofix_result = AutofixEngine::generate_fixes(
        &detections_with_estimates,
        &changes,
        &[], // estimates not used for snippet mode
        AutofixMode::Snippet,
        edition,
    )?;

    if autofix_result.fixes.is_empty() {
        println!("   {} No fix snippets available", "ℹ".bright_blue());
        if !autofix_result.warnings.is_empty() {
            println!();
            println!("{}", "Warnings:".yellow());
            for warning in &autofix_result.warnings {
                println!("   • {}", warning);
            }
        }
        return Ok(());
    }

    println!("   Generated {} fix snippets", autofix_result.fixes.len());
    println!();

    // Display snippets
    for (idx, fix) in autofix_result.fixes.iter().enumerate() {
        println!(
            "{}",
            format!("Fix #{} - {}", idx + 1, fix.resource_id)
                .bold()
                .green()
        );
        println!("{}", "─".repeat(60));

        if args.verbose {
            println!("Resource Type: {}", fix.resource_type);
            println!("Format: {:?}", fix.format);
            println!("\nRationale:");
            println!("{}", fix.rationale);
            println!("\nImpact:");
            println!("{}", fix.impact);
            println!();
        }

        println!("{}", "Before:".yellow());
        println!("{}", fix.before_after.before);
        println!();
        println!("{}", "After:".green());
        println!("{}", fix.before_after.after);
        println!();
        println!("{}", "Change:".cyan());
        println!("{}", fix.before_after.change_description);
        println!();

        if args.verbose {
            println!("Properties:");
            println!(
                "  • Deterministic: {}",
                if fix.deterministic { "✓" } else { "✗" }
            );
            println!("  • Idempotent: {}", if fix.idempotent { "✓" } else { "✗" });
            println!();
        }
    }

    // Summary
    println!("{}", "Summary".bold());
    println!("Total fixes: {}", autofix_result.fixes.len());

    if !autofix_result.warnings.is_empty() {
        println!();
        println!("{}", "Warnings:".yellow());
        for warning in &autofix_result.warnings {
            println!("   • {}", warning);
        }
    }

    Ok(())
}
