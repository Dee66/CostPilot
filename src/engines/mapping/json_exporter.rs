// JSON export for dependency graphs

use super::graph_types::DependencyGraph;
use crate::errors::CostPilotError;
use serde_json::{json, Value};

/// Configuration for JSON export
#[derive(Debug, Clone)]
pub struct JsonExportConfig {
    /// Pretty print with indentation
    pub pretty: bool,
    
    /// Include metadata
    pub include_metadata: bool,
    
    /// Include statistics
    pub include_statistics: bool,
    
    /// Export format variant
    pub format: JsonFormat,
}

impl Default for JsonExportConfig {
    fn default() -> Self {
        Self {
            pretty: true,
            include_metadata: true,
            include_statistics: true,
            format: JsonFormat::Standard,
        }
    }
}

/// JSON export format variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonFormat {
    /// Standard format (nodes and edges arrays)
    Standard,
    
    /// Adjacency list format
    AdjacencyList,
    
    /// Cytoscape.js format
    Cytoscape,
    
    /// D3.js force-directed format
    D3Force,
}

/// JSON exporter for dependency graphs
pub struct JsonExporter {
    config: JsonExportConfig,
}

impl JsonExporter {
    /// Create new exporter with default config
    pub fn new() -> Self {
        Self {
            config: JsonExportConfig::default(),
        }
    }

    /// Create exporter with custom config
    pub fn with_config(config: JsonExportConfig) -> Self {
        Self { config }
    }

    /// Export graph to JSON string
    pub fn export(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let json_value = match self.config.format {
            JsonFormat::Standard => self.export_standard(graph)?,
            JsonFormat::AdjacencyList => self.export_adjacency_list(graph)?,
            JsonFormat::Cytoscape => self.export_cytoscape(graph)?,
            JsonFormat::D3Force => self.export_d3_force(graph)?,
        };

        if self.config.pretty {
            serde_json::to_string_pretty(&json_value)
                .map_err(|e| CostPilotError::SerializationError(e.to_string()))
        } else {
            serde_json::to_string(&json_value)
                .map_err(|e| CostPilotError::SerializationError(e.to_string()))
        }
    }

    /// Export in standard format
    fn export_standard(&self, graph: &DependencyGraph) -> Result<Value, CostPilotError> {
        let mut result = json!({
            "nodes": graph.nodes,
            "edges": graph.edges,
        });

        if self.config.include_metadata {
            result["metadata"] = json!(graph.metadata);
        }

        if self.config.include_statistics {
            result["statistics"] = self.generate_statistics(graph);
        }

        Ok(result)
    }

    /// Export in adjacency list format
    fn export_adjacency_list(&self, graph: &DependencyGraph) -> Result<Value, CostPilotError> {
        use std::collections::HashMap;

        let mut adjacency: HashMap<String, Vec<Value>> = HashMap::new();

        // Initialize with all nodes
        for node in &graph.nodes {
            adjacency.insert(node.id.clone(), Vec::new());
        }

        // Add edges
        for edge in &graph.edges {
            if let Some(neighbors) = adjacency.get_mut(&edge.from) {
                neighbors.push(json!({
                    "to": edge.to,
                    "relationship": edge.relationship,
                    "cost_impact": edge.cost_impact,
                }));
            }
        }

        let mut result = json!({
            "adjacency_list": adjacency,
            "nodes": graph.nodes,
        });

        if self.config.include_metadata {
            result["metadata"] = json!(graph.metadata);
        }

        Ok(result)
    }

    /// Export in Cytoscape.js format
    fn export_cytoscape(&self, graph: &DependencyGraph) -> Result<Value, CostPilotError> {
        let mut elements = Vec::new();

        // Add nodes
        for node in &graph.nodes {
            elements.push(json!({
                "data": {
                    "id": node.id,
                    "label": node.label,
                    "type": node.node_type,
                    "resource_type": node.resource_type,
                    "monthly_cost": node.monthly_cost,
                    "module": node.module,
                },
                "classes": format!("{:?}", node.node_type).to_lowercase(),
            }));
        }

        // Add edges
        for edge in &graph.edges {
            elements.push(json!({
                "data": {
                    "source": edge.from,
                    "target": edge.to,
                    "relationship": edge.relationship,
                    "cost_impact": edge.cost_impact,
                },
                "classes": format!("{:?}", edge.relationship).to_lowercase(),
            }));
        }

        Ok(json!({
            "elements": elements,
            "style": self.generate_cytoscape_style(),
        }))
    }

