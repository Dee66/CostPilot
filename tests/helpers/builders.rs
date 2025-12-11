use costpilot::engines::shared::models::{Detection, Severity, ResourceChange, ChangeAction, RegressionType};
use serde_json::json;

pub fn test_detection(
    rule_id: &str,
    resource_id: &str,
    numeric_severity: f64,
    message: &str,
    estimated_cost: Option<f64>,
) -> Detection {
    Detection::builder()
        .rule_id(rule_id.to_string())
        .resource_id(resource_id.to_string())
        .severity(match numeric_severity {
            x if x <= 3.0 => Severity::Low,
            x if x <= 6.0 => Severity::Medium,
            _ => Severity::High,
        })
        .message(message.to_string())
        .estimated_cost(estimated_cost.unwrap_or(0.0))
        .build()
}

pub fn test_resource_change(
    resource_id: &str,
    resource_type: &str,
    action_str: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    monthly_cost: Option<f64>,
) -> ResourceChange {
    let action_enum = match action_str.to_lowercase().as_str() {
        "create" | "created" => ChangeAction::Create,
        "update" | "modify" => ChangeAction::Update,
        "delete" | "removed" => ChangeAction::Delete,
        "replace" => ChangeAction::Replace,
        _ => ChangeAction::NoOp,
    };

    ResourceChange::builder()
        .resource_id(resource_id.to_string())
        .resource_type(resource_type.to_string())
        .action(action_enum)
        .old_config(before.unwrap_or(json!(null)))
        .new_config(after.unwrap_or(json!(null)))
        .monthly_cost(monthly_cost)
        .build()
}
