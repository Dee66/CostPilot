mod graph_types;
mod graph_builder;
mod mermaid_generator;
mod graphviz_generator;
mod json_exporter;

pub use graph_types::*;
pub use graph_builder::GraphBuilder;
pub use mermaid_generator::{MermaidGenerator, MermaidConfig};
pub use graphviz_generator::{GraphvizGenerator, GraphvizConfig, ColorScheme};
pub use json_exporter::{JsonExporter, JsonExportConfig, JsonFormat};

use crate::engines::detection::ResourceChange;
use crate::errors::CostPilotError;

/// High-level mapping engine for infrastructure dependency visualization
pub struct MappingEngine {
    builder: GraphBuilder,
    generator: MermaidGenerator,
}

impl MappingEngine {
    /// Create a new mapping engine with default configuration
    pub fn new() -> Self {
        Self {
            builder: GraphBuilder::new(),
            generator: MermaidGenerator::new(),
        }
    }

    /// Create a new mapping engine with custom configuration
    pub fn with_config(graph_config: GraphConfig, mermaid_config: MermaidConfig) -> Self {
        Self {
            builder: GraphBuilder::with_config(graph_config),
            generator: MermaidGenerator::with_config(mermaid_config),
        }
    }

    /// Build a dependency graph from infrastructure changes
    pub fn build_graph(&self, changes: &[ResourceChange]) -> Result<DependencyGraph, CostPilotError> {
        self.builder.build_graph(changes)
    }

    /// Generate Mermaid diagram from dependency graph
    pub fn generate_mermaid(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        self.generator.generate(graph)
    }

    /// Generate standalone HTML file with embedded Mermaid diagram
    pub fn generate_html(&self, graph: &DependencyGraph, title: &str) -> Result<String, CostPilotError> {
        self.generator.generate_html(graph, title)
    }

    /// Complete pipeline: build graph and generate Mermaid diagram
    pub fn map_dependencies(&self, changes: &[ResourceChange]) -> Result<String, CostPilotError> {
        let graph = self.build_graph(changes)?;
        self.generate_mermaid(&graph)
    }

    /// Complete pipeline: build graph and generate HTML
    pub fn map_dependencies_html(
        &self,
        changes: &[ResourceChange],
        title: &str,
    ) -> Result<String, CostPilotError> {
        let graph = self.build_graph(changes)?;
        self.generate_html(&graph, title)
    }

    /// Generate Graphviz DOT format from dependency graph
    pub fn generate_graphviz(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let generator = GraphvizGenerator::new();
        generator.generate(graph)
    }

    /// Generate Graphviz DOT format with custom config
    pub fn generate_graphviz_with_config(
        &self,
        graph: &DependencyGraph,
        config: GraphvizConfig,
    ) -> Result<String, CostPilotError> {
        let generator = GraphvizGenerator::with_config(config);
        generator.generate(graph)
    }

    /// Export graph to JSON (standard format)
    pub fn export_json(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        let exporter = JsonExporter::new();
        exporter.export(graph)
    }

    /// Export graph to JSON with custom config
    pub fn export_json_with_config(
        &self,
        graph: &DependencyGraph,
        config: JsonExportConfig,
    ) -> Result<String, CostPilotError> {
        let exporter = JsonExporter::with_config(config);
        exporter.export(graph)
    }

    /// Export graph to JSON with specific format
    pub fn export_json_format(
        &self,
        graph: &DependencyGraph,
        format: JsonFormat,
    ) -> Result<String, CostPilotError> {
        let exporter = JsonExporter::new();
        exporter.export_with_format(graph, format)
    }

    /// Detect cross-service cost impacts
    pub fn detect_cost_impacts(&self, graph: &DependencyGraph) -> Vec<CostImpact> {
        let mut impacts = Vec::new();
        
        // Find expensive resources
        let expensive_resources: Vec<&GraphNode> = graph
            .nodes
            .iter()
            .filter(|n| n.monthly_cost.unwrap_or(0.0) > 100.0)
            .collect();
        
        // Check downstream dependencies
        for resource in expensive_resources {
            let downstream = graph.downstream_nodes(&resource.id);
            
            if downstream.len() > 5 {
                impacts.push(CostImpact {
                    source_id: resource.id.clone(),
                    source_label: resource.label.clone(),
                    source_cost: resource.monthly_cost.unwrap_or(0.0),
                    affected_resources: downstream.len(),
                    severity: if downstream.len() > 10 {
                        ImpactSeverity::High
                    } else {
                        ImpactSeverity::Medium
                    },
                    description: format!(
                        "Resource '{}' (${:.2}/mo) has {} downstream dependencies",
                        resource.label,
                        resource.monthly_cost.unwrap_or(0.0),
                        downstream.len()
                    ),
                });
            }
        }
        
        // Check for cross-service data flows
        for edge in &graph.edges {
            if edge.relationship == EdgeType::DataFlow {
                if let Some(from_node) = graph.find_node(&edge.from) {
                    if let Some(to_node) = graph.find_node(&edge.to) {
                        // Different services means potential cross-service cost
                        if from_node.resource_type != to_node.resource_type {
                            impacts.push(CostImpact {
                                source_id: from_node.id.clone(),
                                source_label: from_node.label.clone(),
                                source_cost: from_node.monthly_cost.unwrap_or(0.0),
                                affected_resources: 1,
                                severity: ImpactSeverity::Low,
                                description: format!(
                                    "Data flow from {} to {} may incur transfer costs",
                                    from_node.label, to_node.label
                                ),
                            });
                        }
                    }
                }
            }
        }
        
        impacts
    }
}

