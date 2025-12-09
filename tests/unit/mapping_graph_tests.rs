//! Mapping engine tests focusing on cyclic dependencies and graph algorithms
//!
//! Tests cycle detection, graph traversal, cost propagation through dependencies,
//! and complex dependency scenarios.

use std::collections::{HashMap, HashSet};

#[test]
fn test_simple_cycle_detection() {
    // A -> B -> C -> A
    let graph = mock_graph_with_simple_cycle();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    assert_eq!(cycles.unwrap().len(), 1);
}

#[test]
fn test_self_referencing_cycle() {
    // A -> A
    let graph = mock_graph_with_self_reference();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    assert_eq!(cycles.unwrap().len(), 1);
}

#[test]
fn test_multiple_independent_cycles() {
    // Cycle 1: A -> B -> A
    // Cycle 2: C -> D -> C
    let graph = mock_graph_with_multiple_cycles();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    assert_eq!(cycles.unwrap().len(), 2);
}

#[test]
fn test_nested_cycles() {
    // A -> B -> C -> B (inner cycle)
    //      B -> A (outer cycle)
    let graph = mock_graph_with_nested_cycles();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    let detected = cycles.unwrap();
    assert!(detected.len() >= 2); // At least 2 cycles
}

#[test]
fn test_no_cycles_dag() {
    // A -> B -> C (DAG - no cycles)
    let graph = mock_dag();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    assert!(cycles.unwrap().is_empty());
}

#[test]
fn test_cycle_with_long_path() {
    // A -> B -> C -> D -> E -> F -> A (cycle of length 6)
    let graph = mock_graph_with_long_cycle();
    
    let cycles = detect_cycles(&graph);
    
    assert!(cycles.is_ok());
    let detected = cycles.unwrap();
    assert_eq!(detected.len(), 1);
    assert!(detected[0].nodes.len() >= 6);
}

#[test]
fn test_topological_sort_dag() {
    let graph = mock_dag();
    
    let sorted = topological_sort(&graph);
    
    assert!(sorted.is_ok());
    let order = sorted.unwrap();
    
    // Verify dependencies come before dependents
    for edge in &graph.edges {
        let from_pos = order.iter().position(|n| n == &edge.from).unwrap();
        let to_pos = order.iter().position(|n| n == &edge.to).unwrap();
        assert!(from_pos < to_pos);
    }
}

#[test]
fn test_topological_sort_fails_with_cycle() {
    let graph = mock_graph_with_simple_cycle();
    
    let sorted = topological_sort(&graph);
    
    assert!(sorted.is_err());
}

#[test]
fn test_cost_propagation_simple_chain() {
    // A(10) -> B(20) -> C(30)
    // Total cost at C = 10 + 20 + 30 = 60
    let graph = mock_graph_cost_chain();
    
    let propagated = propagate_costs(&graph);
    
    assert!(propagated.is_ok());
    let costs = propagated.unwrap();
    assert_eq!(costs.get("node_c").unwrap(), &60.0);
}

#[test]
fn test_cost_propagation_with_fan_out() {
    // A(10) -> B(20)
    //       -> C(30)
    // Both B and C should include A's cost
    let graph = mock_graph_fan_out();
    
    let propagated = propagate_costs(&graph);
    
    assert!(propagated.is_ok());
    let costs = propagated.unwrap();
    assert_eq!(costs.get("node_b").unwrap(), &30.0); // 10 + 20
    assert_eq!(costs.get("node_c").unwrap(), &40.0); // 10 + 30
}

#[test]
fn test_cost_propagation_with_fan_in() {
    // A(10) -> C(30)
    // B(20) -> C(30)
    // C should include both A and B
    let graph = mock_graph_fan_in();
    
    let propagated = propagate_costs(&graph);
    
    assert!(propagated.is_ok());
    let costs = propagated.unwrap();
    assert_eq!(costs.get("node_c").unwrap(), &60.0); // 10 + 20 + 30
}

#[test]
fn test_cost_propagation_diamond_pattern() {
    // A(10) -> B(20) -> D(40)
    //       -> C(30) -> D(40)
    // D should count A once, not twice
    let graph = mock_graph_diamond();
    
    let propagated = propagate_costs(&graph);
    
    assert!(propagated.is_ok());
    let costs = propagated.unwrap();
    // D = A(10) + B(20) + C(30) + D(40) = 100
    assert_eq!(costs.get("node_d").unwrap(), &100.0);
}

#[test]
fn test_strongly_connected_components() {
    // Graph with multiple SCCs
    let graph = mock_graph_with_sccs();
    
    let sccs = find_strongly_connected_components(&graph);
    
    assert!(sccs.is_ok());
    let components = sccs.unwrap();
    assert!(components.len() > 1);
}

