// SLO burn rate command implementation

use colored::*;
use std::path::{Path, PathBuf};

use crate::engines::slo::burn_rate::{BurnRateCalculator, BurnReport, BurnRisk};
use crate::engines::slo::SloManager;
use crate::engines::trend::SnapshotManager;

/// Execute the SLO burn rate analysis command
pub fn execute(
    slo_path: Option<PathBuf>,
    snapshots_dir: Option<PathBuf>,
    format: &str,
    min_snapshots: Option<usize>,
    min_r_squared: Option<f64>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Default paths
    let slo_path = slo_path.unwrap_or_else(|| PathBuf::from(".costpilot/slo.json"));
    let snapshots_dir = snapshots_dir.unwrap_or_else(|| PathBuf::from(".costpilot/snapshots"));

    if verbose {
        println!("  SLO config: {}", slo_path.display());
        println!("  Snapshots:  {}", snapshots_dir.display());
    }

    // Load SLO configuration
    let slo_manager = SloManager::load_from_file(&slo_path)
        .map_err(|e| format!("Failed to load SLO config: {}", e))?;

    // Validate SLOs
    if let Err(errors) = slo_manager.validate() {
        eprintln!("{}", "❌ Invalid SLO configuration:".bright_red().bold());
        for error in errors {
            eprintln!("  - {}", error);
        }
        return Err("SLO validation failed".into());
    }

    // Load historical snapshots
    let snapshot_manager = SnapshotManager::new(&snapshots_dir);
    let history = snapshot_manager.load_history()
        .map_err(|e| format!("Failed to load snapshots: {}", e))?;

    if history.snapshots.is_empty() {
        eprintln!("{}", "⚠️  No historical snapshots found".yellow().bold());
        eprintln!("  Burn rate analysis requires at least 3 snapshots");
        eprintln!("  Run: costpilot snapshot create --plan <plan.json>");
        return Err("Insufficient data".into());
    }

    if verbose {
        println!("  Loaded {} snapshots", history.snapshots.len());
    }

    // Create calculator with custom thresholds if provided
    let calculator = if let (Some(min_snap), Some(min_r2)) = (min_snapshots, min_r_squared) {
        BurnRateCalculator::with_thresholds(min_snap, min_r2)
    } else if let Some(min_snap) = min_snapshots {
        BurnRateCalculator::with_thresholds(min_snap, 0.7)
    } else if let Some(min_r2) = min_r_squared {
        BurnRateCalculator::with_thresholds(3, min_r2)
    } else {
        BurnRateCalculator::new()
    };

    // Run burn rate analysis
    let report = calculator.analyze_all(&slo_manager.config.slos, &history.snapshots);

    // Output results based on format
    match format {
        "json" => output_json(&report)?,
        "markdown" => output_markdown(&report)?,
        _ => output_text(&report)?,
    }

    // Exit with error code if action required
    if report.requires_action() {
        std::process::exit(1);
    }

    Ok(())
}

/// Output report as formatted text
fn output_text(report: &BurnReport) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("{}", "📊 SLO Burn Rate Analysis".bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_black());
    println!();

    if report.analyses.is_empty() {
        println!("{}", "⚠️  No SLOs analyzed (insufficient data)".yellow());
        return Ok(());
    }

    // Display each SLO analysis
    for analysis in &report.analyses {
        // Risk indicator
        let risk_indicator = match analysis.risk {
            BurnRisk::Low => "✅",
            BurnRisk::Medium => "⚠️ ",
            BurnRisk::High => "🔶",
            BurnRisk::Critical => "🔥",
        };

        // Risk color
        let risk_color = match analysis.risk {
            BurnRisk::Low => "GREEN",
            BurnRisk::Medium => "YELLOW",
            BurnRisk::High => "YELLOW",
            BurnRisk::Critical => "RED",
        };

        println!("{} {} (${:.2}/month)", 
            risk_indicator, 
            analysis.slo_name.bright_white().bold(),
            analysis.slo_limit
        );

        println!("  Burn Rate:      ${:.2}/day", analysis.burn_rate);
        println!("  Projected Cost: ${:.2}/month", analysis.projected_cost);
        
        if let Some(days) = analysis.days_to_breach {
            let days_str = format!("{:.1} days", days);
            let colored_days = match analysis.risk {
                BurnRisk::Critical => days_str.bright_red().bold(),
                BurnRisk::High => days_str.bright_yellow(),
                BurnRisk::Medium => days_str.yellow(),
                BurnRisk::Low => days_str.green(),
            };
            println!("  Time to Breach: {}", colored_days);
        } else {
            println!("  Time to Breach: {}", "No breach predicted".green());
        }

        let risk_text = format!("{:?}", analysis.risk);
        let colored_risk = match analysis.risk {
            BurnRisk::Critical => risk_text.bright_red().bold(),
            BurnRisk::High => risk_text.bright_yellow(),
            BurnRisk::Medium => risk_text.yellow(),
            BurnRisk::Low => risk_text.green(),
        };
        println!("  Risk Level:     {}", colored_risk);
        
        println!("  Confidence:     {:.0}% (R² = {:.3})", 
            analysis.confidence * 100.0,
            analysis.r_squared
        );

        println!();
    }

    println!("{}", "━".repeat(60).bright_black());
    println!();

    // Summary
    println!("{}", "Summary".bright_white().bold());
    println!("  Total SLOs:     {}", report.total_slos);
    println!("  At Risk:        {}", report.slos_at_risk);
    
    let overall_risk_text = format!("{:?}", report.overall_risk);
    let colored_overall = match report.overall_risk {
        BurnRisk::Critical => overall_risk_text.bright_red().bold(),
        BurnRisk::High => overall_risk_text.bright_yellow(),
        BurnRisk::Medium => overall_risk_text.yellow(),
        BurnRisk::Low => overall_risk_text.green(),
    };
    println!("  Overall Risk:   {}", colored_overall);

    println!();

    // Action recommendations
    if report.requires_action() {
        println!("{}", "⚠️  Action Required".bright_yellow().bold());
        
        let critical = report.critical_slos();
        if !critical.is_empty() {
            println!();
            println!("{}", "Critical SLOs:".bright_red().bold());
            for analysis in critical {
                if let Some(days) = analysis.days_to_breach {
                    println!("  • {} - {:.1} days to breach", 
                        analysis.slo_name.bright_white(),
                        days
                    );
                } else {
                    println!("  • {} - Already exceeded", 
                        analysis.slo_name.bright_white()
                    );
                }
            }
        }

        println!();
        println!("{}", "Recommended Actions:".bright_white().bold());
        println!("  1. Review cost drivers in critical SLOs");
        println!("  2. Consider scaling down or optimizing resources");
        println!("  3. Update SLO limits if growth is expected");
        println!("  4. Investigate unexpected cost increases");
    } else {
        println!("{}", "✅ All SLOs within acceptable limits".bright_green());
    }

    println!();

    Ok(())
}

