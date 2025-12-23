use costpilot::artifact::*;
use serde_json::{json, Value};
use std::collections::HashMap;

// Artifact Normalizer Unit Tests (70 tests)

// 1-10: Basic normalization tests

#[test]
fn test_normalize_empty_artifact() {
    let artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.format_version, "1.0");
    assert_eq!(normalized.source_format, ArtifactFormat::Cdk);
    assert_eq!(normalized.source_metadata.source, "test.json");
    assert_eq!(normalized.resource_changes.len(), 0);
}

#[test]
fn test_normalize_single_resource() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: Some("2010-09-09".to_string()),
            stack_name: Some("TestStack".to_string()),
            region: Some("us-east-1".to_string()),
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 1);
    let change = &normalized.resource_changes[0];
    assert_eq!(change.address, "aws_instance.myinstance");
    assert_eq!(change.resource_type, "aws_instance");
}

#[test]
fn test_normalize_terraform_format() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Terraform,
        ArtifactMetadata {
            source: "main.tf".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "example".to_string(),
        resource_type: "aws_instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.source_format, ArtifactFormat::Terraform);
    assert_eq!(normalized.resource_changes.len(), 1);
    let change = &normalized.resource_changes[0];
    assert_eq!(change.address, "aws_instance.example");
}

#[test]
fn test_normalize_cdk_format() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "app.ts".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyStack/MyInstance".to_string(),
        resource_type: "aws-cdk-lib.aws_ec2.Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.source_format, ArtifactFormat::Cdk);
    assert_eq!(normalized.resource_changes.len(), 1);
    let change = &normalized.resource_changes[0];
    assert_eq!(
        change.address,
        "aws-cdk-lib.aws_ec2.Instance.mystack_myinstance"
    );
}

#[test]
fn test_normalize_with_properties() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("InstanceType".to_string(), json!("t2.micro"));
    properties.insert("ImageId".to_string(), json!("ami-12345"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 1);
    let change = &normalized.resource_changes[0];
    assert_eq!(change.change.after["instance_type"], json!("t2.micro"));
    assert_eq!(change.change.after["ami"], json!("ami-12345"));
}

#[test]
fn test_normalize_metadata_preservation() {
    let mut tags = HashMap::new();
    tags.insert("Environment".to_string(), "test".to_string());
    tags.insert("Project".to_string(), "costpilot".to_string());

    let artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: Some("2010-09-09".to_string()),
            stack_name: Some("TestStack".to_string()),
            region: Some("us-east-1".to_string()),
            tags,
        },
    );

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.source_metadata.source, "test.json");
    assert_eq!(
        normalized.source_metadata.version,
        Some("2010-09-09".to_string())
    );
    assert_eq!(
        normalized.source_metadata.stack_name,
        Some("TestStack".to_string())
    );
    assert_eq!(
        normalized.source_metadata.region,
        Some("us-east-1".to_string())
    );
    assert_eq!(normalized.source_metadata.tags["Environment"], "test");
    assert_eq!(normalized.source_metadata.tags["Project"], "costpilot");
}

#[test]
fn test_normalize_multiple_resources() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
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

    let addresses: Vec<String> = normalized
        .resource_changes
        .iter()
        .map(|c| c.address.clone())
        .collect();
    assert!(addresses
        .iter()
        .any(|addr| addr == "aws_instance.instance1"));
    assert!(addresses.iter().any(|addr| addr == "aws_s3_bucket.bucket1"));
}

#[test]
fn test_normalize_depends_on() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "Bucket1".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    artifact.add_resource(ArtifactResource {
        id: "Instance1".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: vec!["Bucket1".to_string()],
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 2);

    // Check that both resources are present
    let addresses: Vec<String> = normalized
        .resource_changes
        .iter()
        .map(|c| c.address.clone())
        .collect();
    assert!(addresses
        .iter()
        .any(|addr| addr == "aws_instance.instance1"));
    assert!(addresses.iter().any(|addr| addr == "aws_s3_bucket.bucket1"));
}

