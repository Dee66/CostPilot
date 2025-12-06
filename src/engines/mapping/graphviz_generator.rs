// Graphviz DOT format generator for dependency graphs

use super::graph_types::{DependencyGraph, GraphNode, GraphEdge, NodeType, EdgeType};
use crate::errors::CostPilotError;
use std::collections::HashMap;

/// Configuration for Graphviz generation
#[derive(Debug, Clone)]
pub struct GraphvizConfig {
    /// Graph direction: LR (left-right), TB (top-bottom), RL, BT
    pub rankdir: String,
    
    /// Show cost information on nodes
    pub show_costs: bool,
    
    /// Show module grouping
    pub show_modules: bool,
    
    /// Color scheme for nodes
    pub color_scheme: ColorScheme,
    
    /// Font name
    pub font_name: String,
    
    /// Font size for nodes
    pub node_font_size: u32,
    
    /// Font size for edges
    pub edge_font_size: u32,
    
    /// Include legend
    pub include_legend: bool,
}

impl Default for GraphvizConfig {
    fn default() -> Self {
        Self {
            rankdir: "LR".to_string(),
            show_costs: true,
            show_modules: true,
            color_scheme: ColorScheme::CostBased,
            font_name: "Helvetica".to_string(),
            node_font_size: 12,
            edge_font_size: 10,
            include_legend: true,
        }
    }
}

/// Color scheme for nodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    /// Color based on cost (green=low, yellow=medium, red=high)
    CostBased,
    
    /// Color based on resource type
    TypeBased,
    
    /// Monochrome
    Monochrome,
}

/// Graphviz DOT generator
pub struct GraphvizGenerator {
    config: GraphvizConfig,
}

impl GraphvizGenerator {
    /// Create new generator with default config
    pub fn new() -> Self {
        Self {
            config: GraphvizConfig::default(),
        }
    }

    /// Create generator with custom config
    pub fn with_config(config: GraphvizConfig) -> Self {
        Self { config }
    }

