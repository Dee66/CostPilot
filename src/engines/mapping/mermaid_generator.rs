use super::graph_types::*;
use crate::errors::CostPilotError;
use std::collections::HashSet;

/// Generates Mermaid diagrams from dependency graphs
pub struct MermaidGenerator {
    config: MermaidConfig,
}

/// Configuration for Mermaid diagram generation
#[derive(Debug, Clone)]
pub struct MermaidConfig {
    /// Include cost annotations in nodes
    pub show_costs: bool,
    /// Include module grouping
    pub group_by_module: bool,
    /// Show cycle warnings
    pub highlight_cycles: bool,
    /// Maximum nodes to render (prevent huge diagrams)
    pub max_nodes: usize,
    /// Include cost impact annotations on edges
    pub show_edge_impacts: bool,
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            show_costs: true,
            group_by_module: true,
            highlight_cycles: true,
            max_nodes: 50,
            show_edge_impacts: true,
        }
    }
}

impl MermaidGenerator {
    /// Create a new Mermaid generator with default config
    pub fn new() -> Self {
        Self {
            config: MermaidConfig::default(),
        }
    }

    /// Create a new generator with custom config
    pub fn with_config(config: MermaidConfig) -> Self {
        Self { config }
    }

    /// Generate Mermaid flowchart from dependency graph
    pub fn generate(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let mut output = String::new();

        // Start flowchart
        output.push_str("flowchart TB\n");

        // Warn if graph is too large
        if graph.metadata.node_count > self.config.max_nodes {
            output.push_str(&format!(
                "    warning[\"⚠️ Large graph ({} nodes). Consider filtering.\"]\n",
                graph.metadata.node_count
            ));
            output.push_str("    style warning fill:#fff3cd,stroke:#856404\n\n");
        }

        // Show cycle warning if present
        if self.config.highlight_cycles && graph.metadata.has_cycles {
            output.push_str(&format!(
                "    cycleWarning[\"🔄 {} cycle(s) detected\"]\n",
                graph.metadata.cycles.len()
            ));
            output.push_str("    style cycleWarning fill:#f8d7da,stroke:#721c24\n\n");
        }

        // Collect nodes in cycles for highlighting
        let cycle_nodes = self.collect_cycle_nodes(graph);

        // Group nodes by module if enabled
        if self.config.group_by_module {
            let grouped = self.group_nodes_by_module(graph);

            for (module, nodes) in grouped {
                if let Some(module_name) = module {
                    output.push_str(&format!(
                        "    subgraph {}\n",
                        self.sanitize_id(&module_name)
                    ));

                    for node in nodes {
                        output.push_str(&self.generate_node_definition(node, &cycle_nodes));
                    }

                    output.push_str("    end\n\n");
                } else {
                    // Nodes without module
                    for node in nodes {
                        output.push_str(&self.generate_node_definition(node, &cycle_nodes));
                    }
                    output.push('\n');
                }
            }
        } else {
            // No grouping, just render all nodes
            for node in &graph.nodes {
                output.push_str(&self.generate_node_definition(node, &cycle_nodes));
            }
            output.push('\n');
        }

        // Render edges
        for edge in &graph.edges {
            output.push_str(&self.generate_edge_definition(edge, graph)?);
        }

        // Add styling
        output.push('\n');
        output.push_str(&self.generate_styles(&cycle_nodes));

        Ok(output)
    }

    /// Generate node definition in Mermaid syntax
    fn generate_node_definition(&self, node: &GraphNode, cycle_nodes: &HashSet<String>) -> String {
        let id = self.sanitize_id(&node.id);
        let mut label = node.label.clone();

        // Add cost annotation if enabled
        if self.config.show_costs {
            if let Some(cost) = node.monthly_cost {
                label.push_str(&format!("<br/>${:.2}/mo", cost));
            }
        }

        // Choose shape based on node type
        let (open, close) = match node.node_type {
            NodeType::Resource => ("[", "]"),
            NodeType::Service => ("([", "])"),
            NodeType::Module => ("{{", "}}"),
        };

        let mut line = format!("        {}{}\"{}\"{}\n", id, open, label, close);

        // Add cycle highlighting class if applicable
        if cycle_nodes.contains(&node.id) {
            line.push_str(&format!("        class {} cycle\n", id));
        }

        line
    }

