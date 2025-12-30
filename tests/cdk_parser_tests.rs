use costpilot::artifact::find_cdk_templates;
use costpilot::artifact::is_cdk_output_dir;
use costpilot::artifact::ArtifactFormat;
use costpilot::artifact::ArtifactParser;
use costpilot::artifact::CdkParser;
use std::fs;
use tempfile::TempDir;

// CDK Parser Unit Tests (60 tests)

// 1-20: Manifest parsing tests

#[test]
fn test_parse_cdk_manifest_basic() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0); // No template file
}

#[test]
fn test_parse_cdk_manifest_missing_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0"
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"invalid json"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let result = parser.parse_cdk_output(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_parse_cdk_manifest_missing_version() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "artifacts": {}
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_with_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "runtime": {
            "libraries": {
                "aws-cdk-lib": "2.50.0",
                "constructs": "10.0.0"
            }
        },
        "artifacts": {}
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_non_stack_artifact() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyAsset": {
                "type": "aws:cdk:asset",
                "properties": {}
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_multiple_stacks() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "Stack1": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "Stack1.template.json"
                }
            },
            "Stack2": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "Stack2.template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0); // Templates don't exist
}

#[test]
fn test_parse_cdk_manifest_stack_without_template_file() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {}
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_empty_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {}
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_malformed_artifact() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": "not an object"
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_artifact_without_type() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "properties": {
                    "templateFile": "template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_artifact_wrong_type() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:lambda:function",
                "properties": {
                    "templateFile": "template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_with_environment() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "environment": "aws://123456789012/us-east-1",
                "properties": {
                    "templateFile": "template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "template.json",
                    "tags": {
                        "Environment": "test",
                        "Project": "myproject"
                    }
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_complex() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "21.0.0",
        "runtime": {
            "libraries": {
                "aws-cdk-lib": "2.60.0"
            }
        },
        "artifacts": {
            "StackA": {
                "type": "aws:cloudformation:stack",
                "environment": "aws://111111111111/eu-west-1",
                "properties": {
                    "templateFile": "StackA.template.json",
                    "tags": {"Team": "backend"}
                }
            },
            "Asset1": {
                "type": "aws:cdk:asset",
                "properties": {}
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{"version":"1.0.0","artifacts":{}}"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_large() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let mut artifacts = std::collections::HashMap::new();
    for i in 0..100 {
        artifacts.insert(
            format!("Stack{}", i),
            serde_json::json!({
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": format!("Stack{}.template.json", i)
                }
            }),
        );
    }
    let manifest = serde_json::json!({
        "version": "20.0.0",
        "artifacts": artifacts
    });
    fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json",
                    "tags": {
                        "Name": "测试栈"
                    }
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_manifest_deeply_nested() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json",
                    "nested": {
                        "deep": {
                            "structure": "value"
                        }
                    }
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

// 21-35: Stack extraction tests

#[test]
fn test_parse_cdk_stack_basic() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket"
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].format, ArtifactFormat::Cdk);
    assert_eq!(artifacts[0].resource_count(), 1);
}

#[test]
fn test_parse_cdk_stack_with_environment() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "environment": "aws://123456789012/us-west-2",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyInstance": {
                "Type": "AWS::EC2::Instance"
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].metadata.region, Some("us-west-2".to_string()));
}

#[test]
fn test_parse_cdk_stack_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json",
                    "tags": {
                        "Environment": "prod",
                        "Owner": "team-a"
                    }
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyDB": {
                "Type": "AWS::RDS::DBInstance"
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let tags = &artifacts[0].metadata.tags;
    assert_eq!(tags.get("cdk_tag_Environment"), Some(&"prod".to_string()));
    assert_eq!(tags.get("cdk_tag_Owner"), Some(&"team-a".to_string()));
}

#[test]
fn test_parse_cdk_stack_metadata_enhancement() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyFunction": {
                "Type": "AWS::Lambda::Function",
                "Metadata": {
                    "aws:cdk:path": "MyStack/MyFunction/Resource",
                    "aws:cdk:logicalId": "MyFunction123"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let resource = artifacts[0].get_resource("MyFunction").unwrap();
    assert!(resource.metadata.contains_key("cdk_construct_path"));
    assert!(resource.metadata.contains_key("original_logical_id"));
}

#[test]
fn test_parse_cdk_stack_yaml_template() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.yaml");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.yaml"
                }
            }
        }
    }"#;
    let template_content = r#"AWSTemplateFormatVersion: '2010-09-09'
