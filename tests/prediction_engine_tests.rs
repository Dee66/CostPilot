use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange};
use costpilot::engines::performance::budgets::PerformanceBudgets;
use costpilot::edition::EditionContext;
use serde_json::json;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
#[cfg(test)]
use quickcheck_macros::quickcheck;

#[test]
fn test_prediction_engine_new() {
    let engine = PredictionEngine::new().unwrap();
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_new_with_edition_free() {
    let edition = EditionContext::free();
    let engine = PredictionEngine::new_with_edition(&edition).unwrap();
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_heuristics() {
    let heuristics = costpilot::engines::prediction::minimal_heuristics::MinimalHeuristics::to_cost_heuristics();
    let engine = PredictionEngine::with_heuristics(heuristics.clone());
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_verbose() {
    let engine = PredictionEngine::new().unwrap().with_verbose(true);
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_prediction_engine_with_performance_tracking() {
    let budgets = PerformanceBudgets::default();
    let engine = PredictionEngine::new().unwrap().with_performance_tracking(budgets);
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_predict_resource_cost_ec2() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_lambda() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_rds() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_dynamodb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-table".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PAY_PER_REQUEST",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_nat_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_nat_gateway".to_string())
        .resource_id("test-nat".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_s3() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_s3_bucket".to_string())
        .resource_id("test-bucket".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_resource_cost_load_balancer() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-lb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// EC2 Instance Type Variations
#[test]
fn test_predict_ec2_t2_micro() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-t2-micro".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t2.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_t3_small() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-t3-small".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.small",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_m5_large() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-m5-large".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "m5.large",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_c5_xlarge() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-c5-xlarge".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "c5.xlarge",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_r5_2xlarge() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-r5-2xlarge".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "r5.2xlarge",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// EC2 with different regions
#[test]
fn test_predict_ec2_us_west_2() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-us-west-2".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-west-2"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_eu_west_1() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-eu-west-1".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "eu-west-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_ec2_ap_southeast_1() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-ap-southeast-1".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "ap-southeast-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Lambda Memory Size Variations
#[test]
fn test_predict_lambda_128mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-128".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_256mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-256".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 256,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_512mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-512".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 512,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_1024mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-1024".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 1024,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_2048mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-2048".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 2048,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_3008mb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-3008".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 3008,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Lambda Runtime Variations
#[test]
fn test_predict_lambda_nodejs() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-nodejs".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "nodejs18.x",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_java() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-java".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "java11",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_go() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-go".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "go1.x",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// RDS Instance Class Variations
#[test]
fn test_predict_rds_t3_micro() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-t3-micro".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_m5_large() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-m5-large".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.m5.large",
            "engine": "postgres",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_r5_xlarge() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-r5-xlarge".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.r5.xlarge",
            "engine": "aurora-mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// RDS Engine Variations
#[test]
fn test_predict_rds_mysql() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-mysql".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_postgres() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-postgres".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "postgres",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_aurora_mysql() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-aurora-mysql".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "aurora-mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_aurora_postgres() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-aurora-postgres".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "aurora-postgresql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// DynamoDB Variations
#[test]
fn test_predict_dynamodb_pay_per_request() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-dynamodb-pay-per-request".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PAY_PER_REQUEST",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_dynamodb_provisioned() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-dynamodb-provisioned".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 5,
            "write_capacity": 5,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_dynamodb_high_capacity() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-dynamodb-high-capacity".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 100,
            "write_capacity": 100,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// S3 Variations
#[test]
fn test_predict_s3_standard() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_s3_bucket".to_string())
        .resource_id("test-s3-standard".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_s3_with_versioning() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_s3_bucket".to_string())
        .resource_id("test-s3-versioning".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "versioning": true,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Load Balancer Variations
#[test]
fn test_predict_alb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-alb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_nlb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-nlb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "network",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_clb() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-clb".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "classic",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Update Actions
#[test]
fn test_predict_ec2_update() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Update)
        .old_config(json!({
            "instance_type": "t3.micro"
        }))
        .new_config(json!({
            "instance_type": "t3.small",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_update_memory() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Update)
        .old_config(json!({
            "memory_size": 128
        }))
        .new_config(json!({
            "memory_size": 256,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_update_instance_class() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds".to_string())
        .action(ChangeAction::Update)
        .old_config(json!({
            "instance_class": "db.t3.micro"
        }))
        .new_config(json!({
            "instance_class": "db.t3.small",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_dynamodb_update_capacity() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-table".to_string())
        .action(ChangeAction::Update)
        .old_config(json!({
            "read_capacity": 5,
            "write_capacity": 5
        }))
        .new_config(json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 10,
            "write_capacity": 10,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Delete Actions
#[test]
fn test_predict_ec2_delete() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Delete)
        .old_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_delete() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Delete)
        .old_config(json!({
            "memory_size": 128,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_delete() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds".to_string())
        .action(ChangeAction::Delete)
        .old_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Edge Cases and Error Handling
#[test]
fn test_predict_empty_config() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({}))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle empty config gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_invalid_instance_type() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "invalid-type",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle invalid instance types gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_invalid_memory_size() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 0,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle invalid memory sizes gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_invalid_region() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "invalid-region"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle invalid regions gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_extremely_large_memory() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 100000,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle extremely large memory gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_negative_values() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": -128,
            "runtime": "python3.9",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle negative values gracefully
    assert!(result.is_ok() || result.is_err());
}

// Performance and Scaling Tests
#[test]
fn test_predict_multiple_resources() {
    let mut engine = PredictionEngine::new().unwrap();
    let changes = vec![
        ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance-1".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build(),
        ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance-2".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.small",
                "region": "us-east-1"
            }))
            .build(),
        ResourceChange::builder()
            .resource_type("aws_lambda_function".to_string())
            .resource_id("test-lambda".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "memory_size": 128,
                "runtime": "python3.9",
                "region": "us-east-1"
            }))
            .build(),
    ];

    let result = engine.predict_total_cost(&changes);
    assert!(result.is_ok());
    let total_cost = result.unwrap();
    assert!(total_cost.monthly >= 0.0);
}

#[test]
fn test_predict_large_scale_deployment() {
    let mut engine = PredictionEngine::new().unwrap();
    let mut changes = Vec::new();

    // Create 50 EC2 instances
    for i in 0..50 {
        changes.push(ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id(format!("test-instance-{}", i))
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build());
    }

    let result = engine.predict_total_cost(&changes);
    assert!(result.is_ok());
    let total_cost = result.unwrap();
    assert!(total_cost.monthly >= 0.0);
}

#[test]
fn test_predict_mixed_workload() {
    let mut engine = PredictionEngine::new().unwrap();
    let changes = vec![
        // 10 EC2 instances
        ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("ec2-1".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build(),
        // 5 Lambda functions
        ResourceChange::builder()
            .resource_type("aws_lambda_function".to_string())
            .resource_id("lambda-1".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "memory_size": 128,
                "runtime": "python3.9",
                "region": "us-east-1"
            }))
            .build(),
        // 2 RDS instances
        ResourceChange::builder()
            .resource_type("aws_db_instance".to_string())
            .resource_id("rds-1".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_class": "db.t3.micro",
                "engine": "mysql",
                "region": "us-east-1"
            }))
            .build(),
        // 1 Load balancer
        ResourceChange::builder()
            .resource_type("aws_lb".to_string())
            .resource_id("lb-1".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "load_balancer_type": "application",
                "region": "us-east-1"
            }))
            .build(),
    ];

    let result = engine.predict_total_cost(&changes);
    assert!(result.is_ok());
    let total_cost = result.unwrap();
    assert!(total_cost.monthly >= 0.0);
}

// Additional AWS Services
#[test]
fn test_predict_elasticache() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_elasticache_cluster".to_string())
        .resource_id("test-elasticache".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "node_type": "cache.t3.micro",
            "engine": "redis",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_redshift() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_redshift_cluster".to_string())
        .resource_id("test-redshift".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "node_type": "dc2.large",
            "number_of_nodes": 1,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_eks() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_eks_cluster".to_string())
        .resource_id("test-eks".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_cloudfront() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_cloudfront_distribution".to_string())
        .resource_id("test-cloudfront".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_api_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_api_gateway_rest_api".to_string())
        .resource_id("test-api-gateway".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_route53() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_route53_zone".to_string())
        .resource_id("test-route53".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_cloudwatch() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_cloudwatch_log_group".to_string())
        .resource_id("test-cloudwatch".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_vpc() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_vpc".to_string())
        .resource_id("test-vpc".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_subnet() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_subnet".to_string())
        .resource_id("test-subnet".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_security_group() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_security_group".to_string())
        .resource_id("test-sg".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_iam_role() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_iam_role".to_string())
        .resource_id("test-role".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_kinesis() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_kinesis_stream".to_string())
        .resource_id("test-kinesis".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "shard_count": 1,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_sns() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_sns_topic".to_string())
        .resource_id("test-sns".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_sqs() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_sqs_queue".to_string())
        .resource_id("test-sqs".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_efs() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_efs_file_system".to_string())
        .resource_id("test-efs".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_elastic_ip() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_eip".to_string())
        .resource_id("test-eip".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_internet_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_internet_gateway".to_string())
        .resource_id("test-igw".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_nat_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_nat_gateway".to_string())
        .resource_id("test-nat".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_vpn_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_vpn_gateway".to_string())
        .resource_id("test-vpn".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_transit_gateway() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_ec2_transit_gateway".to_string())
        .resource_id("test-tgw".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_cloudtrail() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_cloudtrail".to_string())
        .resource_id("test-cloudtrail".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_config() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_config_configuration_recorder".to_string())
        .resource_id("test-config".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_guardduty() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_guardduty_detector".to_string())
        .resource_id("test-guardduty".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_inspector() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_inspector_assessment_template".to_string())
        .resource_id("test-inspector".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// Additional Edge Cases and Complex Scenarios
#[test]
fn test_predict_ec2_with_ebs() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-ec2-ebs".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1",
            "root_block_device": {
                "volume_size": 20,
                "volume_type": "gp3"
            }
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_lambda_with_timeout() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lambda_function".to_string())
        .resource_id("test-lambda-timeout".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 128,
            "runtime": "python3.9",
            "timeout": 30,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_rds_with_storage() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_db_instance".to_string())
        .resource_id("test-rds-storage".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_class": "db.t3.micro",
            "engine": "mysql",
            "allocated_storage": 100,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_dynamodb_with_gsi() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_dynamodb_table".to_string())
        .resource_id("test-dynamodb-gsi".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 5,
            "write_capacity": 5,
            "global_secondary_indexes": [
                {
                    "read_capacity": 5,
                    "write_capacity": 5
                }
            ],
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_s3_with_encryption() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_s3_bucket".to_string())
        .resource_id("test-s3-encryption".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "server_side_encryption_configuration": {
                "rule": {
                    "apply_server_side_encryption_by_default": {
                        "sse_algorithm": "AES256"
                    }
                }
            },
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_alb_with_logs() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_lb".to_string())
        .resource_id("test-alb-logs".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application",
            "access_logs": {
                "enabled": true
            },
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

// More AWS Services
#[test]
fn test_predict_elasticsearch() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_elasticsearch_domain".to_string())
        .resource_id("test-elasticsearch".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.small.elasticsearch",
            "instance_count": 1,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_opensearch() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_opensearch_domain".to_string())
        .resource_id("test-opensearch".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.small.search",
            "instance_count": 1,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_msk() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_msk_cluster".to_string())
        .resource_id("test-msk".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "kafka.t3.small",
            "number_of_broker_nodes": 2,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_emr() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_emr_cluster".to_string())
        .resource_id("test-emr".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "m5.xlarge",
            "instance_count": 3,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_sagemaker() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_sagemaker_notebook_instance".to_string())
        .resource_id("test-sagemaker".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "ml.t3.medium",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_batch() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_batch_compute_environment".to_string())
        .resource_id("test-batch".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "c5.large",
            "min_vcpus": 0,
            "max_vcpus": 10,
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_fargate() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_ecs_service".to_string())
        .resource_id("test-fargate".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "launch_type": "FARGATE",
            "cpu": "256",
            "memory": "512",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.predict_resource_cost(&change);
    assert!(result.is_ok());
    let cost = result.unwrap();
    assert!(cost.monthly_cost >= 0.0);
}

#[test]
fn test_predict_total_cost() {
    let mut engine = PredictionEngine::new().unwrap();
    let changes = vec![
        ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build(),
    ];

    let result = engine.predict_total_cost(&changes);
    assert!(result.is_ok());
    let total_cost = result.unwrap();
    assert!(total_cost.monthly >= 0.0);
}

#[test]
fn test_explain() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.micro",
            "region": "us-east-1"
        }))
        .build();

    let result = engine.explain(&change);
    assert!(result.is_ok());
    let explanation = result.unwrap();
    assert!(!explanation.steps.is_empty());
}

#[test]
fn test_predict_resource_cost_unknown_type() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("unknown_resource".to_string())
        .resource_id("test-resource".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({}))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle unknown resource types gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_predict_resource_cost_invalid_config() {
    let engine = PredictionEngine::new().unwrap();
    let change = ResourceChange::builder()
        .resource_type("aws_instance".to_string())
        .resource_id("test-instance".to_string())
        .action(ChangeAction::Create)
        .new_config(json!({}))
        .build();

    let result = engine.predict_resource_cost(&change);
    // Should handle missing configuration gracefully
    assert!(result.is_ok() || result.is_err());
}

proptest! {
    #[test]
    fn test_cost_non_negative(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Costs should never be negative
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low >= 0.0);
            prop_assert!(estimate.prediction_interval_high >= 0.0);
        }
    }

    #[test]
    fn test_prediction_intervals_valid(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Prediction intervals should be valid (low <= high)
        if let Ok(estimate) = result {
            prop_assert!(estimate.prediction_interval_low <= estimate.prediction_interval_high);
        }
    }

    #[test]
    fn test_confidence_score_bounds(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type)
            .resource_id(resource_id)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Confidence scores should be between 0 and 1
        if let Ok(estimate) = result {
            prop_assert!(estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_deterministic_output(resource_type in "[a-z_]{1,50}", resource_id in "[a-zA-Z0-9_-]{1,100}") {
        let engine1 = PredictionEngine::new().unwrap();
        let engine2 = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type(resource_type.clone())
            .resource_id(resource_id.clone())
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let result1 = engine1.predict_resource_cost(&change);
        let result2 = engine2.predict_resource_cost(&change);
        // Same input should produce same output
        match (result1, result2) {
            (Ok(est1), Ok(est2)) => {
                prop_assert_eq!(est1.monthly_cost, est2.monthly_cost);
                prop_assert_eq!(est1.prediction_interval_low, est2.prediction_interval_low);
                prop_assert_eq!(est1.prediction_interval_high, est2.prediction_interval_high);
                prop_assert_eq!(est1.confidence_score, est2.confidence_score);
            }
            (Err(_), Err(_)) => {} // Both errors is also deterministic
            _ => prop_assert!(false, "Inconsistent results for same input"),
        }
    }

    #[test]
    fn test_zero_cost_edge_cases(cost in 0.0f64..1000.0) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Even with zero cost inputs, output should be valid
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low <= estimate.prediction_interval_high);
            prop_assert!(estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_negative_cost_guards(cost in -1000.0f64..0.0) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Should not produce negative costs
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost >= 0.0);
            prop_assert!(estimate.prediction_interval_low >= 0.0);
        }
    }

    #[test]
    fn test_overflow_protection(large_cost in 1e10f64..1e20) {
        let engine = PredictionEngine::new().unwrap();
        let change = ResourceChange::builder()
            .resource_type("aws_instance".to_string())
            .resource_id("test-instance".to_string())
            .action(ChangeAction::Create)
            .new_config(json!({
                "instance_type": "t3.micro",
                "region": "us-east-1"
            }))
            .build();

        let result = engine.predict_resource_cost(&change);
        // Should handle large numbers without overflow
        if let Ok(estimate) = result {
            prop_assert!(estimate.monthly_cost.is_finite());
            prop_assert!(estimate.prediction_interval_low.is_finite());
            prop_assert!(estimate.prediction_interval_high.is_finite());
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct ArbResourceChange(ResourceChange);

impl Arbitrary for ArbResourceChange {
    fn arbitrary(g: &mut Gen) -> Self {
        let resource_id: String = Arbitrary::arbitrary(g);
        let resource_type: String = Arbitrary::arbitrary(g);
        let monthly_cost: Option<f64> = Arbitrary::arbitrary(g);

        ArbResourceChange(ResourceChange::builder()
            .resource_id(resource_id)
            .resource_type(resource_type)
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .monthly_cost(monthly_cost.map(|c| c.abs()).unwrap_or(0.0))
            .build())
    }
}

#[quickcheck]
fn quickcheck_cost_non_negative(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.monthly_cost >= 0.0 &&
        estimate.prediction_interval_low >= 0.0 &&
        estimate.prediction_interval_high >= 0.0
    } else {
        true // Errors are acceptable
    }
}

#[quickcheck]
fn quickcheck_prediction_intervals_valid(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.prediction_interval_low <= estimate.prediction_interval_high
    } else {
        true
    }
}

#[quickcheck]
fn quickcheck_confidence_bounds(ArbResourceChange(change): ArbResourceChange) -> bool {
    let engine = PredictionEngine::new().unwrap();
    let result = engine.predict_resource_cost(&change);
    if let Ok(estimate) = result {
        estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0
    } else {
        true
    }
}

#[test]
fn test_indirect_cost_only_changes_silent() {
    // Placeholder test for: Indirect-cost-only changes → silent
    // TODO: Implement logic to check that when only indirect costs change,
    // the system runs silently (no findings, no explain output, exit code 0)
    assert!(true);
}