    /// Generate edge definition in Mermaid syntax
    fn generate_edge_definition(
        &self,
        edge: &GraphEdge,
        _graph: &DependencyGraph,
    ) -> Result<String, CostPilotError> {
        let from = self.sanitize_id(&edge.from);
        let to = self.sanitize_id(&edge.to);

        // Choose arrow style based on edge type
        let arrow = match edge.relationship {
            EdgeType::DependsOn => "-->",
            EdgeType::DataFlow => "-.->",
            EdgeType::NetworkConnection => "==>",
            EdgeType::CostAttribution => "-.-",
        };

        // Add label with cost impact if enabled
        if self.config.show_edge_impacts {
            if let Some(impact) = &edge.cost_impact {
                return Ok(format!("    {} {}|\"{}\"| {}\n", from, arrow, impact, to));
            }
        }

        Ok(format!("    {} {} {}\n", from, arrow, to))
    }

    /// Generate CSS styling for the diagram
    fn generate_styles(&self, cycle_nodes: &HashSet<String>) -> String {
        let mut styles = String::new();

        // Style for cycle nodes
        if !cycle_nodes.is_empty() {
            styles.push_str("    classDef cycle fill:#f8d7da,stroke:#721c24,stroke-width:3px\n");
        }

        // Resource type styling
        styles.push_str("    classDef resource fill:#d1ecf1,stroke:#0c5460\n");
        styles.push_str("    classDef service fill:#d4edda,stroke:#155724\n");
        styles.push_str("    classDef module fill:#fff3cd,stroke:#856404\n");

        styles
    }

    /// Collect all nodes that are part of cycles
    fn collect_cycle_nodes(&self, graph: &DependencyGraph) -> HashSet<String> {
        let mut cycle_nodes = HashSet::new();

        for cycle in &graph.metadata.cycles {
            for node_id in cycle {
                cycle_nodes.insert(node_id.clone());
            }
        }

        cycle_nodes
    }

