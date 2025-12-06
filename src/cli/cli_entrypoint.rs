// CLI entrypoint for CostPilot

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER: &str = r#"
   ____          _   ____  _ _       _   
  / ___|___  ___| |_|  _ \(_) | ___ | |_ 
 | |   / _ \/ __| __| |_) | | |/ _ \| __|
 | |__| (_) \__ \ |_|  __/| | | (_) | |_ 
  \____\___/|___/\__|_|   |_|_|\___/ \__|
                                          
"#;

#[derive(Parser)]
#[command(name = "costpilot")]
#[command(about = "Zero-IAM FinOps engine for Terraform, CDK, and CloudFormation", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (json, text, markdown)
    #[arg(short = 'f', long, global = true, default_value = "text")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan Terraform plan for cost issues and predictions
    /// 
    /// Examples:
    ///   costpilot scan --plan tfplan.json
    ///   costpilot scan --plan tfplan.json --explain
    ///   costpilot scan --plan tfplan.json --policy policy.yaml
    Scan(crate::cli::scan::ScanCommand),

    /// Compare cost between two Terraform plans
    /// 
    /// Shows the cost difference between a baseline (before) plan and
    /// a proposed (after) plan. Useful for PR reviews and change impact analysis.
    /// 
    /// Examples:
    ///   costpilot diff --before baseline.json --after new-plan.json
    ///   costpilot diff -b baseline.json -a new-plan.json --format json
    ///   costpilot diff -b old.json -a new.json --verbose
    Diff {
        /// Path to baseline (before) plan file
        #[arg(short, long)]
        before: PathBuf,

        /// Path to proposed (after) plan file
        #[arg(short, long)]
        after: PathBuf,
    },

    /// Generate autofix recommendations for cost optimization
    /// 
    /// Modes:
    ///   snippet - Generate code snippets with explanations (default)
    ///   patch   - Generate unified diff patches ready to apply
    /// 
    /// Examples:
    ///   costpilot autofix --plan tfplan.json
    ///   costpilot autofix --mode patch --plan tfplan.json
    ///   costpilot autofix --mode patch --plan tfplan.json --drift-safe
    Autofix {
        /// Mode: snippet or patch
        #[arg(short, long, default_value = "snippet")]
        mode: String,

        /// Path to plan file
        #[arg(short, long)]
        plan: Option<PathBuf>,

        /// Enable drift-safe mode (Beta)
        #[arg(long)]
        drift_safe: bool,
    },

    /// Initialize CostPilot configuration in current directory
    /// 
    /// Creates costpilot.yaml and optionally generates CI/CD templates
    /// for GitHub Actions, GitLab CI, or other providers.
    /// 
    /// Examples:
    ///   costpilot init
    ///   costpilot init --no-ci
    Init {
        /// Skip creating CI template
        #[arg(long)]
        no_ci: bool,
    },

    /// Generate dependency map
    Map(crate::cli::map::MapCommand),

    /// Check cost SLOs
    Slo {
        #[command(subcommand)]
        command: Option<SloCommands>,
    },

    /// Manage policy lifecycle and approvals
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },

    /// Audit log and compliance reporting
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },

    /// Manage cost heuristics
    Heuristics {
        #[command(subcommand)]
        command: crate::cli::heuristics::HeuristicsCommand,
    },

    /// Explain cost predictions with stepwise reasoning
    Explain {
        #[command(subcommand)]
        command: crate::cli::explain::ExplainCommand,
    },

    /// Manage custom policy rules (DSL)
    PolicyDsl {
        #[command(flatten)]
        command: crate::cli::policy_dsl::PolicyDslCommand,
    },

    /// Group resources for cost allocation
    Group(crate::cli::group::GroupCommand),

    /// Validate configuration files
    /// 
    /// Validates configuration files for errors and provides helpful hints.
    /// Supports: costpilot.yaml, policy files, baselines.json, slo.yaml
    /// 
    /// Examples:
    ///   costpilot validate costpilot.yaml
    ///   costpilot validate policy.yaml
    ///   costpilot validate baselines.json --format json
    Validate {
        /// Path to file(s) to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Fail on first error
        #[arg(long)]
        fail_fast: bool,
    },

    /// Show version information
    Version {
        /// Show detailed version info
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum SloCommands {
    /// Check SLO compliance
    Check,
    
    /// Calculate burn rate and predict time-to-breach
    Burn {
        /// Path to SLO configuration file
        #[arg(short, long)]
        slo: Option<PathBuf>,

        /// Path to snapshots directory
        #[arg(short = 'd', long)]
        snapshots: Option<PathBuf>,

        /// Minimum number of snapshots required
        #[arg(long, default_value = "3")]
        min_snapshots: usize,

        /// Minimum R² threshold for predictions
        #[arg(long, default_value = "0.7")]
        min_r_squared: f64,
    },
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Submit policy for approval
    Submit {
        /// Path to policy file
        #[arg(short, long)]
        policy: PathBuf,

        /// Approvers (comma-separated emails)
        #[arg(short, long, value_delimiter = ',')]
        approvers: Vec<String>,
    },

    /// Approve a policy
    Approve {
        /// Policy ID
        policy_id: String,

        /// Approver email/ID
        #[arg(short, long)]
        approver: String,

        /// Optional comment
        #[arg(short, long)]
        comment: Option<String>,
    },

    /// Reject a policy
    Reject {
        /// Policy ID
        policy_id: String,

        /// Approver email/ID
        #[arg(short, long)]
        approver: String,

        /// Reason for rejection
        #[arg(short, long)]
        reason: String,
    },

    /// Activate an approved policy
    Activate {
        /// Policy ID
        policy_id: String,

        /// Actor performing activation
        #[arg(short, long, default_value = "system")]
        actor: String,
    },

    /// Deprecate an active policy
    Deprecate {
        /// Policy ID
        policy_id: String,

        /// Actor performing deprecation
        #[arg(short, long, default_value = "system")]
        actor: String,

        /// Reason for deprecation
        #[arg(short, long)]
        reason: String,
    },

    /// Show policy status
    Status {
        /// Policy ID
        policy_id: String,
    },

    /// Show policy version history
    History {
        /// Policy ID
        policy_id: String,
    },

    /// Compare two policy versions
    Diff {
        /// Policy ID
        policy_id: String,

        /// From version
        #[arg(short, long)]
        from: String,

        /// To version
        #[arg(short, long)]
        to: String,
    },
}

#[derive(Subcommand)]
enum AuditCommands {
    /// View audit log entries
    View {
        /// Filter by event type
        #[arg(long)]
        event_type: Option<String>,

        /// Filter by actor
        #[arg(long)]
        actor: Option<String>,

        /// Filter by resource
        #[arg(long)]
        resource: Option<String>,

        /// Filter by severity (low, medium, high, critical)
        #[arg(long)]
        severity: Option<String>,

        /// Show last N entries
        #[arg(short = 'n', long)]
        last_n: Option<usize>,
    },

    /// Verify audit log integrity
    Verify,

    /// Generate compliance report
    Compliance {
        /// Compliance framework (SOC2, ISO27001, GDPR, HIPAA, PCI_DSS)
        #[arg(short = 'f', long)]
        framework: String,

        /// Report period in days
        #[arg(short, long, default_value = "30")]
        days: u32,
    },

    /// Export audit log
    Export {
        /// Export format (ndjson, csv, json)
        #[arg(short = 'f', long, default_value = "ndjson")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Show audit log statistics
    Stats,

    /// Record a manual audit event
    Record {
        /// Event type
        #[arg(long)]
        event_type: String,

        /// Actor
        #[arg(long)]
        actor: String,

        /// Resource ID
        #[arg(long)]
        resource_id: String,

        /// Resource type
        #[arg(long)]
        resource_type: String,

        /// Description
        #[arg(long)]
        description: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Print banner for interactive mode
    if atty::is(atty::Stream::Stdout) {
        println!("{}", BANNER.bright_cyan());
        println!("{}", format!("v{} | Zero-IAM FinOps Engine", VERSION).bright_black());
        println!();
    }

    let result = match cli.command {
        Commands::Scan(scan_cmd) => {
            scan_cmd.execute().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::Diff { before, after } => {
            cmd_diff(before, after, &cli.format, cli.verbose)
        }
        Commands::Autofix { mode, plan, drift_safe } => {
            cmd_autofix(mode, plan, drift_safe, &cli.format, cli.verbose)
        }
        Commands::Init { no_ci } => {
            cmd_init(no_ci, cli.verbose)
        }
        Commands::Map(map_cmd) => {
            crate::cli::map::execute_map_command(map_cmd)
        }
        Commands::Slo { command } => {
            cmd_slo(command, &cli.format, cli.verbose)
        }
        Commands::Policy { command } => {
            cmd_policy(command, &cli.format, cli.verbose)
        }
        Commands::Audit { command } => {
            cmd_audit(command, &cli.format, cli.verbose)
        }
        Commands::Heuristics { command } => {
            cmd_heuristics(command, &cli.format, cli.verbose)
        }
        Commands::Explain { command } => {
            cmd_explain(command, &cli.format, cli.verbose)
        }
        Commands::PolicyDsl { command } => {
            crate::cli::policy_dsl::execute_policy_dsl_command(command)
        }
        Commands::Group(group_cmd) => {
            crate::cli::group::execute_group_command(group_cmd)
        }
        Commands::Validate { files, fail_fast } => {
            cmd_validate(files, &cli.format, fail_fast)
        }
        Commands::Version { detailed } => {
            cmd_version(detailed);
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".bright_red().bold(), e);
        process::exit(1);
    }
}

fn cmd_diff(
    before: PathBuf,
    after: PathBuf,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::commands::diff;
    diff::execute(before, after, format, verbose)
}

fn cmd_autofix(
    mode: String,
    plan: Option<PathBuf>,
    drift_safe: bool,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if drift_safe {
        println!("{}", "⚠️  Drift-safe mode is not yet implemented (V1)".yellow());
        return Ok(());
    }

    let plan_path = plan.ok_or("--plan argument is required")?;

    match mode.as_str() {
        "snippet" => {
            use crate::cli::commands::autofix_snippet;
            let args = autofix_snippet::AutofixSnippetArgs {
                plan: plan_path,
                verbose,
            };
            autofix_snippet::execute(&args)
        }
        "patch" => {
            use crate::cli::commands::autofix_patch;
            let args = autofix_patch::AutofixPatchArgs {
                plan: plan_path,
                output: None,
                apply: false,
                verbose,
            };
            autofix_patch::execute(&args)
        }
        _ => {
            Err(format!("Unknown mode: {}. Valid modes: snippet, patch", mode).into())
        }
    }
}

fn cmd_init(no_ci: bool, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::init;
    
    let ci_provider = if no_ci {
        "none"
    } else {
        // Auto-detect CI provider or default to GitHub
        if std::path::Path::new(".github").exists() {
            "github"
        } else if std::path::Path::new(".gitlab-ci.yml").exists() {
            "gitlab"
        } else {
            "github"  // Default to GitHub
        }
    };
    
    if verbose {
        println!("  CI Provider: {}", ci_provider);
    }
    
    init::init(".", ci_provider)?;
    
    Ok(())
}

fn cmd_map(
    format: String,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("🗺️  Generating dependency map ({})...", format).bright_blue().bold());
    
    // TODO: Implement map logic
    println!("{}", "✅ Map complete (not yet implemented)".bright_green());
    
    Ok(())
}

fn cmd_slo(
    command: Option<SloCommands>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Some(SloCommands::Check) => {
            println!("{}", "📋 Checking SLO compliance...".bright_blue().bold());
            // TODO: Implement SLO check logic
            println!("{}", "✅ SLO check complete (not yet implemented)".bright_green());
        }
        Some(SloCommands::Burn { slo, snapshots, min_snapshots, min_r_squared }) => {
            println!("{}", "🔥 Calculating burn rate...".bright_blue().bold());
            crate::cli::commands::slo_burn::execute(
                slo,
                snapshots,
                format,
                Some(min_snapshots),
                Some(min_r_squared),
                verbose,
            )?;
        }
        None => {
            println!("{}", "📋 Checking SLO compliance...".bright_blue().bold());
            // TODO: Implement SLO check logic
            println!("{}", "✅ SLO check complete (not yet implemented)".bright_green());
        }
    }
    
    Ok(())
}

fn cmd_policy(
    command: PolicyCommands,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::commands::policy_lifecycle;

    match command {
        PolicyCommands::Submit { policy, approvers } => {
            policy_lifecycle::cmd_submit(policy, approvers, format, verbose)
        }
        PolicyCommands::Approve { policy_id, approver, comment } => {
            policy_lifecycle::cmd_approve(policy_id, approver, comment, format, verbose)
        }
        PolicyCommands::Reject { policy_id, approver, reason } => {
            policy_lifecycle::cmd_reject(policy_id, approver, reason, format, verbose)
        }
        PolicyCommands::Activate { policy_id, actor } => {
            policy_lifecycle::cmd_activate(policy_id, actor, format, verbose)
        }
        PolicyCommands::Deprecate { policy_id, actor, reason } => {
            policy_lifecycle::cmd_deprecate(policy_id, actor, reason, format, verbose)
        }
        PolicyCommands::Status { policy_id } => {
            policy_lifecycle::cmd_status(policy_id, format, verbose)
        }
        PolicyCommands::History { policy_id } => {
            policy_lifecycle::cmd_history(policy_id, format, verbose)
        }
        PolicyCommands::Diff { policy_id, from, to } => {
            policy_lifecycle::cmd_diff(policy_id, from, to, format, verbose)
        }
    }
}

fn cmd_audit(
    command: AuditCommands,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::commands::audit;

    match command {
        AuditCommands::View {
            event_type,
            actor,
            resource,
            severity,
            last_n,
        } => audit::cmd_audit_view(event_type, actor, resource, severity, last_n, format, verbose),
        AuditCommands::Verify => audit::cmd_audit_verify(format, verbose),
        AuditCommands::Compliance { framework, days } => {
            audit::cmd_audit_compliance(framework, days, format, verbose)
        }
        AuditCommands::Export {
            format: output_format,
            output,
        } => audit::cmd_audit_export(output_format, output, format, verbose),
        AuditCommands::Stats => audit::cmd_audit_stats(format, verbose),
        AuditCommands::Record {
            event_type,
            actor,
            resource_id,
            resource_type,
            description,
        } => audit::cmd_audit_record(
            event_type,
            actor,
            resource_id,
            resource_type,
            description,
            format,
            verbose,
        ),
    }
}

fn cmd_heuristics(
    command: crate::cli::heuristics::HeuristicsCommand,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::heuristics::execute_heuristics_command;
    
    let output = execute_heuristics_command(command)?;
    println!("{}", output);
    
    Ok(())
}

fn cmd_explain(
    command: crate::cli::explain::ExplainCommand,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::explain::execute_explain_command;
    
    let output = execute_explain_command(command)?;
    println!("{}", output);
    
    Ok(())
}

fn cmd_validate(
    files: Vec<PathBuf>,
    format: &str,
    fail_fast: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::cli::commands::validate;
    
    if files.len() == 1 {
        validate::execute(files[0].clone(), format.to_string())
    } else {
        validate::execute_batch(files, format.to_string(), fail_fast)
    }
}

fn cmd_version(detailed: bool) {
    if detailed {
        println!("{}", "CostPilot".bright_cyan().bold());
        println!("Version: {}", VERSION);
        println!("Build: {} (deterministic)", env!("CARGO_PKG_VERSION"));
        println!("Features: Zero-IAM, WASM-safe, Offline");
        println!("License: MIT");
    } else {
        println!("costpilot {}", VERSION);
    }
}
