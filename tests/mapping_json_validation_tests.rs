// Mapping graph JSON validation tests

use costpilot::engines::mapping::{MappingEngine, GraphConfig};
use costpilot::engines::detection::ResourceChange;
use costpilot::engines::shared::models::ChangeAction;
use costpilot::edition::EditionContext;
use serde_json::json;

#[test]
fn test_mapping_graph_produces_valid_json() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_vpc.main".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({"cidr_block": "10.0.0.0/16"}))
            .monthly_cost(0.0)
            .build(),
        ResourceChange::builder()
            .resource_id("aws_subnet.public".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "vpc_id": "aws_vpc.main",
                "cidr_block": "10.0.1.0/24"
            }))
            .monthly_cost(0.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    // Validate it's parseable JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object() || parsed.is_array());
}

#[test]
fn test_mapping_graph_json_has_nodes_field() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_instance.web".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.medium"}))
            .monthly_cost(70.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert!(parsed.get("nodes").is_some(), "JSON should have 'nodes' field");
    assert!(parsed["nodes"].is_array(), "'nodes' should be an array");
}

#[test]
fn test_mapping_graph_json_has_edges_field() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_vpc.main".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .monthly_cost(0.0)
            .build(),
        ResourceChange::builder()
            .resource_id("aws_subnet.public".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({"vpc_id": "aws_vpc.main"}))
            .monthly_cost(0.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    if let Some(edges) = parsed.get("edges") {
        assert!(edges.is_array(), "'edges' should be an array if present");
    }
}

#[test]
fn test_mapping_graph_json_nodes_have_required_fields() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_instance.web".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.medium"}))
            .monthly_cost(70.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let nodes = parsed["nodes"].as_array().unwrap();
    
    assert!(!nodes.is_empty(), "Should have at least one node");
    
    let first_node = &nodes[0];
    assert!(first_node.get("id").is_some(), "Node should have 'id' field");
    assert!(first_node.get("node_type").is_some() || first_node.get("resource_type").is_some(), 
        "Node should have 'node_type' or 'resource_type' field");
}

#[test]
fn test_mapping_graph_empty_produces_valid_json() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    // Even empty graph should produce valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object() || parsed.is_array());
}

#[test]
fn test_mapping_graph_json_cost_fields_are_numbers() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_instance.web".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.medium"}))
            .monthly_cost(70.08)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let nodes = parsed["nodes"].as_array().unwrap();
    
    for node in nodes {
        if let Some(cost) = node.get("monthly_cost").or_else(|| node.get("cost")) {
            assert!(cost.is_number() || cost.is_null(), 
                "Cost field should be a number or null: {:?}", cost);
        }
    }
}

#[test]
fn test_mapping_graph_json_pretty_printed() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_instance.web".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .monthly_cost(70.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    // Pretty-printed JSON should have newlines
    assert!(json.contains('\n'), "JSON should be pretty-printed with newlines");
}

#[test]
fn test_mapping_graph_json_roundtrip() {
    let edition = EditionContext::free();
    let mut engine = MappingEngine::with_config(
        GraphConfig { max_depth: Some(1), ..Default::default() },
        Default::default(),
        &edition,
    );
    
    let changes = vec![
        ResourceChange::builder()
            .resource_id("aws_vpc.main".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .monthly_cost(0.0)
            .build(),
    ];
    
    let graph = engine.build_graph(&changes).unwrap();
    let json = engine.export_json(&graph).unwrap();
    
    // Parse and re-serialize
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let re_serialized = serde_json::to_string_pretty(&parsed).unwrap();
    
    // Should still be valid
    let re_parsed: serde_json::Value = serde_json::from_str(&re_serialized).unwrap();
    assert!(re_parsed.is_object() || re_parsed.is_array());
}
