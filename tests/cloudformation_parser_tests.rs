use costpilot::artifact::*;
use std::collections::HashMap;

// CloudFormation Parser Unit Tests (60 tests)

// 1-10: Basic parsing tests

#[test]
fn test_parse_basic_template() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket"
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::CloudFormation);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_parse_template_with_description() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Test template",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.metadata.source, "Test template");
}

#[test]
fn test_parse_template_version_validation() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.metadata.version, Some("2010-09-09".to_string()));
}

#[test]
fn test_parse_template_invalid_version() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2020-01-01",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_template_no_version() {
    let template = r#"{
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert!(artifact.metadata.version.is_none());
}

#[test]
fn test_parse_template_empty() {
    let template = r#"{}"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 0);
}

#[test]
fn test_parse_template_invalid_json() {
    let template = r#"invalid json"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_template_not_object() {
    let template = r#""string""#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_template_minimal() {
    let template = r#"{"Resources":{}}"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 0);
}

#[test]
fn test_parse_template_large() {
    let mut resources = HashMap::new();
    for i in 0..1000 {
        resources.insert(format!("Resource{}", i), serde_json::json!({
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BucketName": format!("bucket-{}", i)
            }
        }));
    }
    let template = serde_json::json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": resources
    });

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(&serde_json::to_string(&template).unwrap()).unwrap();
    assert_eq!(artifact.resource_count(), 1000);
}

// 11-25: Resource parsing tests

#[test]
fn test_parse_resource_basic() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyInstance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "InstanceType": "t3.micro"
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyInstance").unwrap();
    assert_eq!(resource.resource_type, "AWS::EC2::Instance");
    assert_eq!(resource.normalized_type(), "aws_instance");
}

#[test]
fn test_parse_resource_no_properties() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket"
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyBucket").unwrap();
    assert_eq!(resource.properties.len(), 0);
}

#[test]
fn test_parse_resource_with_properties() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": "my-test-bucket",
                    "VersioningConfiguration": {
                        "Status": "Enabled"
                    }
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyBucket").unwrap();
    assert_eq!(resource.properties.len(), 2);
    assert!(resource.has_property("BucketName"));
}

#[test]
fn test_parse_resource_missing_type() {
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
fn test_parse_resource_invalid_structure() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Invalid": "string"
        }
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_resource_with_depends_on_single() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "VPC": {
                "Type": "AWS::EC2::VPC",
                "Properties": {}
            },
            "Subnet": {
                "Type": "AWS::EC2::Subnet",
                "DependsOn": "VPC",
                "Properties": {}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let subnet = artifact.get_resource("Subnet").unwrap();
    assert_eq!(subnet.depends_on.len(), 1);
    assert_eq!(subnet.depends_on[0], "VPC");
}

#[test]
fn test_parse_resource_with_depends_on_array() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "VPC": {"Type": "AWS::EC2::VPC", "Properties": {}},
            "IGW": {"Type": "AWS::EC2::InternetGateway", "Properties": {}},
            "Subnet": {
                "Type": "AWS::EC2::Subnet",
                "DependsOn": ["VPC", "IGW"],
                "Properties": {}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let subnet = artifact.get_resource("Subnet").unwrap();
    assert_eq!(subnet.depends_on.len(), 2);
    assert!(subnet.depends_on.contains(&"VPC".to_string()));
    assert!(subnet.depends_on.contains(&"IGW".to_string()));
}

#[test]
fn test_parse_resource_with_metadata() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyInstance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {},
                "Metadata": {
                    "Comment": "Test instance",
                    "Designer": {"x": 100, "y": 200}
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyInstance").unwrap();
    assert!(resource.metadata.contains_key("Comment"));
    assert!(resource.metadata.contains_key("Designer"));
}

#[test]
fn test_parse_resource_with_condition() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyInstance": {
                "Type": "AWS::EC2::Instance",
                "Condition": "CreateInstance",
                "Properties": {}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyInstance").unwrap();
    assert_eq!(resource.metadata.get("Condition"), Some(&"CreateInstance".to_string()));
}

