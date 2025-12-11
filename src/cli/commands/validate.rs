// Validate command - validate configuration files

use crate::validation::validate_file;
use colored::Colorize;
use std::path::PathBuf;

/// Execute validation for a file
pub fn execute(
    file: PathBuf,
    format: String,
    _edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate the file
    let report = validate_file(&file)?;

    // Output based on format
    match format.as_str() {
        "json" => {
            println!("{}", report.format_json());
        }
        "text" => {
            println!("{}", report.format_text());
        }
        _ => {
            return Err(format!("Unknown format: {}", format).into());
        }
    }

    // Exit with error code if validation failed
    if !report.is_valid {
        std::process::exit(2); // Exit code 2 for validation errors
    }

    Ok(())
}

/// Validate multiple files
pub fn execute_batch(
    files: Vec<PathBuf>,
    format: String,
    fail_fast: bool,
    _edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_valid = true;
    let mut reports = Vec::new();

    for file in &files {
        let report = validate_file(file)?;

        if !report.is_valid {
            all_valid = false;
            if fail_fast {
                // Print immediate error and exit
                println!("{}", report.format_text());
                std::process::exit(2);
            }
        }

        reports.push(report);
    }

    // Output all reports
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&reports)?;
            println!("{}", json);
        }
        "text" => {
            for report in &reports {
                println!("{}", report.format_text());
                println!("{}", "─".repeat(80));
            }

            // Summary
            let valid_count = reports.iter().filter(|r| r.is_valid).count();
            let total_count = reports.len();
            let error_count: usize = reports.iter().map(|r| r.error_count()).sum();
            let warning_count: usize = reports.iter().map(|r| r.warning_count()).sum();

            println!("\n📊 {} Summary\n", "Validation".bold());
            println!("  Files validated: {}", total_count);
            println!("  ✅ Valid: {}", valid_count.to_string().green());
            println!(
                "  ❌ Invalid: {}",
                (total_count - valid_count).to_string().red()
            );
            println!("  🔴 Total errors: {}", error_count.to_string().red());
            println!(
                "  🟡 Total warnings: {}",
                warning_count.to_string().yellow()
            );
        }
        _ => {
            return Err(format!("Unknown format: {}", format).into());
        }
    }

    if !all_valid {
        std::process::exit(2);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_execute_valid_file() {
        let yaml = r#"
id: test_policy
name: Test Policy
description: Test policy for validation
category: budget
severity: warning
owner: test
author: test
rules:
  - name: Budget rule
    description: Enforce budget limits
    enabled: true
    severity: Medium
    conditions:
      - condition_type:
          type: monthly_cost
        operator: greater_than
        value: 1000.0
    action:
      type: warn
      message: "Budget exceeded"
"#;
        let mut file = NamedTempFile::with_suffix(".yaml").unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        // This would exit(0) on success, so we can't test directly
        // But we can test that validation succeeds
        let report = validate_file(file.path()).unwrap();
        if !report.is_valid {
            eprintln!("Validation errors: {:?}", report.errors);
        }
        assert!(report.is_valid);
    }
}
