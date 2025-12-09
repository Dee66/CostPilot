// Golden file tests for explain output

use costpilot::engines::explain::ExplainEngine;
use costpilot::engines::shared::models::{Detection, ResourceChange};

#[test]
fn golden_explain_high_severity_detection() {
    let detection = Detection {
        resource_id: "aws_nat_gateway.main".to_string(),
        resource_type: "aws_nat_gateway".to_string(),
        issue: "NAT Gateway cost accumulation".to_string(),
        severity: 8.5,
        confidence: 0.95,
        monthly_cost: 32.85,
        fix_snippet: None,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_detection_reasoning(&detection);

    insta::assert_snapshot!("high_severity_nat_gateway", explanation);
}

#[test]
fn golden_explain_cost_spike_regression() {
    let change = ResourceChange {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        action: "update".to_string(),
        before: Some(serde_json::json!({"instance_type": "t3.small"})),
        after: Some(serde_json::json!({"instance_type": "t3.xlarge"})),
        monthly_cost: 120.0,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_cost_change(&change).unwrap();

    insta::assert_json_snapshot!("cost_spike_instance_upgrade", explanation);
}

#[test]
fn golden_explain_lambda_invocation_pattern() {
    let resource = ResourceChange {
        resource_id: "aws_lambda_function.api".to_string(),
        resource_type: "aws_lambda_function".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "memory_size": 1024,
            "timeout": 60
        })),
        monthly_cost: 250.0,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_prediction(&resource).unwrap();

    insta::assert_json_snapshot!("lambda_high_cost", explanation);
}

#[test]
fn golden_explain_rds_storage_growth() {
    let change = ResourceChange {
        resource_id: "aws_db_instance.main".to_string(),
        resource_type: "aws_db_instance".to_string(),
        action: "update".to_string(),
        before: Some(serde_json::json!({"allocated_storage": 100})),
        after: Some(serde_json::json!({"allocated_storage": 500})),
        monthly_cost: 180.0,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_cost_change(&change).unwrap();

    insta::assert_json_snapshot!("rds_storage_increase", explanation);
}

#[test]
fn golden_explain_dynamodb_capacity_change() {
    let change = ResourceChange {
        resource_id: "aws_dynamodb_table.users".to_string(),
        resource_type: "aws_dynamodb_table".to_string(),
        action: "update".to_string(),
        before: Some(serde_json::json!({
            "read_capacity": 5,
            "write_capacity": 5
        })),
        after: Some(serde_json::json!({
            "read_capacity": 100,
            "write_capacity": 100
        })),
        monthly_cost: 450.0,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_cost_change(&change).unwrap();

    insta::assert_json_snapshot!("dynamodb_capacity_scale", explanation);
}

#[test]
fn golden_explain_elasticache_node_addition() {
    let change = ResourceChange {
        resource_id: "aws_elasticache_cluster.redis".to_string(),
        resource_type: "aws_elasticache_cluster".to_string(),
        action: "update".to_string(),
        before: Some(serde_json::json!({"num_cache_nodes": 2})),
        after: Some(serde_json::json!({"num_cache_nodes": 5})),
        monthly_cost: 350.0,
    };

    let engine = ExplainEngine::new();
    let explanation = engine.explain_cost_change(&change).unwrap();

    insta::assert_json_snapshot!("elasticache_scale_out", explanation);
}

#[test]
fn golden_explain_multi_detection_summary() {
    let detections = vec![
        Detection {
            resource_id: "aws_nat_gateway.main".to_string(),
            resource_type: "aws_nat_gateway".to_string(),
            issue: "High fixed cost".to_string(),
            severity: 8.0,
            confidence: 0.95,
            monthly_cost: 32.85,
            fix_snippet: None,
        },
        Detection {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            issue: "Oversized instance".to_string(),
            severity: 6.5,
            confidence: 0.80,
            monthly_cost: 120.0,
            fix_snippet: None,
        },
    ];

    let engine = ExplainEngine::new();
    let summary = engine.explain_detection_summary(&detections);

    insta::assert_snapshot!("multi_detection_summary", summary);
}

#[test]
fn golden_explain_baseline_comparison() {
    let current = ResourceChange {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({"instance_type": "t3.large"})),
        monthly_cost: 70.08,
    };

    let baseline_cost = 35.04; // Previous t3.medium cost

    let engine = ExplainEngine::new();
    let explanation = engine.explain_vs_baseline(&current, baseline_cost);

    insta::assert_json_snapshot!("baseline_deviation", explanation);
}
