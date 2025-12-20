// Invariant enforcement tests for detection and prediction engines
// Validates that severity scores, confidence scores, and other invariants are properly bounded

use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange, Severity};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_severity_score_always_within_defined_bounds() {
    let engine = DetectionEngine::new();

    // Create a resource change that should trigger detections
    let change = ResourceChange {
        resource_id: "aws_instance.large".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "m5.24xlarge",  // Very large instance, should trigger cost detection
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let detections = engine.detect(&vec![change]).unwrap();

    // Check that all severity scores are within 0-100 bounds
    for detection in &detections {
        assert!(detection.severity_score <= 100,
            "Severity score {} exceeds maximum bound of 100", detection.severity_score);
        assert!(detection.severity_score >= 0,
            "Severity score {} below minimum bound of 0", detection.severity_score);
    }
}

#[test]
fn test_confidence_score_always_within_defined_bounds() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create resource changes with various configurations
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.test1".to_string(),
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
            resource_id: "aws_lambda_function.test".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "runtime": "python3.9",
                "handler": "index.handler",
                "memory_size": 256
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
            before: None,
            after: None,
        },
    ];

    let estimates = engine.predict(&changes).unwrap();

    // Check that all confidence scores are within 0.0-1.0 bounds
    for estimate in &estimates {
        assert!(estimate.confidence_score >= 0.0,
            "Confidence score {:.3} below minimum bound of 0.0", estimate.confidence_score);
        assert!(estimate.confidence_score <= 1.0,
            "Confidence score {:.3} exceeds maximum bound of 1.0", estimate.confidence_score);
    }
}

#[test]
fn test_severity_monotonically_increases_with_cost_delta() {
    let engine = DetectionEngine::new();

    // Create changes with increasing cost impact
    let small_change = ResourceChange {
        resource_id: "aws_instance.small".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.micro",  // Small instance
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let large_change = ResourceChange {
        resource_id: "aws_instance.large".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "m5.24xlarge",  // Very large instance
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let small_detections = engine.detect(&vec![small_change]).unwrap();
    let large_detections = engine.detect(&vec![large_change]).unwrap();

    // If both have detections, the large change should have higher or equal severity
    if !small_detections.is_empty() && !large_detections.is_empty() {
        let max_small_severity = small_detections.iter().map(|d| d.severity_score).max().unwrap();
        let max_large_severity = large_detections.iter().map(|d| d.severity_score).max().unwrap();

        assert!(max_large_severity >= max_small_severity,
            "Severity should monotonically increase with cost delta: small={}, large={}",
            max_small_severity, max_large_severity);
    }
}

#[test]
fn test_confidence_decreases_under_cold_start_assumptions() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create a change for a known instance type (should have high confidence)
    let known_change = ResourceChange {
        resource_id: "aws_instance.known".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.micro",  // Well-known instance type
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    // Create a change for an unknown instance type (should trigger cold start)
    let unknown_change = ResourceChange {
        resource_id: "aws_instance.unknown".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "custom-instance-type-xyz",  // Unknown instance type
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let known_estimates = engine.predict(&vec![known_change]).unwrap();
    let unknown_estimates = engine.predict(&vec![unknown_change]).unwrap();

    // Both should produce estimates
    assert!(!known_estimates.is_empty(), "Known instance type should produce estimates");
    assert!(!unknown_estimates.is_empty(), "Unknown instance type should produce estimates");

    let known_confidence = known_estimates[0].confidence_score;
    let unknown_confidence = unknown_estimates[0].confidence_score;

    // Confidence should be lower for cold-start inference
    assert!(unknown_confidence <= known_confidence,
        "Cold-start confidence ({:.3}) should be <= known confidence ({:.3})",
        unknown_confidence, known_confidence);
}

#[test]
fn test_incident_classification_consistent_with_severity_and_materiality() {
    let engine = DetectionEngine::new();

    // Create a high-cost change that should trigger critical detection
    let change = ResourceChange {
        resource_id: "aws_instance.critical".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "p3.16xlarge",  // Extremely expensive GPU instance
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
        before: None,
        after: None,
    };

    let detections = engine.detect(&vec![change]).unwrap();

    // Check that high-severity detections are properly classified
    for detection in &detections {
        if detection.severity_score >= 70 {  // High severity threshold
            // Should be classified as High or Critical severity
            assert!(matches!(detection.severity, Severity::High | Severity::Critical),
                "High severity score {} should correspond to High or Critical severity, got {:?}",
                detection.severity_score, detection.severity);
        }
    }
}
