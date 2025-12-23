/// Test model builders for canonical production structs
///
/// These builders provide simple, deterministic construction of model types
/// used across the test suite, minimizing test fragility when struct fields change.

use costpilot::engines::shared::models::*;
use serde_json::json;
use std::collections::HashMap;

/// Create a test CostEstimate with minimal required fields
pub fn make_test_cost_estimate(resource_id: &str, monthly_cost: f64) -> CostEstimate {
    CostEstimate {
        resource_id: resource_id.to_string(),
        monthly_cost,
        prediction_interval_low: monthly_cost * 0.9,
        prediction_interval_high: monthly_cost * 1.1,
        confidence_score: 0.95,
        heuristic_reference: Some("test-default".to_string()),
        cold_start_inference: false,
    }
}

/// Create a test CostEstimate with custom confidence
pub fn make_test_cost_estimate_with_confidence(
    resource_id: &str,
    monthly_cost: f64,
    confidence: f64,
) -> CostEstimate {
    CostEstimate {
        resource_id: resource_id.to_string(),
        monthly_cost,
        prediction_interval_low: monthly_cost * 0.9,
        prediction_interval_high: monthly_cost * 1.1,
        confidence_score: confidence,
        heuristic_reference: Some("test-default".to_string()),
        cold_start_inference: false,
    }
}

/// Create a test ResourceChange with minimal required fields
pub fn make_test_resource_change(
    resource_id: &str,
    resource_type: &str,
    action: ChangeAction,
) -> ResourceChange {
    ResourceChange::builder()
        .resource_id(resource_id)
        .resource_type(resource_type)
        .action(action)
        .old_config(json!({}))
        .new_config(json!({}))
        .monthly_cost(0.0)
        .tags(HashMap::new())
        .build()
}

/// Create a test ResourceChange with monthly cost
pub fn make_test_resource_change_with_cost(
    resource_id: &str,
    resource_type: &str,
    action: ChangeAction,
    monthly_cost: f64,
) -> ResourceChange {
    ResourceChange::builder()
        .resource_id(resource_id)
        .resource_type(resource_type)
        .action(action)
        .new_config(json!({"type": resource_type}))
        .tags(HashMap::new())
        .monthly_cost(monthly_cost)
        .cost_impact(CostImpact {
            delta: monthly_cost,
            confidence: 0.85,
            heuristic_source: Some("test".to_string()),
        })
        .build()
}

/// Create a test Detection with minimal required fields
pub fn make_test_detection(
    rule_id: &str,
    resource_id: &str,
    severity: Severity,
) -> Detection {
    Detection::builder()
        .rule_id(rule_id)
        .severity(severity)
        .resource_id(resource_id)
        .regression_type(RegressionType::Configuration)
        .severity_score(match severity {
            Severity::Critical => 90,
            Severity::High => 70,
            Severity::Medium => 50,
            Severity::Low => 30,
        })
        .message("test".to_string())
        .estimated_cost(0.0)
        .build()
}

/// Create a test Detection with estimated cost
pub fn make_test_detection_with_cost(
    rule_id: &str,
    resource_id: &str,
    severity: Severity,
    estimated_cost: f64,
) -> Detection {
    Detection::builder()
        .rule_id(rule_id)
        .severity(severity)
        .resource_id(resource_id)
        .regression_type(RegressionType::Configuration)
        .severity_score(match severity {
            Severity::Critical => 90,
            Severity::High => 70,
            Severity::Medium => 50,
            Severity::Low => 30,
        })
        .message(format!("Test detection for {}", resource_id))
        .estimated_cost(estimated_cost)
        .build()
}

/// Create a test ScanMetadata
pub fn make_test_scan_metadata() -> ScanMetadata {
    ScanMetadata {
        timestamp: Some("2025-01-01T00:00:00Z".to_string()),
        heuristics_version: "test-1.0".to_string(),
        policy_version: None,
        deterministic: true,
    }
}

/// Create a test ScanResult
pub fn make_test_scan_result(
    resource_changes: Vec<ResourceChange>,
    cost_estimates: Vec<CostEstimate>,
    detections: Vec<Detection>,
) -> ScanResult {
    let total_monthly_delta = cost_estimates.iter().map(|e| e.monthly_cost).sum();

    ScanResult {
        resource_changes,
        cost_estimates,
        detections,
        total_monthly_delta,
        metadata: make_test_scan_metadata(),
    }
}

/// Create a test EditionContext with Premium capabilities (test-only)
#[cfg(test)]
pub fn test_edition_premium() -> costpilot::edition::EditionContext {
    costpilot::edition::EditionContext {
        edition: costpilot::edition::Edition::Premium,
        allow_policy_enforce: true,
        max_mapping_depth: 10,
        allow_advanced_prediction: true,
        allow_trend_analysis: true,
    }
}

/// Create a basic CostEstimate helper
pub fn make_cost_estimate(monthly_cost: f64) -> CostEstimate {
    CostEstimate {
        resource_id: "test-resource".to_string(),
        monthly_cost,
        prediction_interval_low: monthly_cost * 0.9,
        prediction_interval_high: monthly_cost * 1.1,
        confidence_score: 0.95,
        heuristic_reference: Some("test-default".to_string()),
        cold_start_inference: false,
    }
}

/// Create a ResourceChange helper with optional monthly cost
pub fn make_resource_change(action: ChangeAction, monthly: Option<f64>) -> ResourceChange {
    let mut builder = ResourceChange::builder()
        .resource_id("test-resource")
        .resource_type("test-type")
        .action(action)
        .old_config(serde_json::Value::Null)
        .new_config(serde_json::Value::Null)
        .tags(HashMap::new());

    if let Some(cost) = monthly {
        builder = builder.monthly_cost(cost);
    }

    builder.build()
}
