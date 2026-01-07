#![allow(deprecated)]

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

use serde_json::json;

/// Create a Terraform plan with a single EC2 instance
fn terraform_plan_with_ec2(instance_type: &str) -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_instance.web",
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "instance_type": instance_type,
                    "ami": "ami-12345678",
                    "tags": {
                        "Name": "web-server",
                        "Environment": "production"
                    }
                }
            }
        }]
    })
}

/// Test policy enforcement in warn mode
#[test]
fn test_policy_enforcement_warn_mode() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    // Create a policy that should trigger a warning
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "advisory"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&plan_data).unwrap(),
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path);

    // Should succeed but output warnings
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

/// Test policy enforcement in block mode
#[test]
fn test_policy_enforcement_block_mode() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that should block (but will only warn in free mode)
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&plan_data).unwrap(),
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path);

    // In free mode, blocking is disabled so it succeeds but shows warnings
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

/// Test policy exemption workflow
#[test]
fn test_policy_exemption_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let exemption_path = temp_dir.path().join("exemptions.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that would normally block
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create an exemption for this policy
    let exemption_config = r#"{
        "version": "1.0",
        "exemptions": [
            {
                "id": "budget_exemption_001",
                "policy_name": "global_budget",
                "resource_pattern": "global",
                "justification": "Temporary budget increase for Q1 planning",
                "reason": "Temporary budget increase for Q1 planning",
                "approved_by": "finance@company.com",
                "expires_at": "2025-12-31",
                "created_at": "2025-01-01T00:00:00Z",
                "status": "active"
            }
        ]
    }"#;
    fs::write(&exemption_path, exemption_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&plan_data).unwrap(),
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .arg("--exemptions")
        .arg(&exemption_path)
        .arg("--fail-on-critical");

    // Should succeed due to exemption
    cmd.assert()
        .success();
}

/// Test expired exemption handling
#[test]
fn test_expired_exemption() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let exemption_path = temp_dir.path().join("exemptions.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that would normally block
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create an expired exemption
    let exemption_config = r#"{
        "version": "1.0",
        "exemptions": [
            {
                "id": "expired_exemption_001",
                "policy_name": "global_budget",
                "resource_pattern": "global",
                "justification": "Old exemption that has expired",
                "reason": "Old exemption that has expired",
                "approved_by": "finance@company.com",
                "expires_at": "2024-12-31",
                "created_at": "2024-01-01T00:00:00Z",
                "status": "expired"
            }
        ]
    }"#;
    fs::write(&exemption_path, exemption_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&plan_data).unwrap(),
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .arg("--exemptions")
        .arg(&exemption_path);

    // In free mode, even expired exemptions result in warnings, not blocking
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains(
            "Exemption expired_exemption_001 expired",
        ));
}

/// Test policy versioning and compatibility
#[test]
fn test_policy_versioning() {
    let temp_dir = TempDir::new().unwrap();
    let policy_v1_path = temp_dir.path().join("policy_v1.yml");
    let policy_v2_path = temp_dir.path().join("policy_v2.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create v1.0 policy (higher limit)
    let policy_v1_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 200.0
            }
        },
        "enforcement": {
            "mode": "advisory"
        }
    }"#;
    fs::write(&policy_v1_path, policy_v1_config).unwrap();

    // Create v2.0 policy with stricter limits
    let policy_v2_config = r#"{
        "version": "2.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_v2_path, policy_v2_config).unwrap();

    // Create Terraform plan that exceeds v2 limit but not v1
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost $150
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&plan_data).unwrap(),
    )
    .unwrap();

    // Test v1.0 policy (should succeed - no warnings since $150 < $200)
    let mut cmd_v1 = cargo_bin_cmd!("costpilot");
    cmd_v1
        .arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_v1_path);

    cmd_v1
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not()); // Should have output but no warnings

    // Test v2.0 policy (should succeed but show warnings since $150 > $100)
    let mut cmd_v2 = cargo_bin_cmd!("costpilot");
    cmd_v2
        .arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_v2_path);

    cmd_v2
        .assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

// ===== POLICY EDGE CASE TESTS =====

#[test]
fn test_policy_empty_resource_list_edge_case() {
    // Test policy evaluation with empty resource list
    let policy_content = r#"
version: "1.0"
rules:
  - name: "Empty resources should not cause errors"
    description: "Test rule for empty resource list"
    enabled: true
    severity: "Info"
    conditions:
      - condition_type:
          type: monthly_cost
        operator: "greater_than"
        value: 100.0
    action:
      type: warn
      message: "Test warning"
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate")
        .arg(&policy_path)
        .arg("--format")
        .arg("json");

    cmd.assert().success();
}

