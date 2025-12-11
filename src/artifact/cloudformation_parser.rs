use super::artifact_types::*;
use serde_json::Value;
use std::collections::HashMap;

/// Parser for AWS CloudFormation templates (JSON only)
pub struct CloudFormationParser {}

impl CloudFormationParser {
    /// Create a new CloudFormation parser
    pub fn new() -> Self {
        Self {}
    }

    /// Parse CloudFormation template from JSON
    fn parse_json(&self, content: &str) -> ArtifactResult<Artifact> {
        let template: Value = serde_json::from_str(content)?;
        self.parse_template(template, "json")
    }

    /// Parse the template JSON/YAML structure
    fn parse_template(&self, template: Value, source_format: &str) -> ArtifactResult<Artifact> {
        let obj = template
            .as_object()
            .ok_or_else(|| ArtifactError::ParseError("Template must be an object".to_string()))?;

        // Extract metadata
        let metadata = self.extract_metadata(obj, source_format)?;

        // Create artifact
        let mut artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);

        // Parse resources
        if let Some(resources) = obj.get("Resources").and_then(|v| v.as_object()) {
            for (logical_id, resource_def) in resources {
                let resource = self.parse_resource(logical_id, resource_def)?;
                artifact.add_resource(resource);
            }
        }

        // Parse outputs
        if let Some(outputs) = obj.get("Outputs").and_then(|v| v.as_object()) {
            for (name, output_def) in outputs {
                let output = self.parse_output(output_def)?;
                artifact.outputs.insert(name.clone(), output);
            }
        }

        // Parse parameters
        if let Some(parameters) = obj.get("Parameters").and_then(|v| v.as_object()) {
            for (name, param_def) in parameters {
                let parameter = self.parse_parameter(param_def)?;
                artifact.parameters.insert(name.clone(), parameter);
            }
        }

        // Validate artifact
        artifact.validate()?;