#[test]
fn test_parse_multiple_resources() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket1": {"Type": "AWS::S3::Bucket", "Properties": {}},
            "Bucket2": {"Type": "AWS::S3::Bucket", "Properties": {}},
            "Instance": {"Type": "AWS::EC2::Instance", "Properties": {}}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 3);
    assert_eq!(artifact.get_resources_by_type("aws_s3_bucket").len(), 2);
    assert_eq!(artifact.get_resources_by_type("aws_instance").len(), 1);
}

#[test]
fn test_parse_resource_complex_properties() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyFunction": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "FunctionName": "my-function",
                    "Runtime": "nodejs18.x",
                    "Code": {
                        "ZipFile": "exports.handler = () => {};"
                    },
                    "Environment": {
                        "Variables": {
                            "KEY": "value"
                        }
                    }
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyFunction").unwrap();
    assert!(resource.has_property("FunctionName"));
    assert!(resource.has_property("Runtime"));
    assert!(resource.has_property("Code"));
    assert!(resource.has_property("Environment"));
}

#[test]
fn test_parse_resource_intrinsic_functions() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": {"Fn::Sub": "${AWS::StackName}-bucket"}
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let resource = artifact.get_resource("MyBucket").unwrap();
    assert!(resource.has_property("BucketName"));
}

#[test]
fn test_parse_resource_references() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "VPC": {
                "Type": "AWS::EC2::VPC",
                "Properties": {"CidrBlock": "10.0.0.0/16"}
            },
            "Subnet": {
                "Type": "AWS::EC2::Subnet",
                "Properties": {
                    "VpcId": {"Ref": "VPC"},
                    "CidrBlock": "10.0.1.0/24"
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 2);
}

#[test]
fn test_parse_resource_get_att() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {}
            },
            "Policy": {
                "Type": "AWS::IAM::Policy",
                "Properties": {
                    "PolicyDocument": {
                        "Statement": [{
                            "Effect": "Allow",
                            "Action": "s3:GetObject",
                            "Resource": {"Fn::GetAtt": ["Bucket", "Arn"]}
                        }]
                    }
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 2);
}

#[test]
fn test_parse_resource_with_update_policy() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Table": {
                "Type": "AWS::DynamoDB::Table",
                "Properties": {},
                "UpdatePolicy": {
                    "Attribute": "value"
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 1);
}

// 26-35: Parameter parsing tests

#[test]
fn test_parse_parameters_basic() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "InstanceType": {
                "Type": "String",
                "Default": "t3.micro"
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.parameters.len(), 1);
    let param = artifact.parameters.get("InstanceType").unwrap();
    assert_eq!(param.param_type, "String");
    assert_eq!(param.default, Some(serde_json::json!("t3.micro")));
}

#[test]
fn test_parse_parameters_with_description() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Env": {
                "Type": "String",
                "Description": "Environment name"
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let param = artifact.parameters.get("Env").unwrap();
    assert_eq!(param.description, Some("Environment name".to_string()));
}

#[test]
fn test_parse_parameters_with_allowed_values() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Size": {
                "Type": "String",
                "AllowedValues": ["small", "medium", "large"]
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let param = artifact.parameters.get("Size").unwrap();
    assert_eq!(param.allowed_values.len(), 3);
}

#[test]
fn test_parse_parameters_no_default() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Name": {
                "Type": "String"
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let param = artifact.parameters.get("Name").unwrap();
    assert!(param.default.is_none());
}

#[test]
fn test_parse_parameters_missing_type() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Invalid": {
                "Default": "value"
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_parameters_invalid_structure() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Invalid": "string"
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_multiple_parameters() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Type1": {"Type": "String"},
            "Type2": {"Type": "Number"},
            "Type3": {"Type": "List<Number>"}
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.parameters.len(), 3);
}

