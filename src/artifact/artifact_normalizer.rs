use super::artifact_types::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Normalizes different artifact formats to a common internal representation
pub struct ArtifactNormalizer;

impl ArtifactNormalizer {
    /// Normalize an artifact to Terraform-like format for cost analysis
    pub fn normalize(artifact: &Artifact) -> NormalizedPlan {
        let mut plan = NormalizedPlan {
            format_version: "1.0".to_string(),
            source_format: artifact.format,
            source_metadata: artifact.metadata.clone(),
            resource_changes: Vec::new(),
        };
        
        for resource in &artifact.resources {
            if let Some(change) = Self::normalize_resource(resource, &artifact.format) {
                plan.resource_changes.push(change);
            }
        }
        
        plan
    }
    
    /// Normalize a single resource to a resource change
    fn normalize_resource(
        resource: &ArtifactResource,
        format: &ArtifactFormat,
    ) -> Option<NormalizedResourceChange> {
        // Convert to Terraform-style type
        let resource_type = resource.normalized_type();
        
        // Build address (resource identifier)
        let address = Self::build_resource_address(&resource.id, &resource_type, format);
        
        // Normalize properties to Terraform-style after values
        let after = Self::normalize_properties(&resource.properties, &resource_type);
        
        Some(NormalizedResourceChange {
            address,
            mode: "managed".to_string(),
            resource_type: resource_type.clone(),
            name: resource.id.clone(),
            change: ChangeAction {
                actions: vec!["create".to_string()],
                before: Value::Null,
                after,
                after_unknown: HashMap::new(),
            },
            source_metadata: resource.metadata.clone(),
        })
    }
    
    /// Build resource address in Terraform format
    fn build_resource_address(id: &str, resource_type: &str, format: &ArtifactFormat) -> String {
        match format {
            ArtifactFormat::Terraform => {
                // Already in correct format
                format!("{}.{}", resource_type, id)
            }
            ArtifactFormat::CloudFormation | ArtifactFormat::Cdk => {
                // Convert CloudFormation logical ID to Terraform-style
                // AWS::EC2::Instance -> aws_instance.MyInstance
                format!("{}.{}", resource_type, Self::sanitize_name(id))
            }
            ArtifactFormat::Pulumi => {
                format!("{}.{}", resource_type, id)
            }
        }
    }
    
    /// Sanitize resource name to be Terraform-compatible
    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }
    
    /// Normalize CloudFormation/CDK properties to Terraform-style
    fn normalize_properties(properties: &HashMap<String, Value>, resource_type: &str) -> Value {
        let mut normalized = serde_json::Map::new();
        
        for (key, value) in properties {
            let normalized_key = Self::normalize_property_key(key, resource_type);
            let normalized_value = Self::normalize_property_value(value);
            normalized.insert(normalized_key, normalized_value);
        }
        
        Value::Object(normalized)
    }
    
    /// Normalize property key from CloudFormation to Terraform style
    fn normalize_property_key(key: &str, resource_type: &str) -> String {
        // Convert PascalCase to snake_case
        let mut result = String::new();
        let mut prev_lower = false;
        
        for (i, ch) in key.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && prev_lower {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
                prev_lower = false;
            } else {
                result.push(ch);
                prev_lower = true;
            }
        }
        
        // Apply resource-specific mappings
        Self::apply_property_mappings(&result, resource_type)
    }
    
    /// Apply resource-specific property name mappings
    fn apply_property_mappings(key: &str, resource_type: &str) -> String {
        // EC2 Instance mappings
        if resource_type == "aws_instance" || resource_type.contains("ec2_instance") {
            match key {
                "image_id" => return "ami".to_string(),
                "key_name" => return "key_name".to_string(),
                _ => {}
            }
        }
        
        // S3 Bucket mappings  
        if resource_type == "aws_s3_bucket" || resource_type.contains("s3_bucket") {
            match key {
                "bucket_name" => return "bucket".to_string(),
                _ => {}
            }
        }
        
        // RDS mappings
        if resource_type.contains("rds") || resource_type.contains("db_instance") {
            match key {
                "d_b_instance_class" => return "instance_class".to_string(),
                "d_b_instance_identifier" => return "identifier".to_string(),
                _ => {}
            }
        }
        
        key.to_string()
    }
    
    /// Normalize property value (resolve intrinsic functions if possible)
    fn normalize_property_value(value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                // Check for intrinsic functions
                if obj.contains_key("Ref") {
                    if let Some(ref_val) = obj.get("Ref").and_then(|v| v.as_str()) {
                        return json!(format!("${{{}}}", ref_val));
                    }
                }
                
                if obj.contains_key("Fn::GetAtt") {
                    if let Some(arr) = obj.get("Fn::GetAtt").and_then(|v| v.as_array()) {
                        if arr.len() == 2 {
                            if let (Some(resource), Some(attr)) = 
                                (arr[0].as_str(), arr[1].as_str()) {
                                return json!(format!("${{{}.{}}}", resource, attr));
                            }
                        }
                    }
                }
                
                if obj.contains_key("Fn::Sub") {
                    // Try to simplify Sub expressions
                    if let Some(template) = obj.get("Fn::Sub").and_then(|v| v.as_str()) {
                        return json!(template);
                    }
                }
                
                if obj.contains_key("Fn::Join") {
                    if let Some(arr) = obj.get("Fn::Join").and_then(|v| v.as_array()) {
                        if arr.len() == 2 {
                            if let (Some(delim), Some(parts)) = 
                                (arr[0].as_str(), arr[1].as_array()) {
                                let joined = parts.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(delim);
                                return json!(joined);
                            }
                        }
                    }
                }
                
                // Recursively normalize nested objects
                let mut normalized = serde_json::Map::new();
                for (k, v) in obj {
                    normalized.insert(k.clone(), Self::normalize_property_value(v));
                }
                Value::Object(normalized)
            }
            Value::Array(arr) => {
                Value::Array(
                    arr.iter()
                        .map(|v| Self::normalize_property_value(v))
                        .collect()
                )
            }
            _ => value.clone(),
        }
    }
}

