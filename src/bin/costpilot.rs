// CLI entrypoint for CostPilot

use clap::{Parser, Subcommand};
use colored::*;
use costpilot::cli::commands::autofix_patch::AutofixPatchArgs;
use costpilot::cli::commands::autofix_snippet::AutofixSnippetArgs;
use costpilot::engines::policy::ExemptionStatus;
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
#[command(about = "Zero-IAM FinOps engine for Terraform", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short = 'f', long, global = true, default_value = "text")]
    format: String,

    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Scan(costpilot::cli::scan::ScanCommand),
    Diff {
        #[arg(value_name = "BEFORE")]
        before: PathBuf,

        #[arg(value_name = "AFTER")]
        after: PathBuf,
    },
    Init {
        #[arg(long)]
        no_ci: bool,

        #[arg(long)]
        path: Option<PathBuf>,
    },

    Map(costpilot::cli::map::MapCommand),

    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },

    Exemption {
        #[command(subcommand)]
        command: ExemptionCommands,
    },

    Trend {
        #[command(subcommand)]
        command: TrendCommands,
    },

    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },

    Heuristics {
        #[command(subcommand)]
        command: costpilot::cli::heuristics::HeuristicsCommand,
    },

    Explain {
        #[command(subcommand)]
        command: Option<costpilot::cli::explain::ExplainCommand>,

        #[command(flatten)]
        args: Option<costpilot::cli::explain::ExplainArgs>,
    },

    Performance {
        #[command(subcommand)]
        command: Option<PerformanceCli>,
    },

    Slo {
        #[command(subcommand)]
        command: Option<SloCli>,
    },

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

    AutofixSnippet {
        #[arg(long, value_name = "FILE")]
        plan: Option<PathBuf>,

        #[arg(short, long)]
        verbose: bool,
    },

    AutofixPatch(AutofixPatchArgs),

    Escrow {
        #[command(subcommand)]
        command: Option<EscrowCli>,
    },

    PolicyLifecycle {
        #[command(subcommand)]
        command: Option<PolicyLifecycleCli>,
    },

    Usage {
        #[arg(long = "days-in-month")]
        days_in_month: Option<String>,

        #[command(subcommand)]
        command: Option<UsageCli>,
    },

    PolicyDsl {
        #[command(flatten)]
        command: costpilot::cli::policy_dsl::PolicyDslCommand,
    },

    Group(costpilot::cli::group::GroupCommand),
    Validate {
        #[arg(required = true)]
        files: Vec<PathBuf>,

        #[arg(long)]
        fail_fast: bool,
    },

    Version {
        #[arg(long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum SloCommands {
    Check,

    Burn {
        #[arg(short, long)]
        slo: Option<PathBuf>,

        #[arg(short = 'd', long)]
        snapshots: Option<PathBuf>,

        #[arg(long, default_value = "3")]
        min_snapshots: usize,

        #[arg(long, default_value = "0.7")]
        min_r_squared: f64,
    },
}

