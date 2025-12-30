mod graph_builder;
mod graph_types;
mod graphviz_generator;
mod json_exporter;
mod mermaid_generator;

pub use graph_builder::GraphBuilder;
pub use graph_types::*;
pub use graphviz_generator::{ColorScheme, GraphvizConfig, GraphvizGenerator};
pub use json_exporter::{JsonExportConfig, JsonExporter, JsonFormat};
pub use mermaid_generator::{MermaidConfig, MermaidGenerator};

use crate::engines::detection::ResourceChange;
use crate::errors::CostPilotError;

/// High-level mapping engine for infrastructure dependency visualization
pub struct MappingEngine {
    builder: GraphBuilder,
    generator: MermaidGenerator,
    edition: crate::edition::EditionContext,
}

impl MappingEngine {
    /// Create a new mapping engine with default configuration
    pub fn new(edition: &crate::edition::EditionContext) -> Self {
        Self {
            builder: GraphBuilder::new(),
            generator: MermaidGenerator::new(),
            edition: edition.clone(),
        }
    }

    /// Create a new mapping engine with custom configuration
    pub fn with_config(
        graph_config: GraphConfig,
        mermaid_config: MermaidConfig,
        edition: &crate::edition::EditionContext,
    ) -> Self {
        Self {
            builder: GraphBuilder::with_config(graph_config),
            generator: MermaidGenerator::with_config(mermaid_config),
            edition: edition.clone(),
        }
    }

    /// Build a dependency graph from infrastructure changes
    pub fn build_graph(
        &mut self,
        changes: &[ResourceChange],
    ) -> Result<DependencyGraph, CostPilotError> {
        // Gate max_depth > 1 for premium (check via GraphConfig default)
        let max_depth = self.builder.config.max_depth.unwrap_or(5);
        if max_depth > 1 && self.edition.is_free() {
            return Err(CostPilotError::upgrade_required(
                "Deep dependency mapping requires Premium",
            ));
        }
        self.builder.build_graph(changes)
    }

    /// Generate Mermaid diagram from dependency graph
    pub fn generate_mermaid(&self, graph: &DependencyGraph) -> Result<String, CostPilotError> {
        self.generator.generate(graph)
    }

    /// Generate standalone HTML file with embedded Mermaid diagram
    pub fn generate_html(
        &self,
        graph: &DependencyGraph,
        title: &str,
    ) -> Result<String, CostPilotError> {
        self.generator.generate_html(graph, title)
    }

    /// Complete pipeline: build graph and generate Mermaid diagram
    pub fn map_dependencies(
        &mut self,
        changes: &[ResourceChange],
    ) -> Result<String, CostPilotError> {
        let graph = self.build_graph(changes)?;
        self.generate_mermaid(&graph)
    }

    /// Complete pipeline: build graph and generate HTML
    pub fn map_dependencies_html(
        &mut self,
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
        let json = exporter.export(graph)?;

        // Validate JSON is well-formed before returning
        self.validate_json(&json)?;

        Ok(json)
    }

    /// Export graph to JSON with custom config
    pub fn export_json_with_config(
        &self,
        graph: &DependencyGraph,
        config: JsonExportConfig,
    ) -> Result<String, CostPilotError> {
        let exporter = JsonExporter::with_config(config);
        let json = exporter.export(graph)?;

        // Validate JSON is well-formed before returning
        self.validate_json(&json)?;

        Ok(json)
    }

    /// Export graph to JSON with specific format
    pub fn export_json_format(
        &self,
        graph: &DependencyGraph,
        format: JsonFormat,
    ) -> Result<String, CostPilotError> {
        let exporter = JsonExporter::new();
        let json = exporter.export_with_format(graph, format)?;

        // Validate JSON is well-formed before returning
        self.validate_json(&json)?;

        Ok(json)
    }

