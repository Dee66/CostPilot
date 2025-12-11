use std::collections::{HashSet, VecDeque};

use super::graph_types::*;
use crate::engines::detection::ResourceChange;
use crate::engines::performance::budgets::{
    BudgetViolation, PerformanceBudgets, PerformanceTracker, TimeoutAction,
};
use crate::engines::shared::models::ChangeAction;
use crate::errors::CostPilotError;

/// Builds dependency graphs from infrastructure resources
pub struct GraphBuilder {
    pub(super) config: GraphConfig,
    performance_tracker: Option<PerformanceTracker>,
}

impl GraphBuilder {
    /// Create a new graph builder with default config
    pub fn new() -> Self {
        Self {
            config: GraphConfig::default(),
            performance_tracker: None,
        }
    }

    /// Create a new graph builder with custom config
    pub fn with_config(config: GraphConfig) -> Self {
        Self {
            config,
            performance_tracker: None,
        }
    }

    /// Enable performance tracking with budgets
    pub fn with_performance_tracking(mut self, budgets: PerformanceBudgets) -> Self {
        self.performance_tracker = Some(PerformanceTracker::new(budgets.mapping));
        self
    }

    /// Build a dependency graph from resource changes
    pub fn build_graph(
        &mut self,
        changes: &[ResourceChange],
    ) -> Result<DependencyGraph, CostPilotError> {
        // Check budget before starting
        if let Some(tracker) = &self.performance_tracker {
            if let Err(violation) = tracker.check_budget() {
                return self.handle_budget_violation(violation);
            }
        }

        let mut graph = DependencyGraph::new();

        // First pass: Create nodes for all resources
        for change in changes {
            if change.action == ChangeAction::Delete {
                continue; // Skip deleted resources
            }

            let node = self.create_node_from_resource(change)?;
            graph.add_node(node);

            // Check budget periodically
            if let Some(tracker) = &self.performance_tracker {
                if let Err(violation) = tracker.check_budget() {
                    return self.handle_budget_violation_with_partial(violation, graph);
                }
            }
        }

        // Second pass: Infer dependencies and create edges
        for change in changes {
            if change.action == ChangeAction::Delete {
                continue;
            }

            let edges = self.infer_dependencies(change, changes)?;
            for edge in edges {
                graph.add_edge(edge);
            }

            // Check budget periodically
            if let Some(tracker) = &self.performance_tracker {
                if let Err(violation) = tracker.check_budget() {
                    return self.handle_budget_violation_with_partial(violation, graph);
                }
            }
        }

        // Detect cycles if enabled
        if self.config.detect_cycles {
            let cycles = self.detect_cycles(&graph);
            graph.metadata.has_cycles = !cycles.is_empty();
            graph.metadata.cycles = cycles;
        }

        // Calculate max depth
        graph.metadata.max_depth = self.calculate_max_depth(&graph);

        // Update metadata
        graph.update_metadata();

        // Mark completion and collect metrics
        if let Some(tracker) = self.performance_tracker.take() {
            let _metrics = tracker.complete();
            // TODO: Log or return metrics
        }

        Ok(graph)
    }

    /// Handle budget violation based on timeout action
    fn handle_budget_violation(
        &self,
        violation: BudgetViolation,
    ) -> Result<DependencyGraph, CostPilotError> {
        match violation.action {
            TimeoutAction::PartialResults => {
                eprintln!(
                    "⚠️  Mapping budget exceeded: {} ({}ms budget, {}ms elapsed)",
                    violation.violation_type, violation.budget_value, violation.actual_value
                );
                eprintln!("   Returning empty graph");
                Ok(DependencyGraph::new())
            }
            TimeoutAction::Error => Err(CostPilotError::Timeout(format!(
                "Mapping exceeded budget: {} ({}ms budget, {}ms elapsed)",
                violation.violation_type, violation.budget_value, violation.actual_value
            ))),
            TimeoutAction::CircuitBreak => Err(CostPilotError::CircuitBreaker(format!(
                "Mapping circuit breaker triggered: {} ({}ms budget, {}ms elapsed)",
                violation.violation_type, violation.budget_value, violation.actual_value
            ))),
        }
    }

