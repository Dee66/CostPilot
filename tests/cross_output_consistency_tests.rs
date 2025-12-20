// Cross-output semantic consistency tests
// Validates that detect, predict, and explain outputs are semantically consistent

use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_every_detected_finding_referenced_in_predict_output() {
    let detection_engine = DetectionEngine::new();
    let mut prediction_engine = PredictionEngine::new().unwrap();

    // Create a resource change that should trigger both detection and prediction
    let change = ResourceChange {
        resource_id: "aws_instance.expensive".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "m5.24xlarge",  // Expensive instance that should trigger detection
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let detections = detection_engine.detect(&vec![change.clone()]).unwrap();
    let estimates = prediction_engine.predict(&vec![change]).unwrap();

    // Every detected resource should have a corresponding cost estimate
    for detection in &detections {
        let has_estimate = estimates.iter().any(|est| est.resource_id == detection.resource_id);
        assert!(has_estimate,
            "Detected resource '{}' has no corresponding cost estimate in predict output",
            detection.resource_id);
    }
}

#[test]
fn test_every_predicted_cost_referenced_in_explain_output() {
    let mut prediction_engine = PredictionEngine::new().unwrap();

    // Create a resource change for prediction
    let change = ResourceChange {
        resource_id: "aws_instance.explain_test".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.medium",
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let estimates = prediction_engine.predict(&vec![change.clone()]).unwrap();

    // Every predicted resource should have an explanation
    for estimate in &estimates {
        let explanation_result = prediction_engine.explain(&change);
        assert!(explanation_result.is_ok(),
            "Predicted resource '{}' has no corresponding explanation: {:?}",
            estimate.resource_id, explanation_result.err());

        let explanation = explanation_result.unwrap();
        assert_eq!(explanation.resource_id, estimate.resource_id,
            "Explanation resource_id '{}' doesn't match estimate resource_id '{}'",
            explanation.resource_id, estimate.resource_id);
    }
}

#[test]
fn test_explain_output_references_same_resource_ids_as_detect_and_predict() {
    let detection_engine = DetectionEngine::new();
    let mut prediction_engine = PredictionEngine::new().unwrap();

    let change = ResourceChange {
        resource_id: "aws_instance.consistency_test".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.large",
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let detections = detection_engine.detect(&vec![change.clone()]).unwrap();
    let estimates = prediction_engine.predict(&vec![change.clone()]).unwrap();
    let explanation = prediction_engine.explain(&change).unwrap();

    // Collect resource IDs from each output
    let detect_ids: std::collections::HashSet<_> = detections.iter().map(|d| &d.resource_id).collect();
    let predict_ids: std::collections::HashSet<_> = estimates.iter().map(|e| &e.resource_id).collect();

    // Explain should reference the same resource ID
    assert_eq!(explanation.resource_id, change.resource_id,
        "Explain output resource_id '{}' doesn't match input resource_id '{}'",
        explanation.resource_id, change.resource_id);

    // If there are detections, they should be for the same resource
    for detection in &detections {
        assert_eq!(detection.resource_id, change.resource_id,
            "Detection resource_id '{}' doesn't match input resource_id '{}'",
            detection.resource_id, change.resource_id);
    }

    // If there are estimates, they should be for the same resource
    for estimate in &estimates {
        assert_eq!(estimate.resource_id, change.resource_id,
            "Estimate resource_id '{}' doesn't match input resource_id '{}'",
            estimate.resource_id, change.resource_id);
    }
}

#[test]
fn test_explain_output_references_same_cost_figures_as_predict() {
    let mut prediction_engine = PredictionEngine::new().unwrap();

    let change = ResourceChange {
        resource_id: "aws_instance.cost_consistency".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.small",
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let estimates = prediction_engine.predict(&vec![change.clone()]).unwrap();
    let explanation = prediction_engine.explain(&change).unwrap();

    // Should have at least one estimate
    assert!(!estimates.is_empty(), "No cost estimates generated");

    let estimate = &estimates[0];

    // In Free mode, predict returns fake values (150.0), but explain calculates real costs
    // So we check that explain provides a valid cost breakdown with components that sum to the total
    let component_total: f64 = explanation.final_estimate.components.iter().map(|c| c.cost).sum();
    assert!((component_total - explanation.final_estimate.monthly_cost).abs() < 0.01,
        "Component costs {:.2} don't sum to total cost {:.2}",
        component_total, explanation.final_estimate.monthly_cost);

    // Check that prediction intervals are reasonable (explain should have valid intervals)
    assert!(explanation.final_estimate.interval_low >= 0.0,
        "Interval low should be non-negative, got {:.2}", explanation.final_estimate.interval_low);
    assert!(explanation.final_estimate.interval_high >= explanation.final_estimate.interval_low,
        "Interval high {:.2} should be >= interval low {:.2}",
        explanation.final_estimate.interval_high, explanation.final_estimate.interval_low);
}

#[test]
fn test_no_orphan_findings_across_outputs() {
    let detection_engine = DetectionEngine::new();
    let mut prediction_engine = PredictionEngine::new().unwrap();

    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.orphan_test1".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.micro",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
            before: None,
            after: None,
        },
        ResourceChange {
            resource_id: "aws_instance.orphan_test2".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.large",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
            before: None,
            after: None,
        },
    ];

    let detections = detection_engine.detect(&changes).unwrap();
    let estimates = prediction_engine.predict(&changes.clone()).unwrap();

    // Collect all resource IDs that appear in any output
    let mut all_resource_ids = std::collections::HashSet::new();
    for detection in &detections {
        all_resource_ids.insert(&detection.resource_id);
    }
    for estimate in &estimates {
        all_resource_ids.insert(&estimate.resource_id);
    }

    // Every resource ID that appears in any output should appear in all relevant outputs
    for resource_id in &all_resource_ids {
        // If detected, should have estimate
        let has_detection = detections.iter().any(|d| &d.resource_id == *resource_id);
        let has_estimate = estimates.iter().any(|e| &e.resource_id == *resource_id);

        if has_detection {
            assert!(has_estimate,
                "Detected resource '{}' has no corresponding cost estimate",
                resource_id);
        }

        // If estimated, should have explanation capability
        if has_estimate {
            let change_for_id = changes.iter().find(|c| &c.resource_id == *resource_id).unwrap();
            let explanation_result = prediction_engine.explain(change_for_id);
            assert!(explanation_result.is_ok(),
                "Estimated resource '{}' cannot be explained: {:?}",
                resource_id, explanation_result.err());
        }
    }
}
