// Golden file tests for mapping output

use assert_cmd::cargo::cargo_bin_cmd;
use costpilot::engines::mapping::{DependencyGraph, EdgeType, GraphEdge, GraphMetadata, GraphNode};

#[test]
fn golden_map_basic_ec2_instances() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("map").arg("tests/test_golden_plan.json");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!("map_basic_ec2_instances", stdout);
}

#[test]
fn golden_autofix_snippet_basic() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-snippet");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!("autofix_snippet_basic", stdout);
}

#[test]
fn golden_autofix_patch_basic() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-patch");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!("autofix_patch_basic", stdout);
}

#[allow(dead_code)]
fn golden_simple_dependency_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode::new_resource(
                "aws_vpc.main".to_string(),
                "aws_vpc".to_string(),
                "main VPC".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_subnet.public".to_string(),
                "aws_subnet".to_string(),
                "public subnet".to_string(),
            )
            .with_cost(0.0),
        ],
        edges: vec![GraphEdge::new(
            "aws_subnet.public".to_string(),
            "aws_vpc.main".to_string(),
            EdgeType::DependsOn,
        )],
        metadata: GraphMetadata {
            node_count: 2,
            edge_count: 1,
            max_depth: 1,
            has_cycles: false,
            cycles: vec![],
            total_cost: Some(0.0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("simple_vpc_subnet", json_output);
}

#[test]
fn golden_complex_web_app_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode::new_resource(
                "aws_vpc.main".to_string(),
                "aws_vpc".to_string(),
                "main VPC".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_subnet.public".to_string(),
                "aws_subnet".to_string(),
                "public subnet".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_nat_gateway.main".to_string(),
                "aws_nat_gateway".to_string(),
                "main NAT gateway".to_string(),
            )
            .with_cost(32.85),
            GraphNode::new_resource(
                "aws_instance.web".to_string(),
                "aws_instance".to_string(),
                "web instance".to_string(),
            )
            .with_cost(70.08),
            GraphNode::new_resource(
                "aws_lb.main".to_string(),
                "aws_lb".to_string(),
                "main load balancer".to_string(),
            )
            .with_cost(16.20),
        ],
        edges: vec![
            GraphEdge::new(
                "aws_subnet.public".to_string(),
                "aws_vpc.main".to_string(),
                EdgeType::DependsOn,
            ),
            GraphEdge::new(
                "aws_nat_gateway.main".to_string(),
                "aws_subnet.public".to_string(),
                EdgeType::DependsOn,
            )
            .with_cost_impact("$32.85/month".to_string()),
            GraphEdge::new(
                "aws_instance.web".to_string(),
                "aws_subnet.public".to_string(),
                EdgeType::DependsOn,
            )
            .with_cost_impact("$70.08/month".to_string()),
            GraphEdge::new(
                "aws_lb.main".to_string(),
                "aws_subnet.public".to_string(),
                EdgeType::DependsOn,
            )
            .with_cost_impact("$16.20/month".to_string()),
            GraphEdge::new(
                "aws_lb.main".to_string(),
                "aws_instance.web".to_string(),
                EdgeType::DataFlow,
            ),
        ],
        metadata: GraphMetadata {
            node_count: 5,
            edge_count: 5,
            max_depth: 2,
            has_cycles: false,
            cycles: vec![],
            total_cost: Some(119.13),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("web_app_architecture", json_output);
}

#[test]
fn golden_database_tier_graph() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode::new_resource(
                "aws_vpc.main".to_string(),
                "aws_vpc".to_string(),
                "main VPC".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_subnet.private".to_string(),
                "aws_subnet".to_string(),
                "private subnet".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_db_subnet_group.main".to_string(),
                "aws_db_subnet_group".to_string(),
                "main DB subnet group".to_string(),
            )
            .with_cost(0.0),
            GraphNode::new_resource(
                "aws_db_instance.main".to_string(),
                "aws_db_instance".to_string(),
                "main DB instance".to_string(),
            )
            .with_cost(87.60),
        ],
        edges: vec![
            GraphEdge::new(
                "aws_subnet.private".to_string(),
                "aws_vpc.main".to_string(),
                EdgeType::DependsOn,
            ),
            GraphEdge::new(
                "aws_db_subnet_group.main".to_string(),
                "aws_subnet.private".to_string(),
                EdgeType::DependsOn,
            ),
            GraphEdge::new(
                "aws_db_instance.main".to_string(),
                "aws_db_subnet_group.main".to_string(),
                EdgeType::DependsOn,
            )
            .with_cost_impact("$87.60/month".to_string()),
        ],
        metadata: GraphMetadata {
            node_count: 4,
            edge_count: 3,
            max_depth: 3,
            has_cycles: false,
            cycles: vec![],
            total_cost: Some(87.60),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("database_tier", json_output);
}

#[test]
fn golden_empty_graph() {
    let graph = DependencyGraph {
        nodes: vec![],
        edges: vec![],
        metadata: GraphMetadata {
            node_count: 0,
            edge_count: 0,
            max_depth: 0,
            has_cycles: false,
            cycles: vec![],
            total_cost: Some(0.0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("empty_graph", json_output);
}

#[test]
fn golden_isolated_resources() {
    let graph = DependencyGraph {
        nodes: vec![
            GraphNode::new_resource(
                "aws_s3_bucket.data".to_string(),
                "aws_s3_bucket".to_string(),
                "data bucket".to_string(),
            )
            .with_cost(5.0),
            GraphNode::new_resource(
                "aws_dynamodb_table.users".to_string(),
                "aws_dynamodb_table".to_string(),
                "users table".to_string(),
            )
            .with_cost(25.0),
            GraphNode::new_resource(
                "aws_lambda_function.api".to_string(),
                "aws_lambda_function".to_string(),
                "API Lambda".to_string(),
            )
            .with_cost(15.0),
        ],
        edges: vec![],
        metadata: GraphMetadata {
            node_count: 3,
            edge_count: 0,
            max_depth: 0,
            has_cycles: false,
            cycles: vec![],
            total_cost: Some(45.0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
    };

    let json_output = serde_json::to_value(&graph).unwrap();
    insta::assert_json_snapshot!("isolated_resources", json_output);
}