Resources:
  MyBucket:
    Type: AWS::S3::Bucket
"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    // YAML not supported by CloudFormation parser, so no artifacts
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_stack_invalid_template() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"invalid json"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0); // Failed to parse template
}

#[test]
fn test_parse_cdk_stack_missing_template() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0);
}

#[test]
fn test_parse_cdk_stack_nested_stack() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "NestedStack": {
                "Type": "AWS::CloudFormation::Stack",
                "Properties": {
                    "TemplateURL": "https://s3.amazonaws.com/bucket/nested.json"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].resource_count(), 1);
}

#[test]
fn test_parse_cdk_stack_with_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Parameters": {
            "Environment": {
                "Type": "String",
                "Default": "dev"
            }
        },
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket"
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn test_parse_cdk_stack_with_outputs() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket"
            }
        },
        "Outputs": {
            "BucketName": {
                "Value": {"Ref": "MyBucket"},
                "Export": {"Name": "MyBucketName"}
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn test_parse_cdk_stack_complex_template() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Complex CDK Stack",
        "Parameters": {
            "VpcId": {"Type": "AWS::EC2::VPC::Id"}
        },
        "Resources": {
            "SecurityGroup": {
                "Type": "AWS::EC2::SecurityGroup",
                "Properties": {
                    "VpcId": {"Ref": "VpcId"},
                    "GroupDescription": "Security group for app"
                }
            },
            "LoadBalancer": {
                "Type": "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "Properties": {
                    "Type": "application",
                    "SecurityGroups": [{"Ref": "SecurityGroup"}]
                }
            }
        },
        "Outputs": {
            "LoadBalancerDNS": {
                "Value": {"Fn::GetAtt": ["LoadBalancer", "DNSName"]}
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].resource_count(), 2);
}

#[test]
fn test_parse_cdk_stack_empty_template() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09"
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].resource_count(), 0);
}

#[test]
fn test_parse_cdk_stack_template_with_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Metadata": {
            "CDKVersion": "2.50.0"
        },
        "Resources": {
            "MyResource": {
                "Type": "AWS::S3::Bucket"
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn test_parse_cdk_stack_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket": {
                "Type": "AWS::S3::Bucket"
            },
            "BucketPolicy": {
                "Type": "AWS::S3::BucketPolicy",
                "DependsOn": "Bucket",
                "Properties": {
                    "Bucket": {"Ref": "Bucket"}
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].resource_count(), 2);
}

// 36-45: CloudFormation delegation tests

#[test]
fn test_cdk_parser_delegates_to_cfn() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": "my-test-bucket"
                }
            }
        }
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_cdk_parser_handles_cfn_errors() {
    let template = r#"invalid json"#;

    let parser = CdkParser::new();
    let result = parser.parse(template);
    assert!(result.is_err());
}

#[test]
fn test_cdk_parser_preserves_cfn_structure() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Description": "Test template",
        "Parameters": {
            "Env": {"Type": "String"}
        },
        "Resources": {
            "Bucket": {"Type": "AWS::S3::Bucket"},
            "Instance": {"Type": "AWS::EC2::Instance"}
        },
        "Outputs": {
            "BucketName": {"Value": {"Ref": "Bucket"}}
        }
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 2);
}

#[test]
fn test_cdk_parser_with_cfn_intrinsic_functions() {
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

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_cdk_parser_with_cfn_mappings() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Mappings": {
            "RegionMap": {
                "us-east-1": {"AMI": "ami-12345"}
            }
        },
        "Resources": {
            "Instance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "ImageId": {"Fn::FindInMap": ["RegionMap", {"Ref": "AWS::Region"}, "AMI"]}
                }
            }
        }
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_cdk_parser_with_cfn_conditions() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Conditions": {
            "IsProd": {"Fn::Equals": [{"Ref": "Environment"}, "prod"]}
        },
        "Resources": {
            "Bucket": {
                "Type": "AWS::S3::Bucket",
                "Condition": "IsProd"
            }
        }
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_cdk_parser_with_cfn_transforms() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Transform": "AWS::Serverless-2016-10-31",
        "Resources": {
            "Function": {
                "Type": "AWS::Serverless::Function",
                "Properties": {
                    "Runtime": "nodejs18.x"
                }
            }
        }
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1);
}

#[test]
fn test_cdk_parser_empty_template() {
    let template = r#"{
        "AWSTemplateFormatVersion": "2010-09-09"
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 0);
}

