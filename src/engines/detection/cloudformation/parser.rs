// CloudFormation changeset and template parsing

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use crate::engines::shared::models::{ResourceChange, ChangeAction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CloudFormation changeset structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationChangeset {
    /// Changeset ID
    #[serde(rename = "ChangesetId")]
    pub changeset_id: String,
    /// Stack name
    #[serde(rename = "StackName")]
    pub stack_name: String,
    /// Status of the changeset
    #[serde(rename = "Status")]
    pub status: String,
    /// Description
    #[serde(rename = "Description")]
    pub description: Option<String>,
    /// Changes in the changeset
    #[serde(rename = "Changes")]
    pub changes: Vec<CloudFormationChange>,
}

/// Individual change in a CloudFormation changeset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationChange {
    /// Type of change
    #[serde(rename = "Type")]
    pub change_type: String,
    /// Resource change details
    #[serde(rename = "ResourceChange")]
    pub resource_change: CloudFormationResourceChange,
}

/// Resource change details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationResourceChange {
    /// Logical ID of the resource
    #[serde(rename = "LogicalResourceId")]
    pub logical_resource_id: String,
    /// Physical ID (if exists)
    #[serde(rename = "PhysicalResourceId")]
    pub physical_resource_id: Option<String>,
    /// Resource type
    #[serde(rename = "ResourceType")]
    pub resource_type: String,
    /// Action (Add, Modify, Remove)
    #[serde(rename = "Action")]
    pub action: String,
    /// Scope of changes
    #[serde(rename = "Scope")]
    pub scope: Option<Vec<String>>,
    /// Details of the change
    #[serde(rename = "Details")]
    pub details: Option<Vec<CloudFormationChangeDetail>>,
    /// Replacement requirement
    #[serde(rename = "Replacement")]
    pub replacement: Option<String>,
}

/// Change detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationChangeDetail {
    /// Target attribute
    #[serde(rename = "Target")]
    pub target: CloudFormationChangeTarget,
    /// Evaluation result
    #[serde(rename = "Evaluation")]
    pub evaluation: String,
    /// Change source
    #[serde(rename = "ChangeSource")]
    pub change_source: String,
    /// Causing entity
    #[serde(rename = "CausingEntity")]
    pub causing_entity: Option<String>,
}

/// Change target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationChangeTarget {
    /// Attribute name
    #[serde(rename = "Attribute")]
    pub attribute: String,
    /// Attribute requiring update
    #[serde(rename = "RequiresRecreation")]
    pub requires_recreation: Option<String>,
    /// Path to attribute
    #[serde(rename = "Path")]
    pub path: Option<String>,
}

/// CloudFormation template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationTemplate {
    /// AWS template format version
    #[serde(rename = "AWSTemplateFormatVersion")]
    pub aws_template_format_version: Option<String>,
    /// Template description
    #[serde(rename = "Description")]
    pub description: Option<String>,
    /// Template metadata
    #[serde(rename = "Metadata")]
    pub metadata: Option<serde_json::Value>,
    /// Template parameters
    #[serde(rename = "Parameters")]
    pub parameters: Option<HashMap<String, CloudFormationParameter>>,
    /// Mappings
    #[serde(rename = "Mappings")]
    pub mappings: Option<HashMap<String, serde_json::Value>>,
    /// Conditions
    #[serde(rename = "Conditions")]
    pub conditions: Option<HashMap<String, serde_json::Value>>,
    /// Transform
    #[serde(rename = "Transform")]
    pub transform: Option<serde_json::Value>,
    /// Resources
    #[serde(rename = "Resources")]
    pub resources: HashMap<String, CloudFormationResource>,
    /// Outputs
    #[serde(rename = "Outputs")]
    pub outputs: Option<HashMap<String, CloudFormationOutput>>,
}

/// CloudFormation parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationParameter {
    /// Parameter type
    #[serde(rename = "Type")]
    pub param_type: String,
    /// Default value
    #[serde(rename = "Default")]
    pub default: Option<String>,
    /// Allowed values
    #[serde(rename = "AllowedValues")]
    pub allowed_values: Option<Vec<String>>,
    /// Description
    #[serde(rename = "Description")]
    pub description: Option<String>,
}

/// CloudFormation resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationResource {
    /// Resource type
    #[serde(rename = "Type")]
    pub resource_type: String,
    /// Resource properties
    #[serde(rename = "Properties")]
    pub properties: Option<serde_json::Value>,
    /// Depends on
    #[serde(rename = "DependsOn")]
    pub depends_on: Option<serde_json::Value>,
    /// Metadata
    #[serde(rename = "Metadata")]
    pub metadata: Option<serde_json::Value>,
    /// Deletion policy
    #[serde(rename = "DeletionPolicy")]
    pub deletion_policy: Option<String>,
    /// Update replace policy
    #[serde(rename = "UpdateReplacePolicy")]
    pub update_replace_policy: Option<String>,
}

