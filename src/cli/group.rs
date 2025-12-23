// CLI commands for grouping operations

use crate::engines::grouping::GroupingEngine;
// use crate::parser::plan_parser::PlanParser; // TODO: Implement plan parser
use clap::{Args, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type alias for resource tuple: (address, type, service, tags, cost)
pub type ResourceTuple = (String, String, String, HashMap<String, String>, f64);

#[derive(Debug, Args)]
pub struct GroupCommand {
    #[command(subcommand)]
    pub command: GroupSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum GroupSubcommand {
    /// Group resources by Terraform module
    Module {
        /// Path to Terraform plan file (JSON format)
        plan: PathBuf,

        /// Show hierarchical tree view
        #[arg(short = 't', long)]
        tree: bool,

        /// Minimum cost threshold to include
        #[arg(short = 'm', long, default_value = "0.0")]
        min_cost: f64,

        /// Maximum number of groups to show
        #[arg(short = 'n', long)]
        max_groups: Option<usize>,
    },

    /// Group resources by AWS service
    #[command(name = "by-service")]
    Service {
        /// Path to Terraform plan file (JSON format)
        plan: PathBuf,

        /// Group by service category
        #[arg(short, long)]
        by_category: bool,

        /// Minimum cost threshold to include
        #[arg(short = 'm', long, default_value = "0.0")]
        min_cost: f64,

        /// Maximum number of services to show
        #[arg(short = 'n', long)]
        max_groups: Option<usize>,
    },

    /// Group resources by environment (from tags)
    #[command(name = "by-environment")]
    Environment {
        /// Path to Terraform plan file (JSON format)
        plan: PathBuf,

        /// Show detailed breakdown per environment
        #[arg(long)]
        detailed: bool,

        /// Detect cost anomalies
        #[arg(short = 'a', long)]
        detect_anomalies: bool,

        /// Minimum cost threshold to include
        #[arg(short = 'm', long, default_value = "0.0")]
        min_cost: f64,
    },

    /// Generate cost attribution report
    Attribution {
        /// Path to Terraform plan file (JSON format)
        plan: PathBuf,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show top N cost centers
        #[arg(short = 'n', long, default_value = "10")]
        top_n: usize,
    },

    /// Generate comprehensive report across all dimensions
    All {
        /// Path to Terraform plan file (JSON format)
        plan: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

pub fn execute_group_command(
    cmd: GroupCommand,
    edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract plan path and subcommand
    let (plan, subcommand) = match cmd.command {
        GroupSubcommand::Module {
            plan,
            tree,
            min_cost,
            max_groups,
        } => (
            plan,
            GroupExecution::Module {
                tree,
                min_cost,
                max_groups,
            },
        ),
        GroupSubcommand::Service {
            plan,
            by_category,
            min_cost,
            max_groups,
        } => (
            plan,
            GroupExecution::Service {
                by_category,
                min_cost,
                max_groups,
            },
        ),
        GroupSubcommand::Environment {
            plan,
            detailed,
            detect_anomalies,
            min_cost,
        } => {
            // Gate anomaly detection behind premium license
            if detect_anomalies {
                crate::edition::require_premium(edition, "Cost anomaly detection")
                    .map_err(|e| format!("Anomaly detection requires premium license: {}", e))?;
            }
            (
                plan,
                GroupExecution::Environment {
                    detailed,
                    detect_anomalies,
                    min_cost,
                },
            )
        }
        GroupSubcommand::Attribution {
            plan,
            format,
            output,
            top_n,
        } => (
            plan,
            GroupExecution::Attribution {
                format,
                output,
                top_n,
            },
        ),
        GroupSubcommand::All {
            plan,
            format,
            output,
        } => (plan, GroupExecution::All { format, output }),
    };

    // Load and parse the plan using detection engine
    use crate::engines::detection::DetectionEngine;
    let detection = DetectionEngine::new();
    let resources = detection.detect_from_terraform_plan(&plan)?;

    let engine = GroupingEngine::new();

    match subcommand {
        GroupExecution::Module {
            tree,
            min_cost,
            max_groups,
        } => {
            execute_group_module(&engine, &resources, tree, min_cost, max_groups)?;
        }
        GroupExecution::Service {
            by_category,
            min_cost,
            max_groups,
        } => {
            execute_group_service(&engine, &resources, by_category, min_cost, max_groups)?;
        }
        GroupExecution::Environment {
            detailed,
            detect_anomalies,
            min_cost,
        } => {
            execute_group_environment(&engine, &resources, detailed, detect_anomalies, min_cost)?;
        }
        GroupExecution::Attribution {
            format,
            output,
            top_n,
        } => {
            execute_attribution(&engine, &resources, &format, output, top_n)?;
        }
        GroupExecution::All { format, output } => {
            execute_comprehensive(&engine, &resources, &format, output)?;
        }
    }

    Ok(())
}

#[derive(Debug)]
enum GroupExecution {
    Module {
        tree: bool,
        min_cost: f64,
        max_groups: Option<usize>,
    },
    Service {
        by_category: bool,
        min_cost: f64,
        max_groups: Option<usize>,
    },
    Environment {
        detailed: bool,
        detect_anomalies: bool,
        min_cost: f64,
    },
    Attribution {
        format: String,
        output: Option<PathBuf>,
        top_n: usize,
    },
    All {
        format: String,
        output: Option<PathBuf>,
    },
}

fn execute_group_module(
    engine: &GroupingEngine,
    resources: &[crate::engines::shared::models::ResourceChange],
    tree: bool,
    min_cost: f64,
    max_groups: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_resources: Vec<(String, String, f64)> = resources
        .iter()
        .filter_map(|r| {
            if let Some(cost) = r.monthly_cost {
                if cost >= min_cost {
                    return Some((r.resource_id.clone(), r.resource_type.clone(), cost));
                }
            }
            None
        })
        .collect();

    let mut groups = engine.group_by_module(&module_resources);

    if let Some(max) = max_groups {
        groups.truncate(max);
    }

    println!("Module Grouping Report");
    println!("=====================\n");

    let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();
    println!("Total Monthly Cost: ${:.2}\n", total_cost);

    if tree {
        println!(
            "{}",
            crate::engines::grouping::generate_module_tree(&groups)
        );
    } else {
        for (i, group) in groups.iter().enumerate() {
            let percentage = if total_cost > 0.0 {
                (group.monthly_cost / total_cost) * 100.0
            } else {
                0.0
            };
            println!(
                "{}. {} - ${:.2}/mo ({:.1}%, {} resources)",
                i + 1,
                group.module_path,
                group.monthly_cost,
                percentage,
                group.resource_count
            );
        }
    }

    Ok(())
}

fn execute_group_service(
    engine: &GroupingEngine,
    resources: &[crate::engines::shared::models::ResourceChange],
    by_category: bool,
    min_cost: f64,
    max_groups: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_resources: Vec<(String, String, f64)> = resources
        .iter()
        .filter_map(|r| {
            if let Some(cost) = r.monthly_cost {
                if cost >= min_cost {
                    return Some((r.resource_id.clone(), r.resource_type.clone(), cost));
                }
            }
            None
        })
        .collect();

    let mut groups = engine.group_by_service(&service_resources);

    if let Some(max) = max_groups {
        groups.truncate(max);
    }

    if by_category {
        let category_costs = crate::engines::grouping::cost_by_category(&groups);
        let mut categories: Vec<_> = category_costs.into_iter().collect();
        categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("Service Grouping by Category");
        println!("===========================\n");

        let total_cost: f64 = categories.iter().map(|(_, c)| c).sum();
        println!("Total Monthly Cost: ${:.2}\n", total_cost);

        for (category, cost) in categories {
            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            println!(
                "{}: ${:.2}/mo ({:.1}%)",
                category.as_str(),
                cost,
                percentage
            );
        }
    } else {
        println!(
            "{}",
            crate::engines::grouping::generate_service_report(&groups)
        );
    }

    Ok(())
}

fn execute_group_environment(
    engine: &GroupingEngine,
    resources: &[crate::engines::shared::models::ResourceChange],
    detailed: bool,
    detect_anomalies: bool,
    min_cost: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let env_resources: Vec<ResourceTuple> = resources
        .iter()
        .filter_map(|r| {
            if let Some(cost) = r.monthly_cost {
                if cost >= min_cost {
                    let (service, _) = crate::engines::grouping::by_service::extract_service_info(
                        &r.resource_type,
                    );
                    return Some((
                        r.resource_id.clone(),
                        r.resource_type.clone(),
                        service,
                        r.tags.clone(),
                        cost,
                    ));
                }
            }
            None
        })
        .collect();

    let groups = engine.group_by_environment(&env_resources);

    if detailed || detect_anomalies {
        println!(
            "{}",
            crate::engines::grouping::generate_environment_report(&groups)
        );
    } else {
        println!("Environment Grouping Report");
        println!("==========================\n");

        let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();
        println!("Total Monthly Cost: ${:.2}\n", total_cost);

        for group in &groups {
            let percentage = if total_cost > 0.0 {
                (group.monthly_cost / total_cost) * 100.0
            } else {
                0.0
            };
            println!(
                "{}: ${:.2}/mo ({:.1}%, {} resources)",
                group.environment, group.monthly_cost, percentage, group.resource_count
            );
        }
    }

    Ok(())
}

fn execute_attribution(
    engine: &GroupingEngine,
    resources: &[crate::engines::shared::models::ResourceChange],
    format: &str,
    output: Option<PathBuf>,
    _top_n: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let attr_resources: Vec<(String, String, f64, HashMap<String, String>)> = resources
        .iter()
        .filter_map(|r| {
            r.monthly_cost.map(|cost| {
                (
                    r.resource_id.clone(),
                    r.resource_type.clone(),
                    cost,
                    r.tags.clone(),
                )
            })
        })
        .collect();

    let report = engine.generate_attribution_report(&attr_resources);

    let content = match format {
        "json" => report.to_json()?,
        "csv" => report.export_csv(),
        _ => report.format_text(),
    };

    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("Attribution report written successfully");
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn execute_comprehensive(
    engine: &GroupingEngine,
    resources: &[crate::engines::shared::models::ResourceChange],
    format: &str,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let comp_resources: Vec<(String, String, HashMap<String, String>, f64)> = resources
        .iter()
        .filter_map(|r| {
            r.monthly_cost.map(|cost| {
                (
                    r.resource_id.clone(),
                    r.resource_type.clone(),
                    r.tags.clone(),
                    cost,
                )
            })
        })
        .collect();

    let report = engine.generate_comprehensive_report(&comp_resources);

    let content = match format {
        "json" => report.to_json()?,
        _ => report.format_text(),
    };

    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("Comprehensive report written successfully");
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::EditionContext;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_mock_terraform_plan() -> NamedTempFile {
        let plan_content = r#"{
            "format_version": "1.1",
            "terraform_version": "1.5.0",
            "resource_changes": [
                {
                    "address": "aws_instance.web",
                    "mode": "managed",
                    "type": "aws_instance",
                    "name": "web",
                    "provider_name": "registry.terraform.io/hashicorp/aws",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {
                            "instance_type": "t2.micro",
                            "tags": {
                                "Environment": "prod",
                                "Module": "web"
                            }
                        }
                    }
                },
                {
                    "address": "aws_s3_bucket.data",
                    "mode": "managed",
                    "type": "aws_s3_bucket",
                    "name": "data",
                    "provider_name": "registry.terraform.io/hashicorp/aws",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {
                            "tags": {
                                "Environment": "prod",
                                "Module": "data"
                            }
                        }
                    }
                },
                {
                    "address": "aws_db_instance.database",
                    "mode": "managed",
                    "type": "aws_db_instance",
                    "name": "database",
                    "provider_name": "registry.terraform.io/hashicorp/aws",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {
                            "instance_class": "db.t2.micro",
                            "tags": {
                                "Environment": "dev",
                                "Module": "database"
                            }
                        }
                    }
                }
            ],
            "configuration": {
                "root_module": {}
            }
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, plan_content).unwrap();
        temp_file
    }

    #[test]
    fn test_execute_group_command_module() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::Module {
                plan: plan_path,
                tree: false,
                min_cost: 0.0,
                max_groups: Some(5),
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_service() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::Service {
                plan: plan_path,
                by_category: false,
                min_cost: 0.0,
                max_groups: Some(5),
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_environment() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::Environment {
                plan: plan_path,
                detailed: false,
                detect_anomalies: false,
                min_cost: 0.0,
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_attribution() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::Attribution {
                plan: plan_path,
                format: "text".to_string(),
                output: None,
                top_n: 10,
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_attribution_json() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::Attribution {
                plan: plan_path,
                format: "json".to_string(),
                output: None,
                top_n: 10,
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_all() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::All {
                plan: plan_path,
                format: "text".to_string(),
                output: None,
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_group_command_all_json() {
        let temp_file = create_mock_terraform_plan();
        let plan_path = temp_file.path().to_path_buf();
        let edition = EditionContext::default();

        let cmd = GroupCommand {
            command: GroupSubcommand::All {
                plan: plan_path,
                format: "json".to_string(),
                output: None,
            },
        };

        let result = execute_group_command(cmd, &edition);
        assert!(result.is_ok());
    }
}
