// Golden file tests for prediction output

use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::ResourceChange;
use std::fs;
use std::path::PathBuf;

fn load_golden_file(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/golden")
        .join(name);
    fs::read_to_string(path).expect("Failed to read golden file")
}

#[test]
fn golden_ec2_t3_medium_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_instance.web".to_string(),
        resource_type: "aws_instance".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "instance_type": "t3.medium"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("ec2_t3_medium", output);
}

#[test]
fn golden_rds_postgres_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_db_instance.main".to_string(),
        resource_type: "aws_db_instance".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "engine": "postgres",
            "instance_class": "db.t3.medium",
            "allocated_storage": 100
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("rds_postgres", output);
}

#[test]
fn golden_nat_gateway_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_nat_gateway.main".to_string(),
        resource_type: "aws_nat_gateway".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "connectivity_type": "public"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("nat_gateway", output);
}

#[test]
fn golden_lambda_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_lambda_function.api".to_string(),
        resource_type: "aws_lambda_function".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "memory_size": 512,
            "timeout": 30
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("lambda_function", output);
}

#[test]
fn golden_s3_bucket_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_s3_bucket.data".to_string(),
        resource_type: "aws_s3_bucket".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "bucket": "my-data-bucket"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("s3_bucket", output);
}

#[test]
fn golden_dynamodb_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_dynamodb_table.users".to_string(),
        resource_type: "aws_dynamodb_table".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "billing_mode": "PROVISIONED",
            "read_capacity": 20,
            "write_capacity": 20
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("dynamodb_table", output);
}

#[test]
fn golden_elasticache_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_elasticache_cluster.redis".to_string(),
        resource_type: "aws_elasticache_cluster".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "engine": "redis",
            "node_type": "cache.t3.medium",
            "num_cache_nodes": 2
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("elasticache_cluster", output);
}

#[test]
fn golden_alb_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_lb.main".to_string(),
        resource_type: "aws_lb".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "load_balancer_type": "application"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("application_load_balancer", output);
}

#[test]
fn golden_eks_cluster_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_eks_cluster.main".to_string(),
        resource_type: "aws_eks_cluster".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "version": "1.28"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("eks_cluster", output);
}

#[test]
fn golden_cloudfront_prediction() {
    let resource = ResourceChange {
        resource_id: "aws_cloudfront_distribution.cdn".to_string(),
        resource_type: "aws_cloudfront_distribution".to_string(),
        action: "create".to_string(),
        before: None,
        after: Some(serde_json::json!({
            "price_class": "PriceClass_100"
        })),
        monthly_cost: 0.0,
    };

    let engine = PredictionEngine::new();
    let result = engine.predict(&resource).unwrap();
    
    let output = serde_json::json!({
        "resource_id": result.resource_id,
        "predicted_cost": result.predicted_cost,
        "confidence": result.confidence,
    });

    insta::assert_json_snapshot!("cloudfront_distribution", output);
}
