// CDK diff JSON parsing

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use crate::engines::shared::models::{ChangeAction, ResourceChange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CDK diff structure (output from `cdk diff --json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkDiff {
    /// Success status
    pub success: bool,
    /// List of stacks with changes
    pub stacks: Vec<CdkStackDiff>,
    /// Error message if any
    pub error: Option<String>,
}

/// Stack diff in CDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkStackDiff {
    /// Stack name
    pub stack_name: String,
    /// Stack path
    pub stack_path: Option<String>,
    /// Changes in this stack
    pub changes: Vec<CdkResourceChange>,
}

/// Resource change in CDK diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkResourceChange {
    /// Resource logical ID
    pub logical_id: String,
    /// Resource type
    pub resource_type: String,
    /// Change type
    pub change_type: CdkChangeType,
    /// Impact level
    pub impact: Option<String>,
    /// Property changes
    pub property_changes: Option<Vec<CdkPropertyChange>>,
    /// Old values
    pub old_values: Option<serde_json::Value>,
    /// New values
    pub new_values: Option<serde_json::Value>,
}

/// Type of change in CDK
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CdkChangeType {
    Create,
    Update,
    Delete,
}

/// Property change details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkPropertyChange {
    /// Property path
    pub property_path: String,
    /// Old value
    pub old_value: Option<serde_json::Value>,
    /// New value
    pub new_value: Option<serde_json::Value>,
    /// Change impact
    pub change_impact: Option<String>,
}

/// CDK synthesized template structure (from cdk.out/)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkSynthesizedTemplate {
    /// Resources
    #[serde(rename = "Resources")]
    pub resources: Option<HashMap<String, CdkResource>>,
    /// Outputs
    #[serde(rename = "Outputs")]
    pub outputs: Option<HashMap<String, serde_json::Value>>,
    /// Parameters
    #[serde(rename = "Parameters")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

/// CDK resource in synthesized template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkResource {
    /// Resource type
    #[serde(rename = "Type")]
    pub resource_type: String,
    /// Properties
    #[serde(rename = "Properties")]
    pub properties: Option<serde_json::Value>,
    /// Depends on
    #[serde(rename = "DependsOn")]
    pub depends_on: Option<serde_json::Value>,
    /// Metadata
    #[serde(rename = "Metadata")]
    pub metadata: Option<serde_json::Value>,
}

/// Parse CDK diff JSON output
pub fn parse_cdk_diff(json_content: &str) -> Result<CdkDiff> {
    serde_json::from_str(json_content).map_err(|e| {
        CostPilotError::new(
            "CDK_001",
            ErrorCategory::ParseError,
            format!("Failed to parse CDK diff JSON: {}", e),
        )
    })
}

/// Parse CDK synthesized template JSON
pub fn parse_cdk_template(json_content: &str) -> Result<CdkSynthesizedTemplate> {
    serde_json::from_str(json_content).map_err(|e| {
        CostPilotError::new(
            "CDK_002",
            ErrorCategory::ParseError,
            format!("Failed to parse CDK synthesized template: {}", e),
        )
    })
}

/// Convert CDK diff to ResourceChange format
pub fn cdk_diff_to_resource_changes(diff: &CdkDiff) -> Vec<ResourceChange> {
    let mut changes = Vec::new();

    for stack in &diff.stacks {
        for change in &stack.changes {
            // Map CDK change type to CostPilot action
            let action = match change.change_type {
                CdkChangeType::Create => ChangeAction::Create,
                CdkChangeType::Update => ChangeAction::Update,
                CdkChangeType::Delete => ChangeAction::Delete,
            };

            // Map CDK resource type to Terraform-style resource type
            let resource_type = map_cdk_resource_type(&change.resource_type);

            changes.push(ResourceChange {
                resource_id: change.logical_id.clone(),
                resource_type,
                action,
                module_path: stack.stack_path.clone(),
                old_config: change.old_values.clone(),
                new_config: change.new_values.clone(),
                tags: extract_tags_from_cdk_properties(&change.new_values),
                monthly_cost: None, // Will be populated by prediction engine
                cost_impact: None,
                config: change.new_values.clone(),
            });
        }
    }

    changes
}

/// Convert CDK synthesized template to ResourceChange format
pub fn cdk_template_to_resource_changes(
    template: &CdkSynthesizedTemplate,
    stack_name: &str,
) -> Vec<ResourceChange> {
    let mut changes = Vec::new();

    if let Some(resources) = &template.resources {
        for (logical_id, resource) in resources {
            // Map CDK resource type to Terraform-style resource type
            let resource_type = map_cdk_resource_type(&resource.resource_type);

            changes.push(ResourceChange {
                resource_id: logical_id.clone(),
                resource_type,
                action: ChangeAction::Create, // Templates represent desired state
                module_path: Some(stack_name.to_string()),
                old_config: None,
                new_config: resource.properties.clone(),
                tags: extract_tags_from_cdk_properties(&resource.properties),
                monthly_cost: None,
                cost_impact: None,
                config: resource.properties.clone(),
            });
        }
    }

    changes
}

/// Map CDK resource type to Terraform-style resource type
/// CDK uses CloudFormation resource types, so we convert AWS::Service::Resource to aws_service_resource
fn map_cdk_resource_type(cdk_type: &str) -> String {
    // Convert CloudFormation resource type to Terraform-style resource type
    // AWS::S3::Bucket -> aws_s3_bucket
    if cdk_type.starts_with("AWS::") {
        let parts: Vec<&str> = cdk_type.split("::").collect();
        if parts.len() == 3 {
            let service = parts[1].to_lowercase();
            let resource = parts[2].to_lowercase();
            return format!("aws_{}_{}", service, resource);
        }
    }
    // Fallback: return as-is if not a standard AWS resource type
    cdk_type.to_string()
}

/// Extract tags from CDK resource properties
fn extract_tags_from_cdk_properties(
    properties: &Option<serde_json::Value>,
) -> HashMap<String, String> {
    let mut tags = HashMap::new();

    if let Some(props) = properties {
        if let Some(tags_value) = props.get("Tags") {
            if let Some(tags_array) = tags_value.as_array() {
                for tag in tags_array {
                    if let (Some(key), Some(value)) = (
                        tag.get("Key").and_then(|k| k.as_str()),
                        tag.get("Value").and_then(|v| v.as_str()),
                    ) {
                        tags.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cdk_diff() {
        let diff_json = r#"{
            "success": true,
            "stacks": [
                {
                    "stack_name": "TestStack",
                    "changes": [
                        {
                            "logical_id": "MyBucket",
                            "resource_type": "AWS::S3::Bucket",
                            "change_type": "create",
                            "impact": "WILL_CREATE"
                        }
                    ]
                }
            ]
        }"#;

        let diff = parse_cdk_diff(diff_json).unwrap();
        assert!(diff.success);
        assert_eq!(diff.stacks.len(), 1);
        assert_eq!(diff.stacks[0].stack_name, "TestStack");
        assert_eq!(diff.stacks[0].changes.len(), 1);
    }

    #[test]
    fn test_cdk_diff_to_resource_changes() {
        let diff = CdkDiff {
            success: true,
            stacks: vec![CdkStackDiff {
                stack_name: "TestStack".to_string(),
                stack_path: Some("TestStack".to_string()),
                changes: vec![CdkResourceChange {
                    logical_id: "MyBucket".to_string(),
                    resource_type: "AWS::S3::Bucket".to_string(),
                    change_type: CdkChangeType::Create,
                    impact: Some("WILL_CREATE".to_string()),
                    property_changes: None,
                    old_values: None,
                    new_values: None,
                }],
            }],
            error: None,
        };

        let changes = cdk_diff_to_resource_changes(&diff);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].resource_id, "MyBucket");
        assert_eq!(changes[0].resource_type, "aws_s3_bucket");
        assert_eq!(changes[0].action, ChangeAction::Create);
        assert_eq!(changes[0].module_path, Some("TestStack".to_string()));
    }

    #[test]
    fn test_map_cdk_resource_type() {
        assert_eq!(map_cdk_resource_type("AWS::S3::Bucket"), "aws_s3_bucket");
        assert_eq!(
            map_cdk_resource_type("AWS::Lambda::Function"),
            "aws_lambda_function"
        );
    }
}
