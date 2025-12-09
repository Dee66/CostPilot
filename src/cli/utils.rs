// CLI utility functions

use crate::engines::shared::models::ResourceChange;
use serde_json::Value;

/// Extract resource changes from Terraform plan
pub fn extract_resource_changes(plan: &Value) -> Result<Vec<ResourceChange>, Box<dyn std::error::Error>> {
    let mut changes = Vec::new();
    
    // Extract resource_changes array from plan
    let resource_changes = plan
        .get("resource_changes")
        .and_then(|v| v.as_array())
        .ok_or("No resource_changes found in plan")?;

    for resource in resource_changes {
        if let Ok(change) = parse_resource_change(resource) {
            changes.push(change);
        }
    }

    Ok(changes)
}

fn parse_resource_change(resource: &Value) -> Result<ResourceChange, Box<dyn std::error::Error>> {
    use crate::engines::shared::models::ChangeAction;
    use std::collections::HashMap;

    let address = resource
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or("Missing address")?
        .to_string();

    let resource_type = resource
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("Missing type")?
        .to_string();

    let change_obj = resource
        .get("change")
        .ok_or("Missing change object")?;

    let actions = change_obj
        .get("actions")
        .and_then(|v| v.as_array())
        .ok_or("Missing actions")?;

    let action = if actions.contains(&Value::String("create".to_string())) {
        ChangeAction::Create
    } else if actions.contains(&Value::String("delete".to_string())) {
        ChangeAction::Delete
    } else if actions.contains(&Value::String("update".to_string())) {
        ChangeAction::Update
    } else {
        ChangeAction::NoOp
    };

    let before = change_obj.get("before").cloned();
    let after = change_obj.get("after").cloned();

    // Extract module path
    let module_path = if address.starts_with("module.") {
        let parts: Vec<&str> = address.split('.').collect();
        if parts.len() >= 2 {
            Some(parts[1].to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok(ResourceChange {
        resource_id: address,
        resource_type,
        action,
        module_path,
        old_config: before,
        new_config: after,
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
    })
}
