// Golden file tests for mapping output

use costpilot::engines::mapping::{DependencyGraph, GraphNode};

#[test]
fn golden_simple_dependency_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_vpc.main".to_string(),
                resource_type: "aws_vpc".to_string(),
                monthly_cost: 0.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_subnet.public".to_string(),
                resource_type: "aws_subnet".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_vpc.main".to_string()],
            },
        ],
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("simple_vpc_subnet", json_output);
}

#[test]
fn golden_complex_web_app_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_vpc.main".to_string(),
                resource_type: "aws_vpc".to_string(),
                monthly_cost: 0.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_subnet.public".to_string(),
                resource_type: "aws_subnet".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_vpc.main".to_string()],
            },
            GraphNode {
                id: "aws_nat_gateway.main".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                monthly_cost: 32.85,
                dependencies: vec!["aws_subnet.public".to_string()],
            },
            GraphNode {
                id: "aws_instance.web".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 70.08,
                dependencies: vec!["aws_subnet.public".to_string()],
            },
            GraphNode {
                id: "aws_lb.main".to_string(),
                resource_type: "aws_lb".to_string(),
                monthly_cost: 16.20,
                dependencies: vec!["aws_subnet.public".to_string(), "aws_instance.web".to_string()],
            },
        ],
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("web_app_architecture", json_output);
}

#[test]
fn golden_database_tier_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_vpc.main".to_string(),
                resource_type: "aws_vpc".to_string(),
                monthly_cost: 0.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_subnet.private".to_string(),
                resource_type: "aws_subnet".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_vpc.main".to_string()],
            },
            GraphNode {
                id: "aws_db_subnet_group.main".to_string(),
                resource_type: "aws_db_subnet_group".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_subnet.private".to_string()],
            },
            GraphNode {
                id: "aws_db_instance.main".to_string(),
                resource_type: "aws_db_instance".to_string(),
                monthly_cost: 87.60,
                dependencies: vec!["aws_db_subnet_group.main".to_string()],
            },
        ],
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("database_tier", json_output);
}

#[test]
fn golden_cost_propagation_report() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_vpc.main".to_string(),
                resource_type: "aws_vpc".to_string(),
                monthly_cost: 0.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_subnet.public".to_string(),
                resource_type: "aws_subnet".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_vpc.main".to_string()],
            },
            GraphNode {
                id: "aws_nat_gateway.main".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                monthly_cost: 32.85,
                dependencies: vec!["aws_subnet.public".to_string()],
            },
        ],
    };

    let report = graph.cost_propagation_report();
    insta::assert_json_snapshot!("cost_propagation", report);
}

#[test]
fn golden_mermaid_diagram_output() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_vpc.main".to_string(),
                resource_type: "aws_vpc".to_string(),
                monthly_cost: 0.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_subnet.public".to_string(),
                resource_type: "aws_subnet".to_string(),
                monthly_cost: 0.0,
                dependencies: vec!["aws_vpc.main".to_string()],
            },
            GraphNode {
                id: "aws_instance.web".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 70.08,
                dependencies: vec!["aws_subnet.public".to_string()],
            },
        ],
    };

    let mermaid = graph.to_mermaid();
    insta::assert_snapshot!("mermaid_diagram", mermaid);
}

#[test]
fn golden_empty_graph() {
    let graph = DependencyGraph {
        nodes: vec![],
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("empty_graph", json_output);
}

#[test]
fn golden_isolated_resources() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode {
                id: "aws_s3_bucket.data".to_string(),
                resource_type: "aws_s3_bucket".to_string(),
                monthly_cost: 5.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_dynamodb_table.users".to_string(),
                resource_type: "aws_dynamodb_table".to_string(),
                monthly_cost: 25.0,
                dependencies: vec![],
            },
            GraphNode {
                id: "aws_lambda_function.api".to_string(),
                resource_type: "aws_lambda_function".to_string(),
                monthly_cost: 15.0,
                dependencies: vec![],
            },
        ],
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("isolated_resources", json_output);
}
