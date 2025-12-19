use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::shared::models::{ChangeAction, Detection, ResourceChange, Severity};
use std::path::Path;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_detection_engine_new() {
    let engine = DetectionEngine::new();
    // Basic construction test - no assertions needed if it doesn't panic
}

#[test]
fn test_detection_engine_with_verbose() {
    let engine = DetectionEngine::new().with_verbose(true);
    // Test that verbose mode can be set
}

#[test]
fn test_detection_engine_detect_from_terraform_plan_invalid_file() {
    let engine = DetectionEngine::new();
    let result = engine.detect_from_terraform_plan(Path::new("nonexistent.json"));
    assert!(result.is_err());
}

#[test]
fn test_detection_engine_detect_from_terraform_json_invalid_json() {
    let engine = DetectionEngine::new();
    let result = engine.detect_from_terraform_json("invalid json");
    assert!(result.is_err());
}

#[test]
fn test_detection_engine_detect_from_terraform_json_empty_plan() {
    let engine = DetectionEngine::new();
    let json = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": []
    }"#;
    let result = engine.detect_from_terraform_json(json);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_detection_engine_detect_from_terraform_json_with_changes() {
    let engine = DetectionEngine::new();
    let json = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "type": "aws_instance",
                "name": "test",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": "t3.micro",
                        "ami": "ami-12345"
                    }
                }
            }
        ]
    }"#;
    let result = engine.detect_from_terraform_json(json);
    assert!(result.is_ok());
    let changes = result.unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].resource_id, "aws_instance.test");
    assert_eq!(changes[0].action, ChangeAction::Create);
}

#[test]
fn test_detection_engine_analyze_changes_empty() {
    let engine = DetectionEngine::new();
    let result = engine.analyze_changes(&[], &[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_detection_engine_analyze_changes_with_ec2() {
    let engine = DetectionEngine::new();
    let changes = vec![
        ResourceChange::builder()
            .resource_id("test-ec2".to_string())
            .resource_type("aws_instance".to_string())
            .action(ChangeAction::Create)
            .new_config(serde_json::json!({
                "instance_type": "m5.xlarge",  // Use xlarge to trigger overprovisioning detection
                "region": "us-east-1"
            }))
            .build()
    ];
    let cost_estimates = vec![("test-ec2".to_string(), 250.0, 0.9)]; // High cost to trigger detection
    let result = engine.analyze_changes(&changes, &cost_estimates);
    assert!(result.is_ok());
    let detections = result.unwrap();
    // Should detect overprovisioned EC2
    assert!(!detections.is_empty());
    assert_eq!(detections[0].rule_id, "OVERPROVISIONED_EC2");
}

#[test]
fn test_detection_engine_detect_empty_changes() {
    let engine = DetectionEngine::new();
    let result = engine.detect(&[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_detection_engine_detect_with_changes() {
    let engine = DetectionEngine::new();
    let changes = vec![
        ResourceChange::builder()
            .resource_id("test-rds".to_string())
            .resource_type("aws_db_instance".to_string())
            .action(ChangeAction::Create)
            .new_config(serde_json::json!({
                "instance_class": "db.t3.micro",
                "allocated_storage": 20
            }))
            .build()
    ];
    let cost_estimates = vec![("test-rds".to_string(), 400.0, 0.8)]; // High cost to trigger detection
    let result = engine.analyze_changes(&changes, &cost_estimates);
    assert!(result.is_ok());
    let detections = result.unwrap();
    assert!(!detections.is_empty());
    assert_eq!(detections[0].rule_id, "HIGH_COST_CHANGE");
}

#[test]
fn test_detection_engine_detect_from_file() {
    let engine = DetectionEngine::new();

    // Create a temporary file with valid Terraform plan JSON
    let json = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "type": "aws_instance",
                "name": "test",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": "t3.micro"
                    }
                }
            }
        ]
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = engine.detect_from_terraform_plan(temp_file.path());
    assert!(result.is_ok());
    let changes = result.unwrap();
    assert_eq!(changes.len(), 1);
}