        Ok(artifact)
    }

    /// Extract artifact metadata from template
    fn extract_metadata(
        &self,
        template: &serde_json::Map<String, Value>,
        source_format: &str,
    ) -> ArtifactResult<ArtifactMetadata> {
        let version = template
            .get("AWSTemplateFormatVersion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Validate version if present
        if let Some(ref v) = version {
            if v != "2010-09-09" {
                return Err(ArtifactError::InvalidVersion(v.clone()));
            }
        }

        let description = template
            .get("Description")
            .and_then(|v| v.as_str())
            .unwrap_or("CloudFormation Stack");

        let mut tags = HashMap::new();
        tags.insert("format".to_string(), source_format.to_string());

        if let Some(desc) = template.get("Description").and_then(|v| v.as_str()) {
            tags.insert("description".to_string(), desc.to_string());
        }

        Ok(ArtifactMetadata {
            source: description.to_string(),
            version,
            stack_name: None, // Set externally if known
            region: None,     // Set externally if known
            tags,
        })
    }

    /// Parse a single resource definition
    fn parse_resource(
        &self,
        logical_id: &str,
        resource_def: &Value,
    ) -> ArtifactResult<ArtifactResource> {
        let obj = resource_def.as_object().ok_or_else(|| {
            ArtifactError::InvalidResource(format!("Resource {} must be an object", logical_id))
        })?;

        // Extract resource type
        let resource_type = obj
            .get("Type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ArtifactError::MissingField(format!("Resource {} missing Type", logical_id))
            })?
            .to_string();

        // Extract properties
        let mut properties = HashMap::new();
        if let Some(props) = obj.get("Properties").and_then(|v| v.as_object()) {
            for (key, value) in props {
                properties.insert(key.clone(), value.clone());
            }
        }

        // Extract dependencies
        let mut depends_on = Vec::new();
        if let Some(deps) = obj.get("DependsOn") {
            match deps {
                Value::String(s) => depends_on.push(s.clone()),
                Value::Array(arr) => {
                    for dep in arr {
                        if let Some(s) = dep.as_str() {
                            depends_on.push(s.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(meta) = obj.get("Metadata").and_then(|v| v.as_object()) {
            for (key, value) in meta {
                if let Some(s) = value.as_str() {
                    metadata.insert(key.clone(), s.to_string());
                } else {
                    metadata.insert(key.clone(), value.to_string());
                }
            }
        }

        // Extract condition if present
        if let Some(condition) = obj.get("Condition").and_then(|v| v.as_str()) {
            metadata.insert("Condition".to_string(), condition.to_string());
        }

        Ok(ArtifactResource {
            id: logical_id.to_string(),
            resource_type,
            properties,
            depends_on,
            metadata,
        })
    }

    /// Parse an output definition
    fn parse_output(&self, output_def: &Value) -> ArtifactResult<ArtifactOutput> {
        let obj = output_def
            .as_object()
            .ok_or_else(|| ArtifactError::ParseError("Output must be an object".to_string()))?;

        let value = obj
            .get("Value")
            .ok_or_else(|| ArtifactError::MissingField("Output missing Value".to_string()))?
            .clone();

        let description = obj
            .get("Description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let export = obj.get("Export").is_some();

        Ok(ArtifactOutput {
            value,
            description,
            export,
        })
    }

    /// Parse a parameter definition
    fn parse_parameter(&self, param_def: &Value) -> ArtifactResult<ArtifactParameter> {
        let obj = param_def
            .as_object()
            .ok_or_else(|| ArtifactError::ParseError("Parameter must be an object".to_string()))?;

        let param_type = obj
            .get("Type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArtifactError::MissingField("Parameter missing Type".to_string()))?
            .to_string();

        let default = obj.get("Default").cloned();

        let description = obj
            .get("Description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let allowed_values = obj
            .get("AllowedValues")
            .and_then(|v| v.as_array()).cloned()
            .unwrap_or_default();

        Ok(ArtifactParameter {
            param_type,
            default,
            description,
            allowed_values,
        })
    }
}

impl Default for CloudFormationParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactParser for CloudFormationParser {
    fn parse(&self, content: &str) -> ArtifactResult<Artifact> {
        // Try JSON first
        if let Ok(artifact) = self.parse_json(content) {
            return Ok(artifact);
        }

        Err(ArtifactError::ParseError(
            "Failed to parse as JSON. YAML not supported.".to_string(),
        ))
    }

    fn format(&self) -> ArtifactFormat {
        ArtifactFormat::CloudFormation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Description": "Simple EC2 instance",
            "Resources": {
                "MyInstance": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {
                        "InstanceType": "t3.micro",
                        "ImageId": "ami-12345678"
                    }
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.format, ArtifactFormat::CloudFormation);
        assert_eq!(artifact.resource_count(), 1);

        let resource = artifact.get_resource("MyInstance").unwrap();
        assert_eq!(resource.resource_type, "AWS::EC2::Instance");
        assert_eq!(resource.normalized_type(), "aws_ec2_instance");
    }

    #[test]
    fn test_parse_with_dependencies() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "MyVPC": {
                    "Type": "AWS::EC2::VPC",
                    "Properties": {
                        "CidrBlock": "10.0.0.0/16"
                    }
                },
                "MySubnet": {
                    "Type": "AWS::EC2::Subnet",
                    "DependsOn": "MyVPC",
                    "Properties": {
                        "VpcId": {"Ref": "MyVPC"},
                        "CidrBlock": "10.0.1.0/24"
                    }
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.resource_count(), 2);

        let subnet = artifact.get_resource("MySubnet").unwrap();
        assert_eq!(subnet.depends_on.len(), 1);
        assert_eq!(subnet.depends_on[0], "MyVPC");
    }

    #[test]
    fn test_parse_with_parameters() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Parameters": {
                "InstanceType": {
                    "Type": "String",
                    "Default": "t3.micro",
                    "Description": "EC2 instance type",
                    "AllowedValues": ["t3.micro", "t3.small", "t3.medium"]
                }
            },
            "Resources": {
                "MyInstance": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {
                        "InstanceType": {"Ref": "InstanceType"}
                    }
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.parameters.len(), 1);

        let param = artifact.parameters.get("InstanceType").unwrap();
        assert_eq!(param.param_type, "String");
        assert_eq!(param.allowed_values.len(), 3);
    }

    #[test]
    fn test_parse_with_outputs() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "MyInstance": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {
                        "InstanceType": "t3.micro"
                    }
                }
            },
            "Outputs": {
                "InstanceId": {
                    "Description": "Instance ID",
                    "Value": {"Ref": "MyInstance"},
                    "Export": {
                        "Name": "MyInstanceId"
                    }
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.outputs.len(), 1);

        let output = artifact.outputs.get("InstanceId").unwrap();
        assert!(output.description.is_some());
        assert!(output.export);
    }

    #[test]
    fn test_parse_invalid_version() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2020-01-01",
            "Resources": {}
        }"#;

        let parser = CloudFormationParser::new();
        let result = parser.parse(template);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_type() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "Invalid": {
                    "Properties": {}
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let result = parser.parse(template);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_resources() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "Instance1": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {"InstanceType": "t3.micro"}
                },
                "Instance2": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {"InstanceType": "t3.small"}
                },
                "Bucket": {
                    "Type": "AWS::S3::Bucket",
                    "Properties": {"BucketName": "my-bucket"}
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        assert_eq!(artifact.resource_count(), 3);

        let instances = artifact.get_resources_by_type("aws_ec2_instance");
        assert_eq!(instances.len(), 2);

        let buckets = artifact.get_resources_by_type("aws_s3_bucket");
        assert_eq!(buckets.len(), 1);
    }

    #[test]
    fn test_resource_metadata() {
        let template = r#"{
            "AWSTemplateFormatVersion": "2010-09-09",
            "Resources": {
                "MyInstance": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {},
                    "Metadata": {
                        "AWS::CloudFormation::Designer": {
                            "id": "12345"
                        }
                    }
                }
            }
        }"#;

        let parser = CloudFormationParser::new();
        let artifact = parser.parse(template).unwrap();

        let resource = artifact.get_resource("MyInstance").unwrap();
        assert!(!resource.metadata.is_empty());
    }
}
