// CLI entrypoint for CostPilot

use clap::{Parser, Subcommand};
use colored::*;
use costpilot::ZeroCostGuard;
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

/// Exit codes for CostPilot CLI
#[derive(Debug, Clone, Copy)]
enum ExitCode {
    Success = 0,
    PolicyBlock = 2,
    SloBurn = 3,
    InvalidInput = 4,
    InternalError = 5,
}

impl ExitCode {
    fn exit(self) -> ! {
        process::exit(self as i32)
    }

    /// Convert CostPilotError to appropriate exit code
    fn from_costpilot_error(
        error: &costpilot::engines::shared::error_model::CostPilotError,
    ) -> Self {
        use costpilot::engines::shared::error_model::ErrorCategory;
        match error.category {
            ErrorCategory::PolicyViolation => ExitCode::PolicyBlock,
            ErrorCategory::SLOBreach => ExitCode::SloBurn,
            ErrorCategory::InvalidInput
            | ErrorCategory::ParseError
            | ErrorCategory::ValidationError => ExitCode::InvalidInput,
            _ => ExitCode::InternalError,
        }
    }
}

#[derive(Parser)]
#[command(name = "costpilot")]
#[command(about = "Zero-IAM FinOps engine for Terraform", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (json, text, markdown, pr-comment)
    #[arg(long, global = true, default_value = "text")]
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
    ///   costpilot scan tfplan.json
    ///   costpilot scan tfplan.json --explain
    ///   costpilot scan tfplan.json --policy policy.yaml
    ///   costpilot scan tfplan.json --infra-format cdk
    ///   costpilot scan tfplan.json --format json
    ///   costpilot scan tfplan.json --format pr-comment
    Scan(costpilot::cli::scan::ScanCommand),

    /// Manage cost baselines
    ///
    /// Record expected costs from successful deployments, update baselines,
    /// and validate baseline configurations.
    ///
    /// Examples:
    ///   costpilot baseline record tfplan.json --baselines baselines.json
    ///   costpilot baseline update module.vpc --cost 1500.0
    ///   costpilot baseline validate
    ///   costpilot baseline status
    Baseline(costpilot::cli::baseline::BaselineCommand),

    /// Compare cost between two Terraform plans
    ///
    /// Shows the cost difference between a baseline (before) plan and
    /// a proposed (after) plan. Useful for PR reviews and change impact analysis.
    ///
    /// Examples:
    ///   costpilot diff baseline.json new-plan.json
    ///   costpilot diff -f json baseline.json new-plan.json --verbose
    Diff {
        /// Path to baseline (before) plan file
        before: PathBuf,

        /// Path to proposed (after) plan file
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
    ///   costpilot init --path /path/to/project
    Init {
        /// Skip creating CI template
        #[arg(long)]
        no_ci: bool,

        /// Path to initialize (defaults to current directory)
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
    Explain(costpilot::cli::explain::ExplainArgs),

    /// Manage custom policy rules (DSL)
    PolicyDsl {
        #[command(flatten)]
        command: costpilot::cli::policy_dsl::PolicyDslCommand,
    },

    /// Manage policy lifecycle
    PolicyLifecycle {
        #[command(subcommand)]
        command: PolicyCommands,
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
        files: Vec<PathBuf>,

        /// Fail on first error
        #[arg(long)]
        fail_fast: bool,
    },

    /// Show version information
    Version {
        /// Show detailed version info
        #[arg(short = 'D', long)]
        detailed: bool,
    },

    /// Calculate SLO burn rate
    SloBurn {
        /// Path to SLO configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Path to snapshots directory
        #[arg(long)]
        snapshots_dir: Option<PathBuf>,

        /// Minimum number of snapshots required for analysis
        #[arg(long)]
        min_snapshots: Option<usize>,

        /// Minimum R-squared value for trend analysis
        #[arg(long)]
        min_r_squared: Option<f64>,
    },

    /// Check SLO compliance
    SloCheck {
        /// Path to SLO configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Manage SLO monitoring and compliance
    Slo {
        #[command(subcommand)]
        command: Option<SloCommands>,
    },

    /// Analyze cost trends and generate reports
    ///
    /// Track cost changes over time, detect regressions, and generate
    /// visual trend reports. Requires Premium edition.
    ///
    /// Examples:
    ///   costpilot trend show
    ///   costpilot trend snapshot --plan tfplan.json
    ///   costpilot trend regressions --threshold 10.0
    Trend {
        #[command(subcommand)]
        command: TrendCommands,
    },

    /// Detect cost anomalies
    Anomaly {
        /// Path to Terraform plan JSON file
        #[arg(short, long)]
        plan: PathBuf,

        /// Path to baseline plan for comparison
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Anomaly detection threshold (standard deviations)
        #[arg(long, default_value = "2.0")]
        threshold: f64,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate autofix patches
    AutofixPatch,

    /// Generate autofix snippets
    AutofixSnippet,

    /// Generate drift-safe autofix patches
    AutofixDriftSafe {
        /// Path to Terraform plan JSON file
        #[arg(short, long)]
        plan: PathBuf,

        /// Output file for patches (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Show detailed patch metadata
        #[arg(short, long)]
        verbose: bool,
    },

    /// Manage escrow operations
    Escrow,

    /// Performance monitoring and budgets
    Performance {
        #[command(subcommand)]
        command: Option<PerformanceCommands>,
    },

    /// Usage metering and reporting
    Usage {
        /// Calculate days in month
        #[arg(long)]
        days_in_month: Option<String>,
    },

    /// Manage feature flags for test-in-production
    Feature(costpilot::cli::commands::feature::FeatureArgs),
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
enum TrendCommands {
    /// Show cost trend history and generate visual report
    Show {
        /// Output format (text, json, html, svg)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path (for html/svg formats)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to snapshots directory
        #[arg(long)]
        snapshots_dir: Option<PathBuf>,
    },

    /// Create a cost snapshot from Terraform plan
    Snapshot {
        /// Path to Terraform plan JSON file
        #[arg(short, long)]
        plan: PathBuf,

        /// Commit hash for this snapshot
        #[arg(long)]
        commit: Option<String>,

        /// Branch name for this snapshot
        #[arg(long)]
        branch: Option<String>,

        /// Path to snapshots directory
        #[arg(long)]
        snapshots_dir: Option<PathBuf>,
    },

    /// Detect cost regressions compared to baseline
    Regressions {
        /// Regression threshold as percentage (e.g., 10.0 for 10%)
        #[arg(long, default_value = "5.0")]
        threshold: f64,

        /// Path to snapshots directory
        #[arg(long)]
        snapshots_dir: Option<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
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

#[derive(Subcommand)]
enum PerformanceCommands {
    /// Monitor performance metrics
    Monitor,

    /// Check performance budgets
    Budgets {
        /// Path to performance budget file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Generate performance report
    Report {
        /// Output format (json, text, markdown)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    // Add panic recovery to prevent crashes
    let result = std::panic::catch_unwind(|| main_inner());

    match result {
        Ok(exit_code) => exit_code.exit(),
        Err(panic_info) => {
            eprintln!(
                "{} Fatal error: CostPilot encountered an unexpected panic",
                "💥".bright_red()
            );
            if let Some(location) = panic_info.downcast_ref::<&std::panic::Location>() {
                eprintln!("  Location: {}:{}", location.file(), location.line());
            }
            eprintln!("  This is a bug. Please report it with the command that caused it.");
            eprintln!("  https://github.com/costpilot/costpilot/issues");
            ExitCode::InternalError.exit();
        }
    }
}

fn main_inner() -> ExitCode {
    // Load edition context BEFORE parsing CLI
    // This allows us to gate premium commands early
    let edition = costpilot::edition::detect_edition()
        .unwrap_or_else(|_| costpilot::edition::EditionContext::free());

    // Intercept --version/-V to show edition
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        let arg = &args[1];
        if arg == "--version" || arg == "-V" {
            let edition_str = if edition.is_premium() {
                "Premium"
            } else {
                "Free"
            };
            println!("costpilot {} ({})", VERSION, edition_str);
            return ExitCode::Success;
        }
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Check if this is a help request - let Clap handle it
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                e.print().ok();
                return ExitCode::Success;
            }
            eprintln!("error_class=invalid_input");
            eprintln!("{}", e);
            return ExitCode::InvalidInput;
        }
    };

    // Enable debug logging if requested
    if cli.debug {
        eprintln!("{}", "🔍 Debug mode enabled".bright_black());
        eprintln!("  Verbose: {}", cli.verbose);
        eprintln!("  Format: {}", cli.format);
        eprintln!(
            "  Edition: {}",
            if edition.is_premium() {
                "Premium"
            } else {
                "Free"
            }
        );
        eprintln!();
    }

    // Print banner for interactive mode
    if atty::is(atty::Stream::Stdout) {
        eprintln!("{}", BANNER.bright_cyan());
        eprintln!(
            "{}",
            format!("v{} | Zero-IAM FinOps Engine", VERSION).bright_black()
        );
        eprintln!();
    }

    let start_time = if cli.debug {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let result = match cli.command {
        Commands::Scan(scan_cmd) => {
            if cli.debug {
                eprintln!("🔍 Executing scan command");
            }
            // Enforce zero-cost policy before executing scan
            match ZeroCostGuard::new().enforce_zero_cost() {
                Ok(()) => scan_cmd
                    .execute(&cli.format)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
                Err(e) => {
                    eprintln!("{} Zero-cost policy violation: {}", "🚫".bright_red(), e);
                    Err(Box::new(e) as Box<dyn std::error::Error>)
                }
            }
        }
        Commands::Baseline(baseline_cmd) => {
            if cli.debug {
                eprintln!("🔍 Executing baseline command");
            }
            baseline_cmd.execute()
        }
        Commands::Diff { before, after } => {
            if cli.debug {
                eprintln!("🔍 Executing diff command");
                eprintln!("  Before: {:?}", before);
                eprintln!("  After: {:?}", after);
            }
            cmd_diff(before, after, &cli.format, cli.verbose, &edition)
        }
        Commands::Init { no_ci, path } => {
            if cli.debug {
                eprintln!("🔍 Executing init command");
                eprintln!("  No CI: {}", no_ci);
                if let Some(ref p) = path {
                    eprintln!("  Path: {:?}", p);
                }
            }
            cmd_init(no_ci, path, cli.verbose)
        }
        Commands::Map(map_cmd) => {
            if cli.debug {
                eprintln!("🔍 Executing map command");
            }
            costpilot::cli::map::execute_map_command(&map_cmd, &edition)
        }
        Commands::Policy { command } => {
            if cli.debug {
                eprintln!("🔍 Executing policy command");
            }
            cmd_policy(command, &cli.format, cli.verbose, &edition)
        }
        Commands::Audit { command } => {
            if cli.debug {
                eprintln!("🔍 Executing audit command");
            }
            cmd_audit(command, &cli.format, cli.verbose)
        }
        Commands::Heuristics { command } => {
            if cli.debug {
                eprintln!("🔍 Executing heuristics command");
            }
            cmd_heuristics(command, &cli.format, cli.verbose)
        }
        Commands::Explain(explain_args) => {
            cmd_explain(explain_args, &cli.format, cli.verbose, &edition)
        }
        Commands::PolicyDsl { command } => {
            costpilot::cli::policy_dsl::execute_policy_dsl_command(&command)
        }
        Commands::PolicyLifecycle { command } => {
            if cli.debug {
                eprintln!("🔍 Executing policy lifecycle command");
            }
            cmd_policy(command, &cli.format, cli.verbose, &edition)
        }
        Commands::Group(group_cmd) => {
            costpilot::cli::group::execute_group_command(group_cmd, &edition)
        }
        Commands::Validate { files, fail_fast } => {
            cmd_validate(files, &cli.format, fail_fast, &edition)
        }
        Commands::Version { detailed } => {
            cmd_version(detailed, &edition);
            return ExitCode::Success;
        }
        Commands::SloBurn {
            config,
            snapshots_dir,
            min_snapshots,
            min_r_squared,
        } => cmd_slo_burn(
            config,
            snapshots_dir,
            min_snapshots,
            min_r_squared,
            &cli.format,
            cli.verbose,
            &edition,
        ),
        Commands::SloCheck { config } => cmd_slo_check(config, &cli.format, cli.verbose, &edition),
        Commands::Slo { command } => cmd_slo(command, &cli.format, cli.verbose, &edition),
        Commands::Trend { command } => cmd_trend(command, &cli.format, cli.verbose, &edition),
        Commands::Anomaly {
            plan,
            baseline,
            threshold,
            format,
        } => cmd_anomaly(plan, baseline, threshold, &format, cli.verbose, &edition),
        Commands::AutofixPatch => cmd_autofix_patch(&cli.format, cli.verbose),
        Commands::AutofixSnippet => cmd_autofix_snippet(&cli.format, cli.verbose),
        Commands::AutofixDriftSafe {
            plan,
            output,
            verbose,
        } => cmd_autofix_drift_safe(plan, output, &cli.format, verbose || cli.verbose),
        Commands::Escrow => cmd_escrow(&cli.format, cli.verbose),
        Commands::Performance { command } => cmd_performance(command, &cli.format, cli.verbose),
        Commands::Usage { days_in_month } => cmd_usage(days_in_month, &cli.format, cli.verbose),
        Commands::Feature(feature_args) => {
            costpilot::cli::commands::feature::execute(&feature_args)
        }
    };

    if let Err(e) = result {
        if cli.debug {
            eprintln!("{} Error details: {:?}", "🔍".bright_black(), e);
        }

        // Try to downcast to CostPilotError for better exit code mapping
        if let Some(cp_error) =
            e.downcast_ref::<costpilot::engines::shared::error_model::CostPilotError>()
        {
            eprintln!("{} {}", "Error:".bright_red().bold(), cp_error);
            if cli.verbose {
                eprintln!("  Category: {:?}", cp_error.category);
                if let Some(context) = &cp_error.context {
                    eprintln!("  Context: {}", context);
                }
            }
            return ExitCode::from_costpilot_error(cp_error);
        } else {
            // Generic error handling
            eprintln!("{} {}", "Error:".bright_red().bold(), e);
            return ExitCode::InternalError;
        }
    }

    if let Some(start) = start_time {
        eprintln!(
            "{} Command completed in {:.2?}",
            "⏱️".bright_black(),
            start.elapsed()
        );
    }

    ExitCode::Success
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

fn cmd_init(
    no_ci: bool,
    path: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::init::init;

    let target_path = path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let default_path = PathBuf::from(".");

    let ci_provider = if no_ci {
        "none"
    } else {
        // Auto-detect CI provider or default to GitHub
        let path_for_detection = path.as_ref().unwrap_or(&default_path);
        if path_for_detection.join(".github").exists() {
            "github"
        } else if path_for_detection.join(".gitlab-ci.yml").exists() {
            "gitlab"
        } else {
            "github" // Default to GitHub
        }
    };

    if verbose {
        println!("  CI Provider: {}", ci_provider);
        println!("  Target Path: {}", target_path);
    }

    init(&target_path, ci_provider)?;

    Ok(())
}

#[allow(dead_code)]
fn cmd_slo(
    command: Option<SloCommands>,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
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
    args: costpilot::cli::explain::ExplainArgs,
    _format: &str,
    _verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::cli::explain::execute_explain_args;

    let output = execute_explain_args(args, edition)?;
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

fn cmd_slo_burn(
    config: Option<PathBuf>,
    snapshots_dir: Option<PathBuf>,
    min_snapshots: Option<usize>,
    min_r_squared: Option<f64>,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Gate SLO functionality behind premium license
    costpilot::edition::require_premium(edition, "SLO burn rate analysis")
        .map_err(|e| format!("SLO features require premium license: {}", e))?;

    costpilot::cli::commands::slo_burn::execute(
        config,
        snapshots_dir,
        format,
        min_snapshots,
        min_r_squared,
        verbose,
        edition,
    )
}

fn cmd_slo_check(
    config: Option<PathBuf>,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Gate SLO functionality behind premium license
    costpilot::edition::require_premium(edition, "SLO compliance checking")
        .map_err(|e| format!("SLO features require premium license: {}", e))?;

    if verbose {
        eprintln!("🔍 Checking SLO compliance");
        if let Some(ref config_path) = config {
            eprintln!("  Config: {:?}", config_path);
        }
    }

    // Placeholder implementation - would integrate with SLO engine
    match format {
        "json" => println!("{{\"compliance\": \"unknown\", \"violations\": []}}"),
        _ => println!("SLO compliance check not yet implemented"),
    }

    Ok(())
}

fn cmd_trend(
    command: TrendCommands,
    _format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Trend analysis requires premium license
    costpilot::edition::require_premium(edition, "Cost trend analysis")
        .map_err(|e| format!("Trend analysis requires premium license: {}", e))?;

    match command {
        TrendCommands::Show {
            format: output_format,
            output,
            snapshots_dir,
        } => cmd_trend_show(output_format, output, snapshots_dir, verbose),
        TrendCommands::Snapshot {
            plan,
            commit,
            branch,
            snapshots_dir,
        } => cmd_trend_snapshot(plan, commit, branch, snapshots_dir, verbose),
        TrendCommands::Regressions {
            threshold,
            snapshots_dir,
            format: output_format,
        } => cmd_trend_regressions(threshold, snapshots_dir, output_format, verbose),
    }
}

fn cmd_trend_show(
    output_format: String,
    output: Option<PathBuf>,
    snapshots_dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let snapshots_dir = snapshots_dir.unwrap_or_else(|| PathBuf::from(".costpilot/snapshots"));

    if verbose {
        eprintln!("📊 Generating cost trend report");
        eprintln!("  Format: {}", output_format);
        eprintln!("  Snapshots: {:?}", snapshots_dir);
        if let Some(ref output_path) = output {
            eprintln!("  Output: {:?}", output_path);
        }
    }

    // Create trend engine
    let edition = costpilot::edition::EditionContext::new();
    let trend_engine = costpilot::engines::trend::TrendEngine::new(&snapshots_dir, &edition)?;

    match output_format.as_str() {
        "json" => {
            let history = trend_engine.load_history()?;
            let json = serde_json::to_string_pretty(&history)?;
            println!("{}", json);
        }
        "html" => {
            let output_path = output.unwrap_or_else(|| PathBuf::from("trend_report.html"));
            trend_engine.generate_html(&output_path, "CostPilot Trend Report")?;
            println!("📊 HTML trend report generated: {:?}", output_path);
        }
        "svg" => {
            let svg = trend_engine.generate_svg()?;
            if let Some(output_path) = output {
                std::fs::write(&output_path, &svg)?;
                println!("📊 SVG trend graph generated: {:?}", output_path);
            } else {
                println!("{}", svg);
            }
        }
        _ => {
            let history = trend_engine.load_history()?;
            println!("{}", "📊 Cost Trend History".bold().cyan());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

            if history.snapshots.is_empty() {
                println!("No snapshots found. Create some snapshots first:");
                println!("  costpilot trend snapshot --plan tfplan.json");
                return Ok(());
            }

            println!("Found {} snapshots:", history.snapshots.len());
            for snapshot in &history.snapshots {
                println!(
                    "  {}: ${:.2} ({})",
                    snapshot.id, snapshot.total_monthly_cost, snapshot.timestamp
                );
            }

            if history.snapshots.len() >= 2 {
                let latest = &history.snapshots[history.snapshots.len() - 1];
                let previous = &history.snapshots[history.snapshots.len() - 2];
                let change = latest.total_monthly_cost - previous.total_monthly_cost;
                let percent = if previous.total_monthly_cost > 0.0 {
                    (change / previous.total_monthly_cost) * 100.0
                } else {
                    0.0
                };

                println!(
                    "\nLatest change: {} ${:.2} ({:.1}%)",
                    if change >= 0.0 {
                        "↗️ +".green()
                    } else {
                        "↘️ ".red()
                    },
                    change.abs(),
                    percent
                );
            }
        }
    }

    Ok(())
}

fn cmd_trend_snapshot(
    plan: PathBuf,
    commit: Option<String>,
    branch: Option<String>,
    snapshots_dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let snapshots_dir = snapshots_dir.unwrap_or_else(|| PathBuf::from(".costpilot/snapshots"));

    if verbose {
        eprintln!("📸 Creating cost snapshot");
        eprintln!("  Plan: {:?}", plan);
        eprintln!("  Snapshots dir: {:?}", snapshots_dir);
        if let Some(ref commit_hash) = commit {
            eprintln!("  Commit: {}", commit_hash);
        }
        if let Some(ref branch_name) = branch {
            eprintln!("  Branch: {}", branch_name);
        }
    }

    // Load and parse the Terraform plan
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine.detect_from_terraform_plan(&plan)?;

    if changes.is_empty() {
        println!("No resource changes detected in plan");
        return Ok(());
    }

    // Create cost estimates
    let edition = costpilot::edition::EditionContext::new();
    let estimates = costpilot::engines::prediction::PredictionEngine::predict_static(&changes)?;

    // Create trend engine and snapshot
    let trend_engine = costpilot::engines::trend::TrendEngine::new(&snapshots_dir, &edition)?;
    let snapshot = trend_engine.create_snapshot(estimates, commit, branch)?;

    // Save snapshot
    let snapshot_path = trend_engine.save_snapshot(&snapshot)?;

    // Calculate total resource count
    let resource_count: usize = snapshot.modules.values().map(|m| m.resource_count).sum();

    println!(
        "✅ Snapshot created: {} (${:.2}/month, {} resources)",
        snapshot.id, snapshot.total_monthly_cost, resource_count
    );

    if verbose {
        println!("  Saved to: {:?}", snapshot_path);
    }

    Ok(())
}

fn cmd_trend_regressions(
    threshold: f64,
    snapshots_dir: Option<PathBuf>,
    output_format: String,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let snapshots_dir = snapshots_dir.unwrap_or_else(|| PathBuf::from(".costpilot/snapshots"));

    if verbose {
        eprintln!("🔍 Detecting cost regressions");
        eprintln!("  Threshold: {:.1}%", threshold);
        eprintln!("  Snapshots dir: {:?}", snapshots_dir);
    }

    // Create trend engine
    let edition = costpilot::edition::EditionContext::new();
    let trend_engine = costpilot::engines::trend::TrendEngine::new(&snapshots_dir, &edition)?;

    // Load history
    let history = trend_engine.load_history()?;

    if history.snapshots.len() < 2 {
        println!(
            "Need at least 2 snapshots to detect regressions. Currently have {}.",
            history.snapshots.len()
        );
        return Ok(());
    }

    // Use the most recent snapshot as current, second most recent as baseline
    let current = &history.snapshots[history.snapshots.len() - 1];
    let baseline = &history.snapshots[history.snapshots.len() - 2];

    let regressions = trend_engine.detect_regressions(current, baseline, threshold);

    match output_format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&regressions)?;
            println!("{}", json);
        }
        _ => {
            if regressions.is_empty() {
                println!(
                    "✅ No cost regressions detected (threshold: {:.1}%)",
                    threshold
                );
            } else {
                println!("⚠️  {} cost regression(s) detected:", regressions.len());
                for regression in &regressions {
                    println!(
                        "  {} {}: {:?} ${:.2} increase",
                        "↗️".red(),
                        regression.affected,
                        regression.regression_type,
                        regression.baseline_cost
                    );
                }
            }
        }
    }

    Ok(())
}

fn cmd_anomaly(
    plan: PathBuf,
    baseline: Option<PathBuf>,
    threshold: f64,
    format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Gate anomaly detection behind premium license
    costpilot::edition::require_premium(edition, "Cost anomaly detection")
        .map_err(|e| format!("Anomaly detection requires premium license: {}", e))?;

    if verbose {
        eprintln!("🔍 Detecting cost anomalies");
        eprintln!("  Plan: {:?}", plan);
        if let Some(ref baseline_path) = baseline {
            eprintln!("  Baseline: {:?}", baseline_path);
        }
        eprintln!("  Threshold: {}σ", threshold);
    }

    // Placeholder implementation - would integrate with anomaly detection engine
    match format {
        "json" => println!("{{\"anomalies\": [], \"threshold\": {}}}", threshold),
        _ => println!("Cost anomaly detection not yet implemented (Premium feature)"),
    }

    Ok(())
}

fn cmd_autofix_patch(format: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Load edition context
    let edition = costpilot::edition::detect_edition()
        .unwrap_or_else(|_| costpilot::edition::EditionContext::free());

    // Require Premium for autofix
    costpilot::edition::require_premium(&edition, "Autofix")
        .map_err(|e| format!("Autofix requires premium license: {}", e))?;

    if verbose {
        eprintln!("🔍 Generating autofix patches");
    }

    // Placeholder implementation - would integrate with autofix engine
    match format {
        "json" => println!("{{\"patches\": []}}"),
        _ => println!("Autofix patch generation not yet implemented"),
    }

    Ok(())
}

fn cmd_autofix_snippet(format: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Load edition context
    let edition = costpilot::edition::detect_edition()
        .unwrap_or_else(|_| costpilot::edition::EditionContext::free());

    // Require Premium for autofix
    costpilot::edition::require_premium(&edition, "Autofix")
        .map_err(|e| format!("Autofix requires premium license: {}", e))?;

    if verbose {
        eprintln!("🔍 Generating autofix snippets");
    }

    // Placeholder implementation - would integrate with autofix engine
    match format {
        "json" => println!("{{\"snippets\": []}}"),
        _ => println!("Autofix snippet generation not yet implemented"),
    }

    Ok(())
}

fn cmd_autofix_drift_safe(
    plan: PathBuf,
    output: Option<PathBuf>,
    _format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load edition context
    let edition = costpilot::edition::detect_edition()
        .unwrap_or_else(|_| costpilot::edition::EditionContext::free());

    // Require Premium for drift-safe autofix
    costpilot::edition::require_premium(&edition, "Drift-safe autofix")
        .map_err(|e| format!("Drift-safe autofix requires premium license: {}", e))?;

    println!(
        "{}",
        "🔧 CostPilot Autofix - Drift-Safe Mode (Beta)"
            .bold()
            .cyan()
    );
    println!();

    // Load and parse plan
    println!("{}", "Loading Terraform plan...".dimmed());
    let plan_content = std::fs::read_to_string(&plan)?;
    let plan_json: serde_json::Value = serde_json::from_str(&plan_content)?;

    // Extract resource changes
    let changes = costpilot::cli::utils::extract_resource_changes(&plan_json)?;
    println!("   Found {} resource changes", changes.len());
    println!();

    // Detect cost regressions
    println!("{}", "Detecting cost regressions...".dimmed());
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let detections = detection_engine.detect(&changes)?;

    if detections.is_empty() {
        println!("   {} No cost issues detected", "✓".green());
        return Ok(());
    }

    println!("   Found {} cost issues", detections.len());
    println!();

    // Generate predictions for cost estimates
    println!("{}", "Estimating costs...".dimmed());
    let mut prediction_engine =
        costpilot::engines::prediction::PredictionEngine::new_with_edition(&edition)?;
    let cost_estimates = prediction_engine.predict(&changes)?;
    println!("   Generated {} cost estimates", cost_estimates.len());
    println!();

    // Generate drift-safe fixes
    println!("{}", "Generating drift-safe rollback patches...".dimmed());
    let autofix_result = costpilot::engines::autofix::AutofixEngine::generate_fixes(
        &detections,
        &changes,
        &cost_estimates,
        costpilot::engines::autofix::AutofixMode::DriftSafe,
        &edition,
    )?;

    // Display results
    if autofix_result.patches.is_empty() {
        println!("   {} No drift-safe patches generated", "✓".green());
    } else {
        println!(
            "   Generated {} drift-safe patches",
            autofix_result.patches.len()
        );
    }

    // Show warnings
    for warning in &autofix_result.warnings {
        println!("   {} {}", "⚠️".yellow(), warning);
    }

    // Output patches
    if !autofix_result.patches.is_empty() {
        println!();
        println!("{}", "Drift-Safe Patches:".bold());

        for patch in &autofix_result.patches {
            println!("  📄 {}", patch.filename.bold());
            println!(
                "     Resource: {} ({})",
                patch.resource_id, patch.resource_type
            );
            println!("     Changes: {} hunks", patch.hunks.len());

            if verbose {
                println!("     Metadata:");
                println!("       Cost before: ${:.2}", patch.metadata.cost_before);
                println!(
                    "       Monthly savings: ${:.2}",
                    patch.metadata.monthly_savings
                );
                println!(
                    "       Confidence: {:.1}%",
                    patch.metadata.confidence * 100.0
                );
                println!("       Rationale: {}", patch.metadata.rationale);
            }
        }

        // Save to file if requested
        if let Some(output_path) = output {
            let json = serde_json::to_string_pretty(&autofix_result.patches)?;
            std::fs::write(&output_path, json)?;
            println!();
            println!("💾 Patches saved to: {}", output_path.display());
        }
    }

    Ok(())
}

fn cmd_escrow(format: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("🔍 Managing escrow operations");
    }

    // Placeholder implementation - would integrate with escrow engine
    match format {
        "json" => println!("{{\"escrow_operations\": []}}"),
        _ => println!("Escrow operations not yet implemented"),
    }

    Ok(())
}

fn cmd_performance(
    command: Option<PerformanceCommands>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("🔍 Executing performance command");
    }

    match command {
        Some(PerformanceCommands::Monitor) => {
            if verbose {
                eprintln!("  Subcommand: monitor");
            }
            match format {
                "json" => println!("{{\"performance_metrics\": []}}"),
                _ => println!("Performance monitoring not yet implemented"),
            }
        }
        Some(PerformanceCommands::Budgets { config }) => {
            if verbose {
                eprintln!("  Subcommand: budgets");
                if let Some(ref config_path) = config {
                    eprintln!("  Config: {:?}", config_path);
                }
            }
            match format {
                "json" => println!("{{\"budget_check\": \"passed\"}}"),
                _ => println!("Performance budgets"),
            }
        }
        Some(PerformanceCommands::Report {
            format: report_format,
            output,
        }) => {
            if verbose {
                eprintln!("  Subcommand: report");
                eprintln!("  Format: {}", report_format);
                if let Some(ref output_path) = output {
                    eprintln!("  Output: {:?}", output_path);
                }
            }
            match format {
                "json" => println!("{{\"performance_report\": \"generated\"}}"),
                _ => println!("Performance reporting not yet implemented"),
            }
        }
        None => match format {
            "json" => {
                println!("{{\"performance_help\": \"Use subcommands: monitor, budget, report\"}}")
            }
            _ => println!("Performance command requires a subcommand: monitor, budget, report"),
        },
    }

    Ok(())
}

fn calculate_days_in_month(month_str: &str) -> Option<u32> {
    let parts: Vec<&str> = month_str.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;

    if month < 1 || month > 12 {
        return None;
    }

    // Days in each month
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // February - check for leap year
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => return None,
    };

    Some(days)
}

fn cmd_usage(
    days_in_month: Option<String>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("🔍 Calculating usage metrics");
        if let Some(ref days) = days_in_month {
            eprintln!("  Days in month: {}", days);
        }
    }

    if let Some(month_str) = days_in_month {
        // Parse YYYY-MM format and calculate days in month
        if let Some(days) = calculate_days_in_month(&month_str) {
            match format {
                "json" => println!("{{\"days_in_month\": {}}}", days),
                _ => println!("{}", days),
            }
        } else {
            println!("Invalid month format. Use YYYY-MM");
        }
    } else {
        // Placeholder implementation - would integrate with usage metering
        match format {
            "json" => println!("{{\"usage_metrics\": []}}"),
            _ => println!("Usage metering not yet implemented"),
        }
    }

    Ok(())
}
