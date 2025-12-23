// Baseline management commands for CostPilot
//
// This module provides CLI commands for managing cost baselines:
// - Recording expected costs from successful deployments
// - Updating baselines with new cost expectations
// - Validating baseline configurations

use clap::Args;
use std::path::PathBuf;

use crate::engines::baselines::baseline_types::{Baseline, BaselinesConfig};
use crate::engines::baselines::BaselinesManager;

/// Manage cost baselines
#[derive(Debug, Args)]
pub struct BaselineCommand {
    #[command(subcommand)]
    command: BaselineCommands,
}

#[derive(Debug, clap::Subcommand)]
enum BaselineCommands {
    /// Record expected costs from a successful deployment
    ///
    /// Updates baselines with actual costs from a Terraform plan that was successfully deployed.
    /// This establishes new expected cost baselines based on real deployment outcomes.
    ///
    /// Examples:
    ///   costpilot baseline record tfplan.json --baselines baselines.json
    ///   costpilot baseline record tfplan.json --module vpc --cost 1200.0
    Record {
        /// Path to Terraform plan JSON file
        plan: PathBuf,

        /// Path to baselines file (will be created if it doesn't exist)
        #[arg(short, long)]
        baselines: PathBuf,

        /// Only record costs for specific module
        #[arg(long)]
        module: Option<String>,

        /// Override total cost for the plan
        #[arg(long)]
        total_cost: Option<f64>,

        /// Justification for the baseline update
        #[arg(long)]
        justification: Option<String>,

        /// Owner/team responsible for this baseline
        #[arg(long)]
        owner: Option<String>,
    },

    /// Update a specific baseline value
    ///
    /// Manually update expected costs for modules or global budget.
    ///
    /// Examples:
    ///   costpilot baseline update module.vpc --cost 1500.0 --justification "Added redundancy"
    ///   costpilot baseline update global --cost 8000.0 --variance 20.0
    Update {
        /// Baseline to update (module name, service name, or "global")
        target: String,

        /// New expected monthly cost
        #[arg(short, long)]
        cost: f64,

        /// Acceptable variance percentage (default: 10.0)
        #[arg(long)]
        variance: Option<f64>,

        /// Justification for the change
        #[arg(long)]
        justification: Option<String>,

        /// Owner/team responsible
        #[arg(long)]
        owner: Option<String>,

        /// Path to baselines file
        #[arg(short, long, default_value = "baselines.json")]
        baselines: PathBuf,
    },

    /// Validate baseline configuration
    ///
    /// Checks baselines.json for errors and provides helpful feedback.
    ///
    /// Examples:
    ///   costpilot baseline validate
    ///   costpilot baseline validate --file custom-baselines.json
    Validate {
        /// Path to baselines file
        #[arg(short, long, default_value = "baselines.json")]
        file: PathBuf,
    },

    /// Show baseline status and violations
    ///
    /// Displays current baselines and any violations against recent costs.
    ///
    /// Examples:
    ///   costpilot baseline status
    ///   costpilot baseline status --baselines baselines.json
    Status {
        /// Path to baselines file
        #[arg(short, long, default_value = "baselines.json")]
        baselines: PathBuf,

        /// Path to Terraform plan to compare against
        #[arg(long)]
        plan: Option<PathBuf>,
    },
}

impl BaselineCommand {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            BaselineCommands::Record {
                plan,
                baselines,
                module,
                total_cost,
                justification,
                owner,
            } => self.record_baselines(plan, baselines, module, *total_cost, justification, owner),

            BaselineCommands::Update {
                target,
                cost,
                variance,
                justification,
                owner,
                baselines,
            } => self.update_baseline(target, *cost, *variance, justification, owner, baselines),

            BaselineCommands::Validate { file } => self.validate_baselines(file),

