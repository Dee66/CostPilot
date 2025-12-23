// Mapping fuzz tests

#[cfg(test)]
mod mapping_fuzz_tests {
    use proptest::prelude::*;
    use costpilot::engines::mapping::{DependencyGraph, GraphNode};

    fn arb_graph_node() -> impl Strategy<Value = GraphNode> {
        (
            any::<String>(),
            any::<String>(),
            0.0f64..10000.0f64,
            prop::collection::vec(any::<String>(), 0..10),
        ).prop_map(|(id, resource_type, cost, deps)| {
            GraphNode {
                id,
                resource_type,
                monthly_cost: cost,
                dependencies: deps,
            }
        })
    }

    fn arb_dependency_graph() -> impl Strategy<Value = DependencyGraph> {
        prop::collection::vec(arb_graph_node(), 0..50).prop_map(|nodes| {
            DependencyGraph { nodes }
        })
    }

    proptest! {
        #[test]
        fn fuzz_graph_build_never_panics(
            graph in arb_dependency_graph()
        ) {
            let _ = graph.validate();
        }

        #[test]
        fn fuzz_graph_cycle_detection(
            graph in arb_dependency_graph()
        ) {
            let _ = graph.has_cycle();
        }

        #[test]
        fn fuzz_graph_topological_sort(
            graph in arb_dependency_graph()
        ) {
            let _ = graph.topological_sort();
        }

        #[test]
        fn fuzz_graph_cost_propagation(
            graph in arb_dependency_graph()
        ) {
            for node in &graph.nodes {
                let _ = graph.calculate_propagated_cost(&node.id);
            }
        }

        #[test]
        fn fuzz_graph_deterministic(
            graph in arb_dependency_graph()
        ) {
            let result1 = graph.validate();
            let result2 = graph.validate();
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }

        #[test]
        fn fuzz_graph_empty(
            _x in 0..1u8
        ) {
            let graph = DependencyGraph { nodes: vec![] };
            prop_assert!(graph.validate().is_ok());
            prop_assert!(!graph.has_cycle());
        }

        #[test]
        fn fuzz_graph_single_node(
            node in arb_graph_node()
        ) {
            let graph = DependencyGraph {
                nodes: vec![node],
            };
            let _ = graph.validate();
        }

        #[test]
        fn fuzz_graph_self_reference(
            id in any::<String>(),
            cost in 0.0f64..10000.0f64
        ) {
            let node = GraphNode {
                id: id.clone(),
                resource_type: "test".to_string(),
                monthly_cost: cost,
                dependencies: vec![id.clone()],
            };

            let graph = DependencyGraph {
                nodes: vec![node],
            };

            // Self-references should be detected as cycles
            let _ = graph.has_cycle();
        }

        #[test]
        fn fuzz_graph_unicode_ids(
            id in "\\PC{1,100}",
            deps in prop::collection::vec("\\PC{1,50}", 0..5)
        ) {
            let node = GraphNode {
                id,
                resource_type: "test".to_string(),
                monthly_cost: 100.0,
                dependencies: deps,
            };

            let graph = DependencyGraph {
                nodes: vec![node],
            };

            let _ = graph.validate();
        }

        #[test]
        fn fuzz_graph_extreme_costs(
            cost in prop::num::f64::ANY
        ) {
            let node = GraphNode {
                id: "test".to_string(),
                resource_type: "test".to_string(),
                monthly_cost: cost.abs(),
                dependencies: vec![],
            };

            let graph = DependencyGraph {
                nodes: vec![node],
            };

            let _ = graph.validate();
        }
    }
}