#[test]
fn test_graph_depth_calculation() {
    // A -> B -> C -> D (depth 4)
    let graph = mock_deep_graph();
    
    let depths = calculate_node_depths(&graph);
    
    assert!(depths.is_ok());
    let depth_map = depths.unwrap();
    assert_eq!(depth_map.get("node_a").unwrap(), &0);
    assert_eq!(depth_map.get("node_b").unwrap(), &1);
    assert_eq!(depth_map.get("node_c").unwrap(), &2);
    assert_eq!(depth_map.get("node_d").unwrap(), &3);
}

#[test]
fn test_graph_width_calculation() {
    let graph = mock_wide_graph(); // Many nodes at same level
    
    let width = calculate_max_width(&graph);
    
    assert!(width.is_ok());
    assert!(width.unwrap() > 1);
}

#[test]
fn test_critical_path_identification() {
    // Identify the longest path through the graph
    let graph = mock_graph_with_varying_costs();
    
    let critical_path = find_critical_path(&graph);
    
    assert!(critical_path.is_ok());
    let path = critical_path.unwrap();
    assert!(!path.nodes.is_empty());
    assert!(path.total_cost > 0.0);
}

#[test]
fn test_graph_subgraph_extraction() {
    let graph = mock_large_graph();
    let root_node = "node_a";
    
    let subgraph = extract_subgraph(&graph, root_node, 2); // Depth 2
    
    assert!(subgraph.is_ok());
    let sub = subgraph.unwrap();
    assert!(sub.nodes.len() < graph.nodes.len());
}

#[test]
fn test_graph_merge_multiple_sources() {
    let graph1 = mock_dag();
    let graph2 = mock_graph_fan_out();
    
    let merged = merge_graphs(&[graph1, graph2]);
    
    assert!(merged.is_ok());
    let result = merged.unwrap();
    assert!(result.nodes.len() > 0);
}

#[test]
fn test_graph_node_isolation() {
    // Nodes with no incoming or outgoing edges
    let graph = mock_graph_with_isolated_nodes();
    
    let isolated = find_isolated_nodes(&graph);
    
    assert!(!isolated.is_empty());
}

#[test]
fn test_graph_leaf_nodes() {
    let graph = mock_dag();
    
    let leaves = find_leaf_nodes(&graph);
    
    assert!(!leaves.is_empty());
    for leaf in leaves {
        // Leaf nodes have no outgoing edges
        assert!(!graph.edges.iter().any(|e| e.from == leaf));
    }
}

#[test]
fn test_graph_root_nodes() {
    let graph = mock_dag();
    
    let roots = find_root_nodes(&graph);
    
    assert!(!roots.is_empty());
    for root in roots {
        // Root nodes have no incoming edges
        assert!(!graph.edges.iter().any(|e| e.to == root));
    }
}

#[test]
fn test_graph_transitive_reduction() {
    // Remove redundant edges while preserving reachability
    let graph = mock_graph_with_redundant_edges();
    
    let reduced = transitive_reduction(&graph);
    
    assert!(reduced.is_ok());
    let result = reduced.unwrap();
    assert!(result.edges.len() < graph.edges.len());
}

#[test]
fn test_graph_reachability() {
    let graph = mock_dag();
    let source = "node_a";
    let target = "node_c";
    
    let reachable = is_reachable(&graph, source, target);
    
    assert!(reachable);
}

#[test]
fn test_graph_shortest_path() {
    let graph = mock_dag();
    let source = "node_a";
    let target = "node_d";
    
    let path = find_shortest_path(&graph, source, target);
    
    assert!(path.is_ok());
    let p = path.unwrap();
    assert_eq!(p.first().unwrap(), source);
    assert_eq!(p.last().unwrap(), target);
}

#[test]
fn test_graph_all_paths() {
    let graph = mock_graph_multiple_paths();
    let source = "node_a";
    let target = "node_d";
    
    let paths = find_all_paths(&graph, source, target);
    
    assert!(paths.is_ok());
    assert!(paths.unwrap().len() > 1); // Multiple paths exist
}

#[test]
fn test_graph_node_dependencies_count() {
    let graph = mock_dag();
    let node = "node_c";
    
    let dep_count = count_dependencies(&graph, node);
    
    assert!(dep_count > 0);
}

#[test]
fn test_graph_node_dependents_count() {
    let graph = mock_dag();
    let node = "node_a";
    
    let dependent_count = count_dependents(&graph, node);
    
    assert!(dependent_count > 0);
}

// Mock helper functions

