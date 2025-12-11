use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an Infrastructure as Code artifact (Terraform, CDK, CloudFormation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Source format of the artifact
    pub format: ArtifactFormat,

    /// Metadata about the artifact source
    pub metadata: ArtifactMetadata,

    /// Resources defined in the artifact
    pub resources: Vec<ArtifactResource>,

    /// Outputs defined in the artifact
    #[serde(default)]
    pub outputs: HashMap<String, ArtifactOutput>,

    /// Parameters/variables used
    #[serde(default)]
    pub parameters: HashMap<String, ArtifactParameter>,
}

/// Infrastructure as Code format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactFormat {
    /// Terraform JSON plan
    Terraform,

    /// AWS CloudFormation template
    CloudFormation,

    /// AWS CDK synthesized output
    Cdk,

    /// Pulumi program output (future)
    Pulumi,
}

impl ArtifactFormat {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            ArtifactFormat::Terraform => "Terraform",
            ArtifactFormat::CloudFormation => "CloudFormation",
            ArtifactFormat::Cdk => "AWS CDK",
            ArtifactFormat::Pulumi => "Pulumi",
        }
    }

    /// Check if format is supported
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            ArtifactFormat::Terraform | ArtifactFormat::CloudFormation | ArtifactFormat::Cdk
        )
    }
}

/// Metadata about artifact source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Source file path or stack name
    pub source: String,

    /// Artifact format version (e.g., CFN template version, TF version)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Stack name (for CloudFormation/CDK)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_name: Option<String>,

    /// Region where resources will be created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// A resource defined in the artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactResource {
    /// Logical ID in the template
    pub id: String,

    /// Resource type (e.g., AWS::EC2::Instance, aws_instance)
    pub resource_type: String,

    /// Resource properties/configuration
    pub properties: HashMap<String, serde_json::Value>,

    /// Dependencies on other resources
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Resource-level metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ArtifactResource {
    /// Get normalized resource type (convert CFN to Terraform-style)
    pub fn normalized_type(&self) -> String {
        if self.resource_type.starts_with("AWS::") {
            // Convert AWS::EC2::Instance to aws_instance
            let parts: Vec<&str> = self.resource_type.split("::").collect();
            if parts.len() == 3 {
                let service = parts[1].to_lowercase();
                let resource = parts[2].to_lowercase();
                return format!("aws_{}_{}", service, resource);
            }
        }
        self.resource_type.clone()
    }

    /// Get property value as string
    pub fn get_property_string(&self, key: &str) -> Option<String> {
        self.properties.get(key).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        })
    }

    /// Check if resource has a specific property
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

/// Stack output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactOutput {
    /// Output value or reference
    pub value: serde_json::Value,

    /// Output description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether output is exported for cross-stack reference
    #[serde(default)]
    pub export: bool,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactParameter {
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: String,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allowed values
    #[serde(default)]
    pub allowed_values: Vec<serde_json::Value>,
}

/// CloudFormation intrinsic function
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntrinsicFunction {
    /// Ref function: { "Ref": "LogicalName" }
    Ref {
        #[serde(rename = "Ref")]
        reference: String,
    },

    /// GetAtt function: { "Fn::GetAtt": ["LogicalName", "Attribute"] }
    GetAtt {
        #[serde(rename = "Fn::GetAtt")]
        get_att: Vec<String>,
    },

    /// Sub function: { "Fn::Sub": "String with ${placeholder}" }
    Sub {
        #[serde(rename = "Fn::Sub")]
        sub: serde_json::Value,
    },

    /// Join function: { "Fn::Join": ["delimiter", ["value1", "value2"]] }
    Join {
        #[serde(rename = "Fn::Join")]
        join: Vec<serde_json::Value>,
    },

    /// ImportValue function
    ImportValue {
        #[serde(rename = "Fn::ImportValue")]
        import_value: serde_json::Value,
    },

    /// Select function
    Select {
        #[serde(rename = "Fn::Select")]
        select: Vec<serde_json::Value>,
    },

    /// FindInMap function
    FindInMap {
        #[serde(rename = "Fn::FindInMap")]
        find_in_map: Vec<serde_json::Value>,
    },

    /// Base64 function
    Base64 {
        #[serde(rename = "Fn::Base64")]
        base64: serde_json::Value,
    },
}

