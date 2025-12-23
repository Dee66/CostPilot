// Mapping CLI commands for dependency visualization

use crate::engines::mapping::{
    ColorScheme, GraphvizConfig, JsonExportConfig, JsonFormat, MappingEngine,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::validation::OutputValidator;
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct MapCommand {
    /// Path to Terraform plan JSON file
    plan: PathBuf,

    /// Output format: mermaid, graphviz, json, html
    #[arg(short, long, default_value = "mermaid")]
    format: String,

    /// Output file path (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// JSON export format variant (standard, adjacency, cytoscape, d3)
    #[arg(long, default_value = "standard")]
    json_format: String,

    /// Graphviz layout direction (LR, TB, RL, BT)
    #[arg(long, default_value = "LR")]
    rankdir: String,

    /// Color scheme for graphviz (cost, type, mono)
    #[arg(long, default_value = "cost")]
    color_scheme: String,

    /// Maximum depth for dependency traversal (default: 5, Premium required for > 1)
    #[arg(long)]
    max_depth: Option<u32>,

    /// Hide costs in visualization
    #[arg(long)]
    hide_costs: bool,

    /// Hide module grouping
    #[arg(long)]
    no_modules: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Analyze cross-service cost impacts
    #[arg(long)]
    cost_impacts: bool,
}

pub fn execute_map_command(
    cmd: &MapCommand,
    edition: &crate::edition::EditionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check depth gating
    let max_depth = cmd.max_depth.unwrap_or(5);
    if max_depth > 1 {
        crate::edition::require_premium(edition, "Deep mapping")?;
    }

    println!("{}", "📊 CostPilot Dependency Mapper".bold().cyan());
    println!();

    // Load and parse plan
    if cmd.verbose {
        println!("{}", "Loading Terraform plan...".dimmed());
    }
    let plan_content = std::fs::read_to_string(&cmd.plan)?;
    let plan: serde_json::Value = serde_json::from_str(&plan_content)?;

    // Extract resource changes
    let changes = crate::cli::utils::extract_resource_changes(&plan)?;

    if cmd.verbose {
        println!("   Found {} resource changes", changes.len());
        println!();
    }

    // Build dependency graph
    if cmd.verbose {
        println!("{}", "Building dependency graph...".dimmed());
    }

    let graph_config = if edition.capabilities.allow_mapping_deep {
        // Premium: no depth limit
        crate::engines::mapping::GraphConfig::default()
    } else {
        // Free: max depth 1
        crate::engines::mapping::GraphConfig {
            max_depth: Some(1),
            ..Default::default()
        }
    };

    let mut engine = MappingEngine::with_config(
        graph_config,
        crate::engines::mapping::MermaidConfig::default(),
        edition,
    );
    let graph = engine.build_graph(&changes)?;

    if cmd.verbose {
        println!("   Nodes: {}", graph.nodes.len());
        println!("   Edges: {}", graph.edges.len());
        println!();
    }

    // Generate output based on format
    let output_content = match cmd.format.as_str() {
        "mermaid" => {
            if cmd.verbose {
                println!("{}", "Generating Mermaid diagram...".dimmed());
            }
            engine.generate_mermaid(&graph)?
        }
        "graphviz" | "dot" => {
            if cmd.verbose {
                println!("{}", "Generating Graphviz DOT...".dimmed());
            }
            let config = GraphvizConfig {
                rankdir: cmd.rankdir.clone(),
                show_costs: !cmd.hide_costs,
                show_modules: !cmd.no_modules,
                color_scheme: parse_color_scheme(&cmd.color_scheme),
                ..Default::default()
            };
            engine.generate_graphviz_with_config(&graph, config)?
        }
        "json" => {
            if cmd.verbose {
                println!("{}", "Exporting to JSON...".dimmed());
            }
            let json_format = parse_json_format(&cmd.json_format);
            let config = JsonExportConfig {
                pretty: true,
                include_metadata: true,
                include_statistics: cmd.verbose,
                format: json_format,
            };
            let json_output = engine.export_json_with_config(&graph, config)?;

            // Validate JSON output against schema only for standard format
            if json_format == JsonFormat::Standard {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use crate::validation::output::OutputType;
                    let validator = OutputValidator::new()?;
                    validator.validate(OutputType::Mapping, &json_output)?;
                }
            }

            json_output
        }
        "html" => {
            if cmd.verbose {
                println!("{}", "Generating HTML...".dimmed());
            }
            engine.generate_html(&graph, "Infrastructure Dependencies")?
        }
        _ => {
            return Err(format!(
                "Unknown format: {}. Valid formats: mermaid, graphviz, json, html",
                cmd.format
            )
            .into());
        }
    };

    // Write output
    if let Some(output_path) = &cmd.output {
        std::fs::write(output_path, &output_content)?;
        println!(
            "{} Output written to {}",
            "✓".green(),
            output_path.display()
        );
    } else {
        println!("{}", output_content);
    }

    // Show additional information
    if cmd.verbose && cmd.format == "graphviz" {
        println!();
        println!("{}", "Graphviz Tips:".bold());
        println!("  • Render as PNG: dot -Tpng input.dot -o output.png");
        println!("  • Render as SVG: dot -Tsvg input.dot -o output.svg");
        println!("  • Render as PDF: dot -Tpdf input.dot -o output.pdf");
    }

    if cmd.verbose && cmd.format == "json" {
        println!();
        println!("{}", "JSON Format:".bold());
        println!("  Format variant: {}", cmd.json_format);
        match cmd.json_format.as_str() {
            "cytoscape" => println!("  Use with Cytoscape.js for interactive visualization"),
            "d3" => println!("  Use with D3.js force-directed graph layout"),
            "adjacency" => println!("  Adjacency list format for graph algorithms"),
            _ => println!("  Standard nodes/edges format"),
        }
    }

    // Show statistics
    if cmd.verbose {
        println!();
        println!("{}", "Graph Statistics:".bold());
        println!("  Total nodes: {}", graph.metadata.node_count);
        println!("  Total edges: {}", graph.metadata.edge_count);
        println!("  Max depth: {}", graph.metadata.max_depth);

        if let Some(total_cost) = graph.metadata.total_cost {
            println!("  Total monthly cost: ${:.2}", total_cost);
        }

        // Count by type
        let mut resource_count = 0;
        let mut service_count = 0;
        let mut module_count = 0;

        for node in &graph.nodes {
            match node.node_type {
                crate::engines::mapping::NodeType::Resource => resource_count += 1,
                crate::engines::mapping::NodeType::Service => service_count += 1,
                crate::engines::mapping::NodeType::Module => module_count += 1,
            }
        }

        println!();
        println!("  By type:");
        println!("    Resources: {}", resource_count);
        println!("    Services: {}", service_count);
        println!("    Modules: {}", module_count);
    }

    // Show cost impact analysis if requested
    if cmd.cost_impacts {
        println!();
        println!("{}", "Cost Impact Analysis:".bold());

        let impacts = engine.detect_cost_impacts(&graph);

        if impacts.is_empty() {
            println!("  No significant cost impacts detected");
        } else {
            for impact in &impacts {
                let severity_icon = match impact.severity {
                    crate::engines::mapping::ImpactSeverity::High => "🔴",
                    crate::engines::mapping::ImpactSeverity::Medium => "🟡",
                    crate::engines::mapping::ImpactSeverity::Low => "🟢",
                };

                println!(
                    "  {} {} (${:.2}/mo) - {} affected resources",
                    severity_icon,
                    impact.source_label,
                    impact.source_cost,
                    impact.affected_resources
                );

                if cmd.verbose {
                    println!("    {}", impact.description);
                }
            }
        }

        // Show cost propagation if verbose
        if cmd.verbose {
            println!();
            println!("{}", "Cost Propagation:".bold());

            let propagations = engine.cost_propagation_report(&graph);

            if propagations.is_empty() {
                println!("  No cost propagation detected");
            } else {
                for prop in propagations.iter().take(5) {
                    // Show top 5
                    println!(
                        "  {}: ${:.2} direct + ${:.2} downstream = ${:.2} total ({:.1}x)",
                        prop.resource_label,
                        prop.direct_cost,
                        prop.downstream_cost,
                        prop.total_propagated_cost,
                        prop.propagation_factor
                    );
                }

                if propagations.len() > 5 {
                    println!("  ... and {} more", propagations.len() - 5);
                }
            }
        }
    }

    Ok(())
}