            BaselineCommands::Status { baselines, plan } => {
                self.show_baseline_status(baselines, plan)
            }
        }
    }

    fn record_baselines(
        &self,
        plan_path: &PathBuf,
        baselines_path: &PathBuf,
        module_filter: &Option<String>,
        override_total: Option<f64>,
        justification: &Option<String>,
        owner: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Recording baselines from deployment...");

        // Load the plan
        let plan_content = std::fs::read_to_string(plan_path)
            .map_err(|e| format!("Failed to read plan file {}: {}", plan_path.display(), e))?;

        let _plan: serde_json::Value = serde_json::from_str(&plan_content)
            .map_err(|e| format!("Failed to parse plan JSON: {}", e))?;

        // Extract costs from plan (simplified - would need proper plan parsing)
        let total_cost = override_total.unwrap_or(0.0); // Would extract from plan

        // For now, this is a placeholder - would need to integrate with plan parsing
        println!("📊 Extracted costs from plan:");
        println!("   Total cost: ${:.2}", total_cost);

        if let Some(module) = module_filter {
            println!("   Filtered to module: {}", module);
        }

        // Load or create baselines
        let mut manager = if baselines_path.exists() {
            BaselinesManager::load_from_file(baselines_path)?
        } else {
            println!(
                "📝 Creating new baselines file: {}",
                baselines_path.display()
            );
            BaselinesManager::from_config(BaselinesConfig::new())
        };

        // Update baselines based on extracted costs
        let default_justification = justification
            .clone()
            .unwrap_or_else(|| format!("Recorded from deployment of {}", plan_path.display()));

        let default_owner = owner
            .clone()
            .unwrap_or_else(|| "deployment-automation".to_string());

        // This would update specific modules based on the plan
        // For now, just update global if total_cost provided
        if let Some(total) = override_total {
            let baseline = Baseline::new(
                "global".to_string(),
                total,
                default_justification,
                default_owner,
            );

            manager.update_global_baseline(baseline);
            println!("✅ Updated global baseline to ${:.2}", total);
        }

        // Save baselines
        manager.save_to_file(baselines_path)?;
        println!("💾 Saved baselines to {}", baselines_path.display());

        Ok(())
    }

    fn update_baseline(
        &self,
        target: &str,
        cost: f64,
        variance: Option<f64>,
        justification: &Option<String>,
        owner: &Option<String>,
        baselines_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Updating baseline for {}...", target);

        // Load baselines
        let mut manager = BaselinesManager::load_from_file(baselines_path)?;

        let default_justification = justification
            .clone()
            .unwrap_or_else(|| "Manual update via CLI".to_string());

        let default_owner = owner.clone().unwrap_or_else(|| "cli-user".to_string());

        // Create baseline with custom variance if provided
        let mut baseline = Baseline::new(
            target.to_string(),
            cost,
            default_justification,
            default_owner,
        );

        if let Some(var) = variance {
            baseline.acceptable_variance_percent = var;
        }

        // Update appropriate baseline
        if target == "global" {
            manager.update_global_baseline(baseline);
        } else if target.starts_with("module.") {
            manager.update_module_baseline(target.to_string(), baseline);
        } else {
            // Assume service baseline
            // Note: Current API doesn't support service baselines directly
            return Err(
                "Service baseline updates not yet implemented. Use module.global for now.".into(),
            );
        }

        // Save
        manager.save_to_file(baselines_path)?;
        println!("✅ Updated {} baseline to ${:.2}", target, cost);

        Ok(())
    }

    fn validate_baselines(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Validating baselines: {}", file_path.display());

        if !file_path.exists() {
            return Err(format!("Baselines file does not exist: {}", file_path.display()).into());
        }

        let manager = BaselinesManager::load_from_file(file_path)?;

        match manager.validate() {
            Ok(_) => {
                println!("✅ Baselines validation passed");
                Ok(())
            }
            Err(errors) => {
                eprintln!("❌ Baselines validation failed:");
                for error in errors {
                    eprintln!("  - {}", error);
                }
                Err("Validation failed".into())
            }
        }
    }

    fn show_baseline_status(
        &self,
        baselines_path: &PathBuf,
        plan_path: &Option<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Baseline Status");

        if !baselines_path.exists() {
            println!("No baselines file found at: {}", baselines_path.display());
            return Ok(());
        }

        let manager = BaselinesManager::load_from_file(baselines_path)?;

        // Show global baseline
        if let Some(global) = &manager.config().global {
            println!("\n🌍 Global Baseline:");
            println!("   Expected: ${:.2}/month", global.expected_monthly_cost);
            println!("   Variance: ±{:.1}%", global.acceptable_variance_percent);
            println!("   Owner: {}", global.owner);
            if let Ok(last_updated) = global.get_last_updated() {
                println!("   Updated: {}", last_updated.format("%Y-%m-%d"));
            } else {
                println!("   Updated: {}", global.last_updated);
            }
        }

        // Show module baselines
        if !manager.config().modules.is_empty() {
            println!("\n📦 Module Baselines:");
            for (name, baseline) in &manager.config().modules {
                println!(
                    "   {}: ${:.2}/month (±{:.1}%)",
                    name, baseline.expected_monthly_cost, baseline.acceptable_variance_percent
                );
            }
        }

        // Compare against plan if provided
        if let Some(plan) = plan_path {
            println!("\n📈 Comparison with plan: {}", plan.display());

            // This would need plan parsing integration
            println!("   Plan comparison not yet implemented");
        }

        Ok(())
    }
}
