use super::artifact_types::*;
use serde_json::Value;
use std::path::Path;
use std::collections::HashMap;

/// CloudFormation template structure (subset needed for CDK)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct CloudFormationTemplate {
    /// Resources
    #[serde(default)]
    pub resources: HashMap<String, CloudFormationResource>,
}

/// CloudFormation resource
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct CloudFormationResource {
    /// Resource type
    #[serde(rename = "Type")]
    pub resource_type: String,
    /// Resource properties
    #[serde(rename = "Properties")]
    pub properties: Option<serde_json::Value>,
}

/// Parse CloudFormation template JSON
fn parse_cloudformation_template(json_content: &str) -> ArtifactResult<CloudFormationTemplate> {
    serde_json::from_str(json_content).map_err(|e| {
        ArtifactError::ParseError(format!("Failed to parse CloudFormation template: {}", e))
    })
}

/// Map CloudFormation resource type to Terraform-style resource type
fn map_cloudformation_resource_type(cf_type: &str) -> String {
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

/// Parser for AWS CDK synthesized output
pub struct CdkParser;

impl CdkParser {
    /// Create a new CDK parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a CloudFormation template file and return an Artifact
    fn parse_cloudformation_template_file(&self, template_path: &str) -> ArtifactResult<Artifact> {
        let content = std::fs::read_to_string(template_path)
            .map_err(|e| ArtifactError::IoError(format!("Failed to read template file: {}", e)))?;

        let template: CloudFormationTemplate = parse_cloudformation_template(&content)?;

        // Convert template resources to artifact resources
        let mut resources = Vec::new();
        for (logical_id, resource) in &template.resources {
            let resource_type = map_cloudformation_resource_type(&resource.resource_type);
            let properties = resource.properties.as_ref()
                .and_then(|p| p.as_object())
                .map(|m| m.clone().into_iter().collect())
                .unwrap_or_default();
            let mut metadata = HashMap::new();
            
            // Extract tags into metadata
            if let Some(props) = &resource.properties {
                if let Some(tags_value) = props.get("Tags") {
                    if let Some(tags_array) = tags_value.as_array() {
                        for tag in tags_array {
                            if let (Some(key), Some(value)) = (
                                tag.get("Key").and_then(|k| k.as_str()),
                                tag.get("Value").and_then(|v| v.as_str())
                            ) {
                                metadata.insert(format!("tag:{}", key), value.to_string());
                            }
                        }
                    }
                }
            }

            resources.push(ArtifactResource {
                id: logical_id.clone(),
                resource_type: resource.resource_type.clone(), // Keep original CFN type
                properties,
                depends_on: Vec::new(),
                metadata,
            });
        }

        Ok(Artifact {
            format: ArtifactFormat::Cdk, // Will be overridden to CDK
            metadata: ArtifactMetadata {
                source: template_path.to_string(),
                version: None,
                stack_name: None,
                region: None,
                tags: HashMap::new(),
            },
            resources,
            outputs: HashMap::new(),
            parameters: HashMap::new(),
        })
    }

    /// Parse CDK output directory (cdk.out/)
    pub fn parse_cdk_output(&self, output_dir: &str) -> ArtifactResult<Vec<Artifact>> {
        let manifest_path = format!("{}/manifest.json", output_dir);

        // Read CDK manifest
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| ArtifactError::IoError(format!("Failed to read CDK manifest: {}", e)))?;

        let manifest: Value = serde_json::from_str(&manifest_content)?;

        // Extract artifacts from manifest
        let mut artifacts = Vec::new();

        if let Some(artifacts_obj) = manifest.get("artifacts").and_then(|v| v.as_object()) {
            for (artifact_id, artifact_def) in artifacts_obj {
                if let Some(artifact_type) = artifact_def.get("type").and_then(|v| v.as_str()) {
                    if artifact_type == "aws:cloudformation:stack" {
                        if let Ok(artifact) =
                            self.parse_cdk_stack(output_dir, artifact_id, artifact_def)
                        {
                            artifacts.push(artifact);
                        }
                    }
                }
            }
        }

        Ok(artifacts)
    }

    /// Parse a single CDK stack from the output
    fn parse_cdk_stack(
        &self,
        output_dir: &str,
        stack_id: &str,
        stack_def: &Value,
    ) -> ArtifactResult<Artifact> {
        // Get template file path
        let template_file = stack_def
            .get("properties")
            .and_then(|p| p.get("templateFile"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                ArtifactError::MissingField("CDK stack missing templateFile".to_string())
            })?;

        let template_path = format!("{}/{}", output_dir, template_file);

        // Parse the CloudFormation template
        let mut artifact = self.parse_cloudformation_template_file(&template_path)?;

        // Override format to CDK
        artifact.format = ArtifactFormat::Cdk;

        // Enhance metadata with CDK-specific info
        artifact.metadata.stack_name = Some(stack_id.to_string());
        artifact
            .metadata
            .tags
            .insert("cdk_stack_id".to_string(), stack_id.to_string());

        // Extract stack tags if present
        if let Some(tags_obj) = stack_def
            .get("properties")
            .and_then(|p| p.get("tags"))
            .and_then(|t| t.as_object())
        {
            for (key, value) in tags_obj {
                if let Some(v) = value.as_str() {
                    artifact
                        .metadata
                        .tags
                        .insert(format!("cdk_tag_{}", key), v.to_string());
                }
            }
        }

        // Extract environment info
        if let Some(env) = stack_def.get("environment").and_then(|e| e.as_str()) {
            // Environment format: aws://account/region
            if let Some(region) = env.split('/').next_back() {
                artifact.metadata.region = Some(region.to_string());
            }
        }

        // Extract CDK metadata from resources
        self.enhance_with_cdk_metadata(&mut artifact);

        Ok(artifact)
    }

    /// Enhance artifact with CDK-specific metadata from resources
    fn enhance_with_cdk_metadata(&self, artifact: &mut Artifact) {
        for resource in &mut artifact.resources {
            // CDK adds metadata to resources
            if let Some(cdk_path) = resource.metadata.get("aws:cdk:path") {
                resource
                    .metadata
                    .insert("cdk_construct_path".to_string(), cdk_path.clone());
            }

            // Extract logical ID mapping
            if let Some(logical_id) = resource.metadata.get("aws:cdk:logicalId") {
                resource
                    .metadata
                    .insert("original_logical_id".to_string(), logical_id.clone());
            }
        }
    }

    /// Parse CDK assembly metadata
    pub fn parse_assembly_metadata(&self, output_dir: &str) -> ArtifactResult<CdkAssemblyMetadata> {
        let manifest_path = format!("{}/manifest.json", output_dir);
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: Value = serde_json::from_str(&manifest_content)?;

        let version = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let runtime = manifest
            .get("runtime")
            .and_then(|r| r.as_object())
            .and_then(|r| r.get("libraries"))
            .and_then(|l| l.as_object())
            .map(|libs| {
                libs.iter()
                    .map(|(k, v)| format!("{}@{}", k, v.as_str().unwrap_or("?")))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "unknown".to_string());

        Ok(CdkAssemblyMetadata { version, runtime })
    }
}

impl Default for CdkParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactParser for CdkParser {
    fn parse(&self, content: &str) -> ArtifactResult<Artifact> {
        // For CDK, we expect a CloudFormation template
        // (CDK synthesizes to CFN)
        let template: CloudFormationTemplate = parse_cloudformation_template(content)?;
        
        // Convert template resources to artifact resources
        let mut resources = Vec::new();
        for (logical_id, resource) in &template.resources {
            let resource_type = map_cloudformation_resource_type(&resource.resource_type);
            let properties = resource.properties.as_ref()
                .and_then(|p| p.as_object())
                .map(|m| m.clone().into_iter().collect())
                .unwrap_or_default();
            let mut metadata = HashMap::new();
            
            // Extract tags into metadata
            if let Some(props) = &resource.properties {
                if let Some(tags_value) = props.get("Tags") {
                    if let Some(tags_array) = tags_value.as_array() {
                        for tag in tags_array {
                            if let (Some(key), Some(value)) = (
                                tag.get("Key").and_then(|k| k.as_str()),
                                tag.get("Value").and_then(|v| v.as_str())
                            ) {
                                metadata.insert(format!("tag:{}", key), value.to_string());
                            }
                        }
                    }
                }
            }

            resources.push(ArtifactResource {
                id: logical_id.clone(),
                resource_type: resource.resource_type.clone(), // Keep original CFN type
                properties,
                depends_on: Vec::new(),
                metadata,
            });
        }

        Ok(Artifact {
            format: ArtifactFormat::Cdk,
            metadata: ArtifactMetadata {
                source: "cdk-template".to_string(),
                version: None,
                stack_name: None,
                region: None,
                tags: HashMap::new(),
            },
            resources,
            outputs: HashMap::new(),
            parameters: HashMap::new(),
        })
    }

    fn format(&self) -> ArtifactFormat {
        ArtifactFormat::Cdk
    }
}

/// CDK assembly metadata
#[derive(Debug, Clone)]
pub struct CdkAssemblyMetadata {
    pub version: String,
    pub runtime: String,
}

/// Detect if a directory is a CDK output directory
pub fn is_cdk_output_dir(path: &str) -> bool {
    let manifest_path = format!("{}/manifest.json", path);
    Path::new(&manifest_path).exists()
}

/// Find all stack template files in CDK output
pub fn find_cdk_templates(output_dir: &str) -> std::io::Result<Vec<String>> {
    let mut templates = Vec::new();

    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".template.json") || filename.ends_with(".template.yaml") {
                    if let Some(path_str) = path.to_str() {
                        templates.push(path_str.to_string());
                    }
                }
            }
        }
    }

    Ok(templates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cdk_synthesized_template() {
        // CDK outputs standard CloudFormation templates
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Description": "CDK Stack",
            "Resources": {
                "MyFunction": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "FunctionName": "my-function",
                        "Runtime": "nodejs18.x",
                        "Handler": "index.handler"
                    },
                    "Metadata": {
                        "aws:cdk:path": "MyStack/MyFunction/Resource",
                        "aws:asset:path": "asset.1234567890"
                    }
                }
            }
        }"#;

        let parser = CdkParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.format, ArtifactFormat::Cdk);
        assert_eq!(artifact.resource_count(), 1);

        let resource = artifact.get_resource("MyFunction").unwrap();
        assert_eq!(resource.resource_type, "AWS::Lambda::Function");
    }

    #[test]
    fn test_parse_cdk_with_nested_stacks() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "NestedStack": {
                    "Type": "AWS::CloudFormation::Stack",
                    "Properties": {
                        "TemplateURL": "https://s3.amazonaws.com/bucket/nested.template.json"
                    },
                    "Metadata": {
                        "aws:cdk:path": "MyStack/NestedStack"
                    }
                }
            }
        }"#;

        let parser = CdkParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.format, ArtifactFormat::Cdk);

        let nested = artifact.get_resource("NestedStack").unwrap();
        assert_eq!(nested.resource_type, "AWS::CloudFormation::Stack");
    }

    #[test]
    fn test_is_cdk_output_dir() {
        // This will fail in test env since we don't have actual CDK output
        // In real usage, it checks for manifest.json
        assert!(!is_cdk_output_dir("/nonexistent/path"));
    }

    #[test]
    fn test_enhance_metadata() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "MyBucket": {
                    "Type": "AWS::S3::Bucket",
                    "Properties": {},
                    "Metadata": {
                        "aws:cdk:path": "MyStack/MyBucket/Resource",
                        "aws:cdk:logicalId": "MyBucketF68F3FF0"
                    }
                }
            }
        }"#;

        let parser = CdkParser::new();
        let mut artifact = parser.parse(template).unwrap();

        parser.enhance_with_cdk_metadata(&mut artifact);

        let resource = artifact.get_resource("MyBucket").unwrap();
        assert!(resource.metadata.contains_key("cdk_construct_path"));
    }

    #[test]
    fn test_cdk_lambda_with_assets() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "MyFunction": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Code": {
                            "S3Bucket": {"Ref": "AssetParameters1234"},
                            "S3Key": "asset.zip"
                        },
                        "Runtime": "python3.9"
                    },
                    "Metadata": {
                        "aws:cdk:path": "MyStack/MyFunction",
                        "aws:asset:path": "lambda"
                    }
                }
            }
        }"#;

        let parser = CdkParser::new();
        let artifact = parser.parse(template).unwrap();

        let function = artifact.get_resource("MyFunction").unwrap();
        assert!(function.has_property("Code"));
        assert!(function.has_property("Runtime"));
    }

    #[test]
    fn test_cdk_with_constructs() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "Table": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "TableName": "my-table",
                        "BillingMode": "PAY_PER_REQUEST",
                        "AttributeDefinitions": [
                            {"AttributeName": "id", "AttributeType": "S"}
                        ],
                        "KeySchema": [
                            {"AttributeName": "id", "KeyType": "HASH"}
                        ]
                    },
                    "Metadata": {
                        "aws:cdk:path": "MyStack/Table/Resource"
                    }
                },
                "TableReadScaling": {
                    "Type": "AWS::ApplicationAutoScaling::ScalableTarget",
                    "DependsOn": "Table",
                    "Properties": {
                        "ServiceNamespace": "dynamodb",
                        "ResourceId": {"Fn::Join": ["", ["table/", {"Ref": "Table"}]]}
                    }
                }
            }
        }"#;

        let parser = CdkParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.resource_count(), 2);

        let scaling = artifact.get_resource("TableReadScaling").unwrap();
        assert_eq!(scaling.depends_on.len(), 1);
        assert_eq!(scaling.depends_on[0], "Table");
    }
}
