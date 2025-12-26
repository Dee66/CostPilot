// CLI entrypoint for CostPilot

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::process;
use costpilot::cli::commands::autofix_patch::AutofixPatchArgs;
use costpilot::cli::commands::autofix_snippet::AutofixSnippetArgs;

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

    /// Enable debug mode (shows internal operations and timing)
    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan Terraform plan for cost issues and predictions
    ///
    /// Examples:
    ///   costpilot scan --plan tfplan.json
    ///   costpilot scan --plan tfplan.json --explain
    ///   costpilot scan --plan tfplan.json --policy policy.yaml
    Scan(costpilot::cli::scan::ScanCommand),

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
        /// Path to baseline (before) plan file (positional or --before)
        #[arg(value_name = "BEFORE")]
        before: PathBuf,

        /// Path to proposed (after) plan file (positional or --after)
        #[arg(value_name = "AFTER")]
        after: PathBuf,
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

        /// Path to initialize into (optional)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Generate dependency map
    Map(costpilot::cli::map::MapCommand),

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
        command: costpilot::cli::heuristics::HeuristicsCommand,
    },

    /// Explain cost predictions with stepwise reasoning
    Explain {
        /// Either use the structured subcommands (resource/all)
        /// or the simple flattened form: `costpilot explain <resource_type> --instance-type ..`
        #[command(subcommand)]
        command: Option<costpilot::cli::explain::ExplainCommand>,

        /// Flattened arguments for the simple form
        #[command(flatten)]
        args: Option<costpilot::cli::explain::ExplainArgs>,
    },

    /// Performance related commands
    Performance {
        #[command(subcommand)]
        command: Option<PerformanceCli>,
    },

    /// SLO commands (burn/check)
    Slo {
        #[command(subcommand)]
        command: Option<SloCli>,
    },

    /// SLO - legacy flattened commands
    SloCheck,
    SloBurn {
        #[arg(short, long)]
        config: Option<PathBuf>,
        #[arg(short = 's', long = "snapshots-dir")]
        snapshots: Option<PathBuf>,
        #[arg(long)]
        min_snapshots: Option<usize>,
        #[arg(long)]
        min_r_squared: Option<f64>,
        #[arg(short, long)]
        verbose: bool,
    },

    /// Autofix - snippet mode
    AutofixSnippet {
        /// Path to Terraform plan JSON file
        #[arg(long, value_name = "FILE")]
        plan: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Autofix - patch mode
    AutofixPatch(AutofixPatchArgs),

    /// Escrow management
    Escrow {
        #[command(subcommand)]
        command: Option<EscrowCli>,
    },

    /// Policy lifecycle top-level alias
    PolicyLifecycle {
        #[command(subcommand)]
        command: Option<PolicyLifecycleCli>,
    },

    /// Usage and chargeback commands
    Usage {
        /// Days in a given month (YYYY-MM)
        #[arg(long = "days-in-month")]
        days_in_month: Option<String>,

        #[command(subcommand)]
        command: Option<UsageCli>,
    },

    /// Manage custom policy rules (DSL)
    PolicyDsl {
        #[command(flatten)]
        command: costpilot::cli::policy_dsl::PolicyDslCommand,
    },

    /// Group resources for cost allocation
    Group(costpilot::cli::group::GroupCommand),

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
        #[arg(long)]
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

// Lightweight CLI wrappers so top-level commands expose the expected subcommands
#[derive(Subcommand, Debug)]
enum PerformanceCli {
    Budgets,
    SetBaseline {
        #[arg(long = "from-file")]
        from_file: Option<PathBuf>,
    },
    Stats,
    CheckRegressions { report_file: PathBuf },
    History { engine: Option<String>, limit: Option<usize> },
}

#[derive(Subcommand, Debug)]
enum SloCli {
    Check,
    Burn {
        #[arg(short, long)]
        config: Option<PathBuf>,
        #[arg(short = 'd', long)]
        snapshots: Option<PathBuf>,
        #[arg(long)]
        min_snapshots: Option<usize>,
        #[arg(long)]
        min_r_squared: Option<f64>,
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
enum EscrowCli {
    Create { version: String, output_dir: Option<PathBuf>, include_artifacts: bool },
    Verify { package_dir: PathBuf },
    Playbook { package_dir: PathBuf, output: Option<PathBuf> },
    Recover { package_dir: PathBuf, working_dir: PathBuf },
    Configure { vendor_name: String, contact_email: String, support_url: String },
    List { escrow_dir: Option<PathBuf> },
}

#[derive(Subcommand, Debug)]
enum PolicyLifecycleCli {
    Submit { #[arg(short, long)] policy: PathBuf, #[arg(short, long, value_delimiter = ',')] approvers: Vec<String> },
    Approve { policy_id: String, #[arg(short, long)] approver: String, #[arg(short, long)] comment: Option<String> },
    Reject { policy_id: String, #[arg(short, long)] approver: String, #[arg(short, long)] reason: String },
    Activate { policy_id: String, #[arg(short, long, default_value = "system")] actor: String },
    Deprecate { policy_id: String, #[arg(short, long, default_value = "system")] actor: String, #[arg(short, long)] reason: Option<String> },
    Status { policy_id: String },
    History { policy_id: String },
    Diff { policy_id: String, #[arg(short, long)] from: String, #[arg(short, long)] to: String },
}

#[derive(Subcommand, Debug)]
enum UsageCli {
    Report { team_id: String, start: Option<String>, end: Option<String>, format: Option<String> },
    Export { start: String, end: String, format: String, output: Option<PathBuf> },
    Pr { pr_number: u32, repository: Option<String> },
    Chargeback { org_id: String, start: String, end: String, format: String, output: Option<PathBuf> },
    Invoice { team_id: String, start: String, end: String },
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
    // Load edition context BEFORE parsing CLI
    // This allows us to gate premium commands early
    let edition = costpilot::edition::detect_edition().unwrap_or_else(|_| {
        costpilot::edition::EditionContext::free()
    });

    // Intercept --version/-V to show edition
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        let arg = &args[1];
        if arg == "--version" || arg == "-V" {
            let edition_str = if edition.is_premium() { "Premium" } else { "Free" };
            println!("costpilot {} ({})", VERSION, edition_str);
            return;
        }
    }

    // Check for premium commands in Free mode and fail early
    if edition.is_free() && args.len() >= 2 {
        let premium_commands = ["autofix", "patch", "slo"];
        let command = args[1].to_lowercase();

        if premium_commands.contains(&command.as_str()) {
            eprintln!("{} Unknown command '{}'", "Error:".bright_red().bold(), command);
            eprintln!();
            eprintln!("This command requires CostPilot Premium.");
            eprintln!("Upgrade at: https://shieldcraft-ai.com/costpilot/upgrade");
            process::exit(1);
        }
    }

    let cli = Cli::parse();

    // Debug mode removed for release builds (developer-only output suppressed)

    // Print banner for interactive mode
    if atty::is(atty::Stream::Stdout) {
        println!("{}", BANNER.bright_cyan());
        println!(
            "{}",
            format!("v{} | Zero-IAM FinOps Engine", VERSION).bright_black()
        );
        println!();
    }

    let start_time: Option<std::time::Instant> = None;

    let result = match cli.command {
        Commands::Scan(scan_cmd) => {
            scan_cmd
                .execute_with_edition(&edition, &cli.format)
                .map_err(|e| format!("{}", e).into())
        }
        Commands::Diff { before, after } => {
            cmd_diff(before, after, &cli.format, cli.verbose, &edition)
        }
        Commands::Init { no_ci, path } => {
            cmd_init(no_ci, path, cli.verbose)
        }
        Commands::Map(map_cmd) => {
            costpilot::cli::map::execute_map_command(&map_cmd, &edition)
        }
        Commands::Performance { command } => {

            use costpilot::cli::performance as perf;
            let res = match command {
                Some(PerformanceCli::Budgets) => perf::execute_performance_command(perf::PerformanceCommand::Budgets),
                Some(PerformanceCli::SetBaseline { from_file }) => {
                    perf::execute_performance_command(perf::PerformanceCommand::SetBaseline { from_file })
                }
                Some(PerformanceCli::Stats) => perf::execute_performance_command(perf::PerformanceCommand::Stats),
                Some(PerformanceCli::CheckRegressions { report_file }) => perf::execute_performance_command(perf::PerformanceCommand::CheckRegressions { report_file }),
                Some(PerformanceCli::History { engine, limit }) => perf::execute_performance_command(perf::PerformanceCommand::History { engine, limit }),
                None => perf::execute_performance_command(perf::PerformanceCommand::Budgets),
            };
            match res {
                Ok(out) => { println!("{}", out); Ok(()) },
                Err(e) => Err(e.into()),
            }
        }
        Commands::Policy { command } => {
            cmd_policy(command, &cli.format, cli.verbose, &edition)
        }
        Commands::Slo { command } => {

            match command {
                Some(SloCli::Check) => cmd_slo(Some(SloCommands::Check), &cli.format, cli.verbose, &edition),
                Some(SloCli::Burn { config, snapshots, min_snapshots, min_r_squared, verbose }) => {
                    cmd_slo(
                        Some(SloCommands::Burn {
                            slo: config,
                            snapshots,
                            min_snapshots: min_snapshots.unwrap_or(3),
                            min_r_squared: min_r_squared.unwrap_or(0.7),
                        }),
                        &cli.format,
                        verbose || cli.verbose,
                        &edition,
                    )
                }
                None => cmd_slo(None, &cli.format, cli.verbose, &edition),
            }
        }
        Commands::SloCheck => {
            cmd_slo(Some(SloCommands::Check), &cli.format, cli.verbose, &edition)
        }
        Commands::SloBurn { config, snapshots, min_snapshots, min_r_squared, verbose } => {

            cmd_slo(
                Some(SloCommands::Burn {
                    slo: config,
                    snapshots,
                    min_snapshots: min_snapshots.unwrap_or(3),
                    min_r_squared: min_r_squared.unwrap_or(0.7),
                }),
                &cli.format,
                verbose || cli.verbose,
                &edition,
            )
        }
        Commands::Audit { command } => {
            cmd_audit(command, &cli.format, cli.verbose)
        }
        Commands::Heuristics { command } => {
            cmd_heuristics(command, &cli.format, cli.verbose)
        }
        Commands::Explain { command, args } =>
        cmd_explain(command, args, &cli.format, cli.verbose, &edition),
        Commands::AutofixSnippet { plan, verbose } => {

            if plan.is_none() {
                Err("--plan is required for autofix-snippet".into())
            } else {
                let plan_path = plan.unwrap();
                let args = AutofixSnippetArgs { plan: plan_path, verbose };
                match costpilot::cli::commands::autofix_snippet::execute(&args, &edition) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("{}", e).into()),
                }
            }
        }
        Commands::AutofixPatch(args) => {
            match costpilot::cli::commands::autofix_patch::execute(&args, &edition) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}", e).into()),
            }
        }
        Commands::PolicyDsl { command } => {
            costpilot::cli::policy_dsl::execute_policy_dsl_command(&command)
        }
        Commands::Escrow { command } => {

            use costpilot::cli::escrow as ec;
            let res = match command {
                Some(EscrowCli::Create { version, output_dir, include_artifacts }) => ec::execute_escrow_command(ec::EscrowCommand::Create { version, output_dir, include_artifacts }),
                Some(EscrowCli::Verify { package_dir }) => ec::execute_escrow_command(ec::EscrowCommand::Verify { package_dir }),
                Some(EscrowCli::Playbook { package_dir, output }) => ec::execute_escrow_command(ec::EscrowCommand::Playbook { package_dir, output }),
                Some(EscrowCli::Recover { package_dir, working_dir }) => ec::execute_escrow_command(ec::EscrowCommand::Recover { package_dir, working_dir }),
                Some(EscrowCli::Configure { vendor_name, contact_email, support_url }) => ec::execute_escrow_command(ec::EscrowCommand::Configure { vendor_name, contact_email, support_url }),
                Some(EscrowCli::List { escrow_dir }) => ec::execute_escrow_command(ec::EscrowCommand::List { escrow_dir }),
                None => ec::execute_escrow_command(ec::EscrowCommand::List { escrow_dir: None }),
            };
            match res { Ok(out) => { println!("{}", out); Ok(()) }, Err(e) => Err(e.into()) }
        }
        Commands::PolicyLifecycle { command } => {

            use costpilot::cli::commands::policy_lifecycle as pl;
            match command {
                Some(PolicyLifecycleCli::Submit { policy, approvers }) => pl::cmd_submit(policy, approvers, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Approve { policy_id, approver, comment }) => pl::cmd_approve(policy_id, approver, comment, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Reject { policy_id, approver, reason }) => pl::cmd_reject(policy_id, approver, reason, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Activate { policy_id, actor }) => pl::cmd_activate(policy_id, actor, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Deprecate { policy_id, actor, reason }) => pl::cmd_deprecate(policy_id, actor, reason.unwrap_or_default(), &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Status { policy_id }) => pl::cmd_status(policy_id, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::History { policy_id }) => pl::cmd_history(policy_id, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Diff { policy_id, from, to }) => pl::cmd_diff(policy_id, from, to, &cli.format, cli.verbose, &edition).map_err(|e| format!("{}", e).into()),
                None => Err("No policy-lifecycle subcommand provided".into()),
            }
        }
        Commands::Usage { days_in_month, command } => {


            let usage_override: Option<Result<String, String>> = if let Some(d) = days_in_month {
                // Expect YYYY-MM
                let parts: Vec<&str> = d.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(y), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                        let days = costpilot::cli::usage::days_in_month(y, m);
                        Some(Ok(days.to_string()))
                    } else {
                        Some(Err("Invalid --days-in-month format, expected YYYY-MM".to_string()))
                    }
                } else {
                    Some(Err("Invalid --days-in-month format, expected YYYY-MM".to_string()))
                }
            } else {
                None
            };

            use costpilot::cli::usage as usage_mod;

            let res: Result<String, String> = match usage_override {
                Some(r) => r,
                None => match command {
                    Some(UsageCli::Report { team_id, start, end, format: _ }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Report { team_id, start, end, format: usage_mod::OutputFormat::Text }),
                    Some(UsageCli::Export { start, end, format: _, output }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Export { start, end, format: usage_mod::ExportFormat::Json, output }),
                    Some(UsageCli::Pr { pr_number, repository }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Pr { pr_number, repository }),
                    Some(UsageCli::Chargeback { org_id, start, end, format: _, output }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Chargeback { org_id, start, end, format: usage_mod::OutputFormat::Text, output }),
                    Some(UsageCli::Invoice { team_id, start, end }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Invoice { team_id, start, end }),
                    None => usage_mod::execute_usage_command(usage_mod::UsageCommand::Report { team_id: "all".to_string(), start: None, end: None, format: usage_mod::OutputFormat::Text }),
                },
            };

            match res { Ok(out) => { println!("{}", out); Ok(()) }, Err(e) => Err(e.into()) }
        }
        Commands::Group(group_cmd) => {
            costpilot::cli::group::execute_group_command(group_cmd, &edition)
        }
        Commands::Validate { files, fail_fast } => {
            cmd_validate(files, &cli.format, fail_fast, &edition)
        }
        Commands::Version { detailed } => {
            cmd_version(detailed, &edition);
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
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::commands::diff;
    diff::execute(before, after, format, verbose, edition)
}

#[allow(dead_code)]
fn cmd_autofix(
    mode: String,
    plan: Option<PathBuf>,
    drift_safe: bool,
    _format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check edition for premium features
    if edition.is_free() {
        return Err(
            "Autofix requires Premium edition. Upgrade at https://costpilot.io/upgrade".into(),
        );
    }

    if drift_safe {
        println!(
            "{}",
            "⚠️  Drift-safe mode is not yet implemented (V1)".yellow()
        );
        return Ok(());
    }

    let plan_path = plan.ok_or("--plan argument is required")?;

    match mode.as_str() {
        "snippet" => {
            use costpilot::cli::commands::autofix_snippet;
            let args = autofix_snippet::AutofixSnippetArgs {
                plan: plan_path,
                verbose,
            };
            autofix_snippet::execute(&args, edition)
        }
        "patch" => {
            use costpilot::cli::commands::autofix_patch;
            let args = autofix_patch::AutofixPatchArgs {
                plan: plan_path,
                output: None,
                apply: false,
                verbose,
            };
            autofix_patch::execute(&args, edition)
        }
        _ => Err(format!("Unknown mode: {}. Valid modes: snippet, patch", mode).into()),
    }
}

fn cmd_init(no_ci: bool, path: Option<PathBuf>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::init::init;
    let ci_provider = if no_ci {
        "none"
    } else {
        // Auto-detect CI provider or default to GitHub
        if std::path::Path::new(".github").exists() {
            "github"
        } else if std::path::Path::new(".gitlab-ci.yml").exists() {
            "gitlab"
        } else {
            "github" // Default to GitHub
        }
    };

    if verbose {
        println!("  CI Provider: {}", ci_provider);
    }

    let target_path = path.unwrap_or_else(|| PathBuf::from("."));

    init(target_path.to_str().unwrap_or("."), ci_provider)?;

    Ok(())
}

#[allow(dead_code)]
fn cmd_slo(
    command: Option<SloCommands>,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Gate SLO features behind Premium edition to match CLI integration tests
    if edition.is_free() {
        return Err("SLO features require premium license".into());
    }

    match command {
        Some(SloCommands::Check) => {
            println!("{}", "📋 Checking SLO compliance...".bright_blue().bold());
            costpilot::cli::commands::slo_check::execute(None, None, format, verbose, edition)?;
        }
        Some(SloCommands::Burn {
            slo,
            snapshots,
            min_snapshots,
            min_r_squared,
        }) => {
            println!("{}", "🔥 Calculating burn rate...".bright_blue().bold());
            costpilot::cli::commands::slo_burn::execute(
                slo,
                snapshots,
                format,
                Some(min_snapshots),
                Some(min_r_squared),
                verbose,
                edition,
            )?;
        }
        None => {
            println!("{}", "📋 Checking SLO compliance...".bright_blue().bold());
            costpilot::cli::commands::slo_check::execute(None, None, format, verbose, edition)?;
        }
    }

    Ok(())
}

fn cmd_policy(
    command: PolicyCommands,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::commands::policy_lifecycle;

    match command {
        PolicyCommands::Submit { policy, approvers } => {
            policy_lifecycle::cmd_submit(policy, approvers, format, verbose, edition)
        }
        PolicyCommands::Approve {
            policy_id,
            approver,
            comment,
        } => policy_lifecycle::cmd_approve(policy_id, approver, comment, format, verbose, edition),
        PolicyCommands::Reject {
            policy_id,
            approver,
            reason,
        } => policy_lifecycle::cmd_reject(policy_id, approver, reason, format, verbose, edition),
        PolicyCommands::Activate { policy_id, actor } => {
            policy_lifecycle::cmd_activate(policy_id, actor, format, verbose, edition)
        }
        PolicyCommands::Deprecate {
            policy_id,
            actor,
            reason,
        } => policy_lifecycle::cmd_deprecate(policy_id, actor, reason, format, verbose, edition),
        PolicyCommands::Status { policy_id } => {
            policy_lifecycle::cmd_status(policy_id, format, verbose, edition)
        }
        PolicyCommands::History { policy_id } => {
            policy_lifecycle::cmd_history(policy_id, format, verbose, edition)
        }
        PolicyCommands::Diff {
            policy_id,
            from,
            to,
        } => policy_lifecycle::cmd_diff(policy_id, from, to, format, verbose, edition),
    }
}

fn cmd_audit(
    command: AuditCommands,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::commands::audit;

    match command {
        AuditCommands::View {
            event_type,
            actor,
            resource,
            severity,
            last_n,
        } => audit::cmd_audit_view(
            event_type, actor, resource, severity, last_n, format, verbose,
        ),
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
    command: costpilot::cli::heuristics::HeuristicsCommand,
    _format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::heuristics::execute_heuristics_command;

    let output = execute_heuristics_command(command)?;
    println!("{}", output);

    Ok(())
}

fn cmd_explain(
    command: Option<costpilot::cli::explain::ExplainCommand>,
    args: Option<costpilot::cli::explain::ExplainArgs>,
    _format: &str,
    _verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::explain::{execute_explain_args, execute_explain_command};

    let output = if let Some(a) = args {
        execute_explain_args(a, edition)?
    } else if let Some(cmd) = command {
        execute_explain_command(cmd, edition)?
    } else {
        return Err("No explain arguments provided".into());
    };

    println!("{}", output);

    Ok(())
}

fn cmd_validate(
    files: Vec<PathBuf>,
    format: &str,
    fail_fast: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::commands::validate;

    if files.len() == 1 {
        validate::execute(files[0].clone(), format.to_string(), edition)
    } else {
        validate::execute_batch(files, format.to_string(), fail_fast, edition)
    }
}

fn cmd_version(detailed: bool, edition: &costpilot::edition::EditionContext) {
    // Validate version matches Cargo.toml
    let cargo_version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        VERSION, cargo_version,
        "VERSION constant must match CARGO_PKG_VERSION"
    );

    let edition_str = if edition.is_premium() {
        "Premium"
    } else {
        "Free"
    };

    if detailed {
        println!("{}", "CostPilot".bright_cyan().bold());
        println!("Version: {}", VERSION);
        println!("Edition: {}", edition_str);
        println!("Build: {} (deterministic)", cargo_version);
        println!("Features: Zero-IAM, WASM-safe, Offline");
        println!("License: MIT");
    } else {
        println!("costpilot {} ({})", VERSION, edition_str);
    }
}