    /// Group nodes by module for subgraph rendering
    fn group_nodes_by_module<'a>(
        &self,
        graph: &'a DependencyGraph,
    ) -> Vec<(Option<String>, Vec<&'a GraphNode>)> {
        let mut groups: std::collections::HashMap<Option<String>, Vec<&GraphNode>> =
            std::collections::HashMap::new();

        for node in &graph.nodes {
            groups
                .entry(node.module.clone())
                .or_default()
                .push(node);
        }

        groups.into_iter().collect()
    }

    /// Sanitize node ID for Mermaid compatibility
    fn sanitize_id(&self, id: &str) -> String {
        id.replace(['.', '[', ']'], "_")
            .replace('"', "")
            .replace(['-', ':'], "_")
    }

    /// Generate standalone HTML file with Mermaid diagram
    pub fn generate_html(
        &self,
        graph: &DependencyGraph,
        title: &str,
    ) -> Result<String, CostPilotError> {
        let mermaid_code = self.generate(graph)?;

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            padding: 30px;
        }}
        h1 {{
            color: #333;
            margin-top: 0;
            font-size: 28px;
            border-bottom: 3px solid #667eea;
            padding-bottom: 10px;
        }}
        .metadata {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 8px;
            margin: 20px 0;
            font-size: 14px;
        }}
        .metadata strong {{
            color: #495057;
        }}
        .mermaid {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-top: 20px;
        }}
        .legend {{
            margin-top: 30px;
            padding: 15px;
            background: #e7f3ff;
            border-radius: 8px;
            font-size: 13px;
        }}
        .legend h3 {{
            margin-top: 0;
            color: #0056b3;
        }}
        .legend ul {{
            list-style: none;
            padding: 0;
        }}
        .legend li {{
            margin: 8px 0;
            padding-left: 20px;
            position: relative;
        }}
        .legend li:before {{
            content: "→";
            position: absolute;
            left: 0;
            color: #0056b3;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>

        <div class="metadata">
            <strong>Nodes:</strong> {} |
            <strong>Edges:</strong> {} |
            <strong>Max Depth:</strong> {} |
            <strong>Cycles:</strong> {} |
            <strong>Total Cost:</strong> ${}
        </div>

        <div class="mermaid">
{}
        </div>

        <div class="legend">
            <h3>Legend</h3>
            <ul>
                <li><strong>Solid arrows (→)</strong>: Direct dependencies</li>
                <li><strong>Dashed arrows (⇢)</strong>: Data flow</li>
                <li><strong>Double arrows (⇒)</strong>: Network connections</li>
                <li><strong>Dotted arrows (⋯)</strong>: Cost attribution</li>
                <li><strong>Red border</strong>: Part of a dependency cycle</li>
            </ul>
        </div>
    </div>

    <script>
        mermaid.initialize({{
            startOnLoad: true,
            theme: 'default',
            flowchart: {{
                curve: 'basis',
                padding: 20
            }}
        }});
    </script>
</body>
</html>
"#,
            title,
            title,
            graph.metadata.node_count,
            graph.metadata.edge_count,
            graph.metadata.max_depth,
            graph.metadata.cycles.len(),
            graph
                .metadata
                .total_cost
                .map(|c| format!("{:.2}/mo", c))
                .unwrap_or_else(|| "N/A".to_string()),
            mermaid_code
        );

        Ok(html)
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_graph() {
        let generator = MermaidGenerator::new();
        let graph = DependencyGraph::new();

        let result = generator.generate(&graph);
        assert!(result.is_ok());

        let mermaid = result.unwrap();
        assert!(mermaid.starts_with("flowchart TB"));
    }

    #[test]
    fn test_generate_simple_graph() {
        let generator = MermaidGenerator::new();
        let mut graph = DependencyGraph::new();

        graph.add_node(GraphNode::new_resource(
            "vpc_main".to_string(),
            "aws_vpc".to_string(),
            "Main VPC".to_string(),
        ));
        graph.add_node(GraphNode::new_resource(
            "subnet_public".to_string(),
            "aws_subnet".to_string(),
            "Public Subnet".to_string(),
        ));

        graph.add_edge(GraphEdge::new(
            "subnet_public".to_string(),
            "vpc_main".to_string(),
            EdgeType::DependsOn,
        ));

        let result = generator.generate(&graph);
        assert!(result.is_ok());

        let mermaid = result.unwrap();
        assert!(mermaid.contains("vpc_main"));
        assert!(mermaid.contains("subnet_public"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_sanitize_id() {
        let generator = MermaidGenerator::new();

        let sanitized = generator.sanitize_id("aws_instance.web[0]");
        assert_eq!(sanitized, "aws_instance_web_0_");
        assert!(!sanitized.contains('['));
        assert!(!sanitized.contains('.'));
    }

    #[test]
    fn test_generate_with_costs() {
        let generator = MermaidGenerator::new();
        let mut graph = DependencyGraph::new();

        let node = GraphNode::new_resource(
            "instance".to_string(),
            "aws_instance".to_string(),
            "Web Server".to_string(),
        )
        .with_cost(73.0);

        graph.add_node(node);

        let result = generator.generate(&graph);
        assert!(result.is_ok());

        let mermaid = result.unwrap();
        assert!(mermaid.contains("$73.00/mo"));
    }

    #[test]
    fn test_generate_html() {
        let generator = MermaidGenerator::new();
        let mut graph = DependencyGraph::new();

        graph.add_node(GraphNode::new_resource(
            "test".to_string(),
            "aws_instance".to_string(),
            "Test".to_string(),
        ));

        let result = generator.generate_html(&graph, "Test Diagram");
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Diagram"));
        assert!(html.contains("mermaid"));
    }

    #[test]
    fn test_cycle_highlighting() {
        let generator = MermaidGenerator::new();
        let mut graph = DependencyGraph::new();

        graph.add_node(GraphNode::new_resource(
            "a".to_string(),
            "type".to_string(),
            "A".to_string(),
        ));
        graph.add_node(GraphNode::new_resource(
            "b".to_string(),
            "type".to_string(),
            "B".to_string(),
        ));

        // Create a cycle in metadata
        graph.metadata.has_cycles = true;
        graph.metadata.cycles = vec![vec!["a".to_string(), "b".to_string()]];

        let result = generator.generate(&graph);
        assert!(result.is_ok());

        let mermaid = result.unwrap();
        assert!(mermaid.contains("cycle"));
        assert!(mermaid.contains("🔄"));
    }

    #[test]
    fn test_edge_types() {
        let generator = MermaidGenerator::new();
        let mut graph = DependencyGraph::new();

        graph.add_node(GraphNode::new_resource(
            "a".to_string(),
            "type".to_string(),
            "A".to_string(),
        ));
        graph.add_node(GraphNode::new_resource(
            "b".to_string(),
            "type".to_string(),
            "B".to_string(),
        ));

        graph.add_edge(GraphEdge::new(
            "a".to_string(),
            "b".to_string(),
            EdgeType::DataFlow,
        ));

        let result = generator.generate(&graph);
        assert!(result.is_ok());

        let mermaid = result.unwrap();
        assert!(mermaid.contains("-.->")); // DataFlow uses dashed arrow
    }
}
