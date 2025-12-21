use crate::engines::detection::DetectionEngine;
use crate::engines::policy::{ExemptionValidator, PolicyEngine, PolicyLoader};
use crate::engines::prediction::PredictionEngine;
use crate::engines::shared::error_model::{CostPilotError, ErrorCategory};
use crate::engines::shared::models::CostEstimate;
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

/// Scan infrastructure changes for cost issues
#[derive(Debug, Args)]
pub struct ScanCommand {
    /// Path to infrastructure change file (Terraform plan, CDK diff)
    plan: PathBuf,

    /// Infrastructure format: terraform, cdk
    #[arg(short, long, default_value = "terraform")]
    format: String,

    /// Enable detailed explanations
    #[arg(short, long)]
    explain: bool,

    /// Path to policy file
    #[arg(long, value_name = "FILE")]
    policy: Option<PathBuf>,

    /// Path to exemptions file
    #[arg(long, value_name = "FILE")]
    exemptions: Option<PathBuf>,

    /// Fail on critical severity issues
    #[arg(long)]
    fail_on_critical: bool,

    /// Show autofix snippets
    #[arg(long)]
    autofix: bool,

    /// Stack name for CDK synthesized templates (required for CDK format)
    #[arg(long)]
    stack: Option<String>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
    Text,
}

impl ScanCommand {
    pub fn execute(&self) -> Result<(), CostPilotError> {
        let edition = crate::edition::EditionContext::new();
        self.execute_with_edition(&edition)
    }