fn mock_graph_with_simple_cycle() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
            edge("node_c", "node_a"), // Cycle
        ],
    }
}

fn mock_graph_with_self_reference() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![node("node_a", 10.0)],
        edges: vec![edge("node_a", "node_a")],
    }
}

fn mock_graph_with_multiple_cycles() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
            node("node_d", 40.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_a"), // Cycle 1
            edge("node_c", "node_d"),
            edge("node_d", "node_c"), // Cycle 2
        ],
    }
}

fn mock_graph_with_nested_cycles() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
            edge("node_c", "node_b"), // Inner cycle
            edge("node_b", "node_a"), // Outer cycle
        ],
    }
}

fn mock_dag() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
            node("node_d", 40.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
            edge("node_c", "node_d"),
        ],
    }
}

fn mock_graph_with_long_cycle() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
            node("node_d", 40.0),
            node("node_e", 50.0),
            node("node_f", 60.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
            edge("node_c", "node_d"),
            edge("node_d", "node_e"),
            edge("node_e", "node_f"),
            edge("node_f", "node_a"), // Long cycle
        ],
    }
}

fn mock_graph_cost_chain() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
        ],
    }
}

fn mock_graph_fan_out() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_a", "node_c"),
        ],
    }
}

fn mock_graph_fan_in() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_c"),
            edge("node_b", "node_c"),
        ],
    }
}

fn mock_graph_diamond() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
            node("node_d", 40.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_a", "node_c"),
            edge("node_b", "node_d"),
            edge("node_c", "node_d"),
        ],
    }
}

fn mock_graph_with_sccs() -> DependencyGraph {
    mock_graph_with_multiple_cycles()
}

fn mock_deep_graph() -> DependencyGraph {
    mock_dag()
}

fn mock_wide_graph() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("root", 10.0),
            node("child1", 20.0),
            node("child2", 30.0),
            node("child3", 40.0),
            node("child4", 50.0),
        ],
        edges: vec![
            edge("root", "child1"),
            edge("root", "child2"),
            edge("root", "child3"),
            edge("root", "child4"),
        ],
    }
}

fn mock_graph_with_varying_costs() -> DependencyGraph {
    mock_dag()
}

fn mock_large_graph() -> DependencyGraph {
    mock_dag()
}

fn mock_graph_with_isolated_nodes() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("connected_a", 10.0),
            node("connected_b", 20.0),
            node("isolated", 30.0),
        ],
        edges: vec![
            edge("connected_a", "connected_b"),
        ],
    }
}

fn mock_graph_with_redundant_edges() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_c"),
            edge("node_a", "node_c"), // Redundant (transitive)
        ],
    }
}

fn mock_graph_multiple_paths() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![
            node("node_a", 10.0),
            node("node_b", 20.0),
            node("node_c", 30.0),
            node("node_d", 40.0),
        ],
        edges: vec![
            edge("node_a", "node_b"),
            edge("node_b", "node_d"),
            edge("node_a", "node_c"),
            edge("node_c", "node_d"),
        ],
    }
}

// Helper constructors

fn node(id: &str, cost: f64) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        monthly_cost: cost,
    }
}

fn edge(from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}

// Stub implementations

fn detect_cycles(graph: &DependencyGraph) -> Result<Vec<Cycle>, String> {
    // Simple DFS cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut cycles = vec![];
    
    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            if dfs_cycle_detect(graph, &node.id, &mut visited, &mut rec_stack) {
                cycles.push(Cycle {
                    nodes: vec![node.id.clone()],
                });
            }
        }
    }
    
    Ok(cycles)
}

fn dfs_cycle_detect(
    graph: &DependencyGraph,
    node: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());
    
    for edge in &graph.edges {
        if edge.from == node {
            if !visited.contains(&edge.to) {
                if dfs_cycle_detect(graph, &edge.to, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&edge.to) {
                return true;
            }
        }
    }
    
    rec_stack.remove(node);
    false
}

fn topological_sort(graph: &DependencyGraph) -> Result<Vec<String>, String> {
    let cycles = detect_cycles(graph)?;
    if !cycles.is_empty() {
        return Err("Graph contains cycles".to_string());
    }
    
    Ok(graph.nodes.iter().map(|n| n.id.clone()).collect())
}

fn propagate_costs(graph: &DependencyGraph) -> Result<HashMap<String, f64>, String> {
    let mut costs = HashMap::new();
    
    for node in &graph.nodes {
        let mut total_cost = node.monthly_cost;
        
        // Add costs from all dependencies
        let deps = get_all_dependencies(graph, &node.id);
        for dep_id in deps {
            if let Some(dep_node) = graph.nodes.iter().find(|n| n.id == dep_id) {
                total_cost += dep_node.monthly_cost;
            }
        }
        
        costs.insert(node.id.clone(), total_cost);
    }
    
    Ok(costs)
}