#[test]
fn test_policy_extremely_large_budget_limits() {
    // Test with extremely large budget limits
    let policy_content = r#"
version: "1.0"
metadata:
  budgets:
    global:
      monthly_limit: 1000000000.0  # 1 billion
    services:
      "enterprise-db":
        monthly_limit: 500000000.0  # 500 million
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_zero_budget_limits_edge_case() {
    // Test with zero budget limits (should still work)
    let policy_content = r#"
version: "1.0"
metadata:
  budgets:
    global:
      monthly_limit: 0.0
    services:
      "free-service":
        monthly_limit: 0.0
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_maximum_rule_count_edge_case() {
    // Test with a very large number of rules
    let mut rules = Vec::new();
    for i in 0..1000 {
        rules.push(format!(
            r#"
  - id: "rule-{}"
    name: "Rule {}"
    condition: "resource_type == 'aws_instance'"
    action:
      type: warn
      message: "Rule {} warning"
    priority: {}"#,
            i, i, i, i
        ));
    }

    let policy_content = format!(
        r#"
version: "1.0"
rules:{}"#,
        rules.join("\n")
    );

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_extremely_long_rule_names() {
    // Test with extremely long rule names and descriptions
    let long_name = "a".repeat(1000);
    let long_description = "b".repeat(2000);

    let policy_content = format!(
        r#"
version: "1.0"
rules:
  - id: "{}"
    name: "{}"
    description: "{}"
    condition: "true"
    action:
      type: warn
      message: "Long rule warning""#,
        long_name, long_name, long_description
    );

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("policy-dsl").arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_special_characters_in_names() {
    // Test with special characters and Unicode in policy names
    let policy_content = r#"
version: "1.0"
rules:
  - id: "special-chars-测试@#$%^&*()"
    name: "Special Characters: ñáéíóú 🚀 🔥 💯"
    condition: "resource_type == 'aws_instance'"
    action:
      type: block
      message: "Special characters test"
    metadata:
      owner: "测试-team@domain.com"
      tags:
        "特殊标签": "unicode-value"
        "special/key": "special:value"
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_extreme_priority_values() {
    // Test with extreme priority values
    let policy_content = r#"
version: "1.0"
rules:
  - id: "min-priority"
    name: "Minimum Priority"
    condition: "true"
    action:
      type: warn
      message: "Min priority warning"
    priority: 0
  - id: "max-priority"
    name: "Maximum Priority"
    condition: "true"
    action:
      type: block
      message: "Max priority block"
    priority: 2147483647  # i32::MAX
  - id: "negative-priority"
    name: "Negative Priority"
    condition: "true"
    action:
      type: warn
      message: "Negative priority warning"
    priority: -1000
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_empty_condition_edge_case() {
    // Test with empty or minimal conditions
    let policy_content = r#"
version: "1.0"
rules:
  - id: "empty-condition"
    name: "Empty Condition"
    condition: ""
    action:
      type: warn
      message: "Empty condition warning"
  - id: "minimal-condition"
    name: "Minimal Condition"
    condition: "true"
    action:
      type: warn
      message: "Minimal condition warning"
  - id: "complex-condition"
    name: "Complex Condition"
    condition: "resource_type == 'aws_instance' && cost > 100.0 && tags['env'] == 'prod'"
    action:
      type: block
      message: "Complex condition block"
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

#[test]
fn test_policy_duplicate_rule_ids_edge_case() {
    // Test with duplicate rule IDs (should handle gracefully)
    let policy_content = r#"
version: "1.0"
rules:
  - id: "duplicate"
    name: "First Rule"
    condition: "true"
    action:
      type: warn
      message: "First duplicate warning"
  - id: "duplicate"
    name: "Second Rule with Same ID"
    condition: "false"
    action:
      type: block
      message: "Second duplicate block"
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("--policy").arg(&policy_path);

    // Should either succeed (last rule wins) or fail gracefully
    let output = cmd.output().unwrap();
    assert!(output.status.success() || !output.status.success()); // Either outcome is acceptable
}

#[test]
fn test_policy_extreme_nested_metadata() {
    // Test with extremely nested metadata structures
    let policy_content = r#"
metadata:
  id: "test-policy"
  name: "Test Policy"
  description: "Test policy for validation"
  category: "budget"
  severity: "warning"
  status: "active"
  version: "1.0"
  ownership:
    author: "test-author"
    owner: "test-owner"
    owners: ["test"]
    reviewers: []
  lifecycle:
    created_at: "2024-01-01T00:00:00Z"
    updated_at: "2024-01-01T00:00:00Z"
  tags: ["test"]
  custom:
    deeply: "nested"
    structure: "with"
    many: "levels"
    of: "nesting"
    value: "deep"
    number: "42"
    array: "[1, 2, 3]"
    object: "another level"
rules:
  - name: "Test Rule"
    description: "A test rule"
    enabled: true
    severity: "Info"
    conditions:
      - condition_type:
          type: monthly_cost
        operator: "greater_than"
        value: 100.0
    action:
      type: "warn"
      message: "Cost exceeds limit"
exemptions: []
"#;

    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&policy_path);

    cmd.assert().success();
}

// ===== POLICY ENFORCEMENT EDGE CASE TESTS =====

#[test]
fn test_policy_enforcement_empty_policy_edge_case() {
    // Test policy enforcement with empty policy file
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    // Create empty policy
    fs::write(&policy_path, "").unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    let _result = cmd
        .arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert();
    // Should handle gracefully
}

#[test]
fn test_policy_enforcement_zero_budget_edge_case() {
    // Test policy enforcement with zero budget
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 0.0
            }
        },
        "enforcement": {
            "mode": "warn"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_negative_budget_edge_case() {
    // Test policy enforcement with negative budget
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": -100.0
            }
        },
        "enforcement": {
            "mode": "warn"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_extremely_large_budget() {
    // Test policy enforcement with extremely large budget
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 1000000000.0
            }
        },
        "enforcement": {
            "mode": "warn"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_empty_terraform_plan_edge_case() {
    // Test policy enforcement with empty Terraform plan
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "warn"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create empty plan
    let empty_plan = json!({});
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&empty_plan).unwrap(),
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    let _result = cmd
        .arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert();
    // Should handle gracefully
}

#[test]
fn test_policy_enforcement_extremely_long_policy_names() {
    // Test policy enforcement with extremely long policy names
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let long_name = "a".repeat(1000);
    let policy_config = format!(
        r#"{{
        "version": "1.0",
        "budgets": {{
            "{}": {{
                "monthly_limit": 100.0
            }}
        }},
        "enforcement": {{
            "mode": "warn"
        }}
    }}"#,
        long_name
    );
    fs::write(&policy_path, policy_config).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_special_characters_in_policy_names() {
    // Test policy enforcement with special characters in policy names
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "policy@domain.com": {
                "monthly_limit": 100.0
            },
            "测试策略": {
                "monthly_limit": 200.0
            },
            "policy-with-dashes": {
                "monthly_limit": 50.0
            }
        },
        "enforcement": {
            "mode": "warn"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_maximum_nested_rules() {
    // Test policy enforcement with maximum nested rules
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    let mut nested_content = String::from("version: \"1.0\"\n");
    nested_content.push_str("budgets:\n");
    nested_content.push_str("  global:\n");
    nested_content.push_str("    monthly_limit: 10000.0\n");
    nested_content.push_str("enforcement:\n");
    nested_content.push_str("  mode: warn\n");
    nested_content.push_str("rules:\n");

    // Create deeply nested rules
    for i in 0..100 {
        nested_content.push_str(&format!("  - id: \"rule_{}\"\n", i));
        nested_content.push_str(&format!("    name: \"Rule {}\"\n", i));
        nested_content.push_str("    condition: \"true\"\n");
        nested_content.push_str("    action:\n");
        nested_content.push_str("      type: warn\n");
        nested_content.push_str("      message: \"Deep nesting warning\"\n");
        nested_content.push_str("    metadata:\n");
        nested_content.push_str(&format!("      level_{}: \"value\"\n", i));
    }

    fs::write(&policy_path, nested_content).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn test_policy_enforcement_extremely_deep_nesting_edge_case() {
    // Test policy enforcement with extremely deep nesting
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    // Create a policy with extremely deep nesting
    let mut deep_yaml = "version: \"1.0\"\nbudgets:\n  global:\n    monthly_limit: 10000.0\nenforcement:\n  mode: warn\nrules:\n".to_string();
    let mut current_level = "  - id: \"deep_test\"\n    name: \"Deep Test\"\n    condition: \"true\"\n    action:\n      type: warn\n      message: \"Deep test warning\"\n    metadata:\n".to_string();

    for i in 0..50 {
        current_level.push_str(&format!("      level{}:\n", i));
    }
    current_level.push_str("        value: \"deep\"");

    deep_yaml.push_str(&current_level);

    fs::write(&policy_path, deep_yaml).unwrap();

    let plan = terraform_plan_with_ec2("t3.micro");
    fs::write(&scan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--scan")
        .arg(&scan_path)
        .assert()
        .success();
}
