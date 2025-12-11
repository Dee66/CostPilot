// Heuristics split tests - Free uses embedded rules, Premium uses ProEngine

use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ResourceChange, ChangeAction};
use costpilot::edition::EditionContext;

#[test]
fn test_free_prediction_uses_static_method() {
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            before: None,
            after: Some(serde_json::json!({"instance_type": "t3.micro"})),
        },
    ];
    
    // Static prediction should work without filesystem access
    let result = PredictionEngine::predict_static(&changes);
    
    assert!(result.is_ok(), "Static prediction should succeed");
    let estimates = result.unwrap();
    
    // Free tier returns minimal estimates
    assert_eq!(estimates.len(), 1);
    assert_eq!(estimates[0].resource_id, "aws_instance.test");
    assert_eq!(estimates[0].monthly_cost, 0.0, "Free tier doesn't calculate costs");
}

#[test]
fn test_premium_prediction_requires_pro_engine() {
    let edition = EditionContext::free();
    
    // Attempting to use premium prediction without ProEngine should fail
    let result = edition.require_pro("Advanced Prediction");
    assert!(result.is_err());
}

#[test]
fn test_free_engine_no_filesystem_io() {
    // Free prediction should not touch filesystem
    let changes = vec![
        ResourceChange {
            resource_id: "aws_s3_bucket.test".to_string(),
            resource_type: "aws_s3_bucket".to_string(),
            action: ChangeAction::Create,
            before: None,
            after: Some(serde_json::json!({})),
        },
    ];
    
    let result = PredictionEngine::predict_static(&changes);
    assert!(result.is_ok());
    
    // Should succeed even if heuristics files don't exist
}

#[test]
fn test_predict_static_returns_zero_cost() {
    let changes = vec![
        ResourceChange {
            resource_id: "test.resource".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            before: None,
            after: Some(serde_json::json!({})),
        },
    ];
    
    let estimates = PredictionEngine::predict_static(&changes).unwrap();
    
    for estimate in estimates {
        assert_eq!(estimate.monthly_cost, 0.0);
        assert_eq!(estimate.confidence_score, 0.0);
        assert_eq!(estimate.heuristic_reference, Some("free_static".to_string()));
    }
}

#[test]
fn test_free_mode_skips_deleted_resources() {
    let changes = vec![
        ResourceChange {
            resource_id: "test.delete".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Delete,
            before: Some(serde_json::json!({})),
            after: None,
        },
    ];
    
    let estimates = PredictionEngine::predict_static(&changes).unwrap();
    
    // Deleted resources should be skipped
    assert_eq!(estimates.len(), 0);
}