#[derive(Subcommand, Debug)]
enum PerformanceCli {
    Budgets,
    SetBaseline {
        #[arg(long = "from-file")]
        from_file: Option<PathBuf>,
    },
    Stats,
    CheckRegressions {
        report_file: PathBuf,
    },
    History {
        engine: Option<String>,
        limit: Option<usize>,
    },
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
    Create {
        version: String,
        output_dir: Option<PathBuf>,
        include_artifacts: bool,
    },
    Verify {
        package_dir: PathBuf,
    },
    Playbook {
        package_dir: PathBuf,
        output: Option<PathBuf>,
    },
    Recover {
        package_dir: PathBuf,
        working_dir: PathBuf,
    },
    Configure {
        vendor_name: String,
        contact_email: String,
        support_url: String,
    },
    List {
        escrow_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum PolicyLifecycleCli {
    Submit {
        #[arg(short, long)]
        policy: PathBuf,
        #[arg(short, long, value_delimiter = ',')]
        approvers: Vec<String>,
    },
    Approve {
        policy_id: String,
        #[arg(short, long)]
        approver: String,
        #[arg(short, long)]
        comment: Option<String>,
    },
    Reject {
        policy_id: String,
        #[arg(short, long)]
        approver: String,
        #[arg(short, long)]
        reason: String,
    },
    Activate {
        policy_id: String,
        #[arg(short, long, default_value = "system")]
        actor: String,
    },
    Deprecate {
        policy_id: String,
        #[arg(short, long, default_value = "system")]
        actor: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    Status {
        policy_id: String,
    },
    History {
        policy_id: String,
    },
    Diff {
        policy_id: String,
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
    },
}

#[derive(Subcommand, Debug)]
enum UsageCli {
    Report {
        team_id: String,
        start: Option<String>,
        end: Option<String>,
        format: Option<String>,
    },
    Export {
        start: String,
        end: String,
        format: String,
        output: Option<PathBuf>,
    },
    Pr {
        pr_number: u32,
        repository: Option<String>,
    },
    Chargeback {
        org_id: String,
        start: String,
        end: String,
        format: String,
        output: Option<PathBuf>,
    },
    Invoice {
        team_id: String,
        start: String,
        end: String,
    },
}

#[derive(Subcommand)]
enum PolicyCommands {
    Submit {
        #[arg(short, long)]
        policy: PathBuf,

        #[arg(short, long, value_delimiter = ',')]
        approvers: Vec<String>,
    },

    Approve {
        policy_id: String,

        #[arg(short, long)]
        approver: String,

        #[arg(short, long)]
        comment: Option<String>,
    },

    Reject {
        policy_id: String,

        #[arg(short, long)]
        approver: String,

        #[arg(short, long)]
        reason: String,
    },

    Activate {
        policy_id: String,

        #[arg(short, long, default_value = "system")]
        actor: String,
    },

    Deprecate {
        policy_id: String,

        #[arg(short, long, default_value = "system")]
        actor: String,

        #[arg(short, long)]
        reason: String,
    },

    Status {
        policy_id: String,
    },

    History {
        policy_id: String,
    },

    Diff {
        policy_id: String,

        #[arg(short, long)]
        from: String,

        #[arg(short, long)]
        to: String,
    },

    Increment {
        policy_id: String,

        #[arg(short, long)]
        changelog: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExemptionCommands {
    Validate {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    Check {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    List {
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[arg(long)]
        expired: bool,

        #[arg(long)]
        expiring: bool,
    },

    Status {
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[arg(value_name = "ID")]
        exemption_id: String,
    },
}

#[derive(Subcommand)]
enum TrendCommands {
    Snapshot {
        #[arg(short, long, value_name = "FILE")]
        plan: PathBuf,

        #[arg(long)]
        commit: Option<String>,

        #[arg(long)]
        branch: Option<String>,

        #[arg(long)]
        id: Option<String>,
    },

    List {
        #[arg(short, long)]
        verbose: bool,

        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },

    Graph {
        #[arg(short, long, default_value = "svg")]
        format: String,

        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        #[arg(long, default_value = "30")]
        days: u32,

        #[arg(long)]
        branch: Option<String>,
    },

    Diff {
        #[arg(value_name = "FROM")]
        from: String,

        #[arg(value_name = "TO")]
        to: String,

        #[arg(short, long)]
        verbose: bool,
    },

    Clean {
        #[arg(short = 'n', long)]
        keep: Option<usize>,

        #[arg(long)]
        older_than: Option<u32>,

        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum AuditCommands {
    View {
        #[arg(long)]
        event_type: Option<String>,

        #[arg(long)]
        actor: Option<String>,

        #[arg(long)]
        resource: Option<String>,

        #[arg(long)]
        severity: Option<String>,

        #[arg(short = 'n', long)]
        last_n: Option<usize>,
    },

    Verify,

    Compliance {
        #[arg(short = 'f', long)]
        framework: String,

        #[arg(short, long, default_value = "30")]
        days: u32,
    },

    Export {
        #[arg(short = 'f', long, default_value = "ndjson")]
        format: String,

        #[arg(short, long)]
        output: PathBuf,
    },

    Stats,

    Record {
        #[arg(long)]
        event_type: String,

        #[arg(long)]
        actor: String,

        #[arg(long)]
        resource_id: String,

        #[arg(long)]
        resource_type: String,

        #[arg(long)]
        description: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let edition = costpilot::edition::detect_edition()
        .unwrap_or_else(|_| costpilot::edition::EditionContext::free());

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
            return Ok(());
        }
    }

    if edition.is_free() && args.len() >= 2 {
        let premium_commands = ["autofix", "patch", "slo"];
        let command = args[1].to_lowercase();

        if premium_commands.contains(&command.as_str()) {
            eprintln!(
                "{} Unknown command '{}'",
                "Error:".bright_red().bold(),
                command
            );
            eprintln!();
            eprintln!("This command requires CostPilot Premium.");
            eprintln!("Upgrade at: https://shieldcraft-ai.com/costpilot/upgrade");
            process::exit(1);
        }
    }

    let cli = Cli::parse();
    if atty::is(atty::Stream::Stdout) {
        println!("{}", BANNER.bright_cyan());
        println!(
            "{}",
            format!("v{} | Zero-IAM FinOps Engine", VERSION).bright_black()
        );
        println!();
    }

    let _start_time: Option<std::time::Instant> = None;

    let result = match cli.command {
        Commands::Scan(scan_cmd) => scan_cmd
            .execute_with_edition(&edition, &cli.format)
            .map_err(|e| format!("{}", e).into()),
        Commands::Diff { before, after } => {
            cmd_diff(before, after, &cli.format, cli.verbose, &edition)
        }
        Commands::Init { no_ci, path } => cmd_init(no_ci, path, cli.verbose),
        Commands::Map(map_cmd) => costpilot::cli::map::execute_map_command(&map_cmd, &edition),
        Commands::Performance { command } => {
            use costpilot::cli::performance as perf;
            let res = match command {
                Some(PerformanceCli::Budgets) => {
                    perf::execute_performance_command(perf::PerformanceCommand::Budgets)
                }
                Some(PerformanceCli::SetBaseline { from_file }) => {
                    perf::execute_performance_command(perf::PerformanceCommand::SetBaseline {
                        from_file,
                    })
                }
                Some(PerformanceCli::Stats) => {
                    perf::execute_performance_command(perf::PerformanceCommand::Stats)
                }
                Some(PerformanceCli::CheckRegressions { report_file }) => {
                    perf::execute_performance_command(perf::PerformanceCommand::CheckRegressions {
                        report_file,
                    })
                }
                Some(PerformanceCli::History { engine, limit }) => {
                    perf::execute_performance_command(perf::PerformanceCommand::History {
                        engine,
                        limit,
                    })
                }
                None => perf::execute_performance_command(perf::PerformanceCommand::Budgets),
            };
            match res {
                Ok(out) => {
                    println!("{}", out);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Commands::Policy { command } => cmd_policy(command, &cli.format, cli.verbose, &edition),
        Commands::Exemption { command } => {
            cmd_exemption(command, &cli.format, cli.verbose, &edition)
        }
        Commands::Trend { command } => cmd_trend(command, &cli.format, cli.verbose, &edition),
        Commands::Slo { command } => match command {
            Some(SloCli::Check) => {
                cmd_slo(Some(SloCommands::Check), &cli.format, cli.verbose, &edition)
            }
            Some(SloCli::Burn {
                config,
                snapshots,
                min_snapshots,
                min_r_squared,
                verbose,
            }) => cmd_slo(
                Some(SloCommands::Burn {
                    slo: config,
                    snapshots,
                    min_snapshots: min_snapshots.unwrap_or(3),
                    min_r_squared: min_r_squared.unwrap_or(0.7),
                }),
                &cli.format,
                verbose || cli.verbose,
                &edition,
            ),
            None => cmd_slo(None, &cli.format, cli.verbose, &edition),
        },
        Commands::SloCheck => cmd_slo(Some(SloCommands::Check), &cli.format, cli.verbose, &edition),
        Commands::SloBurn {
            config,
            snapshots,
            min_snapshots,
            min_r_squared,
            verbose,
        } => cmd_slo(
            Some(SloCommands::Burn {
                slo: config,
                snapshots,
                min_snapshots: min_snapshots.unwrap_or(3),
                min_r_squared: min_r_squared.unwrap_or(0.7),
            }),
            &cli.format,
            verbose || cli.verbose,
            &edition,
        ),
        Commands::Audit { command } => cmd_audit(command, &cli.format, cli.verbose),
        Commands::Heuristics { command } => cmd_heuristics(command, &cli.format, cli.verbose),
        Commands::Explain { command, args } => {
            cmd_explain(command, args, &cli.format, cli.verbose, &edition)
        }
        Commands::AutofixSnippet { plan, verbose } => {
            let plan_path = plan.ok_or("--plan is required for autofix-snippet")?;
            let args = AutofixSnippetArgs {
                plan: plan_path,
                verbose,
            };
            match costpilot::cli::commands::autofix_snippet::execute(&args, &edition) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}", e).into()),
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
                Some(EscrowCli::Create {
                    version,
                    output_dir,
                    include_artifacts,
                }) => ec::execute_escrow_command(ec::EscrowCommand::Create {
                    version,
                    output_dir,
                    include_artifacts,
                }),
                Some(EscrowCli::Verify { package_dir }) => {
                    ec::execute_escrow_command(ec::EscrowCommand::Verify { package_dir })
                }
                Some(EscrowCli::Playbook {
                    package_dir,
                    output,
                }) => ec::execute_escrow_command(ec::EscrowCommand::Playbook {
                    package_dir,
                    output,
                }),
                Some(EscrowCli::Recover {
                    package_dir,
                    working_dir,
                }) => ec::execute_escrow_command(ec::EscrowCommand::Recover {
                    package_dir,
                    working_dir,
                }),
                Some(EscrowCli::Configure {
                    vendor_name,
                    contact_email,
                    support_url,
                }) => ec::execute_escrow_command(ec::EscrowCommand::Configure {
                    vendor_name,
                    contact_email,
                    support_url,
                }),
                Some(EscrowCli::List { escrow_dir }) => {
                    ec::execute_escrow_command(ec::EscrowCommand::List { escrow_dir })
                }
                None => ec::execute_escrow_command(ec::EscrowCommand::List { escrow_dir: None }),
            };
            match res {
                Ok(out) => {
                    println!("{}", out);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Commands::PolicyLifecycle { command } => {
            use costpilot::cli::commands::policy_lifecycle as pl;
            match command {
                Some(PolicyLifecycleCli::Submit { policy, approvers }) => {
                    pl::cmd_submit(policy, approvers, &cli.format, cli.verbose, &edition)
                        .map_err(|e| format!("{}", e).into())
                }
                Some(PolicyLifecycleCli::Approve {
                    policy_id,
                    approver,
                    comment,
                }) => pl::cmd_approve(
                    policy_id,
                    approver,
                    comment,
                    &cli.format,
                    cli.verbose,
                    &edition,
                )
                .map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Reject {
                    policy_id,
                    approver,
                    reason,
                }) => pl::cmd_reject(
                    policy_id,
                    approver,
                    reason,
                    &cli.format,
                    cli.verbose,
                    &edition,
                )
                .map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Activate { policy_id, actor }) => {
                    pl::cmd_activate(policy_id, actor, &cli.format, cli.verbose, &edition)
                        .map_err(|e| format!("{}", e).into())
                }
                Some(PolicyLifecycleCli::Deprecate {
                    policy_id,
                    actor,
                    reason,
                }) => pl::cmd_deprecate(
                    policy_id,
                    actor,
                    reason.unwrap_or_default(),
                    &cli.format,
                    cli.verbose,
                    &edition,
                )
                .map_err(|e| format!("{}", e).into()),
                Some(PolicyLifecycleCli::Status { policy_id }) => {
                    pl::cmd_status(policy_id, &cli.format, cli.verbose, &edition)
                        .map_err(|e| format!("{}", e).into())
                }
                Some(PolicyLifecycleCli::History { policy_id }) => {
                    pl::cmd_history(policy_id, &cli.format, cli.verbose, &edition)
                        .map_err(|e| format!("{}", e).into())
                }
                Some(PolicyLifecycleCli::Diff {
                    policy_id,
                    from,
                    to,
                }) => pl::cmd_diff(policy_id, from, to, &cli.format, cli.verbose, &edition)
                    .map_err(|e| format!("{}", e).into()),
                None => Err("No policy-lifecycle subcommand provided".into()),
            }
        }
        Commands::Usage {
            days_in_month,
            command,
        } => {
            let usage_override: Option<Result<String, String>> = if let Some(d) = days_in_month {
                let parts: Vec<&str> = d.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(y), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                        let days = costpilot::cli::usage::days_in_month(y, m);
                        Some(Ok(days.to_string()))
                    } else {
                        Some(Err(
                            "Invalid --days-in-month format, expected YYYY-MM".to_string()
                        ))
                    }
                } else {
                    Some(Err(
                        "Invalid --days-in-month format, expected YYYY-MM".to_string()
                    ))
                }
            } else {
                None
            };

            use costpilot::cli::usage as usage_mod;

            let res: Result<String, String> = match usage_override {
                Some(r) => r,
                None => match command {
                    Some(UsageCli::Report {
                        team_id,
                        start,
                        end,
                        format: _,
                    }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Report {
                        team_id,
                        start,
                        end,
                        format: usage_mod::OutputFormat::Text,
                    }),
                    Some(UsageCli::Export {
                        start,
                        end,
                        format: _,
                        output,
                    }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Export {
                        start,
                        end,
                        format: usage_mod::ExportFormat::Json,
                        output,
                    }),
                    Some(UsageCli::Pr {
                        pr_number,
                        repository,
                    }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Pr {
                        pr_number,
                        repository,
                    }),
                    Some(UsageCli::Chargeback {
                        org_id,
                        start,
                        end,
                        format: _,
                        output,
                    }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Chargeback {
                        org_id,
                        start,
                        end,
                        format: usage_mod::OutputFormat::Text,
                        output,
                    }),
                    Some(UsageCli::Invoice {
                        team_id,
                        start,
                        end,
                    }) => usage_mod::execute_usage_command(usage_mod::UsageCommand::Invoice {
                        team_id,
                        start,
                        end,
                    }),
                    None => usage_mod::execute_usage_command(usage_mod::UsageCommand::Report {
                        team_id: "all".to_string(),
                        start: None,
                        end: None,
                        format: usage_mod::OutputFormat::Text,
                    }),
                },
            };

            match res {
                Ok(out) => {
                    println!("{}", out);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Commands::Group(group_cmd) => {
            costpilot::cli::group::execute_group_command(group_cmd, &edition)
        }
        Commands::Validate { files, fail_fast } => {
            cmd_validate(files, &cli.format, fail_fast, &edition)
        }
        Commands::Version { detailed } => {
            cmd_version(detailed, &edition);
            return Ok(());
        }
    };

    result
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
    if edition.is_free() {
        return Err("Autofix requires Premium edition.".into());
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
        return Err("SLO features require CostPilot Premium".into());
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
        PolicyCommands::Increment {
            policy_id,
            changelog,
        } => cmd_policy_increment(policy_id, changelog, format, verbose, edition),
    }
}

fn cmd_exemption(
    command: ExemptionCommands,
    format: &str,
    verbose: bool,
    _edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Exemption workflow requires premium edition
    if !_edition.is_premium() {
        return Err("Exemption workflow requires premium edition. Upgrade to CostPilot Premium for governance features.".into());
    }

    use colored::*;
    use costpilot::engines::policy::exemption_ci;
    use costpilot::engines::policy::exemption_validator::ExemptionValidator;

    match command {
        ExemptionCommands::Validate { file } => {
            println!(
                "{}",
                format!("🔍 Validating exemptions file '{}'...", file.display())
                    .bright_blue()
                    .bold()
            );

            let validator = ExemptionValidator::new();
            let exemptions_file = validator.load_from_file(&file)?;

            println!();
            println!("{}", "✅ Exemptions file is valid".bright_green().bold());
            println!();
            println!("Version: {}", exemptions_file.version);
            println!("Exemptions: {}", exemptions_file.exemptions.len());

            if let Some(metadata) = &exemptions_file.metadata {
                if let Some(owner) = &metadata.owner {
                    println!("Owner: {}", owner);
                }
                if let Some(reviewed) = &metadata.last_reviewed {
                    println!("Last reviewed: {}", reviewed);
                }
            }

            Ok(())
        }
        ExemptionCommands::Check { file } => {
            println!(
                "{}",
                format!("🚦 Checking exemptions for CI: '{}'...", file.display())
                    .bright_blue()
                    .bold()
            );

            let validator = ExemptionValidator::new();
            let exemptions_file = validator.load_from_file(&file)?;
            let check_result = exemption_ci::check_exemptions_for_ci(&exemptions_file)?;

            println!("{}", check_result.summary());

            if check_result.expired_exemptions > 0 {
                eprintln!(
                    "\n❌ CI BLOCKED: {} expired exemption(s) found",
                    check_result.expired_exemptions
                );
                std::process::exit(exemption_ci::EXIT_EXEMPTION_EXPIRED);
            } else if check_result.invalid_exemptions > 0 {
                eprintln!(
                    "\n⚠️  WARNING: {} invalid exemption(s) found",
                    check_result.invalid_exemptions
                );
                std::process::exit(exemption_ci::EXIT_VALIDATION_ERROR);
            } else {
                println!("\n✅ All exemptions are valid");
            }

            Ok(())
        }
        ExemptionCommands::List {
            file,
            expired,
            expiring,
        } => {
            println!(
                "{}",
                format!("📋 Listing exemptions from '{}'...", file.display())
                    .bright_blue()
                    .bold()
            );

            let validator = ExemptionValidator::new();
            let exemptions_file = validator.load_from_file(&file)?;

            println!();
            println!("{}", "Exemption Status Summary".bright_white().bold());
            println!("{}", "━".repeat(80).bright_black());

            let mut active_count = 0;
            let mut expiring_count = 0;
            let mut expired_count = 0;
            let mut invalid_count = 0;

            for exemption in &exemptions_file.exemptions {
                let status = validator.check_status(exemption);

                let should_show = match status {
                    _ if expired && matches!(status, ExemptionStatus::Expired { .. }) => true,
                    _ if expiring && matches!(status, ExemptionStatus::ExpiringSoon { .. }) => true,
                    _ if !expired && !expiring => true,
                    _ => false,
                };

                if !should_show {
                    continue;
                }

                let (status_icon, _status_color) = match status {
                    ExemptionStatus::Active => {
                        active_count += 1;
                        ("✅", "green")
                    }
                    ExemptionStatus::ExpiringSoon { .. } => {
                        expiring_count += 1;
                        ("⚠️ ", "yellow")
                    }
                    ExemptionStatus::Expired { .. } => {
                        expired_count += 1;
                        ("❌", "red")
                    }
                    ExemptionStatus::Invalid { .. } => {
                        invalid_count += 1;
                        ("🔴", "red")
                    }
                };

                let status_text = match status {
                    ExemptionStatus::Active => "Active".green(),
                    ExemptionStatus::ExpiringSoon { expires_in_days } => {
                        format!("Expires in {} days", expires_in_days).yellow()
                    }
                    ExemptionStatus::Expired { expired_on } => {
                        format!("Expired {}", expired_on).red()
                    }
                    ExemptionStatus::Invalid { reason } => format!("Invalid: {}", reason).red(),
                };

                println!(
                    "{} {} - {} ({})",
                    status_icon,
                    exemption.id.bright_white().bold(),
                    exemption.policy_name,
                    status_text
                );
                println!("  Resource: {}", exemption.resource_pattern);
                println!("  Expires: {}", exemption.expires_at);
                if verbose {
                    println!("  Approved by: {}", exemption.approved_by);
                    println!("  Justification: {}", exemption.justification);
                    if let Some(ticket) = &exemption.ticket_ref {
                        println!("  Ticket: {}", ticket);
                    }
                }
                println!();
            }

            if !expired && !expiring {
                println!("Summary:");
                println!("  Active: {}", active_count.to_string().green());
                println!("  Expiring soon: {}", expiring_count.to_string().yellow());
                println!("  Expired: {}", expired_count.to_string().red());
                println!("  Invalid: {}", invalid_count.to_string().red());
            }

            Ok(())
        }
        ExemptionCommands::Status { file, exemption_id } => {
            println!(
                "{}",
                format!("📊 Checking status of exemption '{}'...", exemption_id)
                    .bright_blue()
                    .bold()
            );

            let validator = ExemptionValidator::new();
            let exemptions_file = validator.load_from_file(&file)?;

            let exemption = exemptions_file
                .exemptions
                .iter()
                .find(|e| e.id == exemption_id);
            let exemption = match exemption {
                Some(e) => e,
                None => {
                    return Err(format!("Exemption '{}' not found in file", exemption_id).into());
                }
            };

            let status = validator.check_status(exemption);

            match format {
                "json" => {
                    let status_str = match status {
                        ExemptionStatus::Active => "active",
                        ExemptionStatus::ExpiringSoon { .. } => "expiring_soon",
                        ExemptionStatus::Expired { .. } => "expired",
                        ExemptionStatus::Invalid { .. } => "invalid",
                    };
                    let output = serde_json::json!({
                        "id": exemption.id,
                        "policy_name": exemption.policy_name,
                        "resource_pattern": exemption.resource_pattern,
                        "expires_at": exemption.expires_at,
                        "status": status_str,
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                _ => {
                    println!();
                    println!("{}", "Exemption Status".bright_white().bold());
                    println!("{}", "━".repeat(60).bright_black());
                    println!();
                    println!("ID: {}", exemption.id.bright_white());
                    println!("Policy: {}", exemption.policy_name);
                    println!("Resource: {}", exemption.resource_pattern);
                    println!("Expires: {}", exemption.expires_at);
                    println!("Approved by: {}", exemption.approved_by);
                    println!();

                    let status_display = match status {
                        ExemptionStatus::Active => "✅ Active".green(),
                        ExemptionStatus::ExpiringSoon { expires_in_days } => {
                            format!("⚠️  Expiring soon ({} days)", expires_in_days).yellow()
                        }
                        ExemptionStatus::Expired { expired_on } => {
                            format!("❌ Expired (on {})", expired_on).red()
                        }
                        ExemptionStatus::Invalid { reason } => {
                            format!("🔴 Invalid: {}", reason).red()
                        }
                    };

                    println!("Status: {}", status_display);
                    println!();
                    println!("Justification: {}", exemption.justification);

                    if let Some(ticket) = &exemption.ticket_ref {
                        println!("Ticket: {}", ticket);
                    }
                }
            }

            Ok(())
        }
    }
}

fn cmd_trend(
    command: TrendCommands,
    _format: &str,
    verbose: bool,
    edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use costpilot::engines::detection::DetectionEngine;
    use costpilot::engines::prediction::PredictionEngine;
    use costpilot::engines::trend::{SnapshotManager, TrendEngine};
    use std::path::PathBuf;

    // Trend analysis requires premium edition
    if edition.is_free() {
        return Err("Trend analysis requires CostPilot Premium. Upgrade to access cost tracking and visualization features.".into());
    }

    let snapshots_dir = PathBuf::from(".costpilot/snapshots");

    match command {
        TrendCommands::Snapshot {
            plan,
            commit,
            branch,
            id: _,
        } => {
            println!(
                "{}",
                format!("📸 Creating cost snapshot from '{}'...", plan.display())
                    .bright_blue()
                    .bold()
            );

            // Load and parse the plan
            let plan_content = std::fs::read_to_string(&plan)
                .map_err(|e| format!("Failed to read plan file: {}", e))?;

            let detection_engine = DetectionEngine::new();
            let mut prediction_engine = PredictionEngine::new()?;

            // Parse plan and generate estimates
            let changes = detection_engine.detect_from_terraform_json(&plan_content)?;
            let estimates = prediction_engine.predict(&changes)?;

            // Create trend engine and snapshot
            let trend_engine = TrendEngine::new(&snapshots_dir, edition)?;

            let snapshot = trend_engine.create_snapshot(
                estimates,
                commit.or_else(|| std::env::var("GIT_COMMIT").ok()),
                branch,
            )?;

            let manager = SnapshotManager::new(&snapshots_dir);
            manager.write_snapshot(&snapshot)?;

            println!(
                "{}",
                format!("✅ Snapshot '{}' created successfully", snapshot.id)
                    .bright_green()
                    .bold()
            );

            if verbose {
                if let Ok(ts) = snapshot.get_timestamp() {
                    println!("  Created: {}", ts.format("%Y-%m-%d %H:%M:%S"));
                }
                if let Some(commit) = &snapshot.commit_hash {
                    println!("  Commit: {}", commit);
                }
                if let Some(branch) = &snapshot.branch {
                    println!("  Branch: {}", branch);
                }
                let resource_count: usize =
                    snapshot.modules.values().map(|m| m.resource_count).sum();
                println!("  Total resources: {}", resource_count);
                println!("  Total monthly cost: ${:.2}", snapshot.total_monthly_cost);
            }

            Ok(())
        }

        TrendCommands::List { verbose, limit } => {
            println!("{}", "📋 Listing cost snapshots...".bright_blue().bold());

            let manager = SnapshotManager::new(&snapshots_dir);
            let history = manager.load_history()?;
            let mut snapshots = history.snapshots.clone();

            // Sort by timestamp (newest first)
            snapshots.sort_by_key(|b| std::cmp::Reverse(b.get_timestamp().unwrap()));

            let display_limit = limit.unwrap_or(10);
            let to_display = if snapshots.len() > display_limit {
                &snapshots[0..display_limit]
            } else {
                &snapshots
            };

            if to_display.is_empty() {
                println!("No snapshots found.");
                return Ok(());
            }

            println!();
            println!("{}", "Recent Snapshots".bright_white().bold());
            println!("{}", "━".repeat(80).bright_black());

            for snapshot in to_display {
                let resource_count: usize =
                    snapshot.modules.values().map(|m| m.resource_count).sum();
                println!(
                    "{} {} - ${:.2}/mo ({} resources)",
                    snapshot.get_timestamp().unwrap().format("%Y-%m-%d %H:%M"),
                    snapshot.id.bright_white(),
                    snapshot.total_monthly_cost,
                    resource_count
                );

                if verbose {
                    if let Some(commit) = &snapshot.commit_hash {
                        println!("  Commit: {}", commit);
                    }
                    if let Some(branch) = &snapshot.branch {
                        println!("  Branch: {}", branch);
                    }
                }
            }

            if snapshots.len() > display_limit {
                println!();
                println!("... and {} more snapshots", snapshots.len() - display_limit);
            }

            Ok(())
        }

        TrendCommands::Graph {
            format: output_format,
            output,
            days,
            branch,
        } => {
            println!(
                "{}",
                "📊 Generating trend visualization...".bright_blue().bold()
            );

            let trend_engine = TrendEngine::new(&snapshots_dir, edition)?;
            let mut history = trend_engine.load_history()?;

            // Filter by branch if specified
            if let Some(branch_filter) = &branch {
                history
                    .snapshots
                    .retain(|s| s.branch.as_ref() == Some(branch_filter));
            }

            // Filter by days if specified
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
            history
                .snapshots
                .retain(|s| s.get_timestamp().is_ok_and(|ts| ts > cutoff));

            // Sort by timestamp
            history
                .snapshots
                .sort_by_key(|a| a.get_timestamp().unwrap());

            if history.snapshots.is_empty() {
                return Err("No snapshots found matching the specified criteria.".into());
            }

            match output_format.as_str() {
                "svg" => {
                    let svg_content = trend_engine
                        .svg_generator
                        .generate(&history)
                        .map_err(|e| format!("Failed to generate SVG: {}", e))?;

                    match output {
                        Some(path) => {
                            std::fs::write(&path, &svg_content)
                                .map_err(|e| format!("Failed to write SVG file: {}", e))?;
                            println!(
                                "{}",
                                format!("✅ SVG graph saved to '{}'", path.display())
                                    .bright_green()
                                    .bold()
                            );
                        }
                        None => {
                            println!("{}", svg_content);
                        }
                    }
                }
                "html" => {
                    let svg_content = trend_engine
                        .svg_generator
                        .generate(&history)
                        .map_err(|e| format!("Failed to generate SVG: {}", e))?;
                    let html_content = costpilot::engines::trend::HtmlGenerator::wrap_svg(
                        &svg_content,
                        "Cost Trend Analysis",
                    );

                    match output {
                        Some(path) => {
                            std::fs::write(&path, &html_content)
                                .map_err(|e| format!("Failed to write HTML file: {}", e))?;
                            println!(
                                "{}",
                                format!("✅ HTML report saved to '{}'", path.display())
                                    .bright_green()
                                    .bold()
                            );
                        }
                        None => {
                            println!("{}", html_content);
                        }
                    }
                }
                _ => {
                    return Err(format!("Unsupported output format: {}", output_format).into());
                }
            }

            Ok(())
        }

        TrendCommands::Diff { from, to, verbose } => {
            println!(
                "{}",
                format!("🔍 Comparing snapshots '{}' and '{}'...", from, to)
                    .bright_blue()
                    .bold()
            );

            let manager = SnapshotManager::new(&snapshots_dir);
            let from_snapshot = manager
                .read_snapshot(&from)
                .map_err(|e| format!("Failed to read snapshot '{}': {}", from, e))?;
            let to_snapshot = manager
                .read_snapshot(&to)
                .map_err(|e| format!("Failed to read snapshot '{}': {}", to, e))?;

            let diff = costpilot::engines::trend::TrendDiffGenerator::generate_diff(
                &from_snapshot,
                &to_snapshot,
            );

            println!();
            println!("{}", "Trend Comparison".bright_white().bold());
            println!("{}", "━".repeat(50).bright_black());
            println!(
                "From: {} ({})",
                diff.from_snapshot,
                diff.time_range.split(" → ").next().unwrap_or("")
            );
            println!(
                "To:   {} ({})",
                diff.to_snapshot,
                diff.time_range.split(" → ").last().unwrap_or("")
            );
            println!(
                "Total Cost Change: ${:.2} ({:.1}%)",
                diff.total_cost_delta, diff.total_cost_percent
            );
            println!();

            println!("Module Changes:");
            for change in &diff.module_changes {
                if change.delta.abs() > 0.01 || verbose {
                    println!(
                        "  {}: ${:.2} → ${:.2} ({}${:.2}, {:.1}%) - {}",
                        change.module,
                        change.cost_before,
                        change.cost_after,
                        if change.delta >= 0.0 { "+" } else { "" },
                        change.delta,
                        change.percent,
                        match change.change_type {
                            costpilot::engines::trend::ChangeType::Added => "ADDED",
                            costpilot::engines::trend::ChangeType::Removed => "REMOVED",
                            costpilot::engines::trend::ChangeType::Increased => "INCREASED",
                            costpilot::engines::trend::ChangeType::Decreased => "DECREASED",
                            costpilot::engines::trend::ChangeType::Unchanged => "UNCHANGED",
                        }
                    );
                }
            }

            if verbose {
                println!();
                println!("Service Changes:");
                for change in &diff.service_changes {
                    if change.delta.abs() > 0.01 {
                        println!(
                            "  {}: ${:.2} → ${:.2} ({}${:.2}, {:.1}%)",
                            change.service,
                            change.cost_before,
                            change.cost_after,
                            if change.delta >= 0.0 { "+" } else { "" },
                            change.delta,
                            change.percent
                        );
                    }
                }

                if !diff.new_regressions.is_empty() {
                    println!();
                    println!("New Regressions:");
                    for regression in &diff.new_regressions {
                        println!(
                            "  {}: {} (+${:.2}, {:.1}%) - {}",
                            regression.affected,
                            match regression.regression_type {
                                costpilot::engines::trend::RegressionType::NewResource => "NEW",
                                costpilot::engines::trend::RegressionType::CostIncrease =>
                                    "INCREASE",
                                costpilot::engines::trend::RegressionType::BudgetExceeded =>
                                    "BUDGET_EXCEEDED",
                                costpilot::engines::trend::RegressionType::UnexpectedService =>
                                    "UNEXPECTED_SERVICE",
                            },
                            regression.increase_amount,
                            regression.increase_percent,
                            regression.severity
                        );
                    }
                }
            }

            Ok(())
        }

        TrendCommands::Clean {
            keep,
            older_than,
            dry_run,
        } => {
            println!("{}", "🧹 Cleaning up old snapshots...".bright_blue().bold());

            let manager = SnapshotManager::new(&snapshots_dir);
            let history = manager.load_history()?;

            if dry_run {
                println!("{} Dry run mode - no files will be deleted", "ℹ️".yellow());
                println!();
            }

            let mut snapshots = history.snapshots.clone();
            snapshots.sort_by_key(|b| std::cmp::Reverse(b.get_timestamp().unwrap()));

            let to_delete: Vec<_> = if let Some(keep_count) = keep {
                // Keep the most recent N snapshots
                if snapshots.len() > keep_count {
                    snapshots[keep_count..]
                        .iter()
                        .map(|s| s.id.clone())
                        .collect()
                } else {
                    vec![]
                }
            } else if let Some(days) = older_than {
                // Delete snapshots older than N days
                let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
                snapshots
                    .iter()
                    .filter(|s| s.get_timestamp().is_ok_and(|ts| ts < cutoff))
                    .map(|s| s.id.clone())
                    .collect()
            } else {
                return Err("Must specify either --keep or --older-than".into());
            };

            if to_delete.is_empty() {
                println!("No snapshots to clean up.");
                return Ok(());
            }

            println!(
                "Found {} snapshots to {}",
                to_delete.len(),
                if dry_run {
                    "delete (dry run)"
                } else {
                    "delete"
                }
            );

            for snapshot_id in &to_delete {
                if let Some(snapshot) = snapshots.iter().find(|s| s.id == *snapshot_id) {
                    println!(
                        "  - {} ({})",
                        snapshot.id,
                        snapshot.get_timestamp().unwrap().format("%Y-%m-%d")
                    );
                }
            }

            if !dry_run {
                println!();
                for snapshot_id in &to_delete {
                    manager.delete_snapshot(snapshot_id)?;
                }
                println!(
                    "{}",
                    format!("✅ Deleted {} snapshots", to_delete.len())
                        .bright_green()
                        .bold()
                );
            }

            Ok(())
        }
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

fn cmd_policy_increment(
    policy_id: String,
    changelog: Option<String>,
    format: &str,
    _verbose: bool,
    _edition: &costpilot::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use costpilot::engines::policy::PolicyVersionManager;
    use std::path::PathBuf;

    println!(
        "{}",
        format!("📈 Incrementing version for policy '{}'...", policy_id)
            .bright_blue()
            .bold()
    );

    // For now, assume policy files are in a policies/ directory
    // In production, this would be configurable
    let version_file = PathBuf::from(format!("policies/{}.version.json", policy_id));
    let policy_file = PathBuf::from(format!("policies/{}.yaml", policy_id));

    // Check if policy file exists
    if !policy_file.exists() {
        return Err(format!("Policy file not found: {}", policy_file.display()).into());
    }

    // Read policy content
    let content = std::fs::read_to_string(&policy_file)?;

    // Create version manager
    let manager = PolicyVersionManager::new(&version_file);

    // Increment version (creates 1.0.0 if no version file exists, otherwise increments)
    let new_version = manager.increment_version(&content, changelog)?;

    match format {
        "json" => {
            let output = serde_json::json!({
                "policy_id": policy_id,
                "new_version": new_version.version,
                "content_hash": new_version.content_hash,
                "created_at": new_version.created_at,
                "changelog": new_version.changelog,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!();
            println!(
                "{}",
                "✅ Policy version incremented successfully!"
                    .bright_green()
                    .bold()
            );
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("New Version: {}", new_version.version.bright_green().bold());
            println!("Content Hash: {}", &new_version.content_hash[..16]);
            println!("Created: {}", new_version.created_at);
            if let Some(changelog) = new_version.changelog {
                println!("Changelog: {}", changelog);
            }
            println!();
            println!("Version file: {}", version_file.display());
        }
    }

    Ok(())
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