    /// Handle budget violation with partial graph
    fn handle_budget_violation_with_partial(
        &self,
        violation: BudgetViolation,
        partial: DependencyGraph,
    ) -> Result<DependencyGraph, CostPilotError> {
        match violation.action {
            TimeoutAction::PartialResults => {
                eprintln!("⚠️  Mapping budget exceeded: {} ({}ms budget, {}ms elapsed)",
                    violation.violation_type, violation.budget_value, violation.actual_value);
                eprintln!("   Returning partial graph with {} nodes", partial.nodes.len());
                Ok(partial)
            }
            TimeoutAction::Error => {
                Err(CostPilotError::Timeout(format!(
                    "Mapping exceeded budget: {} ({}ms budget, {}ms elapsed) - partial graph discarded",
                    violation.violation_type, violation.budget_value, violation.actual_value
                )))
            }
            TimeoutAction::CircuitBreak => {
                Err(CostPilotError::CircuitBreaker(format!(
                    "Mapping circuit breaker triggered: {} ({}ms budget, {}ms elapsed) - partial graph discarded",
                    violation.violation_type, violation.budget_value, violation.actual_value
                )))
            }
        }
    }

    /// Create a graph node from a resource change
    fn create_node_from_resource(
        &self,
        change: &ResourceChange,
    ) -> Result<GraphNode, CostPilotError> {
        let node_id = self.generate_stable_id(&change.resource_id);
        let label = self.generate_label(change);

        let mut node = GraphNode::new_resource(node_id, change.resource_type.clone(), label);

        // Add module information
        if let Some(module) = self.extract_module(&change.resource_id) {
            node = node.with_module(module);
        }

        // Cost estimates should be provided externally (no internal prediction)
        // If caller wants cost data, they should predict first and pass to graph

        Ok(node)
    }

    /// Infer dependencies from a resource's configuration
    fn infer_dependencies(
        &self,
        change: &ResourceChange,
        all_changes: &[ResourceChange],
    ) -> Result<Vec<GraphEdge>, CostPilotError> {
        let mut edges = Vec::new();
        let from_id = self.generate_stable_id(&change.resource_id);

        if let Some(config) = &change.new_config {
            // Check for explicit dependencies
            edges.extend(self.extract_explicit_dependencies(&from_id, config, all_changes));

            // Infer dependencies based on resource type
            match change.resource_type.as_str() {
                "aws_lambda_function" => {
                    edges.extend(self.infer_lambda_dependencies(&from_id, config, all_changes));
                }
                "aws_instance" | "aws_autoscaling_group" => {
                    edges.extend(self.infer_compute_dependencies(&from_id, config, all_changes));
                }
                "aws_db_instance" | "aws_rds_cluster" => {
                    edges.extend(self.infer_database_dependencies(&from_id, config, all_changes));
                }
                "aws_ecs_service" | "aws_ecs_task_definition" => {
                    edges.extend(self.infer_container_dependencies(&from_id, config, all_changes));
                }
                _ => {}
            }
        }

        Ok(edges)
    }

    /// Extract explicit dependencies from resource configuration
    fn extract_explicit_dependencies(
        &self,
        from_id: &str,
        config: &serde_json::Value,
        all_changes: &[ResourceChange],
    ) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // Look for common dependency patterns
        if let Some(vpc_id) = config.get("vpc_id").and_then(|v| v.as_str()) {
            if let Some(to_id) = self.find_resource_by_reference(vpc_id, all_changes) {
                edges.push(GraphEdge::new(
                    from_id.to_string(),
                    to_id,
                    EdgeType::DependsOn,
                ));
            }
        }

        if let Some(subnet_ids) = config.get("subnet_ids").and_then(|v| v.as_array()) {
            for subnet in subnet_ids {
                if let Some(subnet_ref) = subnet.as_str() {
                    if let Some(to_id) = self.find_resource_by_reference(subnet_ref, all_changes) {
                        edges.push(GraphEdge::new(
                            from_id.to_string(),
                            to_id,
                            EdgeType::NetworkConnection,
                        ));
                    }
                }
            }
        }