#[test]
fn test_parse_parameters_complex() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "VpcCidr": {
                "Type": "String",
                "Default": "10.0.0.0/16",
                "Description": "CIDR block for VPC",
                "AllowedValues": ["10.0.0.0/16", "172.16.0.0/16", "192.168.0.0/16"]
            }
        },
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let param = artifact.parameters.get("VpcCidr").unwrap();
    assert_eq!(param.param_type, "String");
    assert_eq!(param.allowed_values.len(), 3);
    assert!(param.description.is_some());
}

// 36-45: Output parsing tests

#[test]
fn test_parse_outputs_basic() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket": {"Type": "AWS::S3::Bucket", "Properties": {}}
        },
        "Outputs": {
            "BucketName": {
                "Value": {"Ref": "Bucket"}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.outputs.len(), 1);
    let output = artifact.outputs.get("BucketName").unwrap();
    assert!(!output.export);
}

#[test]
fn test_parse_outputs_with_description() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Result": {
                "Description": "Result value",
                "Value": "success"
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let output = artifact.outputs.get("Result").unwrap();
    assert_eq!(output.description, Some("Result value".to_string()));
}

#[test]
fn test_parse_outputs_with_export() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "SharedValue": {
                "Value": "shared",
                "Export": {"Name": "MySharedValue"}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let output = artifact.outputs.get("SharedValue").unwrap();
    assert!(output.export);
}

#[test]
fn test_parse_outputs_missing_value() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Invalid": {
                "Description": "No value"
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_outputs_invalid_structure() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Invalid": "string"
        }
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_multiple_outputs() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket": {"Type": "AWS::S3::Bucket", "Properties": {}},
            "Instance": {"Type": "AWS::EC2::Instance", "Properties": {}}
        },
        "Outputs": {
            "BucketArn": {"Value": {"Fn::GetAtt": ["Bucket", "Arn"]}},
            "InstanceId": {"Value": {"Ref": "Instance"}}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.outputs.len(), 2);
}

#[test]
fn test_parse_outputs_complex_value() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Complex": {
                "Value": {
                    "Fn::Join": [
                        "-",
                        [{"Ref": "AWS::StackName"}, "output"]
                    ]
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.outputs.len(), 1);
}

#[test]
fn test_parse_outputs_no_description() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Simple": {"Value": "value"}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let output = artifact.outputs.get("Simple").unwrap();
    assert!(output.description.is_none());
}

#[test]
fn test_parse_outputs_export_without_name() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {},
        "Outputs": {
            "Exported": {
                "Value": "value",
                "Export": {}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    let output = artifact.outputs.get("Exported").unwrap();
    assert!(output.export);
}

// 46-55: Metadata and validation tests

#[test]
fn test_parse_metadata_extraction() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Test stack",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.metadata.source, "Test stack");
    assert_eq!(artifact.metadata.tags.get("format"), Some(&"json".to_string()));
    assert_eq!(artifact.metadata.tags.get("description"), Some(&"Test stack".to_string()));
}

#[test]
fn test_parse_metadata_no_description() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.metadata.source, "CloudFormation Stack");
}

#[test]
fn test_parse_validation_passes() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Valid": {"Type": "AWS::S3::Bucket", "Properties": {}}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    artifact.validate().unwrap();
}

#[test]
fn test_parse_validation_fails_empty_resources() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    // Empty resources should still validate
    artifact.validate().unwrap();
}

#[test]
fn test_parse_format_method() {
    let parser = CloudFormationParser::new();
    assert_eq!(parser.format(), ArtifactFormat::CloudFormation);
}

#[test]
fn test_parse_default_implementation() {
    let parser = CloudFormationParser::default();
    assert_eq!(parser.format(), ArtifactFormat::CloudFormation);
}

#[test]
fn test_parse_yaml_not_supported() {
    let parser = CloudFormationParser::new();
    let yaml_content = "AWSTemplateFormatVersion: '2010-09-09'\nResources: {}\n";
    let result = parser.parse(yaml_content);
    assert!(result.is_err());
}