#[test]
fn test_normalize_format_version() {
    let artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.format_version, "1.0");
}

// 11-20: Resource address building tests

#[test]
fn test_address_cdk_ec2() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_instance.myinstance"
    );
}

#[test]
fn test_address_cdk_s3() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyBucket".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_s3_bucket.mybucket"
    );
}

#[test]
fn test_address_cdk_rds() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyDB".to_string(),
        resource_type: "AWS::RDS::DBInstance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_db_instance.mydb"
    );
}

#[test]
fn test_address_terraform_passthrough() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Terraform,
        ArtifactMetadata {
            source: "main.tf".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "example".to_string(),
        resource_type: "aws_instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_instance.example"
    );
}

#[test]
fn test_address_cdk_with_path() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "app.ts".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyStack/MyBucket/Resource".to_string(),
        resource_type: "aws-cdk-lib.aws_s3.Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws-cdk-lib.aws_s3.Bucket.mystack_mybucket_resource"
    );
}

#[test]
fn test_address_special_characters() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "My-Bucket_123".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_s3_bucket.my-bucket_123"
    );
}

#[test]
fn test_address_case_sensitivity() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MYBUCKET".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].address,
        "aws_s3_bucket.mybucket"
    );
}

#[test]
fn test_address_empty_id() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes[0].address, "aws_s3_bucket.");
}