/// Output report as JSON
fn output_json(report: &BurnReport) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Output report as Markdown (useful for PR comments)
fn output_markdown(report: &BurnReport) -> Result<(), Box<dyn std::error::Error>> {
    println!("## 📊 SLO Burn Rate Analysis");
    println!();

    if report.analyses.is_empty() {
        println!("⚠️ No SLOs analyzed (insufficient data)");
        return Ok(());
    }

    println!("| SLO | Burn Rate | Projected | Days to Breach | Risk | Confidence |");
    println!("|-----|-----------|-----------|----------------|------|------------|");

    for analysis in &report.analyses {
        let risk_emoji = match analysis.risk {
            BurnRisk::Low => "✅",
            BurnRisk::Medium => "⚠️",
            BurnRisk::High => "🔶",
            BurnRisk::Critical => "🔥",
        };

        let days_str = if let Some(days) = analysis.days_to_breach {
            format!("{:.1} days", days)
        } else {
            "No breach".to_string()
        };

        println!("| {} {} | ${:.2}/day | ${:.2} | {} | {:?} | {:.0}% |",
            risk_emoji,
            analysis.slo_name,
            analysis.burn_rate,
            analysis.projected_cost,
            days_str,
            analysis.risk,
            analysis.confidence * 100.0
        );
    }

    println!();
    println!("### Summary");
    println!();
    println!("- **Total SLOs:** {}", report.total_slos);
    println!("- **At Risk:** {}", report.slos_at_risk);
    println!("- **Overall Risk:** {:?}", report.overall_risk);
    println!();

    if report.requires_action() {
        println!("### ⚠️ Action Required");
        println!();

        let critical = report.critical_slos();
        if !critical.is_empty() {
            println!("**Critical SLOs:**");
            for analysis in critical {
                if let Some(days) = analysis.days_to_breach {
                    println!("- {} - {:.1} days to breach", analysis.slo_name, days);
                } else {
                    println!("- {} - Already exceeded", analysis.slo_name);
                }
            }
            println!();
        }

        println!("**Recommended Actions:**");
        println!("1. Review cost drivers in critical SLOs");
        println!("2. Consider scaling down or optimizing resources");
        println!("3. Update SLO limits if growth is expected");
        println!("4. Investigate unexpected cost increases");
    } else {
        println!("✅ All SLOs within acceptable limits");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::slo::burn_rate::{BurnAnalysis, BurnRisk};
    use chrono::Utc;

    fn create_test_report() -> BurnReport {
        let analyses = vec![
            BurnAnalysis {
                slo_id: "slo-1".to_string(),
                slo_name: "Production Budget".to_string(),
                burn_rate: 142.86,
                projected_cost: 4428.6,
                slo_limit: 5000.0,
                days_to_breach: Some(8.5),
                risk: BurnRisk::High,
                confidence: 0.95,
                trend_slope: 142.86,
                trend_intercept: 1000.0,
                r_squared: 0.95,
                analyzed_at: Utc::now().to_rfc3339(),
            },
            BurnAnalysis {
                slo_id: "slo-2".to_string(),
                slo_name: "Dev Budget".to_string(),
                burn_rate: 50.0,
                projected_cost: 1500.0,
                slo_limit: 2000.0,
                days_to_breach: Some(25.0),
                risk: BurnRisk::Medium,
                confidence: 0.82,
                trend_slope: 50.0,
                trend_intercept: 500.0,
                r_squared: 0.82,
                analyzed_at: Utc::now().to_rfc3339(),
            },
        ];

        BurnReport::new(analyses)
    }

    #[test]
    fn test_output_json() {
        let report = create_test_report();
        let result = output_json(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_text() {
        let report = create_test_report();
        let result = output_text(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_markdown() {
        let report = create_test_report();
        let result = output_markdown(&report);
        assert!(result.is_ok());
    }
}