impl IntrinsicFunction {
    /// Try to resolve to a simple value (if possible without context)
    pub fn try_resolve(&self) -> Option<String> {
        match self {
            IntrinsicFunction::Ref { reference } => Some(format!("${{{}}}", reference)),
            IntrinsicFunction::GetAtt { get_att } => {
                if get_att.len() == 2 {
                    Some(format!("${{{}.{}}}", get_att[0], get_att[1]))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Result of parsing an artifact
pub type ArtifactResult<T> = Result<T, ArtifactError>;

/// Errors that can occur during artifact parsing
#[derive(Debug, Clone, thiserror::Error)]
pub enum ArtifactError {
    #[error("Unsupported artifact format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to parse artifact: {0}")]
    ParseError(String),

    #[error("Invalid resource definition: {0}")]
    InvalidResource(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid template version: {0}")]
    InvalidVersion(String),

    #[error("Unsupported intrinsic function: {0}")]
    UnsupportedFunction(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for ArtifactError {
    fn from(err: std::io::Error) -> Self {
        ArtifactError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ArtifactError {
    fn from(err: serde_json::Error) -> Self {
        ArtifactError::ParseError(err.to_string())
    }
}

/// Trait for parsing different artifact formats
pub trait ArtifactParser {
    /// Parse artifact from JSON/YAML string
    fn parse(&self, content: &str) -> ArtifactResult<Artifact>;

    /// Parse artifact from file
    fn parse_file(&self, path: &str) -> ArtifactResult<Artifact> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content)
    }

    /// Get the format this parser handles
    fn format(&self) -> ArtifactFormat;
}

impl Artifact {
    /// Create a new artifact
    pub fn new(format: ArtifactFormat, metadata: ArtifactMetadata) -> Self {
        Self {
            format,
            metadata,
            resources: Vec::new(),
            outputs: HashMap::new(),
            parameters: HashMap::new(),
        }
    }

    /// Add a resource to the artifact
    pub fn add_resource(&mut self, resource: ArtifactResource) {
        self.resources.push(resource);
    }

    /// Get resource by ID
    pub fn get_resource(&self, id: &str) -> Option<&ArtifactResource> {
        self.resources.iter().find(|r| r.id == id)
    }

    /// Get all resources of a specific type
    pub fn get_resources_by_type(&self, resource_type: &str) -> Vec<&ArtifactResource> {
        self.resources
            .iter()
            .filter(|r| r.resource_type == resource_type || r.normalized_type() == resource_type)
            .collect()
    }

    /// Count resources by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for resource in &self.resources {
            *counts.entry(resource.normalized_type()).or_insert(0) += 1;
        }
        counts
    }

    /// Get total resource count
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Validate artifact structure
    pub fn validate(&self) -> ArtifactResult<()> {
        // Check for duplicate resource IDs
        let mut seen_ids = std::collections::HashSet::new();
        for resource in &self.resources {
            if !seen_ids.insert(&resource.id) {
                return Err(ArtifactError::InvalidResource(format!(
                    "Duplicate resource ID: {}",
                    resource.id
                )));
            }
        }

        // Check dependencies exist
        for resource in &self.resources {
            for dep in &resource.depends_on {
                if !seen_ids.contains(dep) {
                    return Err(ArtifactError::InvalidResource(format!(
                        "Resource {} depends on non-existent resource {}",
                        resource.id, dep
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_format_name() {
        assert_eq!(ArtifactFormat::Terraform.name(), "Terraform");
        assert_eq!(ArtifactFormat::CloudFormation.name(), "CloudFormation");
        assert_eq!(ArtifactFormat::Cdk.name(), "AWS CDK");
    }

    #[test]
    fn test_artifact_format_supported() {
        assert!(ArtifactFormat::Terraform.is_supported());
        assert!(ArtifactFormat::CloudFormation.is_supported());
        assert!(ArtifactFormat::Cdk.is_supported());
        assert!(!ArtifactFormat::Pulumi.is_supported());
    }

    #[test]
    fn test_normalized_type() {
        let resource = ArtifactResource {
            id: "MyInstance".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        };

        assert_eq!(resource.normalized_type(), "aws_ec2_instance");
    }

    #[test]
    fn test_normalized_type_already_normalized() {
        let resource = ArtifactResource {
            id: "my_instance".to_string(),
            resource_type: "aws_instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        };

        assert_eq!(resource.normalized_type(), "aws_instance");
    }

    #[test]
    fn test_artifact_new() {
        let metadata = ArtifactMetadata {
            source: "template.yaml".to_string(),
            version: Some("2010-09-09".to_string()),
            stack_name: Some("MyStack".to_string()),
            region: Some("us-east-1".to_string()),
            tags: HashMap::new(),
        };

        let artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);
        assert_eq!(artifact.format, ArtifactFormat::CloudFormation);
        assert_eq!(artifact.resources.len(), 0);
    }

    #[test]
    fn test_add_and_get_resource() {
        let metadata = ArtifactMetadata {
            source: "test".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        };

        let mut artifact = Artifact::new(ArtifactFormat::Terraform, metadata);

        let resource = ArtifactResource {
            id: "test_resource".to_string(),
            resource_type: "aws_instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        };

        artifact.add_resource(resource);
        assert_eq!(artifact.resource_count(), 1);

        let retrieved = artifact.get_resource("test_resource");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_resource");
    }

    #[test]
    fn test_get_resources_by_type() {
        let metadata = ArtifactMetadata {
            source: "test".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        };

        let mut artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);

        artifact.add_resource(ArtifactResource {
            id: "Instance1".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        artifact.add_resource(ArtifactResource {
            id: "Instance2".to_string(),
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

        let instances = artifact.get_resources_by_type("aws_ec2_instance");
        assert_eq!(instances.len(), 2);

        let buckets = artifact.get_resources_by_type("aws_s3_bucket");
        assert_eq!(buckets.len(), 1);
    }

    #[test]
    fn test_count_by_type() {
        let metadata = ArtifactMetadata {
            source: "test".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        };

        let mut artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);

        artifact.add_resource(ArtifactResource {
            id: "1".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        artifact.add_resource(ArtifactResource {
            id: "2".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        artifact.add_resource(ArtifactResource {
            id: "3".to_string(),
            resource_type: "AWS::S3::Bucket".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        let counts = artifact.count_by_type();
        assert_eq!(counts.get("aws_ec2_instance"), Some(&2));
        assert_eq!(counts.get("aws_s3_bucket"), Some(&1));
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let metadata = ArtifactMetadata {
            source: "test".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        };

        let mut artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);

        artifact.add_resource(ArtifactResource {
            id: "duplicate".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        artifact.add_resource(ArtifactResource {
            id: "duplicate".to_string(),
            resource_type: "AWS::S3::Bucket".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });

        let result = artifact.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let metadata = ArtifactMetadata {
            source: "test".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        };

        let mut artifact = Artifact::new(ArtifactFormat::CloudFormation, metadata);

        artifact.add_resource(ArtifactResource {
            id: "resource1".to_string(),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: vec!["nonexistent".to_string()],
            metadata: HashMap::new(),
        });

        let result = artifact.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_intrinsic_function_resolve() {
        let ref_fn = IntrinsicFunction::Ref {
            reference: "MyParam".to_string(),
        };
        assert_eq!(ref_fn.try_resolve(), Some("${MyParam}".to_string()));

        let getatt_fn = IntrinsicFunction::GetAtt {
            get_att: vec!["MyInstance".to_string(), "PublicIp".to_string()],
        };
        assert_eq!(
            getatt_fn.try_resolve(),
            Some("${MyInstance.PublicIp}".to_string())
        );
    }
}