/// CloudFormation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFormationOutput {
    /// Description
    #[serde(rename = "Description")]
    pub description: Option<String>,
    /// Value
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
    /// Export name
    #[serde(rename = "Export")]
    pub export: Option<HashMap<String, String>>,
}

/// Parse CloudFormation changeset JSON
pub fn parse_cloudformation_changeset(json_content: &str) -> Result<CloudFormationChangeset> {
    serde_json::from_str(json_content).map_err(|e| {
        CostPilotError::new(
            "CF_001",
            ErrorCategory::ParseError,
            format!("Failed to parse CloudFormation changeset: {}", e),
        )
    })
}

/// Parse CloudFormation template JSON
pub fn parse_cloudformation_template(json_content: &str) -> Result<CloudFormationTemplate> {
    serde_json::from_str(json_content).map_err(|e| {
        CostPilotError::new(
            "CF_002",
            ErrorCategory::ParseError,
            format!("Failed to parse CloudFormation template: {}", e),
        )
    })
}

/// Convert CloudFormation changeset to ResourceChange format
pub fn changeset_to_resource_changes(changeset: &CloudFormationChangeset) -> Vec<ResourceChange> {
    changeset.changes.iter().filter_map(|change| {
        // Map CloudFormation action to CostPilot action
        let action = match change.resource_change.action.as_str() {
            "Add" => ChangeAction::Create,
            "Modify" => ChangeAction::Update,
            "Remove" => ChangeAction::Delete,
            _ => return None, // Skip unknown actions
        };

        // Map CloudFormation resource type to Terraform-style resource type
        let resource_type = map_cloudformation_resource_type(&change.resource_change.resource_type);

        Some(ResourceChange {
            resource_id: change.resource_change.logical_resource_id.clone(),
            resource_type,
            action,
            module_path: None, // CloudFormation doesn't have modules like Terraform
            old_config: None, // Changeset doesn't include full config
            new_config: None, // Changeset doesn't include full config
            tags: HashMap::new(), // Could be extracted from resource properties if available
            monthly_cost: None, // Will be populated by prediction engine
            cost_impact: None,
            config: None,
        })
    }).collect()
}

/// Convert CloudFormation template to ResourceChange format (for full template analysis)
pub fn template_to_resource_changes(template: &CloudFormationTemplate) -> Vec<ResourceChange> {
    template.resources.iter().map(|(logical_id, resource)| {
        // Map CloudFormation resource type to Terraform-style resource type
        let resource_type = map_cloudformation_resource_type(&resource.resource_type);

        ResourceChange {
            resource_id: logical_id.clone(),
            resource_type,
            action: ChangeAction::Create, // Templates represent desired state
            module_path: None,
            old_config: None,
            new_config: Some(resource.properties.clone().unwrap_or(serde_json::Value::Null)),
            tags: extract_tags_from_properties(&resource.properties),
            monthly_cost: None,
            cost_impact: None,
            config: Some(resource.properties.clone().unwrap_or(serde_json::Value::Null)),
        }
    }).collect()
}

