// Policy DSL CLI commands

use clap::Args;
use std::path::PathBuf;
use colored::Colorize;
use crate::engines::policy::parser::{DslParser, PolicyRuleLoader, RuleEvaluator, EvaluationContext};

#[derive(Debug, Args)]
pub struct PolicyDslCommand {
    #[command(subcommand)]
    pub command: PolicyDslSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum PolicyDslSubcommand {
    /// List all loaded policy rules
    List {
        /// Show disabled rules
        #[arg(long)]
        all: bool,
        
        /// Filter by severity
        #[arg(long)]
        severity: Option<String>,
    },

    /// Validate policy rules
    Validate {
        /// Path to policy file or directory
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    /// Test policy rules against sample data
    Test {
        /// Path to policy file
        #[arg(long)]
        policy: PathBuf,

        /// Resource type
        #[arg(long)]
        resource_type: String,

        /// Monthly cost
        #[arg(long)]
        monthly_cost: Option<f64>,

        /// Show detailed evaluation
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show statistics about loaded rules
    Stats {
        /// Path to policy file or directory (default: search paths)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },

    /// Generate example policy rules
    Example {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Format: yaml or json
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
}

pub fn execute_policy_dsl_command(command: &PolicyDslCommand) -> Result<(), Box<dyn std::error::Error>> {
    match &command.command {
        PolicyDslSubcommand::List { all, severity } => {
            execute_list(*all, severity.as_deref())
        }
        PolicyDslSubcommand::Validate { path } => {
            execute_validate(path)
        }
        PolicyDslSubcommand::Test { policy, resource_type, monthly_cost, verbose } => {
            execute_test(policy, resource_type, *monthly_cost, *verbose)
        }
        PolicyDslSubcommand::Stats { path } => {
            execute_stats(path.as_ref())
        }
        PolicyDslSubcommand::Example { output, format } => {
            execute_example(output.as_ref(), format)
        }
    }
}

fn execute_list(show_all: bool, severity_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Policy Rules".bold().cyan());
    println!();

    let loader = PolicyRuleLoader::new();
    let rules = loader.load_all()?;

    if rules.is_empty() {
        println!("{}", "No policy rules found".yellow());
        println!("Search paths:");
        for path in PolicyRuleLoader::default_search_paths() {
            println!("  • {}", path.display());
        }
        return Ok(());
    }

    let filtered_rules: Vec<_> = rules.iter()
        .filter(|r| show_all || r.enabled)
        .filter(|r| {
            if let Some(sev) = severity_filter {
                format!("{:?}", r.severity).to_lowercase() == sev.to_lowercase()
            } else {
                true
            }
        })
        .collect();

    for (idx, rule) in filtered_rules.iter().enumerate() {
        let status = if rule.enabled { "✓".green() } else { "✗".red() };
        let severity_color = match rule.severity {
            crate::engines::policy::parser::RuleSeverity::Critical => "Critical".red(),
            crate::engines::policy::parser::RuleSeverity::High => "High".yellow(),
            crate::engines::policy::parser::RuleSeverity::Medium => "Medium".blue(),
            crate::engines::policy::parser::RuleSeverity::Low => "Low".cyan(),
            crate::engines::policy::parser::RuleSeverity::Info => "Info".white(),
        };

        println!("{} {} [{}]", status, rule.name.bold(), severity_color);
        println!("   {}", rule.description.dimmed());
        println!("   Conditions: {}", rule.conditions.len());
        
        if !rule.metadata.is_empty() {
            println!("   Metadata: {} keys", rule.metadata.len());
        }
        
        if idx < filtered_rules.len() - 1 {
            println!();
        }
    }

    println!();
    println!("{}", format!("Total: {} rules", filtered_rules.len()).dimmed());

    Ok(())
}

fn execute_validate(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Validating Policy Rules".bold().cyan());
    println!("Path: {}", path.display());
    println!();

    let loader = PolicyRuleLoader::new();
    let rules = loader.load_from_path(path)?;

    loader.validate_rules(&rules)?;

    println!("{} All rules valid", "✓".green());
    println!();

    let stats = loader.get_statistics(&rules);
    println!("{}", stats.format_text());

    Ok(())
}

fn execute_test(
    policy_path: &PathBuf,
    resource_type: &str,
    monthly_cost: Option<f64>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Testing Policy Rules".bold().cyan());
    println!();

    let loader = PolicyRuleLoader::new();
    let rules = loader.load_from_path(policy_path)?;

    println!("Loaded {} rules", rules.len());
    println!();

    let evaluator = RuleEvaluator::new(rules);
    
    let mut context = EvaluationContext::new()
        .with_resource_type(resource_type.to_string());

    if let Some(cost) = monthly_cost {
        context = context.with_monthly_cost(cost);
    }

    let result = evaluator.evaluate(&context);

    if result.matches.is_empty() {
        println!("{}", "✓ No rules matched".green());
        return Ok(());
    }

    println!("{}", format!("Found {} matching rules:", result.matches.len()).yellow());
    println!();

    for rule_match in &result.matches {
        let severity_str = match rule_match.severity {
            crate::engines::policy::parser::RuleSeverity::Critical => "CRITICAL".red(),
            crate::engines::policy::parser::RuleSeverity::High => "HIGH".yellow(),
            crate::engines::policy::parser::RuleSeverity::Medium => "MEDIUM".blue(),
            crate::engines::policy::parser::RuleSeverity::Low => "LOW".cyan(),
            crate::engines::policy::parser::RuleSeverity::Info => "INFO".white(),
        };

        println!("{} {} [{}]", "•".bold(), rule_match.rule_name.bold(), severity_str);
        println!("  {}", rule_match.message);
        
        if verbose {
            println!("  Action: {:?}", rule_match.action);
        }
        
        println!();
    }

    if result.has_blocks() {
        println!("{}", "⚠️  Some rules would BLOCK this resource".red().bold());
    } else if result.requires_approval() {
        println!("{}", "⚠️  Some rules require APPROVAL".yellow().bold());
    }

    Ok(())
}

fn execute_stats(path: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Policy Rule Statistics".bold().cyan());
    println!();

    let loader = PolicyRuleLoader::new();
    let rules = if let Some(p) = path {
        println!("Loading from: {}", p.display());
        loader.load_from_path(p)?
    } else {
        println!("Searching default paths...");
        loader.load_all()?
    };

    let stats = loader.get_statistics(&rules);
    
    println!();
    println!("{}", stats.format_text());

    Ok(())
}

fn execute_example(output: Option<&PathBuf>, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let example = if format == "json" {
        generate_example_json()
    } else {
        generate_example_yaml()
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &example)?;
        println!("{} Example written to {}", "✓".green(), output_path.display());
    } else {
        println!("{}", example);
    }

    Ok(())
}

fn generate_example_yaml() -> String {
    r#"# CostPilot Policy Rules Example

- name: "Block Expensive EC2 Instances"
  description: "Prevent creation of EC2 instances that cost more than $1000/month"
  enabled: true
  severity: High
  conditions:
    - condition_type:
        type: resource_type
      operator: equals
      value: "aws_instance"
    - condition_type:
        type: monthly_cost
      operator: greater_than
      value: 1000.0
  action:
    type: block
    message: "EC2 instance exceeds monthly budget of $1000"

- name: "Require Approval for NAT Gateways"
  description: "NAT Gateways require approval from network team"
  enabled: true
  severity: Medium
  conditions:
    - condition_type:
        type: resource_type
      operator: equals
      value: "aws_nat_gateway"
  action:
    type: require_approval
    approvers:
      - "network-team@example.com"
    message: "NAT Gateway creation requires network team approval"

- name: "Warn on High Cost Increase"
  description: "Warn when cost increases by more than 50%"
  enabled: true
  severity: Medium
  conditions:
    - condition_type:
        type: cost_increase
      operator: greater_than
      value: 50.0
  action:
    type: warn
    message: "Cost increase exceeds 50% threshold"

- name: "Restrict Instance Types"
  description: "Only allow t3, t4g, or m6i instance families"
  enabled: true
  severity: High
  conditions:
    - condition_type:
        type: resource_type
      operator: equals
      value: "aws_instance"
    - condition_type:
        type: resource_attribute
        attribute: "instance_type"
      operator: not_in
      value:
        - "t3"
        - "t4g"
        - "m6i"
  action:
    type: block
    message: "Only t3, t4g, or m6i instance families are allowed"
"#.to_string()
}

fn generate_example_json() -> String {
    serde_json::to_string_pretty(&serde_json::json!([
        {
            "name": "Block Expensive EC2 Instances",
            "description": "Prevent creation of EC2 instances that cost more than $1000/month",
            "enabled": true,
            "severity": "High",
            "conditions": [
                {
                    "condition_type": { "type": "resource_type" },
                    "operator": "equals",
                    "value": "aws_instance"
                },
                {
                    "condition_type": { "type": "monthly_cost" },
                    "operator": "greater_than",
                    "value": 1000.0
                }
            ],
            "action": {
                "type": "block",
                "message": "EC2 instance exceeds monthly budget of $1000"
            }
        }
    ])).unwrap_or_default()
}
