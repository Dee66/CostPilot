// Golden file tests for prediction output

use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ResourceChange, ChangeAction};
use serde_json::json;

#[test]
fn golden_ec2_t3_medium_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Create)
        .new_config(json!({
            "instance_type": "t3.medium"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("ec2_t3_medium", output);
}

#[test]
fn golden_rds_postgres_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_db_instance.main")
        .resource_type("aws_db_instance")
        .action(ChangeAction::Create)
        .new_config(json!({
            "engine": "postgres",
            "instance_class": "db.t3.medium",
            "allocated_storage": 100
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("rds_postgres", output);
}

#[test]
fn golden_nat_gateway_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_nat_gateway.main")
        .resource_type("aws_nat_gateway")
        .action(ChangeAction::Create)
        .new_config(json!({
            "connectivity_type": "public"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("nat_gateway", output);
}

#[test]
fn golden_lambda_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_lambda_function.api")
        .resource_type("aws_lambda_function")
        .action(ChangeAction::Create)
        .new_config(json!({
            "memory_size": 512,
            "timeout": 30
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("lambda_function", output);
}

#[test]
fn golden_s3_bucket_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_s3_bucket.data")
        .resource_type("aws_s3_bucket")
        .action(ChangeAction::Create)
        .new_config(json!({
            "bucket": "my-data-bucket"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("s3_bucket", output);
}

#[test]
fn golden_dynamodb_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_dynamodb_table.users")
        .resource_type("aws_dynamodb_table")
        .action(ChangeAction::Create)
        .new_config(json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 20,
            "write_capacity": 20
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("dynamodb_table", output);
}

#[test]
fn golden_elasticache_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_elasticache_cluster.redis")
        .resource_type("aws_elasticache_cluster")
        .action(ChangeAction::Create)
        .new_config(json!({
            "engine": "redis",
            "node_type": "cache.t3.medium",
            "num_cache_nodes": 2
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("elasticache_cluster", output);
}

#[test]
fn golden_alb_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_lb.main")
        .resource_type("aws_lb")
        .action(ChangeAction::Create)
        .new_config(json!({
            "load_balancer_type": "application"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("application_load_balancer", output);
}

#[test]
fn golden_eks_cluster_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_eks_cluster.main")
        .resource_type("aws_eks_cluster")
        .action(ChangeAction::Create)
        .new_config(json!({
            "version": "1.28"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("eks_cluster", output);
}

#[test]
fn golden_cloudfront_prediction() {
    let resource = ResourceChange::builder()
        .resource_id("aws_cloudfront_distribution.cdn")
        .resource_type("aws_cloudfront_distribution")
        .action(ChangeAction::Create)
        .new_config(json!({
            "price_class": "PriceClass_100"
        }))
        .monthly_cost(0.0)
        .build();

    let mut engine = PredictionEngine::new().unwrap();
    let results = engine.predict(&[resource]).unwrap();
    let result = &results[0];

    let output = json!({
        "resource_id": result.resource_id,
        "monthly_cost": result.monthly_cost,
        "prediction_interval_low": result.prediction_interval_low,
        "prediction_interval_high": result.prediction_interval_high,
        "confidence_score": result.confidence_score,
    });

    insta::assert_json_snapshot!("cloudfront_distribution", output);
}
