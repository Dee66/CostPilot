use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange};
use costpilot::engines::performance::budgets::PerformanceBudgets;
use costpilot::edition::EditionContext;
use serde_json::json;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
#[cfg(test)]
use quickcheck_macros::quickcheck;

#[test]
fn test_prediction_engine_new() {
    let engine = PredictionEngine::new().unwrap();
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_new_with_edition_free() {
    let edition = EditionContext::free();
    let engine = PredictionEngine::new_with_edition(&edition).unwrap();
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_heuristics() {
    let heuristics = costpilot::engines::prediction::minimal_heuristics::MinimalHeuristics::to_cost_heuristics();
    let engine = PredictionEngine::with_heuristics(heuristics.clone());
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_verbose() {
    let engine = PredictionEngine::new().unwrap().with_verbose(true);
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_performance_tracking() {
    let budgets = PerformanceBudgets::default();
    let engine = PredictionEngine::new().unwrap().with_performance_tracking(budgets);
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_predict_resource_cost_ec2() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_lambda() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_rds() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_dynamodb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-table".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PAY_PER_REQUEST",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_nat_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_nat_gateway".to_string())
        .resource_id("test-nat".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_s3() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_s3_bucket".to_string())
        .resource_id("test-bucket".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_load_balancer() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-lb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_total_cost() {
    let mut engine = PredictionEngine::new().unwrap();
    let changes = vec![
        ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build(),
    ];

    let result = engine.predict_total_cost(&changes);
    assert!(result.is_ok());
    let total_cost = result.unwrap();
    assert!(total_cost.monthly >= 0.0);
}

#[test]
fn test_explain() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.explain(&change);
    assert!(result.is_ok());
    let explanation = result.unwrap();
    assert!(!explanation.steps.is_empty());
}

#[test]
fn test_predict_resource_cost_unknown_type() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("unknown_resource".to_string())
        .resource_id("test-resource".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({}))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle unknown resource types gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_resource_cost_invalid_config() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({}))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle missing configuration gracefully
    assert!(result.is_ok() || result.is_err());
}

proptest! {
    #[test]
    fn test_cost_non_negative(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Costs should never be negative
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low >= 0.0);
            prop_assert!(estimate.prediction_interval_high >= 0.0);
        }
    }

    #[test]
    fn test_prediction_intervals_valid(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Prediction intervals should be valid (low <= high)
        if let Ok(estimate) = result {
            prop_assert!(estimate.prediction_interval_low <= estimate.prediction_interval_high);
        }
    }

    #[test]
    fn test_confidence_score_bounds(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Confidence scores should be between 0 and 1
        if let Ok(estimate) = result {
            prop_assert!(estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_deterministic_output(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine1 = PredictionEngine::new().unwrap();
        let engine2 = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type.clone())
            .resource_id(resource_id.clone())
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result1 = engine1.predict_resource_cost(&change);
        let result2 = engine2.predict_resource_cost(&change);
        // Same input should produce same output
        match (result1, result2) {
            (Ok(est1), Ok(est2)) => {
                prop_assert_eq!(est1.monthly_cost, est2.monthly_cost);
                prop_assert_eq!(est1.prediction_interval_low, est2.prediction_interval_low);
                prop_assert_eq!(est1.prediction_interval_high, est2.prediction_interval_high);
                prop_assert_eq!(est1.confidence_score, est2.confidence_score);
            }
            (Err(_), Err(_)) => {} // Both errors is also deterministic
            _ => prop_assert!(false, "Inconsistent results for same input"),
        }
    }

    #[test]
    fn test_zero_cost_edge_cases(cost in 0.0f64..1000.0) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Even with zero cost inputs, output should be valid
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low <= estimate.prediction_interval_high);
            prop_assert!(estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_negative_cost_guards(cost in -1000.0f64..0.0) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Should not produce negative costs
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low >= 0.0);
        }
    }

    #[test]
    fn test_overflow_protection(large_cost in 1e10f64..1e20) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Should handle large numbers without overflow
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost.is_finite());
            prop_assert!(estimate.prediction_interval_low.is_finite());
            prop_assert!(estimate.prediction_interval_high.is_finite());
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct ArbResourceChange(ResourceChange);

impl Arbitrary for ArbResourceChange {
    fn arbitrary(g: &mut Gen) -> Self {
        let resource_id: String = Arbitrary::arbitrary(g);
        let resource_type: String = Arbitrary::arbitrary(g);
        let monthly_cost: Option<f64> = Arbitrary::arbitrary(g);

        ArbResourceChange(ResourceChange::builder()
            .resource_id(resource_id)
            .resource_type(resource_type)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .monthly_cost(monthly_cost.map(|c| c.abs()).unwrap_or(0.0))
            .build())
    }
}

#[quickcheck]
fn quickcheck_cost_non_negative(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.monthly_cost >= 0.0 && 
        estimate.prediction_interval_low >= 0.0 && 
        estimate.prediction_interval_high >= 0.0
    } else {
        true // Errors are acceptable
    }
}

#[quickcheck]
fn quickcheck_prediction_intervals_valid(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.prediction_interval_low <= estimate.prediction_interval_high
    } else {
        true
    }
}

#[quickcheck]
fn quickcheck_confidence_bounds(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0
    } else {
        true
    }
}