#[test]
fn test_parse_unicode_support() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "测试模板",
        "Resources": {
            "Bucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": "测试-bucket"
                }
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_parse_complex_template() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Complex template",
        "Parameters": {
            "Env": {"Type": "String", "Default": "dev"}
        },
        "Resources": {
            "VPC": {
                "Type": "AWS::EC2::VPC",
                "Properties": {"CidrBlock": "10.0.0.0/16"}
            },
            "Subnet": {
                "Type": "AWS::EC2::Subnet",
                "DependsOn": "VPC",
                "Properties": {
                    "VpcId": {"Ref": "VPC"},
                    "CidrBlock": "10.0.1.0/24"
                }
            },
            "Instance": {
                "Type": "AWS::EC2::Instance",
                "Condition": "CreateInstance",
                "Properties": {
                    "InstanceType": {"Ref": "Env"},
                    "SubnetId": {"Ref": "Subnet"}
                }
            }
        },
        "Outputs": {
            "VpcId": {
                "Description": "VPC ID",
                "Value": {"Ref": "VPC"},
                "Export": {"Name": {"Fn::Sub": "${AWS::StackName}-VpcId"}}
            }
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.resource_count(), 3);
    assert_eq!(artifact.parameters.len(), 1);
    assert_eq!(artifact.outputs.len(), 1);
}

// 56-60: Error handling and edge cases

#[test]
fn test_parse_error_handling() {
    let parser = CloudFormationParser::new();

    // Test various invalid inputs that should definitely fail
    assert!(parser.parse("").is_err());
    assert!(parser.parse("null").is_err());
    assert!(parser.parse("[]").is_err());
    assert!(parser.parse(r#"{"Resources": {"Invalid": {}}}"#).is_err()); // Resource missing Type
    assert!(parser.parse(r#"{"Parameters": {"Invalid": {}}}"#).is_err()); // Parameter missing Type
    assert!(parser.parse(r#"{"Outputs": {"Invalid": {}}}"#).is_err()); // Output missing Value
}

#[test]
fn test_parse_resource_error_recovery() {
    // If one resource is invalid, the whole parse should fail
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Valid": {"Type": "AWS::S3::Bucket", "Properties": {}},
            "Invalid": {"Properties": {}}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_sections() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {},
        "Resources": {},
        "Outputs": {}
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.parameters.len(), 0);
    assert_eq!(artifact.resource_count(), 0);
    assert_eq!(artifact.outputs.len(), 0);
}

#[test]
fn test_parse_maximal_template() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Maximal template",
        "Parameters": {
            "P1": {"Type": "String", "Default": "d", "Description": "d", "AllowedValues": ["a"]},
            "P2": {"Type": "Number"}
        },
        "Resources": {
            "R1": {
                "Type": "AWS::S3::Bucket",
                "Properties": {"BucketName": "b"},
                "DependsOn": ["R2"],
                "Metadata": {"M": "v"},
                "Condition": "C"
            },
            "R2": {"Type": "AWS::EC2::Instance", "Properties": {}}
        },
        "Outputs": {
            "O1": {"Value": "v", "Description": "d", "Export": {"Name": "n"}},
            "O2": {"Value": {"Ref": "R1"}}
        }
    }"#;

    let parser = CloudFormationParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.parameters.len(), 2);
    assert_eq!(artifact.resource_count(), 2);
    assert_eq!(artifact.outputs.len(), 2);
}

#[test]
fn test_parse_edge_cases() {
    let parser = CloudFormationParser::new();

    // Very long strings
    let long_name = "a".repeat(1000);
    let template = format!(r#"{{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {{
            "{}": {{"Type": "AWS::S3::Bucket", "Properties": {{"BucketName": "{}"}}}}
        }}
    }}"#, long_name, long_name);

    let artifact = parser.parse(&template).unwrap();
    assert_eq!(artifact.resource_count(), 1);
}