    /// Export in D3.js force-directed format
    fn export_d3_force(&self, graph: &DependencyGraph) -> Result<Value, CostPilotError> {
        use std::collections::HashMap;

        // Create node index map
        let node_indices: HashMap<&str, usize> = graph.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.as_str(), i))
            .collect();

        // Format nodes for D3
        let nodes: Vec<Value> = graph.nodes.iter().map(|node| {
            json!({
                "id": node.id,
                "label": node.label,
                "type": node.node_type,
                "resource_type": node.resource_type,
                "monthly_cost": node.monthly_cost,
                "module": node.module,
                "group": self.get_node_group(node),
            })
        }).collect();

        // Format edges for D3 (use indices)
        let links: Vec<Value> = graph.edges.iter().filter_map(|edge| {
            let source_idx = node_indices.get(edge.from.as_str())?;
            let target_idx = node_indices.get(edge.to.as_str())?;
            
            Some(json!({
                "source": source_idx,
                "target": target_idx,
                "relationship": edge.relationship,
                "cost_impact": edge.cost_impact,
                "value": self.get_edge_strength(edge),
            }))
        }).collect();

        Ok(json!({
            "nodes": nodes,
            "links": links,
        }))
    }

    /// Generate statistics about the graph
    fn generate_statistics(&self, graph: &DependencyGraph) -> Value {
        use std::collections::HashMap;

        let mut node_type_counts: HashMap<String, usize> = HashMap::new();
        let mut edge_type_counts: HashMap<String, usize> = HashMap::new();
        let mut total_cost = 0.0;
        let mut costed_nodes = 0;

        for node in &graph.nodes {
            let type_name = format!("{:?}", node.node_type);
            *node_type_counts.entry(type_name).or_insert(0) += 1;

            if let Some(cost) = node.monthly_cost {
                total_cost += cost;
                costed_nodes += 1;
            }
        }

        for edge in &graph.edges {
            let type_name = format!("{:?}", edge.relationship);
            *edge_type_counts.entry(type_name).or_insert(0) += 1;
        }

        json!({
            "total_nodes": graph.nodes.len(),
            "total_edges": graph.edges.len(),
            "node_types": node_type_counts,
            "edge_types": edge_type_counts,
            "total_monthly_cost": total_cost,
            "costed_nodes": costed_nodes,
            "average_node_cost": if costed_nodes > 0 { total_cost / costed_nodes as f64 } else { 0.0 },
        })
    }

    /// Generate Cytoscape.js style
    fn generate_cytoscape_style(&self) -> Value {
        json!([
            {
                "selector": "node",
                "style": {
                    "label": "data(label)",
                    "text-valign": "center",
                    "text-halign": "center",
                    "background-color": "#3498db",
                    "color": "#fff",
                }
            },
            {
                "selector": "node.resource",
                "style": {
                    "shape": "rectangle",
                    "background-color": "#3498db",
                }
            },
            {
                "selector": "node.service",
                "style": {
                    "shape": "ellipse",
                    "background-color": "#2ecc71",
                }
            },
            {
                "selector": "node.module",
                "style": {
                    "shape": "roundrectangle",
                    "background-color": "#f39c12",
                }
            },
            {
                "selector": "edge",
                "style": {
                    "width": 2,
                    "line-color": "#95a5a6",
                    "target-arrow-color": "#95a5a6",
                    "target-arrow-shape": "triangle",
                    "curve-style": "bezier",
                }
            },
            {
                "selector": "edge.dependson",
                "style": {
                    "line-color": "#34495e",
                }
            },
            {
                "selector": "edge.dataflow",
                "style": {
                    "line-color": "#3498db",
                    "line-style": "dashed",
                }
            },
            {
                "selector": "edge.networkconnection",
                "style": {
                    "line-color": "#2ecc71",
                    "line-style": "dotted",
                }
            },
            {
                "selector": "edge.costattribution",
                "style": {
                    "line-color": "#e74c3c",
                    "width": 3,
                }
            },
        ])
    }

    /// Get node group for D3 visualization
    fn get_node_group(&self, node: &super::graph_types::GraphNode) -> usize {
        use super::graph_types::NodeType;
        
        match node.node_type {
            NodeType::Resource => 1,
            NodeType::Service => 2,
            NodeType::Module => 3,
        }
    }

    /// Get edge strength for D3 force simulation
    fn get_edge_strength(&self, edge: &super::graph_types::GraphEdge) -> f64 {
        use super::graph_types::EdgeType;
        
        match edge.relationship {
            EdgeType::DependsOn => 1.0,
            EdgeType::DataFlow => 0.8,
            EdgeType::NetworkConnection => 0.6,
            EdgeType::CostAttribution => 1.2,
        }
    }

    /// Export with custom format
    pub fn export_with_format(&self, graph: &DependencyGraph, format: JsonFormat) -> Result<String, CostPilotError> {
        let mut config = self.config.clone();
        config.format = format;
        let exporter = JsonExporter::with_config(config);
        exporter.export(graph)
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to export graph to JSON file
pub fn export_to_file(
    graph: &DependencyGraph,
    path: &std::path::Path,
    config: Option<JsonExportConfig>,
) -> Result<(), CostPilotError> {
    let exporter = if let Some(cfg) = config {
        JsonExporter::with_config(cfg)
    } else {
        JsonExporter::new()
    };

    let json_string = exporter.export(graph)?;
    
    std::fs::write(path, json_string)
        .map_err(|e| CostPilotError::IoError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::mapping::graph_types::*;

    fn create_test_graph() -> DependencyGraph {
        DependencyGraph {
            nodes: vec![
                GraphNode {
                    id: "node1".to_string(),
                    label: "EC2".to_string(),
                    node_type: NodeType::Resource,
                    resource_type: Some("aws_instance".to_string()),
                    monthly_cost: Some(100.0),
                    module: None,
                },
                GraphNode {
                    id: "node2".to_string(),
                    label: "RDS".to_string(),
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
                total_cost: Some(300.0),
                timestamp: None,
            },
        }
    }

    #[test]
    fn test_export_standard() {
        let graph = create_test_graph();
        let exporter = JsonExporter::new();
        let result = exporter.export(&graph);
        
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("nodes"));
        assert!(json.contains("edges"));
    }

    #[test]
    fn test_export_formats() {
        let graph = create_test_graph();
        let exporter = JsonExporter::new();

        // Test all formats
        assert!(exporter.export_with_format(&graph, JsonFormat::Standard).is_ok());
        assert!(exporter.export_with_format(&graph, JsonFormat::AdjacencyList).is_ok());
        assert!(exporter.export_with_format(&graph, JsonFormat::Cytoscape).is_ok());
        assert!(exporter.export_with_format(&graph, JsonFormat::D3Force).is_ok());
    }
}
