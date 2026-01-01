use costpilot::engines::explain::prediction_explainer::PredictionExplainer;
use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, CostEstimate, ResourceChange};
use serde_json::json;

#[test]
fn test_prediction_explainer_new() {
    let heuristics =
        costpilot::engines::prediction::minimal_heuristics::MinimalHeuristics::to_cost_heuristics();
    let _explainer = PredictionExplainer::new(&heuristics);
    // Just test that it creates successfully
}

#[test]
fn test_prediction_explainer_from_engine() {
    let engine = PredictionEngine::new().unwrap();
    let _explainer = PredictionExplainer::from_engine(&engine);
    // Just test that it creates successfully
}

#[test]
fn test_prediction_explainer_explain_ec2() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-ec2".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-ec2".to_string(),
        monthly_cost: 10.0,
        prediction_interval_low: 8.0,
        prediction_interval_high: 12.0,
        confidence_score: 0.8,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-ec2");
    assert_eq!(reasoning.resource_type, "aws_instance");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_rds() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-rds".to_string())
        .resource_type("aws_rds_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "allocated_storage": 20,
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-rds".to_string(),
        monthly_cost: 50.0,
        prediction_interval_low: 40.0,
        prediction_interval_high: 60.0,
        confidence_score: 0.9,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-rds");
    assert_eq!(reasoning.resource_type, "aws_rds_instance");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_lambda() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-lambda".to_string())
        .resource_type("aws_lambda_function".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "timeout": 30,
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-lambda".to_string(),
        monthly_cost: 5.0,
        prediction_interval_low: 4.0,
        prediction_interval_high: 6.0,
        confidence_score: 0.7,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: true,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-lambda");
    assert_eq!(reasoning.resource_type, "aws_lambda_function");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_dynamodb() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-dynamodb".to_string())
        .resource_type("aws_dynamodb_table".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PAY_PER_REQUEST",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-dynamodb".to_string(),
        monthly_cost: 15.0,
        prediction_interval_low: 12.0,
        prediction_interval_high: 18.0,
        confidence_score: 0.85,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-dynamodb");
    assert_eq!(reasoning.resource_type, "aws_dynamodb_table");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_nat_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-nat".to_string())
        .resource_type("aws_nat_gateway".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-nat".to_string(),
        monthly_cost: 30.0,
        prediction_interval_low: 25.0,
        prediction_interval_high: 35.0,
        confidence_score: 0.9,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-nat");
    assert_eq!(reasoning.resource_type, "aws_nat_gateway");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_load_balancer() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-alb".to_string())
        .resource_type("aws_lb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-alb".to_string(),
        monthly_cost: 20.0,
        prediction_interval_low: 16.0,
        prediction_interval_high: 24.0,
        confidence_score: 0.8,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-alb");
    assert_eq!(reasoning.resource_type, "aws_lb");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_s3() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-s3".to_string())
        .resource_type("aws_s3_bucket".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "storage_class": "STANDARD",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-s3".to_string(),
        monthly_cost: 2.0,
        prediction_interval_low: 1.5,
        prediction_interval_high: 2.5,
        confidence_score: 0.6,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-s3");
    assert_eq!(reasoning.resource_type, "aws_s3_bucket");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_generic() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-generic".to_string())
        .resource_type("aws_unknown_service".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-generic".to_string(),
        monthly_cost: 0.0,
        prediction_interval_low: 0.0,
        prediction_interval_high: 0.0,
        confidence_score: 0.0,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: true,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert_eq!(reasoning.resource_id, "test-generic");
    assert_eq!(reasoning.resource_type, "aws_unknown_service");
    assert!(!reasoning.steps.is_empty());
}

#[test]
fn test_prediction_explainer_explain_with_cold_start() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-cold-start".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "unknown-type",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-cold-start".to_string(),
        monthly_cost: 25.0,
        prediction_interval_low: 20.0,
        prediction_interval_high: 30.0,
        confidence_score: 0.5,
        heuristic_reference: Some("cold-start".to_string()),
        cold_start_inference: true,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning
        .steps
        .iter()
        .any(|step| step.description.contains("cold-start")));
}

#[test]
fn test_prediction_explainer_explain_with_high_confidence() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-high-confidence".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-high-confidence".to_string(),
        monthly_cost: 10.0,
        prediction_interval_low: 9.5,
        prediction_interval_high: 10.5,
        confidence_score: 0.95,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.9);
    assert!(reasoning.steps.iter().any(|step| step
        .output_value
        .as_ref()
        .is_some_and(|ov| ov.name == "confidence_score" && ov.value == "95%")));
}

#[test]
fn test_prediction_explainer_explain_with_low_confidence() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-low-confidence".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-low-confidence".to_string(),
        monthly_cost: 10.0,
        prediction_interval_low: 5.0,
        prediction_interval_high: 20.0,
        confidence_score: 0.3,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence < 0.5);
    assert!(reasoning.steps.iter().any(|step| step
        .output_value
        .as_ref()
        .is_some_and(|ov| ov.name == "confidence_score" && ov.value == "30%")));
    // Check for wide interval
    assert!(reasoning.steps.iter().any(|step| step
        .output_value
        .as_ref()
        .is_some_and(|ov| ov.name == "interval" && ov.value == "$5.00 - $20.00")));
}

// ===== PREDICTION EXPLAINER EDGE CASE TESTS =====

#[test]
fn test_prediction_explainer_zero_cost_edge_case() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-zero".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-zero".to_string(),
        monthly_cost: 0.0,
        prediction_interval_low: 0.0,
        prediction_interval_high: 0.0,
        confidence_score: 1.0,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}

#[test]
fn test_prediction_explainer_negative_cost_edge_case() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-negative".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-negative".to_string(),
        monthly_cost: -50.0,
        prediction_interval_low: -60.0,
        prediction_interval_high: -40.0,
        confidence_score: 0.8,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}

#[test]
fn test_prediction_explainer_extremely_high_cost() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-high".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "p4d.24xlarge",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-high".to_string(),
        monthly_cost: 100000.0,
        prediction_interval_low: 90000.0,
        prediction_interval_high: 110000.0,
        confidence_score: 0.9,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}

#[test]
fn test_prediction_explainer_empty_resource_id_edge_case() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "".to_string(),
        monthly_cost: 10.0,
        prediction_interval_low: 8.0,
        prediction_interval_high: 12.0,
        confidence_score: 0.8,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}

#[test]
fn test_prediction_explainer_extremely_long_resource_names() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let long_name = "a".repeat(1000);
    let change = ResourceChange::builder()
        .resource_id(long_name.clone())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: long_name,
        monthly_cost: 10.0,
        prediction_interval_low: 8.0,
        prediction_interval_high: 12.0,
        confidence_score: 0.8,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}

#[test]
fn test_prediction_explainer_zero_confidence_edge_case() {
    let engine = PredictionEngine::new().unwrap();
    let explainer = PredictionExplainer::from_engine(&engine);

    let change = ResourceChange::builder()
        .resource_id("test-zero-confidence".to_string())
        .resource_type("aws_instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let estimate = CostEstimate {
        resource_id: "test-zero-confidence".to_string(),
        monthly_cost: 10.0,
        prediction_interval_low: 0.0,
        prediction_interval_high: 100.0,
        confidence_score: 0.0,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let reasoning = explainer.explain(&change, &estimate);
    assert!(reasoning.overall_confidence >= 0.0);
}
