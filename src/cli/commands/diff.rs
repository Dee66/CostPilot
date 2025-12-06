// costpilot diff command implementation

use crate::engines::detection::DetectionEngine;
use crate::engines::prediction::PredictionEngine;
use crate::errors::CostPilotError;
use colored::Colorize;
use std::path::PathBuf;

/// Execute the diff command to compare two Terraform plans
pub fn execute(
    before: PathBuf,
    after: PathBuf,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("{}", "📊 Computing cost differences...".bright_blue().bold());
        println!("   Before: {}", before.display());
        println!("   After: {}", after.display());
        println!();
    } else {
        println!("{}", "📊 costpilot diff".bright_blue().bold());
        println!();
    }

    // Validate both files exist
    if !before.exists() {
        return Err(format!("Before plan not found: {}", before.display()).into());
    }
    if !after.exists() {
        return Err(format!("After plan not found: {}", after.display()).into());
    }

    // Initialize engines
    let detection_engine = DetectionEngine::new();
    let prediction_engine = PredictionEngine::new()
        .map_err(|e| format!("Failed to initialize prediction engine: {}", e))?;

    // Parse both plans
    let before_changes = detection_engine.detect_from_terraform_plan(&before)?;
    let after_changes = detection_engine.detect_from_terraform_plan(&after)?;

    // Compute costs
    let before_cost = prediction_engine.predict_total_cost(&before_changes)?;
    let after_cost = prediction_engine.predict_total_cost(&after_changes)?;

    // Calculate delta
    let delta = after_cost.monthly - before_cost.monthly;
    let percentage = if before_cost.monthly > 0.0 {
        (delta / before_cost.monthly) * 100.0
    } else {
        0.0
    };

    match format {
        "json" => print_diff_json(&before_cost, &after_cost, delta, percentage),
        "markdown" => print_diff_markdown(&before_cost, &after_cost, delta, percentage, &before_changes, &after_changes),
        _ => print_diff_text(&before_cost, &after_cost, delta, percentage, verbose),
    }

    Ok(())
}

fn print_diff_text(
    before: &crate::engines::prediction::TotalCost,
    after: &crate::engines::prediction::TotalCost,
    delta: f64,
    percentage: f64,
    verbose: bool,
) {
    println!("{}", "Cost Comparison".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    println!("  {} ${:.2}/month", "Before:".bold(), before.monthly);
    println!("  {}  ${:.2}/month", "After:".bold(), after.monthly);
    println!();

    let delta_str = if delta >= 0.0 {
        format!("+${:.2}/month", delta)
    } else {
        format!("-${:.2}/month", delta.abs())
    };

    let percentage_str = if percentage >= 0.0 {
        format!("(+{:.1}%)", percentage)
    } else {
        format!("(-{:.1}%)", percentage.abs())
    };

    if delta > 0.0 {
        println!("  {}   {} {}", "Delta:".bold(), delta_str.bright_red(), percentage_str.bright_red());
    } else if delta < 0.0 {
        println!("  {}   {} {}", "Delta:".bold(), delta_str.bright_green(), percentage_str.bright_green());
    } else {
        println!("  {}   {} {}", "Delta:".bold(), delta_str, percentage_str);
    }
    
    println!();

    // Severity assessment
    let severity = if percentage.abs() >= 50.0 {
        ("HIGH", "🔴")
    } else if percentage.abs() >= 20.0 {
        ("MEDIUM", "🟡")
    } else if percentage.abs() >= 5.0 {
        ("LOW", "🔵")
    } else {
        ("INFO", "⚪")
    };

    println!("  {}  {} {}", "Severity:".bold(), severity.1, severity.0);
    
    println!();
    println!("{}", "Confidence".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Before: {:.0}%", before.confidence * 100.0);
    println!("  After:  {:.0}%", after.confidence * 100.0);

    if verbose {
        println!();
        println!("{}", "💡 Tip".bright_cyan());
        println!("   Use {} to see detailed breakdown", "'costpilot scan --explain'".bright_white());
        println!("   Use {} for autofix suggestions", "'costpilot autofix patch'".bright_white());
    }
}

fn print_diff_json(
    before: &crate::engines::prediction::TotalCost,
    after: &crate::engines::prediction::TotalCost,
    delta: f64,
    percentage: f64,
) {
    use serde_json::json;
    
    let diff = json!({
        "before": {
            "monthly_cost": before.monthly,
            "confidence": before.confidence,
        },
        "after": {
            "monthly_cost": after.monthly,
            "confidence": after.confidence,
        },
        "delta": {
            "absolute": delta,
            "percentage": percentage,
        },
        "severity": if percentage.abs() >= 50.0 {
            "HIGH"
        } else if percentage.abs() >= 20.0 {
            "MEDIUM"
        } else if percentage.abs() >= 5.0 {
            "LOW"
        } else {
            "INFO"
        },
    });
    
    println!("{}", serde_json::to_string_pretty(&diff).unwrap());
}

fn print_diff_markdown(
    before: &crate::engines::prediction::TotalCost,
    after: &crate::engines::prediction::TotalCost,
    delta: f64,
    percentage: f64,
    before_changes: &[crate::engines::detection::ResourceChange],
    after_changes: &[crate::engines::detection::ResourceChange],
) {
    println!("# Cost Difference Report");
    println!();
    println!("## Summary");
    println!();
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| **Before** | ${:.2}/month |", before.monthly);
    println!("| **After** | ${:.2}/month |", after.monthly);
    
    let delta_str = if delta >= 0.0 {
        format!("+${:.2}", delta)
    } else {
        format!("-${:.2}", delta.abs())
    };
    let percentage_str = if percentage >= 0.0 {
        format!("(+{:.1}%)", percentage)
    } else {
        format!("(-{:.1}%)", percentage.abs())
    };
    
    println!("| **Delta** | {} {} |", delta_str, percentage_str);
    
    let severity = if percentage.abs() >= 50.0 {
        "🔴 HIGH"
    } else if percentage.abs() >= 20.0 {
        "🟡 MEDIUM"
    } else if percentage.abs() >= 5.0 {
        "🔵 LOW"
    } else {
        "⚪ INFO"
    };
    
    println!("| **Severity** | {} |", severity);
    println!();
    
    println!("## Details");
    println!();
    println!("### Before Plan");
    println!("- Resources: {}", before_changes.len());
    println!("- Monthly Cost: ${:.2}", before.monthly);
    println!("- Confidence: {:.0}%", before.confidence * 100.0);
    println!();
    
    println!("### After Plan");
    println!("- Resources: {}", after_changes.len());
    println!("- Monthly Cost: ${:.2}", after.monthly);
    println!("- Confidence: {:.0}%", after.confidence * 100.0);
    println!();
    
    println!("---");
    println!();
    println!("💡 **Next Steps**");
    println!("- Run `costpilot scan --explain` for detailed analysis");
    println!("- Run `costpilot autofix patch` for cost optimization suggestions");
}
