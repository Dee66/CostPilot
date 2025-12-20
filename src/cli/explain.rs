// CLI command for explaining cost predictions

use crate::engines::detection::DetectionEngine;
use crate::engines::prediction::PredictionEngine;
use crate::engines::shared::models::ChangeAction;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum ExplainCommand {
    /// Explain a specific resource's cost prediction
    Resource {
        /// Path to Terraform plan JSON
        #[arg(short, long)]
        plan: PathBuf,

        /// Resource address to explain (e.g., aws_instance.web)
        #[arg(short, long)]
        resource: String,

        /// Show verbose step-by-step reasoning
        #[arg(short, long)]
        verbose: bool,
    },

    /// Explain all resources in a plan
    All {
        /// Path to Terraform plan JSON
        #[arg(short, long)]
        plan: PathBuf,

        /// Only explain resources above this cost threshold
        #[arg(long, default_value = "0.0")]
        min_cost: f64,

        /// Limit number of explanations
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
}

#[derive(Debug, clap::Args)]
pub struct ExplainArgs {
    /// Resource type to explain (e.g., aws_instance, aws_rds_instance)
    pub resource_type: String,

    /// Instance type for EC2 instances
    #[arg(long)]
    pub instance_type: Option<String>,

    /// Database engine for RDS instances
    #[arg(long)]
    pub engine: Option<String>,

    /// Storage size in GB
    #[arg(long)]
    pub storage_gb: Option<u32>,

    /// Number of vCPUs
    #[arg(long)]
    pub vcpu: Option<u32>,

    /// Memory size in GB
    #[arg(long)]
    pub memory_gb: Option<f32>,
}

pub fn execute_explain_command(
    command: ExplainCommand,
    edition: &crate::edition::EditionContext,
) -> Result<String, String> {
    match command {
        ExplainCommand::Resource {
            plan,
            resource,
            verbose,
        } => {
            // Gate verbose mode for Premium
            if verbose {
                crate::edition::require_premium(edition, "Advanced Explain")
                    .map_err(|e| e.to_string())?;
            }

            if edition.capabilities.allow_explain_full {
                execute_explain_resource(plan, resource, verbose, edition)
            } else {
                // Free edition: top patterns only
                execute_explain_lite(plan)
            }
        }
        ExplainCommand::All {
            plan,
            min_cost,
            limit,
        } => {
            if edition.capabilities.allow_explain_full {
                execute_explain_all(plan, min_cost, limit, edition)
            } else {
                execute_explain_lite(plan)
            }
        }
    }
}

fn execute_explain_resource(
    plan_path: PathBuf,
    resource_id: String,
    verbose: bool,
    edition: &crate::edition::EditionContext,
) -> Result<String, String> {
    // Load plan
    let detection_engine = DetectionEngine::new();
    let changes = detection_engine
        .detect_from_file(&plan_path)
        .map_err(|e| format!("Failed to load plan: {}", e))?;

    // Find the resource
    let change = changes
        .iter()
        .find(|c| c.resource_id == resource_id)
        .ok_or_else(|| format!("Resource not found: {}", resource_id))?;

    // Initialize prediction engine
    let prediction_engine = PredictionEngine::new_with_edition(edition)
        .map_err(|e| format!("Failed to initialize prediction engine: {}", e))?;

    // Generate explanation
    let chain = prediction_engine
        .explain(change)
        .map_err(|e| format!("Failed to generate explanation: {}", e))?;

    // Format output
    let mut output = String::new();

    if verbose {
        output.push_str(&chain.format_text());
    } else {
        output.push_str(&format!("🔍 Cost Explanation: {}\n\n", chain.resource_id));
        output.push_str(&format!("Resource Type: {}\n", chain.resource_type));
        output.push_str(&format!(
            "Monthly Cost: ${:.2}\n",
            chain.final_estimate.monthly_cost
        ));
        output.push_str(&format!(
            "Range: ${:.2} - ${:.2}\n",
            chain.final_estimate.interval_low, chain.final_estimate.interval_high
        ));
        output.push_str(&format!(
            "Confidence: {:.0}%\n\n",
            chain.overall_confidence * 100.0
        ));

        if !chain.final_estimate.components.is_empty() {
            output.push_str("Cost Breakdown:\n");
            for component in &chain.final_estimate.components {
                output.push_str(&format!(
                    "  • {}: ${:.2} ({:.1}%)\n",
                    component.name, component.cost, component.percentage
                ));
            }
            output.push('\n');
        }

        output.push_str(&format!(
            "💡 Tip: Use --verbose for step-by-step reasoning ({}  steps)\n",
            chain.step_count()
        ));
    }

    Ok(output)
}

fn execute_explain_all(
    plan_path: PathBuf,
    min_cost: f64,
    limit: Option<usize>,
    edition: &crate::edition::EditionContext,
) -> Result<String, String> {
    // Load plan
    let detection_engine = DetectionEngine::new();
    let changes = detection_engine
        .detect_from_file(&plan_path)
        .map_err(|e| format!("Failed to load plan: {}", e))?;

    // Initialize prediction engine
    let prediction_engine = PredictionEngine::new_with_edition(edition)
        .map_err(|e| format!("Failed to initialize prediction engine: {}", e))?;

    // Generate predictions and filter
    let mut explanations = Vec::new();

    for change in &changes {
        if change.action == ChangeAction::NoOp {
            continue;
        }

        match prediction_engine.explain(change) {
            Ok(chain) => {
                if chain.final_estimate.monthly_cost >= min_cost {
                    explanations.push(chain);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to explain {}: {}", change.resource_id, e);
            }
        }
    }

    // Sort by cost (descending)
    explanations.sort_by(|a, b| {
        b.final_estimate
            .monthly_cost
            .partial_cmp(&a.final_estimate.monthly_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    if let Some(n) = limit {
        explanations.truncate(n);
    }

    // Format output
    let mut output = String::new();
    output.push_str(&format!(
        "🔍 Cost Explanations ({} resources)\n",
        explanations.len()
    ));
    output.push_str(&"=".repeat(50));
    output.push_str("\n\n");

    for (idx, chain) in explanations.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", idx + 1, chain.resource_id));
        output.push_str(&format!("   Type: {}\n", chain.resource_type));
        output.push_str(&format!(
            "   Monthly Cost: ${:.2} (±${:.2})\n",
            chain.final_estimate.monthly_cost,
            (chain.final_estimate.interval_high - chain.final_estimate.interval_low) / 2.0
        ));
        output.push_str(&format!(
            "   Confidence: {:.0}%\n",
            chain.overall_confidence * 100.0
        ));

        if !chain.final_estimate.components.is_empty() && chain.final_estimate.components.len() > 1
        {
            output.push_str("   Components: ");
            let comp_names: Vec<String> = chain
                .final_estimate
                .components
                .iter()
                .map(|c| format!("{} ({:.0}%)", c.name, c.percentage))
                .collect();
            output.push_str(&comp_names.join(", "));
            output.push('\n');
        }

        if !chain.key_assumptions.is_empty() {
            output.push_str(&format!(
                "   Assumptions: {} key assumption(s)\n",
                chain.key_assumptions.len()
            ));
        }

        output.push('\n');
    }

    output.push_str(&"\n💡 Use 'costpilot explain resource --resource <id>' for detailed reasoning\n".to_string());

    Ok(output)
}

fn execute_explain_lite(plan_path: PathBuf) -> Result<String, String> {
    // Free edition: top 5 patterns only
    use crate::engines::explain::ExplainEngine;

    let detection_engine = DetectionEngine::new();
    let changes = detection_engine
        .detect_from_file(&plan_path)
        .map_err(|e| format!("Failed to load plan: {}", e))?;

    let detection_engine_inner = DetectionEngine::new();
    let detections = detection_engine_inner
        .detect(&changes)
        .map_err(|e| format!("Failed to detect: {}", e))?;

    let patterns = ExplainEngine::explain_top_patterns(&detections);

    let mut output = String::new();
    output.push_str("🔍 Cost Pattern Summary (Free Edition)\n\n");

    if patterns.is_empty() {
        output.push_str("✅ No cost issues detected\n");
    } else {
        output.push_str("Top detected patterns:\n\n");
        for pattern in patterns {
            output.push_str(&format!("{}\n", pattern));
        }
        output.push_str(&"\n💎 Upgrade to Premium for full explanations with:\n".to_string());
        output.push_str("   • Stepwise reasoning chains\n");
        output.push_str("   • Cost component breakdowns\n");
        output.push_str("   • Root cause analysis\n");
    }

    Ok(output)
}

pub fn execute_explain_args(
    args: ExplainArgs,
    edition: &crate::edition::EditionContext,
) -> Result<String, String> {
    // Build config from provided arguments
    let mut config = serde_json::Map::new();

    if let Some(instance_type) = &args.instance_type {
        config.insert("instance_type".to_string(), serde_json::Value::String(instance_type.clone()));
    }
    if let Some(engine) = &args.engine {
        config.insert("engine".to_string(), serde_json::Value::String(engine.clone()));
    }
    if let Some(storage_gb) = args.storage_gb {
        config.insert("storage_gb".to_string(), serde_json::Value::Number(storage_gb.into()));
    }
    if let Some(vcpu) = args.vcpu {
        config.insert("vcpu".to_string(), serde_json::Value::Number(vcpu.into()));
    }
    if let Some(memory_gb) = args.memory_gb {
        config.insert("memory_gb".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(memory_gb as f64).unwrap()));
    }

    let config_value = serde_json::Value::Object(config);

    // Create a mock resource change for explanation
    let change = crate::engines::shared::models::ResourceChange::builder()
        .resource_id(format!("{}.example", args.resource_type))
        .resource_type(args.resource_type.clone())
        .action(crate::engines::shared::models::ChangeAction::Create)
        .new_config(config_value)
        .monthly_cost(0.0) // Will be predicted
        .build();

    // Initialize prediction engine
    let prediction_engine = PredictionEngine::new_with_edition(edition)
        .map_err(|e| format!("Failed to initialize prediction engine: {}", e))?;

    // Get prediction
    let prediction = prediction_engine.predict_resource_cost(&change)
        .map_err(|e| format!("Failed to predict cost: {}", e))?;

    // Get explanation
    let explainer = crate::engines::explain::PredictionExplainer::from_engine(&prediction_engine);
    let explanation = explainer.explain(&change, &prediction);

    // Format output
    let mut output = format!("Explanation for {}:\n\n", args.resource_type);
    output.push_str(&format!("Predicted monthly cost: ${:.2}\n", prediction.monthly_cost));
    output.push_str(&format!("Confidence: {:.1}%\n\n", prediction.confidence_score * 100.0));

    output.push_str("Reasoning:\n");
    for step in &explanation.steps {
        output.push_str(&format!("• {}\n", step.title));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::EditionContext;

    #[test]
    fn test_explain_command_structure() {
        // Test that command structure compiles
        // Actual functionality requires valid plan files
    }

    #[test]
    fn test_execute_explain_args_basic() {
        let args = ExplainArgs {
            resource_type: "aws_instance".to_string(),
            instance_type: Some("t3.micro".to_string()),
            engine: None,
            storage_gb: None,
            vcpu: None,
            memory_gb: None,
        };

        let edition = EditionContext::default();

        let result = execute_explain_args(args, &edition);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Explanation for aws_instance"));
        assert!(output.contains("Predicted monthly cost"));
        assert!(output.contains("Confidence"));
        assert!(output.contains("Reasoning"));
    }

    #[test]
    fn test_execute_explain_args_unknown_resource() {
        let args = ExplainArgs {
            resource_type: "unknown_resource_type".to_string(),
            instance_type: None,
            engine: None,
            storage_gb: None,
            vcpu: None,
            memory_gb: None,
        };

        let edition = EditionContext::default();

        let result = execute_explain_args(args, &edition);
        // Should still succeed but with a low confidence prediction
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Explanation for unknown_resource_type"));
    }
}