    pub fn execute_with_edition(
        &self,
        edition: &crate::edition::EditionContext,
    ) -> Result<(), CostPilotError> {
        // Validate input file exists
        if !self.plan.exists() {
            let hint = match self.format.as_str() {
                "terraform" => "Run 'terraform plan -out=tfplan && terraform show -json tfplan > tfplan.json'",
                "cdk" => "Run 'cdk diff --json' and save the output",
                _ => "Ensure the input file exists and is readable",
            };
            return Err(CostPilotError::new(
                "SCAN_001",
                crate::errors::ErrorCategory::FileSystemError,
                format!("{} file not found: {}", self.format, self.plan.display()),
            )
            .with_hint(hint.to_string()));
        }

        // Validate format-specific requirements
        match self.format.as_str() {
            "terraform" | "cdk" => {}
            _ => {
                return Err(CostPilotError::new(
                    "SCAN_003",
                    crate::errors::ErrorCategory::ValidationError,
                    format!("Unsupported format: {}", self.format),
                )
                .with_hint("Supported formats: terraform, cdk".to_string()));
            }
        }

        // For CDK format, stack name is required
        if self.format == "cdk" && self.stack.is_none() {
            return Err(CostPilotError::new(
                "SCAN_004",
                crate::errors::ErrorCategory::ValidationError,
                "Stack name is required for CDK format".to_string(),
            )
            .with_hint("Use --stack <name> to specify the CDK stack name".to_string()));
        }

        println!("{}", "🔍 CostPilot Scan".bold().cyan());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Step 1: Detection
        println!("{}", "📊 Step 1: Detection".bold());
        let detection_engine = DetectionEngine::new();
        let changes = match self.format.as_str() {
            "terraform" => {
                println!("   Format: Terraform plan");
                detection_engine.detect_from_terraform_plan(&self.plan)?
            }
            "cdk" => {
                let stack_name = self.stack.as_ref().unwrap();
                println!("   Format: CDK diff (stack: {})", stack_name);
                detection_engine.detect_from_cdk_diff(&self.plan)?
            }
            _ => unreachable!(),
        };
        println!("   Found {} resource changes\n", changes.len());

        if changes.is_empty() {
            println!("{}", "✅ No resource changes detected".green());
            return Ok(());
        }

        // Step 2: Prediction
        println!("{}", "💰 Step 2: Cost Prediction".bold());

        let estimates = match edition.pro.as_ref() {
            Some(pro) => {
                // Premium: use ProEngine
                use crate::cli::pro_serde;
                let input = pro_serde::serialize(&changes).map_err(|e| {
                    CostPilotError::new(
                        "E_SERIALIZE",
                        ErrorCategory::PredictionError,
                        e.to_string(),
                    )
                })?;
                let output = pro.scan(input.as_bytes()).map_err(|e| {
                    CostPilotError::new("E_PRO_SCAN", ErrorCategory::PredictionError, e.to_string())
                })?;
                let output_str = std::str::from_utf8(&output).map_err(|e| {
                    CostPilotError::new("E_UTF8", ErrorCategory::PredictionError, e.to_string())
                })?;
                pro_serde::deserialize::<Vec<CostEstimate>>(output_str).map_err(|e| {
                    CostPilotError::new(
                        "E_DESERIALIZE",
                        ErrorCategory::PredictionError,
                        e.to_string(),
                    )
                })?
            }
            None => {
                // Free: use static prediction
                PredictionEngine::predict_static(&changes)?
            }
        };

        let total_monthly: f64 = estimates.iter().map(|e| e.monthly_cost).sum();
        println!("   Estimated monthly cost: ${:.2}", total_monthly);
        println!("   ({} resources analyzed)\n", estimates.len());

        // Step 3: Policy Evaluation (if policy file provided)
        // Convert TotalCost to CostEstimate for policy/SLO evaluation
        let total_cost_estimate = CostEstimate {
            resource_id: "total".to_string(),
            monthly_cost: total_monthly,
            prediction_interval_low: 0.0,
            prediction_interval_high: 0.0,
            confidence_score: 0.0,
            heuristic_reference: None,
            cold_start_inference: false,
            one_time: None,
            breakdown: None,
            hourly: None,
            daily: None,
        };

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
                        crate::engines::policy::ExemptionStatus::ExpiringSoon {
                            expires_in_days,
                        } => {
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

                PolicyEngine::with_exemptions(policy_config, exemptions, edition)
            } else {
                PolicyEngine::new(policy_config, edition)
            };

            // Convert TotalCost to CostEstimate for policy evaluation

            let mut policy_result = policy_engine.evaluate(&changes, &total_cost_estimate);

            // Free edition: downgrade all violations to warnings
            if !edition.capabilities.allow_policy_enforce {
                let violations_to_convert = policy_result.violations.clone();
                for violation in &violations_to_convert {
                    policy_result.add_warning(format!(
                        "[{}] {} - {} (actual: {}, expected: {})",
                        violation.severity,
                        violation.policy_name,
                        violation.message,
                        violation.actual_value,
                        violation.expected_value
                    ));
                }
                policy_result.violations.clear();
                policy_result.passed = true;
            }

            if !policy_result.violations.is_empty() {
                println!(
                    "   {} {}",
                    "⚠".yellow(),
                    format!(
                        "{} policy violations detected",
                        policy_result.violations.len()
                    )
                    .yellow()
                );
                for violation in &policy_result.violations {
                    println!(
                        "     • {} [{}] {}",
                        violation.resource_id.bright_black(),
                        violation.severity.yellow(),
                        violation.message
                    );
                }
            } else if !policy_result.warnings.is_empty() {
                println!(
                    "   {} {}",
                    "⚠".yellow(),
                    format!(
                        "{} policy warnings (Free edition: enforcement disabled)",
                        policy_result.warnings.len()
                    )
                    .yellow()
                );
                for warning in &policy_result.warnings {
                    println!(
                        "     • {}",
                        warning
                    );
                }
            } else {
                println!("   {} All policies passed", "✅".green());
            }

            // Display applied exemptions
            if !policy_result.applied_exemptions.is_empty() {
                println!(
                    "   {} {}",
                    "ℹ".blue(),
                    format!(
                        "{} exemptions applied",
                        policy_result.applied_exemptions.len()
                    )
                    .blue()
                );
                for exemption_id in &policy_result.applied_exemptions {
                    println!(
                        "     • EXEMPTION_APPLIED: {}",
                        exemption_id.bright_blue()
                    );
                }
            }
            println!();
        }

        // Step 4: SLO Evaluation (if SLO config exists)
        let slo_config_path = std::path::PathBuf::from(".costpilot/slo.json");
        if slo_config_path.exists() {
            println!("{}", "📏 Step 4: SLO Evaluation".bold());
            match self.evaluate_slos(&total_cost_estimate, &estimates, edition) {
                Ok(slo_passed) => {
                    if slo_passed {
                        println!("   {} All SLOs passed", "✅".green());
                    } else {
                        println!("   {} SLO violations detected", "❌".red());
                        // In premium edition, this would block deployment
                        if edition.capabilities.allow_slo_enforce {
                            println!("   {} Deployment blocked by SLO violation", "🚫".red());
                        } else {
                            println!("   {} Free edition: SLO violations logged only", "⚠️".yellow());
                        }
                    }
                }
                Err(e) => {
                    println!("   {} SLO evaluation failed: {}", "❌".red(), e);
                }
            }
            println!();
        }

