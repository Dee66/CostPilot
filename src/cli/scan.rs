use crate::engines::detection::DetectionEngine;
use crate::engines::prediction::PredictionEngine;
use crate::engines::policy::{ExemptionValidator, PolicyEngine, PolicyLoader};
use crate::errors::CostPilotError;
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

/// Scan Terraform plan for cost issues
#[derive(Debug, Args)]
pub struct ScanCommand {
    /// Path to Terraform plan JSON file
    #[arg(short, long, value_name = "FILE")]
    plan: PathBuf,

    /// Enable detailed explanations
    #[arg(short, long)]
    explain: bool,

    /// Path to policy file
    #[arg(long, value_name = "FILE")]
    policy: Option<PathBuf>,

    /// Path to exemptions file
    #[arg(long, value_name = "FILE")]
    exemptions: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "markdown")]
    format: OutputFormat,

    /// Fail on critical severity issues
    #[arg(long)]
    fail_on_critical: bool,

    /// Show autofix snippets
    #[arg(long)]
    autofix: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
    Text,
}

impl ScanCommand {
    pub fn execute(&self) -> Result<(), CostPilotError> {
        println!("{}", "🔍 CostPilot Scan".bold().cyan());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Validate plan file exists
        if !self.plan.exists() {
            return Err(CostPilotError::new(
                "SCAN_001",
                crate::errors::ErrorCategory::FileSystemError,
                format!("Terraform plan file not found: {}", self.plan.display()),
            )
            .with_hint("Run 'terraform plan -out=tfplan && terraform show -json tfplan > tfplan.json'".to_string()));
        }

        // Step 1: Detection
        println!("{}", "📊 Step 1: Detection".bold());
        let detection_engine = DetectionEngine::new();
        let changes = detection_engine.detect_from_terraform_plan(&self.plan)?;
        println!("   Found {} resource changes\n", changes.len());

        if changes.is_empty() {
            println!("{}", "✅ No resource changes detected".green());
            return Ok(());
        }

        // Step 2: Prediction
        println!("{}", "💰 Step 2: Cost Prediction".bold());
        let prediction_engine = PredictionEngine::new().map_err(|e| format!("Failed to initialize prediction engine: {}", e))?;
        let total_cost = prediction_engine.predict_total_cost(&changes)?;
        
        println!("   Estimated monthly cost: ${:.2}", total_cost.monthly);
        println!("   Confidence: {:.0}%\n", total_cost.confidence * 100.0);

        // Step 3: Policy Evaluation (if policy file provided)
        if let Some(policy_path) = &self.policy {
            println!("{}", "📋 Step 3: Policy Evaluation".bold());
            let policy_config = PolicyLoader::load_from_file(policy_path)?;
            PolicyLoader::validate(&policy_config)?;
            
            // Load exemptions if provided
            let policy_engine = if let Some(exemptions_path) = &self.exemptions {
                let exemption_validator = ExemptionValidator::new();
                let exemptions = exemption_validator.load_from_file(exemptions_path)?;
                
                // Check for expiring exemptions and warn
                let mut expiring_count = 0;
                for exemption in &exemptions.exemptions {
                    match exemption_validator.check_status(exemption) {
                        crate::engines::policy::ExemptionStatus::ExpiringSoon { expires_in_days } => {
                            println!(
                                "   {} Exemption {} expires in {} days",
                                "⚠".yellow(),
                                exemption.id.bright_black(),
                                expires_in_days
                            );
                            expiring_count += 1;
                        }
                        crate::engines::policy::ExemptionStatus::Expired { expired_on } => {
                            println!(
                                "   {} Exemption {} expired on {}",
                                "⚠".red(),
                                exemption.id.bright_black(),
                                expired_on
                            );
                        }
                        _ => {}
                    }
                }
                if expiring_count > 0 {
                    println!();
                }
                
                PolicyEngine::with_exemptions(policy_config, exemptions)
            } else {
                PolicyEngine::new(policy_config)
            };
            
            let policy_result = policy_engine.evaluate(&changes, &total_cost);
            
            if !policy_result.violations.is_empty() {
                println!(
                    "   {} {}",
                    "⚠".yellow(),
                    format!("{} policy violations detected", policy_result.violations.len()).yellow()
                );
                for violation in &policy_result.violations {
                    println!(
                        "     • {} [{}] {}",
                        violation.resource_id.bright_black(),
                        violation.severity.yellow(),
                        violation.message
                    );
                }
            } else {
                println!("   {} All policies passed", "✅".green());
            }
            println!();
        }

        // Step 4: Explanation (if requested)
        if self.explain {
            println!("{}", "💡 Step 4: Explanation".bold());
            let explain_engine = crate::engines::explain::ExplainEngine::new();
            let anti_patterns = explain_engine.detect_anti_patterns(&changes);
            
            if !anti_patterns.is_empty() {
                println!("   Detected {} anti-patterns:\n", anti_patterns.len());
                for pattern in &anti_patterns {
                    println!("   {} {}", "▸".bright_cyan(), pattern.pattern_name.bold());
                    println!("     {}", pattern.description.bright_black());
                    println!("     Resource: {}", pattern.detected_in.yellow());
                    println!("     Severity: {}", self.format_severity(&pattern.severity));
                    
                    if let Some(fix) = &pattern.suggested_fix {
                        println!("     Fix: {}", fix.bright_green());
                    }
                    println!();
                }
            } else {
                println!("   {} No anti-patterns detected", "✅".green());
            }
        }

        // Step 5: Autofix snippets (if requested)
        if self.autofix {
            println!("{}", "🔧 Step 5: Autofix Snippets".bold());
            let autofix_engine = crate::engines::autofix::AutofixEngine::new();
            let autofix_result = autofix_engine.generate_fixes(&changes)?;
            
            if !autofix_result.fixes.is_empty() {
                println!("   Generated {} fix snippets:\n", autofix_result.fixes.len());
                for fix in &autofix_result.fixes {
                    println!("   {} {}", "▸".bright_cyan(), fix.resource_id.bold());
                    println!("     Rationale: {}", fix.rationale.bright_black());
                    println!("     Deterministic: {}", if fix.deterministic { "✅" } else { "❌" });
                    println!("     Idempotent: {}", if fix.idempotent { "✅" } else { "❌" });
                    
                    if let Some(impact) = &fix.cost_impact {
                        println!("     Estimated savings: ${:.2}/month", impact.monthly_savings);
                    }
                    
                    println!("\n     Before:");
                    println!("     {}", self.format_code_block(&fix.before_after.before));
                    println!("\n     After:");
                    println!("     {}", self.format_code_block(&fix.before_after.after));
                    println!();
                }
            } else {
                println!("   {} No autofix suggestions available", "ℹ".bright_blue());
            }
        }

        // Summary
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
        println!("{}", "📈 Summary".bold());
        println!("   Resources changed: {}", changes.len());
        println!("   Monthly cost: ${:.2}", total_cost.monthly);
        
        if let Some(policy_path) = &self.policy {
            let policy_config = PolicyLoader::load_from_file(policy_path)?;
            let policy_engine = PolicyEngine::new(policy_config);
            let policy_result = policy_engine.evaluate(&changes, &total_cost);
            
            if !policy_result.passed {
                println!("   Policy status: {}", "FAILED".red());
                if self.fail_on_critical {
                    let has_critical = policy_result
                        .violations
                        .iter()
                        .any(|v| v.severity == "CRITICAL");
                    if has_critical {
                        return Err(CostPilotError::new(
                            "SCAN_002",
                            crate::errors::ErrorCategory::PolicyViolation,
                            "Critical policy violations detected".to_string(),
                        ));
                    }
                }
            } else {
                println!("   Policy status: {}", "PASSED".green());
            }
        }
        
        println!();
        Ok(())
    }

    fn format_severity(&self, severity: &str) -> String {
        match severity {
            "CRITICAL" => severity.red().to_string(),
            "HIGH" => severity.bright_red().to_string(),
            "MEDIUM" => severity.yellow().to_string(),
            "LOW" => severity.bright_black().to_string(),
            _ => severity.to_string(),
        }
    }

    fn format_code_block(&self, code: &str) -> String {
        code.lines()
            .map(|line| format!("       {}", line.bright_black()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_command_validation() {
        let cmd = ScanCommand {
            plan: PathBuf::from("nonexistent.json"),
            explain: false,
            policy: None,
            format: OutputFormat::Markdown,
            fail_on_critical: false,
            autofix: false,
        };

        let result = cmd.execute();
        assert!(result.is_err());
    }
}