fn parse_color_scheme(scheme: &str) -> ColorScheme {
    match scheme.to_lowercase().as_str() {
        "cost" => ColorScheme::CostBased,
        "type" => ColorScheme::TypeBased,
        "mono" | "monochrome" => ColorScheme::Monochrome,
        _ => ColorScheme::CostBased,
    }
}

fn parse_json_format(format: &str) -> JsonFormat {
    match format.to_lowercase().as_str() {
        "adjacency" | "adjacency_list" => JsonFormat::AdjacencyList,
        "cytoscape" | "cyto" => JsonFormat::Cytoscape,
        "d3" | "d3force" | "d3-force" => JsonFormat::D3Force,
        _ => JsonFormat::Standard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::EditionContext;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_edition() -> EditionContext {
        EditionContext {
            mode: crate::edition::EditionMode::Premium,
            license: None,
            pro_engine: None,
            capabilities: crate::edition::Capabilities {
                allow_predict: true,
                allow_explain_full: true,
                allow_autofix: true,
                allow_mapping_deep: true,
                allow_trend: true,
                allow_policy_enforce: true,
                allow_slo_enforce: true,
            },
            pro: Some(crate::edition::ProEngineHandle::stub(
                std::path::PathBuf::from("/tmp/stub"),
            )),
            paths: crate::edition::EditionPaths::default(),
        }
    }

    fn create_test_terraform_plan() -> serde_json::Value {
        serde_json::json!({
            "resource_changes": [
                {
                    "address": "aws_instance.example",
                    "change": {
                        "actions": ["create"],
                        "after": {
                            "instance_type": "t2.micro",
                            "ami": "ami-12345"
                        }
                    },
                    "type": "aws_instance"
                },
                {
                    "address": "aws_s3_bucket.example",
                    "change": {
                        "actions": ["create"],
                        "after": {
                            "bucket": "my-bucket"
                        }
                    },
                    "type": "aws_s3_bucket"
                }
            ]
        })
    }

    #[test]
    fn test_execute_map_command_mermaid_format() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "mermaid".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_graphviz_format() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "graphviz".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "TB".to_string(),
            color_scheme: "type".to_string(),
            max_depth: None,
            hide_costs: true,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_json_format() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "json".to_string(),
            output: None,
            json_format: "cytoscape".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_html_format() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "html".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_invalid_format() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "invalid".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("Unknown format: invalid"));
    }

    #[test]
    fn test_execute_map_command_nonexistent_plan() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("nonexistent.json");

        let cmd = MapCommand {
            plan: plan_path,
            format: "mermaid".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_map_command_with_output_file() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");
        let output_path = temp_dir.path().join("output.mmd");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "mermaid".to_string(),
            output: Some(output_path.clone()),
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_execute_map_command_verbose_mode() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "mermaid".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: true,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_max_depth_premium() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "mermaid".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: Some(10), // This should trigger premium check
            hide_costs: false,
            no_modules: false,
            verbose: false,
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_graphviz_verbose_tips() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "graphviz".to_string(),
            output: None,
            json_format: "standard".to_string(),
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: true, // This should trigger graphviz tips
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_map_command_json_verbose_tips() {
        let temp_dir = tempdir().unwrap();
        let plan_path = temp_dir.path().join("plan.json");

        let plan = create_test_terraform_plan();
        fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

        let cmd = MapCommand {
            plan: plan_path,
            format: "json".to_string(),
            output: None,
            json_format: "d3".to_string(), // Different format to trigger different tips
            rankdir: "LR".to_string(),
            color_scheme: "cost".to_string(),
            max_depth: None,
            hide_costs: false,
            no_modules: false,
            verbose: true, // This should trigger json tips
            cost_impacts: false,
        };

        let edition = create_test_edition();
        let result = execute_map_command(&cmd, &edition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_color_scheme() {
        assert!(matches!(parse_color_scheme("cost"), ColorScheme::CostBased));
        assert!(matches!(parse_color_scheme("type"), ColorScheme::TypeBased));
        assert!(matches!(
            parse_color_scheme("mono"),
            ColorScheme::Monochrome
        ));
        assert!(matches!(
            parse_color_scheme("monochrome"),
            ColorScheme::Monochrome
        ));
        assert!(matches!(
            parse_color_scheme("invalid"),
            ColorScheme::CostBased
        )); // default
    }

    #[test]
    fn test_parse_json_format() {
        assert!(matches!(
            parse_json_format("standard"),
            JsonFormat::Standard
        ));
        assert!(matches!(
            parse_json_format("adjacency"),
            JsonFormat::AdjacencyList
        ));
        assert!(matches!(
            parse_json_format("adjacency_list"),
            JsonFormat::AdjacencyList
        ));
        assert!(matches!(
            parse_json_format("cytoscape"),
            JsonFormat::Cytoscape
        ));
        assert!(matches!(parse_json_format("cyto"), JsonFormat::Cytoscape));
        assert!(matches!(parse_json_format("d3"), JsonFormat::D3Force));
        assert!(matches!(parse_json_format("d3force"), JsonFormat::D3Force));
        assert!(matches!(parse_json_format("d3-force"), JsonFormat::D3Force));
        assert!(matches!(parse_json_format("invalid"), JsonFormat::Standard)); // default
    }
}