        // Step 5: Explanation (if requested)
        if self.explain {
            println!("{}", "💡 Step 5: Explanation".bold());
            let _explain_engine = crate::engines::explain::ExplainEngine::new();
            // TODO: Implement detect_anti_patterns
            let anti_patterns: Vec<String> = Vec::new();

            if !anti_patterns.is_empty() {
                println!("   Detected {} anti-patterns:\n", anti_patterns.len());
            } else {
                println!("   {} No anti-patterns detected", "✅".green());
            }
        }

        // Step 5: Autofix snippets (if requested)
        if self.autofix {
            println!("{}", "🔧 Step 6: Autofix Snippets".bold());
            let _autofix_engine = crate::engines::autofix::AutofixEngine::new();
            // TODO: generate_fixes requires 4 args: detections, changes, estimates, mode
            // Stub for now
            println!("   Autofix not yet implemented in scan command");
        }

        // Summary
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
        );
        println!("{}", "📈 Summary".bold());
        println!("   Resources changed: {}", changes.len());
        println!("   Monthly cost: ${:.2}", total_monthly);

        if let Some(policy_path) = &self.policy {
            let policy_config = PolicyLoader::load_from_file(policy_path)?;
            let policy_engine = if let Some(exemptions_path) = &self.exemptions {
                let exemption_validator = ExemptionValidator::new();
                let exemptions = exemption_validator.load_from_file(exemptions_path)?;
                PolicyEngine::with_exemptions(policy_config, exemptions, edition)
            } else {
                PolicyEngine::new(policy_config, edition)
            };

            // Convert estimates to CostEstimate for policy evaluation
            let total_cost_estimate = CostEstimate {
                resource_id: "total".to_string(),
                monthly_cost: total_monthly,
                prediction_interval_low: 0.0,
                prediction_interval_high: 0.0,
                confidence_score: 0.0,
                heuristic_reference: None,
                cold_start_inference: false,
                one_time: None,
                breakdown: None,
                hourly: None,
                daily: None,
            };

            let policy_result = policy_engine.evaluate(&changes, &total_cost_estimate);

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

    /// Evaluate SLOs against the current cost estimates
    fn evaluate_slos(
        &self,
        total_cost: &CostEstimate,
        estimates: &[CostEstimate],
        edition: &crate::edition::EditionContext,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::engines::slo::{SloEngine, SloDefinition};

        // Load SLO config
        let slo_config_path = std::path::PathBuf::from(".costpilot/slo.json");
        let content = std::fs::read_to_string(&slo_config_path)?;
        let config: crate::engines::slo::SloConfig = serde_json::from_str(&content)?;

        // Convert to SloDefinitions for the engine
        let definitions: Vec<SloDefinition> = config.slos.into_iter().map(|slo| {
            SloDefinition {
                id: slo.id,
                name: slo.name,
                description: slo.description,
                slo_type: slo.slo_type,
                target: slo.target,
                threshold: slo.threshold.max_value,
                enforcement: slo.enforcement,
            }
        }).collect();

        // Create total cost structure
        let total_cost_struct = crate::engines::shared::models::TotalCost {
            monthly: total_cost.monthly_cost,
            prediction_interval_low: total_cost.prediction_interval_low,
            prediction_interval_high: total_cost.prediction_interval_high,
            confidence_score: total_cost.confidence_score,
            resource_count: estimates.len(),
        };

        // Create SLO engine and evaluate
        let engine = SloEngine::new(definitions, edition);
        let result = engine.check_slo(&total_cost_struct, estimates);

        // Print evaluation details
        for evaluation in &result.evaluations {
            let icon = match evaluation.status {
                crate::engines::slo::SloStatus::Pass => "✅",
                crate::engines::slo::SloStatus::Warning => "⚠️",
                crate::engines::slo::SloStatus::Violation => "❌",
                crate::engines::slo::SloStatus::NoData => "❓",
            };
            println!("     {} {}: {}", icon, evaluation.slo_name, evaluation.message);
        }

        Ok(result.passed)
    }

    /// Format severity with color (utility for future output modes)
    #[allow(dead_code)]
    fn _format_severity(&self, severity: &str) -> String {
        match severity {
            "CRITICAL" => severity.red().to_string(),
            "HIGH" => severity.bright_red().to_string(),
            "MEDIUM" => severity.yellow().to_string(),
            "LOW" => severity.bright_black().to_string(),
            _ => severity.to_string(),
        }
    }

    /// Format code block with syntax highlighting (utility for future output modes)
    #[allow(dead_code)]
    fn _format_code_block(&self, code: &str) -> String {
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
            format: "terraform".to_string(),
            explain: false,
            policy: None,
            exemptions: None,
            fail_on_critical: false,
            autofix: false,
            stack: None,
        };

        let result = cmd.execute();
        assert!(result.is_err());
    }
}