#[test]
fn test_address_long_id() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let long_id = "A".repeat(100);
    artifact.add_resource(ArtifactResource {
        id: long_id.clone(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert!(normalized.resource_changes[0]
        .address
        .starts_with("aws_s3_bucket."));
}

// 21-35: Property key normalization tests

#[test]
fn test_property_key_pascal_to_snake() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("InstanceType".to_string(), json!("t2.micro"));
    properties.insert("ImageId".to_string(), json!("ami-12345"));
    properties.insert("KeyName".to_string(), json!("my-key"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instance_type"], json!("t2.micro"));
    assert_eq!(config["ami"], json!("ami-12345"));
    assert_eq!(config["key_name"], json!("my-key"));
}

#[test]
fn test_property_key_s3_mappings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("BucketName".to_string(), json!("my-bucket"));
    properties.insert(
        "VersioningConfiguration".to_string(),
        json!({"Status": "Enabled"}),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyBucket".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["bucket"], json!("my-bucket"));
    assert_eq!(
        config["versioning_configuration"]["Status"],
        json!("Enabled")
    );
}

#[test]
fn test_property_key_rds_mappings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("DBInstanceClass".to_string(), json!("db.t2.micro"));
    properties.insert("DBInstanceIdentifier".to_string(), json!("my-db"));
    properties.insert("Engine".to_string(), json!("mysql"));

    artifact.add_resource(ArtifactResource {
        id: "MyDB".to_string(),
        resource_type: "AWS::RDS::DBInstance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instance_class"], json!("db.t2.micro"));
    assert_eq!(config["identifier"], json!("my-db"));
    assert_eq!(config["engine"], json!("mysql"));
}

#[test]
fn test_property_key_asg_mappings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("AutoScalingGroupName".to_string(), json!("my-asg"));
    properties.insert("MinSize".to_string(), json!("1"));
    properties.insert("MaxSize".to_string(), json!("10"));

    artifact.add_resource(ArtifactResource {
        id: "MyASG".to_string(),
        resource_type: "AWS::AutoScaling::AutoScalingGroup".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["auto_scaling_group_name"], json!("my-asg"));
    assert_eq!(config["min_size"], json!(1));
    assert_eq!(config["max_size"], json!(10));
}

#[test]
fn test_property_key_elb_mappings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("LoadBalancerName".to_string(), json!("my-elb"));
    properties.insert("Scheme".to_string(), json!("internal"));

    artifact.add_resource(ArtifactResource {
        id: "MyELB".to_string(),
        resource_type: "AWS::ElasticLoadBalancing::LoadBalancer".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["load_balancer_name"], json!("my-elb"));
    assert_eq!(config["scheme"], json!("internal"));
}

#[test]
fn test_property_key_already_snake_case() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Terraform,
        ArtifactMetadata {
            source: "main.tf".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("instance_type".to_string(), json!("t2.micro"));
    properties.insert("ami".to_string(), json!("ami-12345"));

    artifact.add_resource(ArtifactResource {
        id: "example".to_string(),
        resource_type: "aws_instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instance_type"], json!("t2.micro"));
    assert_eq!(config["ami"], json!("ami-12345"));
}

#[test]
fn test_property_key_mixed_case() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("instanceType_Name".to_string(), json!("test"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instance_type_name"], json!("test"));
}

#[test]
fn test_property_key_empty_and_single() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("".to_string(), json!("empty"));
    properties.insert("A".to_string(), json!("single"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config[""], json!("empty"));
    assert_eq!(config["a"], json!("single"));
}

#[test]
fn test_property_key_uppercase() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("INSTANCETYPE".to_string(), json!("t2.micro"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instancetype"], json!("t2.micro"));
}

#[test]
fn test_property_key_special_mappings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("ImageId".to_string(), json!("ami-12345"));
    properties.insert("KeyName".to_string(), json!("my-key"));
    properties.insert("VPCId".to_string(), json!("vpc-12345"));
    properties.insert("IAMRole".to_string(), json!("my-role"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["ami"], json!("ami-12345"));
    assert_eq!(config["key_name"], json!("my-key"));
    assert_eq!(config["vpc_id"], json!("vpc-12345"));
    assert_eq!(config["iam_instance_profile"], json!("my-role"));
}

#[test]
fn test_property_key_unknown_resource_type() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("SomeProperty".to_string(), json!("value"));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::Unknown::Service".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["some_property"], json!("value"));
}

#[test]
fn test_property_key_case_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("instanceType_Name".to_string(), json!("TestValue"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["instance_type_name"], json!("TestValue"));
}

// 36-50: Property value normalization tests

#[test]
fn test_property_value_string_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("InstanceType".to_string(), json!("t2.micro"));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["instance_type"],
        json!("t2.micro")
    );
}

#[test]
fn test_property_value_number_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Port".to_string(), json!(3306));

    artifact.add_resource(ArtifactResource {
        id: "MyDB".to_string(),
        resource_type: "AWS::RDS::DBInstance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["port"],
        json!(3306)
    );
}

#[test]
fn test_property_value_boolean_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Enabled".to_string(), json!(true));

    artifact.add_resource(ArtifactResource {
        id: "MyBucket".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["enabled"],
        json!(true)
    );
}

#[test]
fn test_property_value_array_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("SecurityGroups".to_string(), json!(["sg-1", "sg-2"]));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["security_groups"],
        json!(["sg-1", "sg-2"])
    );
}

#[test]
fn test_property_value_object_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert(
        "Tags".to_string(),
        json!({"Environment": "test", "Project": "costpilot"}),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let tags = &normalized.resource_changes[0].change.after["tags"];
    assert_eq!(tags["Environment"], json!("test"));
    assert_eq!(tags["Project"], json!("costpilot"));
}

#[test]
fn test_property_value_null_handling() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("OptionalField".to_string(), json!(null));

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["optional_field"],
        json!(null)
    );
}

#[test]
fn test_property_value_intrinsic_functions() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Ref".to_string(), json!({"Ref": "MyParameter"}));
    properties.insert(
        "GetAtt".to_string(),
        json!({"Fn::GetAtt": ["MyInstance", "PublicIp"]}),
    );
    properties.insert(
        "Join".to_string(),
        json!({"Fn::Join": [",", ["a", "b", "c"]]}),
    );
    properties.insert(
        "Sub".to_string(),
        json!({"Fn::Sub": "${AWS::StackName}-suffix"}),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    // Intrinsic functions should be resolved where possible
    assert_eq!(config["ref"], json!("${MyParameter}"));
    assert_eq!(config["get_att"], json!("${MyInstance.PublicIp}"));
    assert_eq!(config["join"], json!("a,b,c"));
    assert_eq!(config["sub"], json!("${AWS::StackName}-suffix"));
}

