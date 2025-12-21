// Terraform plan JSON parser

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use crate::engines::shared::models::{ChangeAction, ResourceChange};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Terraform plan JSON structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerraformPlan {
    pub format_version: String,
    pub terraform_version: Option<String>,
    pub resource_changes: Option<Vec<TerraformResourceChange>>,
    pub configuration: Option<Value>,
}

/// Terraform resource change
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerraformResourceChange {
    pub address: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub name: String,
    pub provider_name: Option<String>,
    pub change: TerraformChange,
    pub module_address: Option<String>,
}

/// Terraform change details
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerraformChange {
    pub actions: Vec<String>,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub after_unknown: Option<Value>,
}

/// Parse Terraform plan JSON from string
pub fn parse_terraform_plan(json_content: &str) -> Result<TerraformPlan> {
    let plan: TerraformPlan = serde_json::from_str(json_content).map_err(|e| {
        CostPilotError::new(
            "PARSE_001",
            ErrorCategory::ParseError,
            format!("Failed to parse Terraform plan JSON: {}", e),
        )
        .with_hint("Ensure the input is a valid Terraform plan JSON file generated with 'terraform show -json plan.out'")
    })?;

    // Validate required fields for a meaningful plan
    if plan.resource_changes.is_none() {
        return Err(CostPilotError::new(
            "PARSE_002",
            ErrorCategory::ParseError,
            "Terraform plan must contain resource_changes field".to_string(),
        )
        .with_hint("Ensure the plan contains a resource_changes field"));
    }

    Ok(plan)
}

/// Convert Terraform plan to canonical ResourceChange format
pub fn convert_to_resource_changes(plan: &TerraformPlan) -> Result<Vec<ResourceChange>> {
    let mut changes = Vec::new();

    if let Some(resource_changes) = &plan.resource_changes {
        for tf_change in resource_changes {
            let action = determine_action(&tf_change.change.actions)?;

            // Skip no-op changes
            if action == ChangeAction::NoOp {
                continue;
            }

            let tags = extract_tags(&tf_change.change.after);

            // Extract module path from address if not provided
            let module_path = tf_change.module_address.clone()
                .or_else(|| extract_module_path_from_address(&tf_change.address));

            changes.push(ResourceChange {
                resource_id: tf_change.address.clone(),
                resource_type: tf_change.resource_type.clone(),
                action,
                module_path,
                old_config: tf_change.change.before.clone(),
                new_config: tf_change.change.after.clone(),
                tags,
                monthly_cost: None,
                config: None,
                cost_impact: None,
            });
        }
    }

    Ok(changes)
}

/// Extract module path from resource address
/// For address "module.vpc.aws_instance.test", returns "module.vpc"
/// For address "aws_instance.test", returns None
fn extract_module_path_from_address(address: &str) -> Option<String> {
    if let Some(module_part) = address.strip_prefix("module.") {
        // Find the first dot after "module." to get the module name
        if let Some(dot_pos) = module_part.find('.') {
            Some(format!("module.{}", &module_part[..dot_pos]))
        } else {
            // If no dot found, the whole thing after "module." is the module name
            Some(format!("module.{}", module_part))
        }
    } else {
        None
    }
}
fn determine_action(actions: &[String]) -> Result<ChangeAction> {
    if actions.is_empty() {
        return Ok(ChangeAction::NoOp);
    }

    // Handle common action patterns
    let action_strs: Vec<&str> = actions.iter().map(|s| s.as_str()).collect();
    match action_strs.as_slice() {
        ["no-op"] => Ok(ChangeAction::NoOp),
        ["create"] => Ok(ChangeAction::Create),
        ["delete"] => Ok(ChangeAction::Delete),
        ["update"] => Ok(ChangeAction::Update),
        ["delete", "create"] | ["create", "delete"] => Ok(ChangeAction::Replace),
        _ => {
            // Default to update if we have multiple actions
            Ok(ChangeAction::Update)
        }
    }
}

/// Extract tags from Terraform resource configuration
fn extract_tags(config: &Option<Value>) -> HashMap<String, String> {
    let mut tags = HashMap::new();

    if let Some(Value::Object(obj)) = config {
        // Check common tag locations
        if let Some(Value::Object(tag_obj)) = obj.get("tags") {
            for (key, value) in tag_obj {
                if let Value::String(s) = value {
                    tags.insert(key.clone(), s.clone());
                }
            }
        }

        // Also check tags_all (AWS provider)
        if let Some(Value::Object(tag_obj)) = obj.get("tags_all") {
            for (key, value) in tag_obj {
                if let Value::String(s) = value {
                    tags.entry(key.clone()).or_insert_with(|| s.clone());
                }
            }
        }
    }

    tags
}

/// Handle unknown and computed values conservatively
pub fn handle_unknown_values(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), handle_unknown_values(v));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(handle_unknown_values).collect()),
        // Unknown values are represented as null in our conservative approach
        Value::String(s) if s == "(known after apply)" => Value::Null,
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_plan() {
        let plan_json = r#"{
            "format_version": "1.2",
            "terraform_version": "1.5.0",
            "resource_changes": [
                {
                    "address": "aws_instance.example",
                    "type": "aws_instance",
                    "name": "example",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {
                            "instance_type": "t3.micro",
                            "tags": {
                                "Name": "example"
                            }
                        }
                    }
                }
            ]
        }"#;

        let result = parse_terraform_plan(plan_json);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.format_version, "1.2");
    }

    #[test]
    fn test_action_determination() {
        assert_eq!(determine_action(&["create".to_string()]).unwrap(), ChangeAction::Create);
        assert_eq!(determine_action(&["delete".to_string()]).unwrap(), ChangeAction::Delete);
        assert_eq!(determine_action(&["update".to_string()]).unwrap(), ChangeAction::Update);
        assert_eq!(
            determine_action(&["delete".to_string(), "create".to_string()]).unwrap(),
            ChangeAction::Replace
        );
    }
}