#[test]
fn test_cdk_parser_minimal_template() {
    let template = r#"{"Resources":{}}"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(template).unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 0);
}

#[test]
fn test_cdk_parser_large_template() {
    let mut resources = serde_json::Map::new();
    for i in 0..1000 {
        resources.insert(
            format!("Resource{}", i),
            serde_json::json!({
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": format!("bucket-{}", i)
                }
            }),
        );
    }
    let template = serde_json::json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": resources
    });

    let parser = CdkParser::new();
    let artifact = parser
        .parse(&serde_json::to_string(&template).unwrap())
        .unwrap();
    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 1000);
}

// 46-50: Error handling tests

#[test]
fn test_cdk_parser_missing_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let parser = CdkParser::new();
    let result = parser.parse_cdk_output(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_cdk_parser_invalid_manifest_path() {
    let parser = CdkParser::new();
    let result = parser.parse_cdk_output("/nonexistent/path");
    assert!(result.is_err());
}

#[test]
fn test_cdk_parser_manifest_read_error() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    // Create a directory instead of a file to cause read error
    fs::create_dir(&manifest_path).unwrap();

    let parser = CdkParser::new();
    let result = parser.parse_cdk_output(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_cdk_parser_template_read_error() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();
    fs::create_dir(&template_path).unwrap(); // Directory instead of file

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 0); // Should skip the failing stack
}

#[test]
fn test_cdk_parser_assembly_metadata_missing_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let parser = CdkParser::new();
    let result = parser.parse_assembly_metadata(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

// 51-55: Metadata enhancement tests

#[test]
fn test_enhance_metadata_basic() {
    // Since enhance_with_cdk_metadata is private, test the public behavior
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Metadata": {
                    "aws:cdk:path": "MyStack/MyBucket/Resource"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let resource = artifacts[0].get_resource("MyBucket").unwrap();
    // The CDK path should be in metadata
    assert!(resource.metadata.contains_key("cdk_construct_path"));
}

#[test]
fn test_enhance_metadata_logical_id() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Metadata": {
                    "aws:cdk:logicalId": "MyBucketABC123"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let resource = artifacts[0].get_resource("MyBucket").unwrap();
    assert!(resource.metadata.contains_key("original_logical_id"));
}

#[test]
fn test_enhance_metadata_both_fields() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyFunction": {
                "Type": "AWS::Lambda::Function",
                "Metadata": {
                    "aws:cdk:path": "MyStack/MyFunction",
                    "aws:cdk:logicalId": "MyFunctionXYZ789"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let resource = artifacts[0].get_resource("MyFunction").unwrap();
    assert!(resource.metadata.contains_key("cdk_construct_path"));
    assert!(resource.metadata.contains_key("original_logical_id"));
}

#[test]
fn test_enhance_metadata_no_cdk_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "MyBucket": {
                "Type": "AWS::S3::Bucket",
                "Metadata": {
                    "custom:field": "value"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let resource = artifacts[0].get_resource("MyBucket").unwrap();
    assert!(!resource.metadata.contains_key("cdk_construct_path"));
    assert!(!resource.metadata.contains_key("original_logical_id"));
}

#[test]
fn test_enhance_metadata_multiple_resources() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let template_path = temp_dir.path().join("MyStack.template.json");

    let manifest_content = r#"{
        "version": "20.0.0",
        "artifacts": {
            "MyStack": {
                "type": "aws:cloudformation:stack",
                "properties": {
                    "templateFile": "MyStack.template.json"
                }
            }
        }
    }"#;
    let template_content = r#"{
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Bucket1": {
                "Type": "AWS::S3::Bucket",
                "Metadata": {
                    "aws:cdk:path": "Stack/Bucket1"
                }
            },
            "Bucket2": {
                "Type": "AWS::S3::Bucket",
                "Metadata": {
                    "aws:cdk:path": "Stack/Bucket2",
                    "aws:cdk:logicalId": "Bucket2ID"
                }
            }
        }
    }"#;

    fs::write(&manifest_path, manifest_content).unwrap();
    fs::write(&template_path, template_content).unwrap();

    let parser = CdkParser::new();
    let artifacts = parser
        .parse_cdk_output(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let bucket1 = artifacts[0].get_resource("Bucket1").unwrap();
    assert!(bucket1.metadata.contains_key("cdk_construct_path"));

    let bucket2 = artifacts[0].get_resource("Bucket2").unwrap();
    assert!(bucket2.metadata.contains_key("cdk_construct_path"));
    assert!(bucket2.metadata.contains_key("original_logical_id"));
}