/// Map CloudFormation resource type to Terraform-style resource type
pub fn map_cloudformation_resource_type(cf_type: &str) -> String {
    match cf_type {
        "AWS::EC2::Instance" => "aws_instance".to_string(),
        "AWS::EC2::LaunchTemplate" => "aws_launch_template".to_string(),
        "AWS::EC2::SecurityGroup" => "aws_security_group".to_string(),
        "AWS::EC2::Subnet" => "aws_subnet".to_string(),
        "AWS::EC2::VPC" => "aws_vpc".to_string(),
        "AWS::EC2::InternetGateway" => "aws_internet_gateway".to_string(),
        "AWS::EC2::NatGateway" => "aws_nat_gateway".to_string(),
        "AWS::EC2::RouteTable" => "aws_route_table".to_string(),
        "AWS::EC2::NetworkAcl" => "aws_network_acl".to_string(),
        "AWS::RDS::DBInstance" => "aws_db_instance".to_string(),
        "AWS::RDS::DBCluster" => "aws_rds_cluster".to_string(),
        "AWS::RDS::DBSubnetGroup" => "aws_db_subnet_group".to_string(),
        "AWS::Lambda::Function" => "aws_lambda_function".to_string(),
        "AWS::S3::Bucket" => "aws_s3_bucket".to_string(),
        "AWS::S3::BucketPolicy" => "aws_s3_bucket_policy".to_string(),
        "AWS::IAM::Role" => "aws_iam_role".to_string(),
        "AWS::IAM::Policy" => "aws_iam_policy".to_string(),
        "AWS::IAM::InstanceProfile" => "aws_iam_instance_profile".to_string(),
        "AWS::ELB::LoadBalancer" => "aws_elb".to_string(),
        "AWS::ELBv2::LoadBalancer" => "aws_lb".to_string(),
        "AWS::ELBv2::TargetGroup" => "aws_lb_target_group".to_string(),
        "AWS::ELBv2::Listener" => "aws_lb_listener".to_string(),
        "AWS::CloudWatch::Alarm" => "aws_cloudwatch_metric_alarm".to_string(),
        "AWS::CloudWatch::LogGroup" => "aws_cloudwatch_log_group".to_string(),
        "AWS::SNS::Topic" => "aws_sns_topic".to_string(),
        "AWS::SQS::Queue" => "aws_sqs_queue".to_string(),
        "AWS::DynamoDB::Table" => "aws_dynamodb_table".to_string(),
        "AWS::Kinesis::Stream" => "aws_kinesis_stream".to_string(),
        "AWS::ApiGateway::RestApi" => "aws_api_gateway_rest_api".to_string(),
        "AWS::ApiGateway::Resource" => "aws_api_gateway_resource".to_string(),
        "AWS::ApiGateway::Method" => "aws_api_gateway_method".to_string(),
        "AWS::ApiGateway::Deployment" => "aws_api_gateway_deployment".to_string(),
        "AWS::CloudFront::Distribution" => "aws_cloudfront_distribution".to_string(),
        "AWS::Route53::HostedZone" => "aws_route53_zone".to_string(),
        "AWS::Route53::RecordSet" => "aws_route53_record".to_string(),
        "AWS::EFS::FileSystem" => "aws_efs_file_system".to_string(),
        "AWS::EFS::MountTarget" => "aws_efs_mount_target".to_string(),
        "AWS::EKS::Cluster" => "aws_eks_cluster".to_string(),
        "AWS::EKS::Nodegroup" => "aws_eks_nodegroup".to_string(),
        "AWS::ECS::Cluster" => "aws_ecs_cluster".to_string(),
        "AWS::ECS::Service" => "aws_ecs_service".to_string(),
        "AWS::ECS::TaskDefinition" => "aws_ecs_task_definition".to_string(),
        // Add more mappings as needed
        _ => {
            // For unknown types, create a generic mapping
            format!("aws_{}", cf_type.split("::").last().unwrap_or("unknown").to_lowercase())
        }
    }
}

/// Extract tags from CloudFormation resource properties
fn extract_tags_from_properties(properties: &Option<serde_json::Value>) -> HashMap<String, String> {
    let mut tags = HashMap::new();

    if let Some(props) = properties {
        if let Some(tags_value) = props.get("Tags") {
            if let Some(tags_array) = tags_value.as_array() {
                for tag in tags_array {
                    if let (Some(key), Some(value)) = (
                        tag.get("Key").and_then(|k| k.as_str()),
                        tag.get("Value").and_then(|v| v.as_str())
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
    fn test_parse_cloudformation_changeset() {
        let changeset_json = r#"{
            "ChangesetId": "test-changeset",
            "StackName": "test-stack",
            "Status": "CREATE_COMPLETE",
            "Changes": [
                {
                    "Type": "Resource",
                    "ResourceChange": {
                        "LogicalResourceId": "MyInstance",
                        "ResourceType": "AWS::EC2::Instance",
                        "Action": "Add"
                    }
                }
            ]
        }"#;

        let changeset = parse_cloudformation_changeset(changeset_json).unwrap();
        assert_eq!(changeset.changeset_id, "test-changeset");
        assert_eq!(changeset.stack_name, "test-stack");
        assert_eq!(changeset.changes.len(), 1);
    }

    #[test]
    fn test_changeset_to_resource_changes() {
        let changeset = CloudFormationChangeset {
            changeset_id: "test".to_string(),
            stack_name: "test-stack".to_string(),
            status: "CREATE_COMPLETE".to_string(),
            description: None,
            changes: vec![CloudFormationChange {
                change_type: "Resource".to_string(),
                resource_change: CloudFormationResourceChange {
                    logical_resource_id: "MyInstance".to_string(),
                    physical_resource_id: None,
                    resource_type: "AWS::EC2::Instance".to_string(),
                    action: "Add".to_string(),
                    scope: None,
                    details: None,
                    replacement: None,
                },
            }],
        };

        let changes = changeset_to_resource_changes(&changeset);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].resource_id, "MyInstance");
        assert_eq!(changes[0].resource_type, "aws_instance");
        assert_eq!(changes[0].action, ChangeAction::Create);
    }

    #[test]
    fn test_map_cloudformation_resource_type() {
        assert_eq!(map_cloudformation_resource_type("AWS::EC2::Instance"), "aws_instance");
        assert_eq!(map_cloudformation_resource_type("AWS::S3::Bucket"), "aws_s3_bucket");
        assert_eq!(map_cloudformation_resource_type("AWS::Lambda::Function"), "aws_lambda_function");
        assert_eq!(map_cloudformation_resource_type("AWS::Unknown::Type"), "aws_type");
    }
}