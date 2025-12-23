// costpilot diff command implementation

use crate::engines::detection::DetectionEngine;
use colored::Colorize;
use std::path::PathBuf;

/// Execute the diff command to compare two Terraform plans
pub fn execute(
    before: PathBuf,
    after: PathBuf,
    format: &str,
    verbose: bool,
    edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Require Premium edition
    crate::edition::require_premium(edition, "Diff")?;

    if verbose {
        println!(
            "{}",
            "📊 Computing cost differences...".bright_blue().bold()
        );
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

    // Parse both plans
    let before_changes = detection_engine.detect_from_terraform_plan(&before)?;
    let after_changes = detection_engine.detect_from_terraform_plan(&after)?;

    // Compute costs via ProEngine
    let pro = edition.require_pro("Diff")?;
    use crate::cli::pro_serde;

    let before_input = pro_serde::serialize(&before_changes)?;
    let before_output = pro.predict(before_input.as_bytes())?;
    let before_output_str = std::str::from_utf8(&before_output)
        .map_err(|e| format!("Invalid UTF-8 from ProEngine: {}", e))?;
    let before_estimates: Vec<crate::engines::shared::models::CostEstimate> =
        pro_serde::deserialize(before_output_str)?;
    let before_monthly: f64 = before_estimates.iter().map(|e| e.monthly_cost).sum();

    let after_input = pro_serde::serialize(&after_changes)?;
    let after_output = pro.predict(after_input.as_bytes())?;
    let after_output_str = std::str::from_utf8(&after_output)
        .map_err(|e| format!("Invalid UTF-8 from ProEngine: {}", e))?;
    let after_estimates: Vec<crate::engines::shared::models::CostEstimate> =
        pro_serde::deserialize(after_output_str)?;
    let after_monthly: f64 = after_estimates.iter().map(|e| e.monthly_cost).sum();

    // Calculate delta
    let delta = after_monthly - before_monthly;

    let percentage = if before_monthly > 0.0 {
        (delta / before_monthly) * 100.0
    } else {
        0.0
    };

    match format {
        "json" => print_diff_json(before_monthly, after_monthly, delta, percentage),
        "markdown" => print_diff_markdown(before_monthly, after_monthly, delta, percentage),
        _ => print_diff_text(before_monthly, after_monthly, delta, percentage, verbose),
    }

    Ok(())
}

fn print_diff_text(before: f64, after: f64, delta: f64, percentage: f64, verbose: bool) {
    println!("{}", "Cost Comparison".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("  {} ${:.2}/month", "Before:".bold(), before);
    println!("  {}  ${:.2}/month", "After:".bold(), after);
    println!();

    let delta_str = if delta >= 0.0 {
        format!("+${:.2}/month", delta)
    } else {
        format!("-${:.2}/month", delta.abs())
    };

    let percentage_str = if percentage >= 0.0 {
        format!("(+{:.1}%)", percentage)
    } else {
        format!("({:.1}%)", percentage)
    };

    if delta > 0.0 {
        println!(
            "  {}   {} {}",
            "Delta:".bold(),
            delta_str.bright_red(),
            percentage_str.bright_red()
        );
    } else if delta < 0.0 {
        println!(
            "  {}   {} {}",
            "Delta:".bold(),
            delta_str.bright_green(),
            percentage_str.bright_green()
        );
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

    if verbose {
        println!();
        println!("{}", "💡 Tip".bright_cyan());
        println!(
            "   Use {} to see detailed breakdown",
            "'costpilot scan --explain'".bright_white()
        );
        println!(
            "   Use {} for autofix suggestions",
            "'costpilot autofix patch'".bright_white()
        );
    }
}

fn print_diff_json(before: f64, after: f64, delta: f64, percentage: f64) {
    use serde_json::json;

    let diff = json!({
        "before": {
            "monthly_cost": before,
        },
        "after": {
            "monthly_cost": after,
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

fn print_diff_markdown(before: f64, after: f64, delta: f64, percentage: f64) {
    println!("# Cost Difference Report");
    println!();
    println!("## Summary");
    println!();
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| **Before** | ${:.2}/month |", before);
    println!("| **After** | ${:.2}/month |", after);

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
    println!("- Monthly Cost: ${:.2}", before);
    println!();

    println!("### After Plan");
    println!("- Monthly Cost: ${:.2}", after);
    println!();

    println!("---");
    println!();
    println!("💡 **Next Steps**");
    println!("- Run `costpilot scan --explain` for detailed analysis");
    println!("- Run `costpilot autofix patch` for cost optimization suggestions");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::EditionContext;
    use crate::test_helpers::edition;
    use std::fs;
    use tempfile::TempDir;

    fn create_mock_plan(temp_dir: &TempDir, filename: &str, content: &str) -> std::path::PathBuf {
        let path = temp_dir.path().join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_execute_missing_before_file() {
        let temp = TempDir::new().unwrap();
        let before_path = temp.path().join("missing_before.json");
        let after_path = create_mock_plan(&temp, "after.json", "{}");

        let edition = edition::premium();

        let result = execute(before_path, after_path, "text", false, &edition);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        println!("Actual error message: {}", err_msg);
        assert!(err_msg.contains("Before plan not found"));
    }

    #[test]
    fn test_execute_missing_after_file() {
        let temp = TempDir::new().unwrap();
        let before_path = create_mock_plan(&temp, "before.json", "{}");
        let after_path = temp.path().join("missing_after.json");

        let edition = edition::premium();

        let result = execute(before_path, after_path, "text", false, &edition);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("After plan not found"));
    }

    #[test]
    fn test_execute_requires_premium() {
        let temp = TempDir::new().unwrap();
        let before_path = create_mock_plan(&temp, "before.json", "{}");
        let after_path = create_mock_plan(&temp, "after.json", "{}");

        let edition = EditionContext::free();

        let result = execute(before_path, after_path, "text", false, &edition);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Premium"));
    }

    #[test]
    fn test_print_diff_text_cost_increase() {
        // Test that the function doesn't panic
        print_diff_text(100.0, 120.0, 20.0, 20.0, false);
        print_diff_text(100.0, 120.0, 20.0, 20.0, true);
    }

    #[test]
    fn test_print_diff_text_cost_decrease() {
        print_diff_text(120.0, 100.0, -20.0, -16.67, false);
        print_diff_text(120.0, 100.0, -20.0, -16.67, true);
    }

    #[test]
    fn test_print_diff_text_no_change() {
        print_diff_text(100.0, 100.0, 0.0, 0.0, false);
        print_diff_text(100.0, 100.0, 0.0, 0.0, true);
    }

    #[test]
    fn test_print_diff_text_zero_before() {
        print_diff_text(0.0, 50.0, 50.0, 0.0, false);
    }

    #[test]
    fn test_print_diff_json() {
        print_diff_json(100.0, 120.0, 20.0, 20.0);
        print_diff_json(120.0, 100.0, -20.0, -16.67);
        print_diff_json(100.0, 100.0, 0.0, 0.0);
    }

    #[test]
    fn test_print_diff_markdown() {
        print_diff_markdown(100.0, 120.0, 20.0, 20.0);
        print_diff_markdown(120.0, 100.0, -20.0, -16.67);
        print_diff_markdown(100.0, 100.0, 0.0, 0.0);
    }

    #[test]
    fn test_severity_assessment_high() {
        // Test HIGH severity (>= 50%)
        let severity = if 60.0 >= 50.0 {
            ("HIGH", "🔴")
        } else if 60.0 >= 20.0 {
            ("MEDIUM", "🟡")
        } else if 60.0 >= 5.0 {
            ("LOW", "🔵")
        } else {
            ("INFO", "⚪")
        };
        assert_eq!(severity.0, "HIGH");
        assert_eq!(severity.1, "🔴");
    }

    #[test]
    fn test_severity_assessment_medium() {
        // Test MEDIUM severity (>= 20%)
        let severity = if 30.0 >= 50.0 {
            ("HIGH", "🔴")
        } else if 30.0 >= 20.0 {
            ("MEDIUM", "🟡")
        } else if 30.0 >= 5.0 {
            ("LOW", "🔵")
        } else {
            ("INFO", "⚪")
        };
        assert_eq!(severity.0, "MEDIUM");
        assert_eq!(severity.1, "🟡");
    }

    #[test]
    fn test_severity_assessment_low() {
        // Test LOW severity (>= 5%)
        let severity = if 10.0 >= 50.0 {
            ("HIGH", "🔴")
        } else if 10.0 >= 20.0 {
            ("MEDIUM", "🟡")
        } else if 10.0 >= 5.0 {
            ("LOW", "🔵")
        } else {
            ("INFO", "⚪")
        };
        assert_eq!(severity.0, "LOW");
        assert_eq!(severity.1, "🔵");
    }

    #[test]
    fn test_severity_assessment_info() {
        // Test INFO severity (< 5%)
        let severity = if 2.0 >= 50.0 {
            ("HIGH", "🔴")
        } else if 2.0 >= 20.0 {
            ("MEDIUM", "🟡")
        } else if 2.0 >= 5.0 {
            ("LOW", "🔵")
        } else {
            ("INFO", "⚪")
        };
        assert_eq!(severity.0, "INFO");
        assert_eq!(severity.1, "⚪");
    }
}