        edges
    }

    /// Infer Lambda function dependencies
    fn infer_lambda_dependencies(
        &self,
        from_id: &str,
        config: &serde_json::Value,
        all_changes: &[ResourceChange],
    ) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // VPC configuration implies network dependency
        if let Some(vpc_config) = config.get("vpc_config") {
            if let Some(vpc_id) = vpc_config.get("vpc_id").and_then(|v| v.as_str()) {
                if let Some(to_id) = self.find_resource_by_reference(vpc_id, all_changes) {
                    edges.push(
                        GraphEdge::new(from_id.to_string(), to_id, EdgeType::DependsOn)
                            .with_cost_impact(
                                "VPC networking may incur data transfer costs".to_string(),
                            ),
                    );
                }
            }
        }

        // Environment variables might reference other resources
        if let Some(env) = config.get("environment") {
            if let Some(vars) = env.get("variables").and_then(|v| v.as_object()) {
                for (key, value) in vars {
                    if key.contains("BUCKET") || key.contains("S3") {
                        if let Some(bucket_ref) = value.as_str() {
                            if let Some(to_id) =
                                self.find_resource_by_reference(bucket_ref, all_changes)
                            {
                                edges.push(
                                    GraphEdge::new(from_id.to_string(), to_id, EdgeType::DataFlow)
                                        .with_cost_impact(
                                            "S3 requests and data transfer".to_string(),
                                        ),
                                );
                            }
                        }
                    }
                }
            }
        }

        edges
    }

    /// Infer compute resource dependencies
    fn infer_compute_dependencies(
        &self,
        from_id: &str,
        config: &serde_json::Value,
        all_changes: &[ResourceChange],
    ) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // Security groups imply network dependencies
        if let Some(sg_ids) = config.get("security_groups").and_then(|v| v.as_array()) {
            for sg in sg_ids {
                if let Some(sg_ref) = sg.as_str() {
                    if let Some(to_id) = self.find_resource_by_reference(sg_ref, all_changes) {
                        edges.push(GraphEdge::new(
                            from_id.to_string(),
                            to_id,
                            EdgeType::DependsOn,
                        ));
                    }
                }
            }
        }

        edges
    }

    /// Infer database dependencies
    fn infer_database_dependencies(
        &self,
        from_id: &str,
        config: &serde_json::Value,
        all_changes: &[ResourceChange],
    ) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // DB subnet group implies network dependency
        if let Some(subnet_group) = config.get("db_subnet_group_name").and_then(|v| v.as_str()) {
            if let Some(to_id) = self.find_resource_by_reference(subnet_group, all_changes) {
                edges.push(GraphEdge::new(
                    from_id.to_string(),
                    to_id,
                    EdgeType::DependsOn,
                ));
            }
        }

        edges
    }

    /// Infer container service dependencies
    fn infer_container_dependencies(
        &self,
        from_id: &str,
        config: &serde_json::Value,
        all_changes: &[ResourceChange],
    ) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // ECS service depends on task definition
        if let Some(task_def) = config.get("task_definition").and_then(|v| v.as_str()) {
            if let Some(to_id) = self.find_resource_by_reference(task_def, all_changes) {
                edges.push(GraphEdge::new(
                    from_id.to_string(),
                    to_id,
                    EdgeType::DependsOn,
                ));
            }
        }

        edges
    }

    /// Find a resource by its Terraform reference
    fn find_resource_by_reference(
        &self,
        reference: &str,
        all_changes: &[ResourceChange],
    ) -> Option<String> {
        // Handle Terraform references like "${aws_vpc.main.id}"
        let resource_ref = reference
            .trim_start_matches("${")
            .trim_end_matches("}")
            .split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join(".");

        for change in all_changes {
            if change.resource_id.contains(&resource_ref) {
                return Some(self.generate_stable_id(&change.resource_id));
            }
        }

        None
    }

    /// Detect cycles in the graph using DFS
    fn detect_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in &graph.nodes {
            if !visited.contains(&node.id) {
                self.dfs_cycle_detect(
                    &node.id,
                    graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detect(
        &self,
        node_id: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        path.push(node_id.to_string());

        for edge in graph.edges_from(node_id) {
            if !visited.contains(&edge.to) {
                self.dfs_cycle_detect(&edge.to, graph, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(&edge.to) {
                // Found a cycle
                if let Some(cycle_start) = path.iter().position(|id| id == &edge.to) {
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node_id);
    }

    /// Calculate maximum depth in the graph
    fn calculate_max_depth(&self, graph: &DependencyGraph) -> usize {
        let mut max_depth = 0;

        // Find root nodes (no incoming edges)
        let mut has_incoming: HashSet<String> = HashSet::new();
        for edge in &graph.edges {
            has_incoming.insert(edge.to.clone());
        }

        let root_nodes: Vec<&GraphNode> = graph
            .nodes
            .iter()
            .filter(|n| !has_incoming.contains(&n.id))
            .collect();

        // BFS from each root to find max depth
        for root in root_nodes {
            let depth = self.calculate_depth_from(&root.id, graph);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Calculate depth from a specific node using BFS
    fn calculate_depth_from(&self, start_id: &str, graph: &DependencyGraph) -> usize {
        let mut max_depth = 0;
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((start_id.to_string(), 0));
        visited.insert(start_id.to_string());

        while let Some((node_id, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth);

            for edge in graph.edges_from(&node_id) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    queue.push_back((edge.to.clone(), depth + 1));
                }
            }
        }

        max_depth
    }

    /// Generate a stable ID for a resource
    fn generate_stable_id(&self, resource_id: &str) -> String {
        // Normalize resource ID to stable format
        resource_id
            .replace(['[', ']'], "_")
            .replace('"', "")
            .replace('.', "_")
    }

    /// Generate a human-readable label
    fn generate_label(&self, change: &ResourceChange) -> String {
        // Extract a nice label from resource ID
        let parts: Vec<&str> = change.resource_id.split('.').collect();
        if let Some(last) = parts.last() {
            return last
                .trim_matches(|c: char| c == '[' || c == ']' || c == '"')
                .to_string();
        }
        change.resource_id.clone()
    }

    /// Extract module name from resource ID
    fn extract_module(&self, resource_id: &str) -> Option<String> {
        if resource_id.starts_with("module.") {
            let parts: Vec<&str> = resource_id.split('.').collect();
            if parts.len() >= 2 {
                return Some(format!("module.{}", parts[1]));
            }
        }
        None
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_resource(id: &str, resource_type: &str) -> ResourceChange {
        use std::collections::HashMap;
        #[allow(deprecated)]
        ResourceChange {
            resource_id: id.to_string(),
            resource_type: resource_type.to_string(),
            action: crate::engines::shared::models::ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({})),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
            before: None,
            after: None,
        }
    }

    #[test]
    fn test_build_empty_graph() {
        let mut builder = GraphBuilder::new();
        let changes = vec![];

        let result = builder.build_graph(&changes);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.metadata.node_count, 0);
    }

    #[test]
    fn test_build_graph_with_resources() {
        let mut builder = GraphBuilder::new();
        let changes = vec![
            create_test_resource("aws_vpc.main", "aws_vpc"),
            create_test_resource("aws_subnet.public", "aws_subnet"),
        ];

        let result = builder.build_graph(&changes);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.metadata.node_count, 2);
    }

    #[test]
    fn test_generate_stable_id() {
        let builder = GraphBuilder::new();

        let id1 = builder.generate_stable_id("aws_instance.web[0]");
        let id2 = builder.generate_stable_id("aws_instance.web[0]");

        assert_eq!(id1, id2);
        assert!(!id1.contains('['));
    }

    #[test]
    fn test_extract_module() {
        let builder = GraphBuilder::new();

        assert_eq!(
            builder.extract_module("module.vpc.aws_nat_gateway.main"),
            Some("module.vpc".to_string())
        );
        assert_eq!(builder.extract_module("aws_instance.web"), None);
    }

    #[test]
    fn test_detect_cycles_no_cycle() {
        let builder = GraphBuilder::new();
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
            EdgeType::DependsOn,
        ));

        let cycles = builder.detect_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_calculate_max_depth() {
        let builder = GraphBuilder::new();
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

        let max_depth = builder.calculate_max_depth(&graph);
        assert_eq!(max_depth, 2);
    }
}