// 56-58: Assembly metadata parsing tests

#[test]
fn test_parse_assembly_metadata_basic() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "21.0.0",
        "runtime": {
            "libraries": {
                "aws-cdk-lib": "2.60.0",
                "constructs": "10.2.0"
            }
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let metadata = parser
        .parse_assembly_metadata(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(metadata.version, "21.0.0");
    assert!(metadata.runtime.contains("aws-cdk-lib@2.60.0"));
    assert!(metadata.runtime.contains("constructs@10.2.0"));
}

#[test]
fn test_parse_assembly_metadata_no_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0"
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let metadata = parser
        .parse_assembly_metadata(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(metadata.version, "20.0.0");
    assert_eq!(metadata.runtime, "unknown");
}

#[test]
fn test_parse_assembly_metadata_empty_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_content = r#"{
        "version": "20.0.0",
        "runtime": {
            "libraries": {}
        }
    }"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let parser = CdkParser::new();
    let metadata = parser
        .parse_assembly_metadata(temp_dir.path().to_str().unwrap())
        .unwrap();
    assert_eq!(metadata.version, "20.0.0");
    assert_eq!(metadata.runtime, "");
}

// 59: Directory detection test

#[test]
fn test_is_cdk_output_dir() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    fs::write(&manifest_path, r#"{"version": "1.0.0"}"#).unwrap();

    assert!(is_cdk_output_dir(temp_dir.path().to_str().unwrap()));
}

#[test]
fn test_is_cdk_output_dir_no_manifest() {
    let temp_dir = TempDir::new().unwrap();
    assert!(!is_cdk_output_dir(temp_dir.path().to_str().unwrap()));
}

#[test]
fn test_is_cdk_output_dir_invalid_path() {
    assert!(!is_cdk_output_dir("/nonexistent/path"));
}

// 60: Template finding test

#[test]
fn test_find_cdk_templates() {
    let temp_dir = TempDir::new().unwrap();
    let template1 = temp_dir.path().join("Stack1.template.json");
    let template2 = temp_dir.path().join("Stack2.template.yaml");
    let other_file = temp_dir.path().join("not-a-template.txt");

    fs::write(&template1, "{}").unwrap();
    fs::write(&template2, "{}").unwrap();
    fs::write(&other_file, "content").unwrap();

    let templates = find_cdk_templates(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(templates.len(), 2);
    assert!(templates.contains(&template1.to_string_lossy().to_string()));
    assert!(templates.contains(&template2.to_string_lossy().to_string()));
}

#[test]
fn test_find_cdk_templates_empty_dir() {
    let temp_dir = TempDir::new().unwrap();
    let templates = find_cdk_templates(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(templates.len(), 0);
}

#[test]
fn test_find_cdk_templates_invalid_dir() {
    let result = find_cdk_templates("/nonexistent/path");
    assert!(result.is_err());
}

// 61-62: CDK diff parsing tests

#[test]
fn test_parse_cdk_diff_json() {
    let diff_json = r#"{
        "success": true,
        "stacks": [
            {
                "stack_name": "MyStack",
                "changes": [
                    {
                        "logical_id": "MyBucket",
                        "resource_type": "AWS::S3::Bucket",
                        "change_type": "create",
                        "new_values": {
                            "BucketName": "my-test-bucket"
                        }
                    }
                ]
            }
        ]
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(diff_json).unwrap();

    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resources.len(), 1);

    // Check the created bucket
    let bucket = &artifact.resources[0];
    assert_eq!(bucket.id, "MyBucket");
    assert_eq!(bucket.resource_type, "AWS::S3::Bucket");
    assert_eq!(
        bucket
            .properties
            .get("BucketName")
            .unwrap()
            .as_str()
            .unwrap(),
        "my-test-bucket"
    );
}

#[test]
fn test_parse_cdk_diff_empty() {
    let diff_json = r#"{
        "success": true,
        "stacks": [],
        "error": null
    }"#;

    let parser = CdkParser::new();
    let artifact = parser.parse(diff_json).unwrap();

    assert_eq!(artifact.format, ArtifactFormat::Cdk);
    assert_eq!(artifact.resource_count(), 0);
}

#[test]
fn test_parse_cdk_diff_invalid_json() {
    let invalid_json = r#"invalid json"#;

    let parser = CdkParser::new();
    let result = parser.parse(invalid_json);
    assert!(result.is_err());
}
