// SLO check command implementation

use crate::engines::slo::{SloConfig, SloManager};
use crate::engines::trend::TrendEngine;
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(
    slo_path: Option<PathBuf>,
    snapshots_path: Option<PathBuf>,
    format: &str,
    verbose: bool,
    edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine SLO config path
    let slo_file = slo_path.unwrap_or_else(|| PathBuf::from(".costpilot/slo.json"));

    if !slo_file.exists() {
        eprintln!(
            "{} SLO configuration not found: {}",
            "❌".red(),
            slo_file.display()
        );
        eprintln!("  Create one with: costpilot init");
        return Err("SLO configuration not found".into());
    }

    // Load SLO config
    let content = std::fs::read_to_string(&slo_file)?;
    let config: SloConfig = serde_json::from_str(&content)?;

    if verbose {
        println!(
            "📂 Loaded {} SLOs from {}",
            config.slos.len(),
            slo_file.display()
        );
    }

    // Load latest snapshot
    let snapshots_dir = snapshots_path.unwrap_or_else(|| PathBuf::from(".costpilot/snapshots"));

    if !snapshots_dir.exists() {
        eprintln!(
            "{} Snapshots directory not found: {}",
            "❌".red(),
            snapshots_dir.display()
        );
        eprintln!("  Run a scan first to generate snapshots");
        return Err("No snapshots available".into());
    }

    // Require Premium for trend tracking
    crate::edition::require_premium(edition, "Trend tracking")?;

    let trend_engine = TrendEngine::new(snapshots_dir.to_str().unwrap(), edition)?;
    let history = trend_engine.load_history()?;

    if history.snapshots.is_empty() {
        eprintln!("{} No snapshots found", "❌".red());
        eprintln!("  Run a scan first: costpilot scan --plan <file>");
        return Err("No snapshots available".into());
    }

    let latest_snapshot = history.snapshots.last().unwrap();

    if verbose {
        println!(
            "📊 Evaluating against snapshot from {}",
            latest_snapshot.timestamp
        );
        println!("   Total cost: ${:.2}", latest_snapshot.total_monthly_cost);
    }

    // Create SLO manager and evaluate
    let slo_manager = SloManager::new(config, edition);
    let mut report = slo_manager.evaluate_snapshot(latest_snapshot);

    // Free edition: convert all violations/warnings to non-blocking validation messages
    if !edition.capabilities.allow_slo_enforce {
        for eval in &mut report.evaluations {
            if eval.status == crate::engines::slo::slo_types::SloStatus::Violation {
                eval.status = crate::engines::slo::slo_types::SloStatus::Warning;
            }
        }
        // Update summary counts
        report.summary.violation_count = 0;
        report.summary.warning_count = report
            .evaluations
            .iter()
            .filter(|e| e.status == crate::engines::slo::slo_types::SloStatus::Warning)
            .count();
    }

    // Output report
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            // Text format
            println!("\n{}", "SLO Compliance Report".bold().underline());
            println!("Evaluated: {}\n", latest_snapshot.timestamp);

            // Summary
            println!("{}", "Summary:".bold());
            println!("  Total SLOs: {}", report.summary.total_slos);
            println!("  {} Passed: {}", "✅".green(), report.summary.pass_count);
            println!(
                "  {} Warnings: {}",
                "⚠️".yellow(),
                report.summary.warning_count
            );
            println!(
                "  {} Violations: {}",
                "❌".red(),
                report.summary.violation_count
            );
            println!(
                "  {} No Data: {}",
                "❓".bright_black(),
                report.summary.no_data_count
            );
            println!();

            // Show violations first
            let violations: Vec<_> = report
                .evaluations
                .iter()
                .filter(|e| e.status == crate::engines::slo::slo_types::SloStatus::Violation)
                .collect();
            if !violations.is_empty() {
                println!("{}", "Violations:".red().bold());
                for eval in &violations {
                    println!("  {} {}", "❌".red(), eval.slo_name.bold());
                    println!("     {}", eval.message);
                    println!(
                        "     Actual: ${:.2} | Threshold: ${:.2} ({:.1}%)",
                        eval.actual_value, eval.threshold_value, eval.threshold_usage_percent
                    );
                    if !eval.affected.is_empty() {
                        println!("     Affected: {}", eval.affected.join(", "));
                    }
                    println!();
                }
            }

            // Show warnings
            let warnings: Vec<_> = report
                .evaluations
                .iter()
                .filter(|e| e.status == crate::engines::slo::slo_types::SloStatus::Warning)
                .collect();
            if !warnings.is_empty() && verbose {
                println!("{}", "Warnings:".yellow().bold());
                for eval in &warnings {
                    println!("  {} {}", "⚠️".yellow(), eval.slo_name.bold());
                    println!("     {}", eval.message);
                    println!(
                        "     Actual: ${:.2} | Threshold: ${:.2} ({:.1}%)",
                        eval.actual_value, eval.threshold_value, eval.threshold_usage_percent
                    );
                    println!();
                }
            }

            // Show passes if verbose (filter evaluations by status)
            let passes: Vec<_> = report
                .evaluations
                .iter()
                .filter(|e| e.status == crate::engines::slo::slo_types::SloStatus::Pass)
                .collect();
            if !passes.is_empty() && verbose {
                println!("{}", "Passing:".green().bold());
                for eval in &passes {
                    println!(
                        "  {} {} ({:.1}%)",
                        "✅".green(),
                        eval.slo_name,
                        eval.threshold_usage_percent
                    );
                }
                println!();
            }
        }
    }

    // Check if deployment should be blocked
    if slo_manager.should_block_deployment(&report) {
        eprintln!(
            "\n{} Deployment blocked due to SLO violations",
            "🛑".red().bold()
        );
        let blocking = slo_manager.get_blocking_violations(&report);
        eprintln!("  {} blocking violation(s) detected", blocking.len());
        std::process::exit(1);
    }

    if report.summary.violation_count > 0 {
        eprintln!(
            "\n{} SLO violations detected (non-blocking)",
            "⚠️".yellow().bold()
        );
        std::process::exit(1);
    }

    println!("\n{} All SLOs passed", "✅".green().bold());
    Ok(())
}