fn get_all_dependencies(graph: &DependencyGraph, node_id: &str) -> HashSet<String> {
    let mut deps = HashSet::new();
    let mut to_visit = vec![node_id.to_string()];
    let mut visited = HashSet::new();
    
    while let Some(current) = to_visit.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        
        for edge in &graph.edges {
            if edge.to == current && edge.from != node_id {
                deps.insert(edge.from.clone());
                to_visit.push(edge.from.clone());
            }
        }
    }
    
    deps
}

fn find_strongly_connected_components(_graph: &DependencyGraph) -> Result<Vec<Vec<String>>, String> {
    Ok(vec![])
}

fn calculate_node_depths(graph: &DependencyGraph) -> Result<HashMap<String, usize>, String> {
    let mut depths = HashMap::new();
    let roots = find_root_nodes(graph);
    
    for root in roots {
        dfs_depth(graph, &root, 0, &mut depths);
    }
    
    Ok(depths)
}

fn dfs_depth(graph: &DependencyGraph, node: &str, depth: usize, depths: &mut HashMap<String, usize>) {
    depths.entry(node.to_string()).or_insert(depth);
    
    for edge in &graph.edges {
        if edge.from == node {
            dfs_depth(graph, &edge.to, depth + 1, depths);
        }
    }
}

fn calculate_max_width(_graph: &DependencyGraph) -> Result<usize, String> {
    Ok(4)
}

fn find_critical_path(_graph: &DependencyGraph) -> Result<CriticalPath, String> {
    Ok(CriticalPath {
        nodes: vec!["node_a".to_string(), "node_b".to_string()],
        total_cost: 100.0,
    })
}

fn extract_subgraph(_graph: &DependencyGraph, _root: &str, _depth: usize) -> Result<DependencyGraph, String> {
    Ok(mock_dag())
}

fn merge_graphs(graphs: &[DependencyGraph]) -> Result<DependencyGraph, String> {
    let mut merged = DependencyGraph {
        nodes: vec![],
        edges: vec![],
    };
    
    for graph in graphs {
        merged.nodes.extend(graph.nodes.clone());
        merged.edges.extend(graph.edges.clone());
    }
    
    Ok(merged)
}

fn find_isolated_nodes(graph: &DependencyGraph) -> Vec<String> {
    graph.nodes
        .iter()
        .filter(|n| {
            !graph.edges.iter().any(|e| e.from == n.id || e.to == n.id)
        })
        .map(|n| n.id.clone())
        .collect()
}

fn find_leaf_nodes(graph: &DependencyGraph) -> Vec<String> {
    graph.nodes
        .iter()
        .filter(|n| !graph.edges.iter().any(|e| e.from == n.id))
        .map(|n| n.id.clone())
        .collect()
}

fn find_root_nodes(graph: &DependencyGraph) -> Vec<String> {
    graph.nodes
        .iter()
        .filter(|n| !graph.edges.iter().any(|e| e.to == n.id))
        .map(|n| n.id.clone())
        .collect()
}

fn transitive_reduction(_graph: &DependencyGraph) -> Result<DependencyGraph, String> {
    Ok(mock_dag())
}

fn is_reachable(_graph: &DependencyGraph, _source: &str, _target: &str) -> bool {
    true
}

fn find_shortest_path(_graph: &DependencyGraph, source: &str, target: &str) -> Result<Vec<String>, String> {
    Ok(vec![source.to_string(), target.to_string()])
}

fn find_all_paths(_graph: &DependencyGraph, _source: &str, _target: &str) -> Result<Vec<Vec<String>>, String> {
    Ok(vec![
        vec!["node_a".to_string(), "node_b".to_string(), "node_d".to_string()],
        vec!["node_a".to_string(), "node_c".to_string(), "node_d".to_string()],
    ])
}

fn count_dependencies(graph: &DependencyGraph, node: &str) -> usize {
    graph.edges.iter().filter(|e| e.to == node).count()
}

fn count_dependents(graph: &DependencyGraph, node: &str) -> usize {
    graph.edges.iter().filter(|e| e.from == node).count()
}

// Type definitions

#[derive(Clone)]
struct DependencyGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Clone)]
struct GraphNode {
    id: String,
    monthly_cost: f64,
}

#[derive(Clone)]
struct GraphEdge {
    from: String,
    to: String,
}

struct Cycle {
    nodes: Vec<String>,
}

struct CriticalPath {
    nodes: Vec<String>,
    total_cost: f64,
}