impl Default for MappingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a detected cost impact in the dependency graph
#[derive(Debug, Clone)]
pub struct CostImpact {
    /// ID of the source resource causing the impact
    pub source_id: String,
    /// Human-readable label of the source
    pub source_label: String,
    /// Monthly cost of the source resource
    pub source_cost: f64,
    /// Number of resources affected downstream
    pub affected_resources: usize,
    /// Severity of the impact
    pub severity: ImpactSeverity,
    /// Description of the impact
    pub description: String,
}

/// Severity levels for cost impacts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_resource(id: &str, resource_type: &str) -> ResourceChange {
        ResourceChange {
            resource_id: id.to_string(),
            resource_type: resource_type.to_string(),
            change_type: "create".to_string(),
            old_config: None,
            new_config: Some(json!({})),
        }
    }

    #[test]
    fn test_new_engine() {
        let engine = MappingEngine::new();
        assert!(true); // Just check it constructs
    }

    #[test]
    fn test_map_dependencies() {
        let engine = MappingEngine::new();
        let changes = vec![
            create_test_resource("aws_vpc.main", "aws_vpc"),
            create_test_resource("aws_subnet.public", "aws_subnet"),
        ];
        
        let result = engine.map_dependencies(&changes);
        assert!(result.is_ok());
        
        let mermaid = result.unwrap();
        assert!(mermaid.contains("flowchart TB"));
    }

    #[test]
    fn test_map_dependencies_html() {
        let engine = MappingEngine::new();
        let changes = vec![
            create_test_resource("aws_instance.web", "aws_instance"),
        ];
        
        let result = engine.map_dependencies_html(&changes, "Test Infrastructure");
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Infrastructure"));
    }

    #[test]
    fn test_detect_cost_impacts() {
        let engine = MappingEngine::new();
        let mut graph = DependencyGraph::new();
        
        // Create expensive resource
        let expensive = GraphNode::new_resource(
            "expensive".to_string(),
            "aws_instance".to_string(),
            "Expensive Server".to_string(),
        )
        .with_cost(200.0);
        
        graph.add_node(expensive);
        
        // Add many downstream dependencies
        for i in 0..8 {
            let downstream = GraphNode::new_resource(
                format!("downstream_{}", i),
                "aws_lambda".to_string(),
                format!("Lambda {}", i),
            );
            graph.add_node(downstream);
            graph.add_edge(GraphEdge::new(
                format!("downstream_{}", i),
                "expensive".to_string(),
                EdgeType::DependsOn,
            ));
        }
        
        let impacts = engine.detect_cost_impacts(&graph);
        assert!(!impacts.is_empty());
        assert_eq!(impacts[0].affected_resources, 8);
        assert_eq!(impacts[0].severity, ImpactSeverity::Medium);
    }

    #[test]
    fn test_detect_cross_service_data_flow() {
        let engine = MappingEngine::new();
        let mut graph = DependencyGraph::new();
        
        graph.add_node(GraphNode::new_resource(
            "lambda".to_string(),
            "aws_lambda_function".to_string(),
            "Lambda".to_string(),
        ));
        
        graph.add_node(GraphNode::new_resource(
            "s3".to_string(),
            "aws_s3_bucket".to_string(),
            "S3 Bucket".to_string(),
        ));
        
        graph.add_edge(GraphEdge::new(
            "lambda".to_string(),
            "s3".to_string(),
            EdgeType::DataFlow,
        ));
        
        let impacts = engine.detect_cost_impacts(&graph);
        assert!(!impacts.is_empty());
        assert!(impacts[0].description.contains("transfer costs"));
    }
}