    /// Generate DOT format string
    pub fn generate(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let mut output = String::new();

        // Graph header
        output.push_str("digraph CostPilot {\n");
        output.push_str(&format!("  rankdir={};\n", self.config.rankdir));
        output.push_str(&format!("  node [fontname=\"{}\", fontsize={}];\n", 
            self.config.font_name, self.config.node_font_size));
        output.push_str(&format!("  edge [fontname=\"{}\", fontsize={}];\n", 
            self.config.font_name, self.config.edge_font_size));
        output.push_str("  graph [pad=\"0.5\", nodesep=\"1\", ranksep=\"1\"];\n");
        output.push_str("\n");

        // Group nodes by module if enabled
        if self.config.show_modules {
            self.generate_module_subgraphs(graph, &mut output)?;
        } else {
            self.generate_nodes(graph, &mut output)?;
        }

        // Generate edges
        self.generate_edges(graph, &mut output)?;

        // Generate legend if enabled
        if self.config.include_legend {
            self.generate_legend(&mut output)?;
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate module-based subgraphs
    fn generate_module_subgraphs(&self, graph: &DependencyGraph, output: &mut String) -> Result<(), CostPilotError> {
        // Group nodes by module
        let mut module_nodes: HashMap<String, Vec<&GraphNode>> = HashMap::new();
        let mut ungrouped_nodes = Vec::new();

        for node in &graph.nodes {
            if let Some(module) = &node.module {
                module_nodes.entry(module.clone()).or_default().push(node);
            } else {
                ungrouped_nodes.push(node);
            }
        }

        // Generate subgraphs for each module
        for (module_name, nodes) in module_nodes.iter() {
            let cluster_name = format!("cluster_{}", sanitize_id(module_name));
            output.push_str(&format!("  subgraph {} {{\n", cluster_name));
            output.push_str(&format!("    label=\"Module: {}\";\n", escape_label(module_name)));
            output.push_str("    style=filled;\n");
            output.push_str("    color=lightgray;\n");
            output.push_str("\n");

            for node in nodes {
                self.generate_node(node, output, 2)?;
            }

            output.push_str("  }\n\n");
        }

        // Generate ungrouped nodes
        for node in ungrouped_nodes {
            self.generate_node(node, output, 1)?;
        }

        Ok(())
    }

    /// Generate all nodes (without module grouping)
    fn generate_nodes(&self, graph: &DependencyGraph, output: &mut String) -> Result<(), CostPilotError> {
        for node in &graph.nodes {
            self.generate_node(node, output, 1)?;
        }
        Ok(())
    }

    /// Generate single node
    fn generate_node(&self, node: &GraphNode, output: &mut String, indent: usize) -> Result<(), CostPilotError> {
        let indent_str = "  ".repeat(indent);
        let node_id = sanitize_id(&node.id);
        
        // Build label
        let mut label_parts = vec![escape_label(&node.label)];
        
        if let Some(ref resource_type) = node.resource_type {
            label_parts.push(format!("\\n{}", resource_type));
        }
        
        if self.config.show_costs {
            if let Some(cost) = node.monthly_cost {
                label_parts.push(format!("\\n${:.2}/mo", cost));
            }
        }
        
        let label = label_parts.join("");

        // Determine node style
        let (shape, color, style) = self.get_node_style(node);

        output.push_str(&format!(
            "{}\"{}\" [label=\"{}\", shape={}, fillcolor=\"{}\", style=\"{}\"];\n",
            indent_str, node_id, label, shape, color, style
        ));

        Ok(())
    }

    /// Get node visual style
    fn get_node_style(&self, node: &GraphNode) -> (&str, String, &str) {
        let shape = match node.node_type {
            NodeType::Resource => "box",
            NodeType::Service => "ellipse",
            NodeType::Module => "folder",
        };

        let color = match self.config.color_scheme {
            ColorScheme::CostBased => {
                if let Some(cost) = node.monthly_cost {
                    if cost > 1000.0 {
                        "indianred1".to_string()
                    } else if cost > 100.0 {
                        "gold".to_string()
                    } else {
                        "lightgreen".to_string()
                    }
                } else {
                    "lightblue".to_string()
                }
            }
            ColorScheme::TypeBased => {
                match node.node_type {
                    NodeType::Resource => "lightblue".to_string(),
                    NodeType::Service => "lightgreen".to_string(),
                    NodeType::Module => "lightyellow".to_string(),
                }
            }
            ColorScheme::Monochrome => "white".to_string(),
        };

        let style = "filled";

        (shape, color, style)
    }

    /// Generate edges
    fn generate_edges(&self, graph: &DependencyGraph, output: &mut String) -> Result<(), CostPilotError> {
        output.push_str("  // Edges\n");
        
        for edge in &graph.edges {
            let from_id = sanitize_id(&edge.from);
            let to_id = sanitize_id(&edge.to);
            
            let (style, color, label) = self.get_edge_style(edge);
            
            if let Some(label_text) = label {
                output.push_str(&format!(
                    "  \"{}\" -> \"{}\" [style=\"{}\", color=\"{}\", label=\"{}\"];\n",
                    from_id, to_id, style, color, escape_label(&label_text)
                ));
            } else {
                output.push_str(&format!(
                    "  \"{}\" -> \"{}\" [style=\"{}\", color=\"{}\"];\n",
                    from_id, to_id, style, color
                ));
            }
        }
        
        output.push_str("\n");
        Ok(())
    }

    /// Get edge visual style
    fn get_edge_style(&self, edge: &GraphEdge) -> (&str, &str, Option<String>) {
        let (style, color, edge_label) = match edge.relationship {
            EdgeType::DependsOn => ("solid", "black", Some("depends".to_string())),
            EdgeType::DataFlow => ("dashed", "blue", Some("data".to_string())),
            EdgeType::NetworkConnection => ("dotted", "green", Some("network".to_string())),
            EdgeType::CostAttribution => ("bold", "red", Some("cost".to_string())),
        };

        let label = if let Some(impact) = &edge.cost_impact {
            Some(format!("{} ({})", edge_label.unwrap_or_default(), impact))
        } else {
            edge_label
        };

        (style, color, label)
    }

    /// Generate legend
    fn generate_legend(&self, output: &mut String) -> Result<(), CostPilotError> {
        output.push_str("  // Legend\n");
        output.push_str("  subgraph cluster_legend {\n");
        output.push_str("    label=\"Legend\";\n");
        output.push_str("    style=dashed;\n");
        output.push_str("    color=gray;\n");
        output.push_str("\n");

        // Node type legend
        output.push_str("    legend_resource [label=\"Resource\", shape=box, fillcolor=\"lightblue\", style=\"filled\"];\n");
        output.push_str("    legend_service [label=\"Service\", shape=ellipse, fillcolor=\"lightgreen\", style=\"filled\"];\n");
        output.push_str("    legend_module [label=\"Module\", shape=folder, fillcolor=\"lightyellow\", style=\"filled\"];\n");
        output.push_str("\n");

        // Edge type legend
        output.push_str("    legend_edge1 [label=\"\", shape=point, width=0];\n");
        output.push_str("    legend_edge2 [label=\"\", shape=point, width=0];\n");
        output.push_str("    legend_edge3 [label=\"\", shape=point, width=0];\n");
        output.push_str("    legend_edge4 [label=\"\", shape=point, width=0];\n");
        output.push_str("    legend_edge5 [label=\"\", shape=point, width=0];\n");
        output.push_str("\n");
        output.push_str("    legend_edge1 -> legend_edge2 [label=\"depends\", style=\"solid\"];\n");
        output.push_str("    legend_edge2 -> legend_edge3 [label=\"data\", style=\"dashed\", color=\"blue\"];\n");
        output.push_str("    legend_edge3 -> legend_edge4 [label=\"network\", style=\"dotted\", color=\"green\"];\n");
        output.push_str("    legend_edge4 -> legend_edge5 [label=\"cost\", style=\"bold\", color=\"red\"];\n");

        output.push_str("  }\n\n");

        Ok(())
    }

    /// Generate SVG output (requires dot command)
    pub fn generate_svg(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let dot = self.generate(graph)?;
        
        // This would call `dot -Tsvg` in production
        // For now, return a placeholder
        Ok(format!("<!-- SVG generation requires dot command -->\n<!-- DOT source:\n{}\n-->", dot))
    }

    /// Generate PNG output (requires dot command)
    pub fn generate_png_command(&self, graph: &DependencyGraph, output_path: &str) -> Result<String, CostPilotError> {
        let dot = self.generate(graph)?;
        
        // Return the command that user should run
        Ok(format!(
            "# Save this DOT content to a file, then run:\n\
             # dot -Tpng input.dot -o {}\n\n{}",
            output_path, dot
        ))
    }
}

impl Default for GraphvizGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize node ID for DOT format
fn sanitize_id(id: &str) -> String {
    id.replace('.', "_")
        .replace(':', "_")
        .replace('/', "_")
        .replace('[', "_")
        .replace(']', "_")
}

/// Escape label text for DOT format
fn escape_label(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::mapping::graph_types::*;

    #[test]
    fn test_generate_simple_graph() {
        let graph = DependencyGraph {
            nodes: vec![
                GraphNode {
                    id: "node1".to_string(),
                    label: "EC2 Instance".to_string(),
                    node_type: NodeType::Resource,
                    resource_type: Some("aws_instance".to_string()),
                    monthly_cost: Some(50.0),
                    module: None,
                },
                GraphNode {
                    id: "node2".to_string(),
                    label: "RDS Database".to_string(),
                    node_type: NodeType::Resource,
                    resource_type: Some("aws_rds_instance".to_string()),
                    monthly_cost: Some(200.0),
                    module: None,
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "node1".to_string(),
                    to: "node2".to_string(),
                    relationship: EdgeType::DependsOn,
                    cost_impact: None,
                },
            ],
            metadata: GraphMetadata {
                node_count: 2,
                edge_count: 1,
                max_depth: 1,
                total_cost: Some(250.0),
                timestamp: None,
            },
        };

        let generator = GraphvizGenerator::new();
        let result = generator.generate(&graph);
        
        assert!(result.is_ok());
        let dot = result.unwrap();
        assert!(dot.contains("digraph CostPilot"));
        assert!(dot.contains("node1"));
        assert!(dot.contains("node2"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("aws.ec2.instance"), "aws_ec2_instance");
        assert_eq!(sanitize_id("module:web/server"), "module_web_server");
    }

    #[test]
    fn test_escape_label() {
        assert_eq!(escape_label("test\"quote"), "test\\\"quote");
        assert_eq!(escape_label("line1\nline2"), "line1\\nline2");
    }
}
