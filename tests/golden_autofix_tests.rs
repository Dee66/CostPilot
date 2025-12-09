// Golden file tests for autofix output

use costpilot::engines::autofix::AutofixEngine;
use costpilot::engines::shared::models::Detection;

#[test]
fn golden_autofix_nat_gateway_to_vpc_endpoint() {
    let detection = Detection {
        resource_id: "aws_nat_gateway.main".to_string(),
        resource_type: "aws_nat_gateway".to_string(),
        issue: "High fixed cost for NAT Gateway".to_string(),
        severity: 8.5,
        confidence: 0.95,
        monthly_cost: 32.85,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("nat_gateway_to_vpc_endpoint", fix);
}

#[test]
fn golden_autofix_oversized_instance() {
    let detection = Detection {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        issue: "Instance type too large for workload".to_string(),
        severity: 6.5,
        confidence: 0.80,
        monthly_cost: 140.16,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("downsize_instance", fix);
}

#[test]
fn golden_autofix_rds_provisioned_to_serverless() {
    let detection = Detection {
        resource_id: "aws_db_instance.main".to_string(),
        resource_type: "aws_db_instance".to_string(),
        issue: "Low utilization RDS instance".to_string(),
        severity: 7.0,
        confidence: 0.85,
        monthly_cost: 175.20,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("rds_to_serverless", fix);
}

#[test]
fn golden_autofix_s3_lifecycle_policy() {
    let detection = Detection {
        resource_id: "aws_s3_bucket.logs".to_string(),
        resource_type: "aws_s3_bucket".to_string(),
        issue: "Missing lifecycle policy for old data".to_string(),
        severity: 5.5,
        confidence: 0.90,
        monthly_cost: 50.0,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("s3_add_lifecycle", fix);
}

#[test]
fn golden_autofix_lambda_memory_optimization() {
    let detection = Detection {
        resource_id: "aws_lambda_function.processor".to_string(),
        resource_type: "aws_lambda_function".to_string(),
        issue: "Lambda memory overprovisioned".to_string(),
        severity: 4.5,
        confidence: 0.75,
        monthly_cost: 85.0,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("lambda_reduce_memory", fix);
}

#[test]
fn golden_autofix_dynamodb_ondemand() {
    let detection = Detection {
        resource_id: "aws_dynamodb_table.events".to_string(),
        resource_type: "aws_dynamodb_table".to_string(),
        issue: "Underutilized provisioned capacity".to_string(),
        severity: 6.0,
        confidence: 0.82,
        monthly_cost: 120.0,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_fix(&detection, "snippet").unwrap();

    insta::assert_snapshot!("dynamodb_to_ondemand", fix);
}

#[test]
fn golden_autofix_patch_mode_nat_gateway() {
    let detection = Detection {
        resource_id: "aws_nat_gateway.main".to_string(),
        resource_type: "aws_nat_gateway".to_string(),
        issue: "High fixed cost for NAT Gateway".to_string(),
        severity: 8.5,
        confidence: 0.95,
        monthly_cost: 32.85,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let patch = engine.generate_fix(&detection, "patch").unwrap();

    insta::assert_snapshot!("nat_gateway_patch", patch);
}

#[test]
fn golden_autofix_patch_mode_instance_type() {
    let detection = Detection {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        issue: "Oversized instance".to_string(),
        severity: 6.5,
        confidence: 0.80,
        monthly_cost: 140.16,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let patch = engine.generate_fix(&detection, "patch").unwrap();

    insta::assert_snapshot!("instance_type_patch", patch);
}

#[test]
fn golden_autofix_batch_fixes() {
    let detections = vec![
        Detection {
            resource_id: "aws_nat_gateway.main".to_string(),
            resource_type: "aws_nat_gateway".to_string(),
            issue: "High cost".to_string(),
            severity: 8.5,
            confidence: 0.95,
            monthly_cost: 32.85,
            fix_snippet: None,
        },
        Detection {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            issue: "Oversized".to_string(),
            severity: 6.5,
            confidence: 0.80,
            monthly_cost: 140.16,
            fix_snippet: None,
        },
    ];

    let engine = AutofixEngine::new();
    let fixes = engine.generate_batch_fixes(&detections, "snippet").unwrap();

    insta::assert_json_snapshot!("batch_fixes", fixes);
}

#[test]
fn golden_autofix_drift_safe_mode() {
    let detection = Detection {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        issue: "Oversized instance".to_string(),
        severity: 6.5,
        confidence: 0.80,
        monthly_cost: 140.16,
        fix_snippet: None,
    };

    let engine = AutofixEngine::new();
    let fix = engine.generate_drift_safe_fix(&detection).unwrap();

    insta::assert_json_snapshot!("drift_safe_fix", fix);
}