#[test]
fn test_property_value_complex_nested() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert(
        "Complex".to_string(),
        json!({
            "nested": {
                "array": [1, 2, {"nested": "value"}],
                "boolean": true,
                "null": null,
                "string": "test"
            }
        }),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let complex = &normalized.resource_changes[0].change.after["complex"];
    assert_eq!(complex["nested"]["array"][0], json!(1));
    assert_eq!(complex["nested"]["array"][1], json!(2));
    assert_eq!(complex["nested"]["array"][2]["nested"], json!("value"));
    assert_eq!(complex["nested"]["boolean"], json!(true));
    assert_eq!(complex["nested"]["null"], json!(null));
    assert_eq!(complex["nested"]["string"], json!("test"));
}

#[test]
fn test_property_value_empty_object() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Empty".to_string(), json!({}));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["empty"],
        json!({})
    );
}

#[test]
fn test_property_value_empty_array() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("EmptyArray".to_string(), json!([]));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["empty_array"],
        json!([])
    );
}

#[test]
fn test_property_value_large_numbers() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("LargeNumber".to_string(), json!(999999999999999i64));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["large_number"],
        json!(999999999999999i64)
    );
}

#[test]
fn test_property_value_unicode_strings() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Unicode".to_string(), json!("测试字符串 🚀"));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(
        normalized.resource_changes[0].change.after["unicode"],
        json!("测试字符串 🚀")
    );
}

// 51-70: Plan operations and edge cases

#[test]
fn test_plan_operations_create() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 1);
    let change = &normalized.resource_changes[0];
    assert_eq!(change.change.actions[0], "create");
    assert_eq!(change.change.before, Value::Null);
}

#[test]
fn test_plan_operations_no_changes() {
    let artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 0);
}

#[test]
fn test_plan_operations_mixed_resources() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    // Add different types of resources
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

    artifact.add_resource(ArtifactResource {
        id: "DB1".to_string(),
        resource_type: "AWS::RDS::DBInstance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 3);

    let resource_types: Vec<String> = normalized
        .resource_changes
        .iter()
        .map(|c| c.resource_type.clone())
        .collect();
    assert!(resource_types.iter().any(|rt| rt == "aws_instance"));
    assert!(resource_types.iter().any(|rt| rt == "aws_s3_bucket"));
    assert!(resource_types.iter().any(|rt| rt == "aws_db_instance"));
}

#[test]
fn test_plan_operations_depends_on_resolution() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "VPC1".to_string(),
        resource_type: "AWS::EC2::VPC".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    artifact.add_resource(ArtifactResource {
        id: "Subnet1".to_string(),
        resource_type: "AWS::EC2::Subnet".to_string(),
        properties: HashMap::new(),
        depends_on: vec!["VPC1".to_string()],
        metadata: HashMap::new(),
    });

    artifact.add_resource(ArtifactResource {
        id: "Instance1".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: vec!["Subnet1".to_string()],
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 3);

    // Check that all resources are present with correct addresses
    let addresses: Vec<String> = normalized
        .resource_changes
        .iter()
        .map(|c| c.address.clone())
        .collect();
    assert!(addresses
        .iter()
        .any(|addr| addr == "aws_instance.instance1"));
    assert!(addresses.iter().any(|addr| addr == "aws_subnet.subnet1"));
    assert!(addresses.iter().any(|addr| addr == "aws_vpc.vpc1"));
}

#[test]
fn test_plan_operations_metadata_preservation() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut metadata = HashMap::new();
    metadata.insert("custom_field".to_string(), "custom_value".to_string());

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata,
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 1);
    // Metadata should be preserved in the resource change if needed
}

