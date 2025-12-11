// Test-only compatibility helpers for legacy struct literal patterns

use costpilot::engines::shared::models::{
    CostEstimate, ResourceChange, Detection, ChangeAction, Severity,
};
use serde_json::Value;

/// Create a CostEstimate from legacy field names
pub fn make_cost_estimate_from_legacy(
    monthly: f64,
    lower: Option<f64>,
    upper: Option<f64>,
    confidence: Option<f64>,
    resource_id: Option<&str>,
) -> CostEstimate {
    CostEstimate::builder()
        .resource_id(resource_id.unwrap_or("test://resource").to_string())
        .monthly_cost(monthly)
        .prediction_interval_low(lower.unwrap_or(monthly * 0.9))
        .prediction_interval_high(upper.unwrap_or(monthly * 1.1))
        .confidence_score(confidence.unwrap_or(0.75))
        .heuristic_reference("test".to_string())
        .cold_start_inference(false)
        .build()
}

/// Create a ResourceChange from legacy field patterns
pub fn make_resource_change_from_legacy(
    before: Option<Value>,
    after: Option<Value>,
    action_str: Option<&str>,
    monthly: Option<f64>,
) -> ResourceChange {
    let action = match action_str {
        Some("create") | Some("created") => ChangeAction::Create,
        Some("update") | Some("modify") => ChangeAction::Update,
        Some("delete") | Some("removed") => ChangeAction::Delete,
        Some("replace") => ChangeAction::Replace,
        _ => ChangeAction::NoOp,
    };

    let mut builder = ResourceChange::builder()
        .resource_id("test_resource".to_string())
        .action(action);

    if let Some(b) = before {
        builder = builder.old_config(b);
    }
    if let Some(a) = after {
        builder = builder.new_config(a);
    }
    if let Some(cost) = monthly {
        builder = builder.monthly_cost(cost);
    }

    builder.build()
}

/// Create a Detection from legacy field patterns
pub fn make_detection_from_legacy(
    rule_id: &str,
    issue: Option<&str>,
    severity_num: Option<f64>,
    estimated: Option<f64>,
) -> Detection {
    let severity = match severity_num.unwrap_or(5.0) {
        s if s <= 3.0 => Severity::Low,
        s if s <= 6.0 => Severity::Medium,
        _ => Severity::High,
    };

    Detection::builder()
        .rule_id(rule_id.to_string())
        .severity(severity)
        .message(issue.unwrap_or("test detection").to_string())
        .estimated_cost(estimated.unwrap_or(0.0))
        .build()
}