/// Normalized plan in Terraform-like format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NormalizedPlan {
    /// Format version
    pub format_version: String,
    
    /// Original source format
    pub source_format: ArtifactFormat,
    
    /// Source metadata
    pub source_metadata: ArtifactMetadata,
    
    /// Resource changes
    pub resource_changes: Vec<NormalizedResourceChange>,
}

/// Normalized resource change
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NormalizedResourceChange {
    /// Resource address (e.g., "aws_instance.web")
    pub address: String,
    
    /// Resource mode ("managed" or "data")
    pub mode: String,
    
    /// Resource type (normalized to Terraform style)
    #[serde(rename = "type")]
    pub resource_type: String,
    
    /// Resource name
    pub name: String,
    
    /// Change details
    pub change: ChangeAction,
    
    /// Source metadata from original artifact
    #[serde(default)]
    pub source_metadata: HashMap<String, String>,
}

/// Change action details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangeAction {
    /// Actions to be performed (e.g., ["create"], ["update"], ["delete", "create"])
    pub actions: Vec<String>,
    
    /// State before change
    pub before: Value,
    
    /// State after change
    pub after: Value,
    
    /// Unknown values after change
    #[serde(default)]
    pub after_unknown: HashMap<String, bool>,
}

impl NormalizedPlan {
    /// Convert to Terraform JSON plan format
    pub fn to_terraform_plan(&self) -> Value {
        json!({
            "format_version": self.format_version,
            "terraform_version": "1.0.0",
            "resource_changes": self.resource_changes,
            "source_format": self.source_format,
        })
    }
    
    /// Get all resources being created
    pub fn created_resources(&self) -> Vec<&NormalizedResourceChange> {
        self.resource_changes
            .iter()
            .filter(|r| r.change.actions.contains(&"create".to_string()))
            .collect()
    }
    
    /// Count resources by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for change in &self.resource_changes {
            *counts.entry(change.resource_type.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(ArtifactNormalizer::sanitize_name("MyInstance"), "myinstance");
        assert_eq!(ArtifactNormalizer::sanitize_name("My-Instance"), "my-instance");
        assert_eq!(ArtifactNormalizer::sanitize_name("My_Instance"), "my_instance");
        assert_eq!(ArtifactNormalizer::sanitize_name("My.Instance"), "my_instance");
    }