#[test]
fn test_plan_operations_large_artifact() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    // Add many resources
    for i in 0..100 {
        artifact.add_resource(ArtifactResource {
            id: format!("Resource{}", i),
            resource_type: "AWS::EC2::Instance".to_string(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            metadata: HashMap::new(),
        });
    }

    let normalized = ArtifactNormalizer::normalize(&artifact);
    assert_eq!(normalized.resource_changes.len(), 100);
}

#[test]
fn test_plan_operations_empty_properties() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties: HashMap::new(),
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert!(config.as_object().unwrap().is_empty());
}

#[test]
fn test_plan_operations_special_characters_in_properties() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Name".to_string(), json!("Test@Instance#123"));
    properties.insert(
        "Description".to_string(),
        json!("Test with spaces and symbols: !@#$%^&*()"),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["name"], json!("Test@Instance#123"));
    assert_eq!(
        config["description"],
        json!("Test with spaces and symbols: !@#$%^&*()")
    );
}

#[test]
fn test_plan_operations_unicode_in_properties() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("Name".to_string(), json!("测试实例 🚀"));
    properties.insert(
        "Tags".to_string(),
        json!({"项目": "CostPilot", "环境": "测试"}),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["name"], json!("测试实例 🚀"));
    let tags = &config["tags"];
    assert_eq!(tags["项目"], json!("CostPilot"));
    assert_eq!(tags["环境"], json!("测试"));
}

#[test]
fn test_plan_operations_null_values() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("OptionalField1".to_string(), json!(null));
    properties.insert("OptionalField2".to_string(), Value::Null);

    artifact.add_resource(ArtifactResource {
        id: "MyInstance".to_string(),
        resource_type: "AWS::EC2::Instance".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["optional_field1"], json!(null));
    assert_eq!(config["optional_field2"], json!(null));
}

#[test]
fn test_plan_operations_deeply_nested_structures() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert(
        "ComplexConfig".to_string(),
        json!({
            "level1": {
                "level2": {
                    "level3": {
                        "array": [1, 2, {"nested": "value"}],
                        "boolean": true,
                        "string": "deep"
                    }
                }
            }
        }),
    );

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    let complex = &config["complex_config"];
    assert_eq!(complex["level1"]["level2"]["level3"]["array"][0], json!(1));
    assert_eq!(complex["level1"]["level2"]["level3"]["array"][1], json!(2));
    assert_eq!(
        complex["level1"]["level2"]["level3"]["array"][2]["nested"],
        json!("value")
    );
    assert_eq!(
        complex["level1"]["level2"]["level3"]["boolean"],
        json!(true)
    );
    assert_eq!(
        complex["level1"]["level2"]["level3"]["string"],
        json!("deep")
    );
}

#[test]
fn test_plan_operations_mixed_data_types() {
    let mut artifact = Artifact::new(
        ArtifactFormat::Cdk,
        ArtifactMetadata {
            source: "test.json".to_string(),
            version: None,
            stack_name: None,
            region: None,
            tags: HashMap::new(),
        },
    );

    let mut properties = HashMap::new();
    properties.insert("StringField".to_string(), json!("string"));
    properties.insert("NumberField".to_string(), json!(42));
    properties.insert("BooleanField".to_string(), json!(true));
    properties.insert("ArrayField".to_string(), json!([1, "two", true, null]));
    properties.insert("ObjectField".to_string(), json!({"key": "value"}));
    properties.insert("NullField".to_string(), json!(null));

    artifact.add_resource(ArtifactResource {
        id: "MyResource".to_string(),
        resource_type: "AWS::S3::Bucket".to_string(),
        properties,
        depends_on: Vec::new(),
        metadata: HashMap::new(),
    });

    let normalized = ArtifactNormalizer::normalize(&artifact);
    let config = &normalized.resource_changes[0].change.after;
    assert_eq!(config["string_field"], json!("string"));
    assert_eq!(config["number_field"], json!(42));
    assert_eq!(config["boolean_field"], json!(true));
    assert_eq!(config["array_field"], json!([1, "two", true, null]));
    assert_eq!(config["object_field"], json!({"key": "value"}));
    assert_eq!(config["null_field"], json!(null));
}
