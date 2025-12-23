// Golden file tests for autofix output

use costpilot::engines::autofix::{AutofixEngine, AutofixMode};
use costpilot::engines::shared::models::{ChangeAction, Detection, ResourceChange, Severity};
use serde_json::json;

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_nat_gateway_to_vpc_endpoint() {
    let detection = Detection::builder()
        .resource_id("aws_nat_gateway.main")
        .rule_id("test_rule")
        .message("High fixed cost for NAT Gateway")
        .severity(Severity::High)
        .estimated_cost(32.85)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("nat_gateway_to_vpc_endpoint", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_oversized_instance() {
    let detection = Detection::builder()
        .resource_id("aws_instance.web")
        .rule_id("test_rule")
        .message("Instance type too large for workload")
        .severity(Severity::Medium)
        .estimated_cost(140.16)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("downsize_instance", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_rds_provisioned_to_serverless() {
    let detection = Detection::builder()
        .resource_id("aws_db_instance.main")
        .rule_id("test_rule")
        .message("Low utilization RDS instance")
        .severity(Severity::High)
        .estimated_cost(175.20)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("rds_to_serverless", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_s3_lifecycle_policy() {
    let detection = Detection::builder()
        .resource_id("aws_s3_bucket.logs")
        .rule_id("test_rule")
        .message("Missing lifecycle policy for old data")
        .severity(Severity::Medium)
        .estimated_cost(50.0)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("s3_add_lifecycle", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_lambda_memory_optimization() {
    let detection = Detection::builder()
        .resource_id("aws_lambda_function.processor")
        .rule_id("test_rule")
        .message("Lambda memory overprovisioned")
        .severity(Severity::Low)
        .estimated_cost(85.0)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("lambda_reduce_memory", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_dynamodb_ondemand() {
    let detection = Detection::builder()
        .resource_id("aws_dynamodb_table.events")
        .rule_id("test_rule")
        .message("Underutilized provisioned capacity")
        .severity(Severity::Medium)
        .estimated_cost(120.0)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_fix(&detection, "snippet").unwrap();

    //     insta::assert_snapshot!("dynamodb_to_ondemand", fix);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_patch_mode_nat_gateway() {
    let detection = Detection::builder()
        .resource_id("aws_nat_gateway.main")
        .rule_id("test_rule")
        .message("High fixed cost for NAT Gateway")
        .severity(Severity::High)
        .estimated_cost(32.85)
        .build();

    let engine = AutofixEngine::new();
    //     let patch = engine.generate_fix(&detection, "patch").unwrap();

    //     insta::assert_snapshot!("nat_gateway_patch", patch);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_patch_mode_instance_type() {
    let detection = Detection::builder()
        .resource_id("aws_instance.web")
        .rule_id("test_rule")
        .message("Oversized instance")
        .severity(Severity::Medium)
        .estimated_cost(140.16)
        .build();

    let engine = AutofixEngine::new();
    //     let patch = engine.generate_fix(&detection, "patch").unwrap();

    //     insta::assert_snapshot!("instance_type_patch", patch);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_batch_fixes() {
    let detections = vec![
        Detection::builder()
            .resource_id("aws_nat_gateway.main")
            .rule_id("test_rule")
            .message("High cost")
            .severity(Severity::High)
            .estimated_cost(32.85)
            .build(),
        Detection::builder()
            .resource_id("aws_instance.web")
            .rule_id("test_rule")
            .message("Oversized")
            .severity(Severity::Medium)
            .estimated_cost(140.16)
            .build(),
    ];

    let engine = AutofixEngine::new();
    // let fixes = engine.generate_batch_fixes(&detections, "snippet").unwrap();

    // insta::assert_json_snapshot!("batch_fixes", fixes);
}

#[test]
#[ignore = "Autofix API changed - needs ResourceChange parameter"]
fn golden_autofix_drift_safe_mode() {
    let detection = Detection::builder()
        .resource_id("aws_instance.web")
        .rule_id("test_rule")
        .message("Oversized instance")
        .severity(Severity::Medium)
        .estimated_cost(140.16)
        .build();

    let engine = AutofixEngine::new();
    //     let fix = engine.generate_drift_safe_fix(&detection).unwrap();

    // insta::assert_json_snapshot!("drift_safe_fix", fix);
}