    #[test]
    fn test_normalize_property_key() {
        assert_eq!(
            ArtifactNormalizer::normalize_property_key("InstanceType", "aws_instance"),
            "instance_type"
        );
        assert_eq!(
            ArtifactNormalizer::normalize_property_key("ImageId", "aws_instance"),
            "ami"
        );
        assert_eq!(
            ArtifactNormalizer::normalize_property_key("BucketName", "aws_s3_bucket"),
            "bucket"
        );
    }

    #[test]
    fn test_normalize_ref_function() {
        let value = json!({"Ref": "MyParameter"});
        let normalized = ArtifactNormalizer::normalize_property_value(&value);
        assert_eq!(normalized, json!("${MyParameter}"));
    }

    #[test]
    fn test_normalize_getatt_function() {
        let value = json!({"Fn::GetAtt": ["MyInstance", "PublicIp"]});
        let normalized = ArtifactNormalizer::normalize_property_value(&value);
        assert_eq!(normalized, json!("${MyInstance.PublicIp}"));
    }

    #[test]
    fn test_normalize_join_function() {
        let value = json!({"Fn::Join": ["-", ["prefix", "suffix"]]});
        let normalized = ArtifactNormalizer::normalize_property_value(&value);
        assert_eq!(normalized, json!("prefix-suffix"));
    }

    #[test]
    fn test_normalize_artifact() {
        let mut artifact = Artifact::new(
            ArtifactFormat::CloudFormation,
            ArtifactMetadata {
                source: "test.yaml".to_string(),
                version: Some("2010-09-09".to_string()),
                stack_name: Some("TestStack".to_string()),
                region: Some("us-east-1".to_string()),
                tags: HashMap::new(),
            },
        );
        
        let mut properties = HashMap::new();
        properties.insert("InstanceType".to_string(), json!("t3.micro"));
        properties.insert("ImageId".to_string(), json!("ami-12345"));
        
        artifact.add_resource(ArtifactResource {
            id: "MyInstance".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties,
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });
        
        let normalized = ArtifactNormalizer::normalize(&artifact);
        
        assert_eq!(normalized.source_format, ArtifactFormat::CloudFormation);
        assert_eq!(normalized.resource_changes.len(), 1);
        
        let change = &normalized.resource_changes[0];
        assert_eq!(change.resource_type, "aws_ec2_instance");
        assert!(change.address.contains("aws_ec2_instance"));
    }

    #[test]
    fn test_normalize_multiple_resources() {
        let mut artifact = Artifact::new(
            ArtifactFormat::CloudFormation,
            ArtifactMetadata {
                source: "test.yaml".to_string(),
                version: None,
                stack_name: None,
                region: None,
                tags: HashMap::new(),
            },
        );
        
        artifact.add_resource(ArtifactResource {
            id: "Instance1".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });
        
        artifact.add_resource(ArtifactResource {
            id: "Bucket1".to_string(),
            resource_type: "AWS::S3::Bucket".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });
        
        let normalized = ArtifactNormalizer::normalize(&artifact);
        assert_eq!(normalized.resource_changes.len(), 2);
        
        let counts = normalized.count_by_type();
        assert_eq!(counts.get("aws_ec2_instance"), Some(&1));
        assert_eq!(counts.get("aws_s3_bucket"), Some(&1));
    }

    #[test]
    fn test_build_resource_address() {
        let address = ArtifactNormalizer::build_resource_address(
            "MyInstance",
            "aws_instance",
            &ArtifactFormat::CloudFormation,
        );
        assert_eq!(address, "aws_instance.myinstance");
        
        let address = ArtifactNormalizer::build_resource_address(
            "my_instance",
            "aws_instance",
            &ArtifactFormat::Terraform,
        );
        assert_eq!(address, "aws_instance.my_instance");
    }

    #[test]
    fn test_to_terraform_plan() {
        let normalized = NormalizedPlan {
            format_version: "1.0".to_string(),
            source_format: ArtifactFormat::CloudFormation,
            source_metadata: ArtifactMetadata {
                source: "test".to_string(),
                version: None,
                stack_name: None,
                region: None,
                tags: HashMap::new(),
            },
            resource_changes: Vec::new(),
        };
        
        let plan = normalized.to_terraform_plan();
        assert!(plan.is_object());
        assert!(plan.get("format_version").is_some());
        assert!(plan.get("resource_changes").is_some());
    }
}
