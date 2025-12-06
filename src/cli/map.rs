// Mapping CLI commands for dependency visualization

use clap::Args;
use std::path::PathBuf;
use colored::Colorize;
use crate::engines::mapping::{MappingEngine, GraphvizConfig, JsonExportConfig, JsonFormat, ColorScheme};

#[derive(Debug, Args)]
pub struct MapCommand {
    /// Path to Terraform plan JSON file
    #[arg(long, value_name = "FILE")]
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

    /// Hide costs in visualization
    #[arg(long)]
    hide_costs: bool,

    /// Hide module grouping
    #[arg(long)]
    no_modules: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

pub fn execute_map_command(cmd: &MapCommand) -> Result<(), Box<dyn std::error::Error>> {
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
    
    let engine = MappingEngine::new();
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
            engine.export_json_with_config(&graph, config)?
        }
        "html" => {
            if cmd.verbose {
                println!("{}", "Generating HTML...".dimmed());
            }
            engine.generate_html(&graph, "Infrastructure Dependencies")?
        }
        _ => {
            return Err(format!("Unknown format: {}. Valid formats: mermaid, graphviz, json, html", cmd.format).into());
        }
    };

    // Write output
    if let Some(output_path) = &cmd.output {
        std::fs::write(output_path, &output_content)?;
        println!("{} Output written to {}", "✓".green(), output_path.display());
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