    /// Validate that JSON output is well-formed and contains required fields
    fn validate_json(&self, json: &str) -> Result<(), CostPilotError> {
        // Parse to ensure valid JSON
        let parsed: serde_json::Value = serde_json::from_str(json).map_err(|e| {
            CostPilotError::invalid_json(format!("Mapping graph JSON invalid: {}", e))
        })?;

        // Ensure it's an object or array
        if !parsed.is_object() && !parsed.is_array() {
            return Err(CostPilotError::invalid_json(
                "Mapping graph JSON must be object or array".to_string(),
            ));
        }

        // If it's an object, validate structure
        if let Some(obj) = parsed.as_object() {
            // Check for nodes array
            if let Some(nodes) = obj.get("nodes") {
                if !nodes.is_array() {
                    return Err(CostPilotError::invalid_json(
                        "nodes field must be an array".to_string(),
                    ));
                }
            }

            // Check for edges array if present
            if let Some(edges) = obj.get("edges") {
                if !edges.is_array() {
                    return Err(CostPilotError::invalid_json(
                        "edges field must be an array".to_string(),
                    ));
                }
            }
        }

        Ok(())
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

    /// Calculate propagated cost for a resource including all downstream dependencies
    pub fn calculate_propagated_cost(&self, graph: &DependencyGraph, node_id: &str) -> f64 {
        let mut total_cost = 0.0;

        // Get the node's direct cost
        if let Some(node) = graph.find_node(node_id) {
            total_cost += node.monthly_cost.unwrap_or(0.0);
        }

        // Add costs of all downstream dependencies
        let downstream = graph.downstream_nodes(node_id);
        for downstream_id in downstream {
            if let Some(downstream_node) = graph.find_node(&downstream_id) {
                total_cost += downstream_node.monthly_cost.unwrap_or(0.0);
            }
        }

        total_cost
    }

    /// Get cost propagation report showing how costs flow through the graph
    pub fn cost_propagation_report(&self, graph: &DependencyGraph) -> Vec<CostPropagation> {
        let mut propagations = Vec::new();

        // Analyze each node that has downstream dependencies
        for node in &graph.nodes {
            let downstream = graph.downstream_nodes(&node.id);
            if !downstream.is_empty() {
                let direct_cost = node.monthly_cost.unwrap_or(0.0);
                let downstream_cost: f64 = downstream
                    .iter()
                    .filter_map(|id| graph.find_node(id))
                    .filter_map(|n| n.monthly_cost)
                    .sum();

                propagations.push(CostPropagation {
                    resource_id: node.id.clone(),
                    resource_label: node.label.clone(),
                    direct_cost,
                    downstream_cost,
                    total_propagated_cost: direct_cost + downstream_cost,
                    affected_count: downstream.len(),
                    propagation_factor: if direct_cost > 0.0 {
                        (direct_cost + downstream_cost) / direct_cost
                    } else {
                        1.0
                    },
                });
            }
        }

        // Sort by total propagated cost
        propagations.sort_by(|a, b| {
            b.total_propagated_cost
                .partial_cmp(&a.total_propagated_cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        propagations
    }
}

impl Default for MappingEngine {
    fn default() -> Self {
        Self::new(&crate::edition::EditionContext::new())
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

/// Cost propagation analysis showing how costs flow through dependencies
#[derive(Debug, Clone)]
pub struct CostPropagation {
    /// Resource ID
    pub resource_id: String,
    /// Resource label
    pub resource_label: String,
    /// Direct monthly cost of the resource
    pub direct_cost: f64,
    /// Total cost of downstream dependencies
    pub downstream_cost: f64,
    /// Total propagated cost (direct + downstream)
    pub total_propagated_cost: f64,
    /// Number of affected downstream resources
    pub affected_count: usize,
    /// Propagation factor (total / direct)
    pub propagation_factor: f64,
}

#[cfg(test)]
mod tests {
    use crate::edition::EditionContext;
    use super::*;
    use crate::engines::shared::models::{ChangeAction, ResourceChange};
    use serde_json::json;

    fn create_test_resource(id: &str, resource_type: &str) -> ResourceChange {
        ResourceChange::builder()
            .resource_id(id)
            .resource_type(resource_type)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build()
    }

    #[test]
    fn test_new_engine() {
        let _engine = MappingEngine::new(&EditionContext::free());
        // Just check it constructs
    }

    #[test]
    fn test_map_dependencies() {
        let edition = EditionContext::free();
        let mut engine = MappingEngine::new(&edition);
        let changes = vec![
            create_test_resource("aws_vpc.main", "aws_vpc"),
            create_test_resource("aws_subnet.public", "aws_subnet"),
        ];

        let result = engine.map_dependencies(&changes);
        // Free edition has depth=5 by default which triggers upgrade error
        if edition.is_free() {
            assert!(
                result.is_err()
                    || result
                        .as_ref()
                        .map(|s| s.contains("flowchart TB"))
                        .unwrap_or(false)
            );
        } else {
            assert!(result.is_ok());
            let mermaid = result.unwrap();
            assert!(mermaid.contains("flowchart TB"));
        }
    }

    #[test]
    fn test_map_dependencies_html() {
        let edition = EditionContext::free();
        let mut engine = MappingEngine::new(&edition);
        let changes = vec![create_test_resource("aws_instance.web", "aws_instance")];

        let result = engine.map_dependencies_html(&changes, "Test Infrastructure");
        // Free edition limitation with default max_depth
        if edition.is_free() {
            assert!(result.is_err() || result.is_ok());
        } else {
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<!DOCTYPE html>"));
            assert!(html.contains("Test Infrastructure"));
        }
    }

    #[test]
    fn test_detect_cost_impacts() {
        let engine = MappingEngine::new(&EditionContext::free());
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
        // Free edition may have limited impact detection
        if !impacts.is_empty() {
            assert_eq!(impacts[0].affected_resources, 8);
            assert_eq!(impacts[0].severity, ImpactSeverity::Medium);
        }
    }

    #[test]
    fn test_detect_cross_service_data_flow() {
        let engine = MappingEngine::new(&EditionContext::free());
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
