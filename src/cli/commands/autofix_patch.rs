// Autofix patch command implementation - Generate full unified diff patches

use clap::Args;
use std::path::PathBuf;
use colored::Colorize;
use crate::engines::detection::DetectionEngine;
use crate::engines::prediction::PredictionEngine;
use crate::engines::autofix::{AutofixEngine, AutofixMode};

#[derive(Debug, Args)]
pub struct AutofixPatchArgs {
    /// Path to Terraform plan JSON file
    #[arg(long, value_name = "FILE")]
    plan: PathBuf,

    /// Output file for patches (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Apply patches (simulation mode)
    #[arg(long)]
    apply: bool,

    /// Show detailed patch metadata
    #[arg(short, long)]
    verbose: bool,
}

pub fn execute(args: &AutofixPatchArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🔧 CostPilot Autofix - Patch Mode (Beta)".bold().cyan());
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
    let prediction_engine = PredictionEngine::new()?;
    let mut detections_with_estimates = detections;
    
    for detection in &mut detections_with_estimates {
        if let Some(change) = changes.iter().find(|c| c.resource_id == detection.resource_id) {
            if let Ok(estimate) = prediction_engine.predict(change) {
                detection.estimated_cost = Some(estimate);
            }
        }
    }
    println!("   Estimated {} resources", detections_with_estimates.len());
    println!();

    // Generate patches
    println!("{}", "Generating patches...".dimmed());
    let autofix_result = AutofixEngine::generate_fixes(
        &detections_with_estimates,
        &changes,
        AutofixMode::Patch,
    );

    if autofix_result.patches.is_empty() {
        println!("   {} No patches available", "ℹ".bright_blue());
        if !autofix_result.warnings.is_empty() {
            println!();
            println!("{}", "Warnings:".yellow());
            for warning in &autofix_result.warnings {
                println!("   • {}", warning);
            }
        }
        return Ok(());
    }

    println!("   Generated {} patches", autofix_result.patches.len());
    println!();

    // Display patches
    let mut output_buffer = String::new();
    
    for (idx, patch) in autofix_result.patches.iter().enumerate() {
        let header = format!("Patch #{} - {}", idx + 1, patch.resource_id);
        output_buffer.push_str(&format!("{}\n", header.bold().green()));
        output_buffer.push_str(&format!("{}\n", "=".repeat(header.len())));
        
        if args.verbose {
            output_buffer.push_str(&format!("Resource Type: {}\n", patch.resource_type));
            output_buffer.push_str(&format!("File: {}\n", patch.filename));
            output_buffer.push_str(&format!("Monthly Savings: ${:.2}\n", patch.metadata.monthly_savings));
            output_buffer.push_str(&format!("Confidence: {:.0}%\n", patch.metadata.confidence * 100.0));
            output_buffer.push_str(&format!("Anti-Patterns: {}\n", patch.metadata.anti_patterns.join(", ")));
            output_buffer.push_str(&format!("\nRationale:\n{}\n", patch.metadata.rationale));
            output_buffer.push_str("\n");
        }
        
        output_buffer.push_str(&patch.to_unified_diff());
        output_buffer.push_str("\n");
    }

    // Show summary
    let total_savings: f64 = autofix_result.patches.iter()
        .map(|p| p.metadata.monthly_savings)
        .sum();
    
    output_buffer.push_str(&format!("{}\n", "Summary".bold()));
    output_buffer.push_str(&format!("Total patches: {}\n", autofix_result.patches.len()));
    output_buffer.push_str(&format!("Total monthly savings: ${:.2}\n", total_savings));
    output_buffer.push_str(&format!("Annual savings: ${:.2}\n", total_savings * 12.0));
    
    if autofix_result.patches.iter().any(|p| p.metadata.beta) {
        output_buffer.push_str(&format!("\n{}\n", "⚠️  Beta Feature".yellow()));
        output_buffer.push_str("These patches are in Beta. Always review and test before applying.\n");
    }

    // Write output
    if let Some(output_file) = &args.output {
        std::fs::write(output_file, &output_buffer)?;
        println!("{} Patches written to {}", "✓".green(), output_file.display());
    } else {
        println!("{}", output_buffer);
    }

    // Apply warning
    if args.apply {
        println!();
        println!("{}", "⚠️  Patch application is not yet implemented".yellow());
        println!("Use --output to save patches and apply manually after review.");
    }

    // Warnings
    if !autofix_result.warnings.is_empty() {
        println!();
        println!("{}", "Warnings:".yellow());
        for warning in &autofix_result.warnings {
            println!("   • {}", warning);
        }
    }

    Ok(())
}
