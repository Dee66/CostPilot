use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A node in the dependency graph representing a resource or service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    /// Unique stable identifier for the node
    pub id: String,

    /// Human-readable label
    pub label: String,

    /// Type of node (resource, service, module)
    pub node_type: NodeType,

    /// AWS resource type (e.g., "aws_nat_gateway")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// Monthly cost estimate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_cost: Option<f64>,

    /// Module name if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

/// Type of graph node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// AWS resource
    Resource,

    /// AWS service (aggregated)
    Service,

    /// Terraform module
    Module,
}

/// An edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Type of relationship
    pub relationship: EdgeType,

    /// Optional cost impact description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_impact: Option<String>,
}

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Direct dependency (e.g., Lambda depends on VPC)
    DependsOn,

    /// Data flow (e.g., S3 bucket read by Lambda)
    DataFlow,

    /// Network connection
    NetworkConnection,

    /// Cost attribution
    CostAttribution,
}

/// Complete dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,

    /// All edges in the graph
    pub edges: Vec<GraphEdge>,

    /// Metadata about the graph
    pub metadata: GraphMetadata,
}

/// Metadata about the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Schema version
    pub version: String,

    /// Generation timestamp
    pub timestamp: String,

    /// Total number of nodes
    pub node_count: usize,

    /// Total number of edges
    pub edge_count: usize,

    /// Maximum depth detected
    pub max_depth: usize,

    /// Whether cycles were detected
    pub has_cycles: bool,

    /// List of cycles if detected
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cycles: Vec<Vec<String>>,

    /// Total monthly cost across all nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<f64>,
}

/// Configuration for graph construction
#[derive(Debug, Clone)]
pub struct GraphConfig {
    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,

    /// Whether to detect cycles
    pub detect_cycles: bool,

    /// Whether to infer downstream services
    pub infer_downstream: bool,

    /// Whether to aggregate by service
    pub aggregate_by_service: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(5),
            detect_cycles: true,
            infer_downstream: true,
            aggregate_by_service: false,
        }
    }
}

impl GraphNode {
    /// Create a new resource node
    pub fn new_resource(id: String, resource_type: String, label: String) -> Self {
        Self {
            id,
            label,
            node_type: NodeType::Resource,
            resource_type: Some(resource_type),
            monthly_cost: None,
            module: None,
        }
    }

    /// Create a new service node
    pub fn new_service(id: String, service_name: String) -> Self {
        Self {
            id,
            label: service_name,
            node_type: NodeType::Service,
            resource_type: None,
            monthly_cost: None,
            module: None,
        }
    }

    /// Create a new module node
    pub fn new_module(id: String, module_name: String) -> Self {
        Self {
            id,
            label: module_name.clone(),
            node_type: NodeType::Module,
            resource_type: None,
            monthly_cost: None,
            module: Some(module_name),
        }
    }

    /// Set the monthly cost for this node
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.monthly_cost = Some(cost);
        self
    }

    /// Set the module for this node
    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }
}

impl GraphEdge {
    /// Create a new edge
    pub fn new(from: String, to: String, relationship: EdgeType) -> Self {
        Self {
            from,
            to,
            relationship,
            cost_impact: None,
        }
    }

    /// Add cost impact description
    pub fn with_cost_impact(mut self, impact: String) -> Self {
        self.cost_impact = Some(impact);
        self
    }
}

impl DependencyGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                node_count: 0,
                edge_count: 0,
                max_depth: 0,
                has_cycles: false,
                cycles: Vec::new(),
                total_cost: None,
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
            self.metadata.node_count = self.nodes.len();
        }
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
        self.metadata.edge_count = self.edges.len();
    }

    /// Find a node by ID
    pub fn find_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Find all edges to a node
    pub fn edges_to(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// Get all downstream nodes from a given node
    pub fn downstream_nodes(&self, node_id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut to_visit = vec![node_id.to_string()];

        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for edge in self.edges_from(&current) {
                if !visited.contains(&edge.to) {
                    to_visit.push(edge.to.clone());
                }
            }
        }

        visited.remove(node_id);
        visited
    }

    /// Calculate total cost
    pub fn calculate_total_cost(&mut self) {
        let total: f64 = self.nodes.iter().filter_map(|n| n.monthly_cost).sum();

        if total > 0.0 {
            self.metadata.total_cost = Some(total);
        }
    }

    /// Update metadata
    pub fn update_metadata(&mut self) {
        self.metadata.node_count = self.nodes.len();
        self.metadata.edge_count = self.edges.len();
        self.calculate_total_cost();
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource_node() {
        let node = GraphNode::new_resource(
            "vpc-1".to_string(),
            "aws_vpc".to_string(),
            "Main VPC".to_string(),
        );

        assert_eq!(node.id, "vpc-1");
        assert_eq!(node.node_type, NodeType::Resource);
        assert_eq!(node.resource_type, Some("aws_vpc".to_string()));
    }

    #[test]
    fn test_create_service_node() {
        let node = GraphNode::new_service("ec2-service".to_string(), "EC2".to_string());

        assert_eq!(node.node_type, NodeType::Service);
        assert_eq!(node.label, "EC2");
    }

    #[test]
    fn test_add_node_to_graph() {
        let mut graph = DependencyGraph::new();
        let node = GraphNode::new_resource(
            "nat-1".to_string(),
            "aws_nat_gateway".to_string(),
            "NAT Gateway".to_string(),
        );

        graph.add_node(node);
        assert_eq!(graph.metadata.node_count, 1);
    }

    #[test]
    fn test_add_edge_to_graph() {
        let mut graph = DependencyGraph::new();
        let edge = GraphEdge::new(
            "lambda-1".to_string(),
            "vpc-1".to_string(),
            EdgeType::DependsOn,
        );

        graph.add_edge(edge);
        assert_eq!(graph.metadata.edge_count, 1);
    }

    #[test]
    fn test_find_node() {
        let mut graph = DependencyGraph::new();
        let node = GraphNode::new_resource(
            "s3-1".to_string(),
            "aws_s3_bucket".to_string(),
            "Data Bucket".to_string(),
        );

        graph.add_node(node);

        let found = graph.find_node("s3-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label, "Data Bucket");
    }

    #[test]
    fn test_downstream_nodes() {
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
        graph.add_node(GraphNode::new_resource(
            "c".to_string(),
            "type".to_string(),
            "C".to_string(),
        ));

        graph.add_edge(GraphEdge::new(
            "a".to_string(),
            "b".to_string(),
            EdgeType::DependsOn,
        ));
        graph.add_edge(GraphEdge::new(
            "b".to_string(),
            "c".to_string(),
            EdgeType::DependsOn,
        ));

        let downstream = graph.downstream_nodes("a");
        assert_eq!(downstream.len(), 2);
        assert!(downstream.contains("b"));
        assert!(downstream.contains("c"));
    }

    #[test]
    fn test_calculate_total_cost() {
        let mut graph = DependencyGraph::new();

        graph.add_node(
            GraphNode::new_resource("n1".to_string(), "type".to_string(), "N1".to_string())
                .with_cost(100.0),
        );
        graph.add_node(
            GraphNode::new_resource("n2".to_string(), "type".to_string(), "N2".to_string())
                .with_cost(200.0),
        );

        graph.calculate_total_cost();
        assert_eq!(graph.metadata.total_cost, Some(300.0));
    }
}
