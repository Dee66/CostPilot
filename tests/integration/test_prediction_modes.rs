// Prediction mode integration tests
// Tests that verify scan command uses appropriate prediction modes based on edition

use assert_cmd::cargo::cargo_bin_cmd;
use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ResourceChange, ChangeAction};
use costpilot::edition::EditionContext;
use serde_json;
use std::fs;
use tempfile::TempDir;

/// Test that free tier scan uses static prediction with zero costs
#[test]
fn test_free_tier_scan_uses_static_prediction() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan with multiple expensive resources
    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.large",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "large",
                        "values": {
                            "instance_type": "t3.2xlarge"
                        }
                    },
                    {
                        "address": "aws_rds_instance.db",
                        "mode": "managed",
                        "type": "aws_db_instance",
                        "name": "db",
                        "values": {
                            "instance_class": "db.r5.large"
                        }
                    }
                ]
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.large",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.2xlarge"
                    }
                }
            },
            {
                "address": "aws_rds_instance.db",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_class": "db.r5.large"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path)
        .arg("--format")
        .arg("json");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Free tier should show $0.00 for all costs
    assert_eq!(json["summary"]["total_monthly_cost"], 0.0);

    // Should have 2 resources analyzed
    assert_eq!(json["summary"]["resources_analyzed"], 2);

    // Each resource should have $0.00 cost
    let estimates = json["estimates"].as_array().unwrap();
    for estimate in estimates {
        assert_eq!(estimate["monthly_cost"], 0.0);
        assert_eq!(estimate["confidence_score"], 0.0);
    }
}

/// Test that PredictionEngine::predict_static returns zero costs
#[test]
fn test_prediction_engine_static_returns_zero_costs() {
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({"instance_type": "t3.2xlarge"})),
            tags: Default::default(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_db_instance.test".to_string(),
            resource_type: "aws_db_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({"instance_class": "db.r5.large"})),
            tags: Default::default(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
    ];

    let result = PredictionEngine::predict_static(&changes);
    assert!(result.is_ok());

    let estimates = result.unwrap();
    assert_eq!(estimates.len(), 2);

    for estimate in estimates {
        // Free tier returns zero costs
        assert_eq!(estimate.monthly_cost, 0.0);
        assert_eq!(estimate.confidence_score, 0.0);
        assert_eq!(estimate.prediction_interval_low, 0.0);
        assert_eq!(estimate.prediction_interval_high, 0.0);
        assert_eq!(estimate.heuristic_reference, Some("free_static".to_string()));
    }
}

/// Test that scan command indicates prediction mode in output
#[test]
fn test_scan_indicates_prediction_mode() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.test",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "test",
                        "values": {
                            "instance_type": "t3.micro"
                        }
                    }
                ]
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.micro"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path)
        .arg("--format")
        .arg("json");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should indicate static prediction mode
    let prediction = &json["prediction"];
    assert!(prediction["mode"].as_str().unwrap().contains("static") ||
            prediction["engine"].as_str().unwrap().contains("static"));
}

/// Test that free tier scan output includes upgrade messaging
#[test]
fn test_free_tier_scan_includes_upgrade_messaging() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.expensive",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "expensive",
                        "values": {
                            "instance_type": "t3.2xlarge"
                        }
                    }
                ]
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.expensive",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.2xlarge"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path);

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should mention free tier limitations or upgrade
    assert!(stdout.contains("Free") ||
            stdout.contains("static") ||
            stdout.contains("$0.00") ||
            stdout.contains("upgrade"));
}

/// Test that scan with policy shows enforcement downgrade in free tier
#[test]
fn test_scan_policy_enforcement_downgrade() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");
    let policy_path = temp_dir.path().join("policy.yaml");

    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.large",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "large",
                        "values": {
                            "instance_type": "t3.2xlarge"
                        }
                    }
                ]
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.large",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.2xlarge"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let policy_content = r#"version: "1.0"
rules:
  - id: large-instance-check
    severity: critical
    resource_type: aws_instance
    condition: "instance_type == 't3.2xlarge'"
    message: "Large instance detected - requires approval"
"#;
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path)
        .arg("--policy")
        .arg(policy_path);

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // In free tier, critical violations should become warnings
    assert!(stdout.contains("warning") || stdout.contains("Free edition"));
    assert!(!stdout.contains("FAILED") || stdout.contains("warning"));
}</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/tests/integration/test_prediction_modes.rs
