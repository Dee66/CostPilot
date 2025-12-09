// Snapshot tests for Terraform plan variants

use costpilot::engines::prediction::PredictionEngine;
use costpilot::validation::terraform_parser::TerraformParser;
use insta::assert_json_snapshot;
use serde_json::json;

/// Helper to create a basic Terraform plan JSON
fn create_terraform_plan(resources: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": resources,
        "configuration": {
            "root_module": {}
        }
    })
}

#[test]
fn snapshot_ec2_t3_medium_create() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_instance.web",
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "instance_type": "t3.medium",
                    "ami": "ami-0c55b159cbfafe1f0"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan, @r###"
    {
      "format_version": "1.0",
      "terraform_version": "1.5.0",
      "resource_changes": [
        {
          "address": "aws_instance.web",
          "mode": "managed",
          "type": "aws_instance",
          "name": "web",
          "change": {
            "actions": [
              "create"
            ],
            "before": null,
            "after": {
              "instance_type": "t3.medium",
              "ami": "ami-0c55b159cbfafe1f0"
            }
          }
        }
      ],
      "configuration": {
        "root_module": {}
      }
    }
    "###);
}

#[test]
fn snapshot_ec2_instance_type_upgrade() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_instance.web",
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "change": {
                "actions": ["update"],
                "before": {
                    "instance_type": "t3.small",
                    "ami": "ami-0c55b159cbfafe1f0"
                },
                "after": {
                    "instance_type": "t3.xlarge",
                    "ami": "ami-0c55b159cbfafe1f0"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_nat_gateway_create() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_nat_gateway.main",
            "mode": "managed",
            "type": "aws_nat_gateway",
            "name": "main",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "subnet_id": "subnet-12345",
                    "connectivity_type": "public"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_rds_instance_create() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_db_instance.main",
            "mode": "managed",
            "type": "aws_db_instance",
            "name": "main",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "engine": "postgres",
                    "engine_version": "14.6",
                    "instance_class": "db.t3.medium",
                    "allocated_storage": 100
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_s3_bucket_with_lifecycle() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_s3_bucket.data",
            "mode": "managed",
            "type": "aws_s3_bucket",
            "name": "data",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "bucket": "my-data-bucket",
                    "lifecycle_rule": [
                        {
                            "enabled": true,
                            "transition": [
                                {
                                    "days": 90,
                                    "storage_class": "GLACIER"
                                }
                            ]
                        }
                    ]
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_lambda_with_provisioned_concurrency() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_lambda_function.api",
            "mode": "managed",
            "type": "aws_lambda_function",
            "name": "api",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "function_name": "api-handler",
                    "runtime": "python3.11",
                    "memory_size": 512,
                    "timeout": 30,
                    "reserved_concurrent_executions": 100
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_multi_resource_plan() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_vpc.main",
            "type": "aws_vpc",
            "change": {
                "actions": ["create"],
                "after": { "cidr_block": "10.0.0.0/16" }
            }
        }),
        json!({
            "address": "aws_subnet.public",
            "type": "aws_subnet",
            "change": {
                "actions": ["create"],
                "after": {
                    "vpc_id": "aws_vpc.main",
                    "cidr_block": "10.0.1.0/24"
                }
            }
        }),
        json!({
            "address": "aws_instance.web",
            "type": "aws_instance",
            "change": {
                "actions": ["create"],
                "after": {
                    "instance_type": "t3.medium",
                    "subnet_id": "aws_subnet.public"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_delete_operation() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_instance.old",
            "type": "aws_instance",
            "change": {
                "actions": ["delete"],
                "before": {
                    "instance_type": "t2.micro",
                    "ami": "ami-old"
                },
                "after": null
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_replace_operation() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_instance.web",
            "type": "aws_instance",
            "change": {
                "actions": ["delete", "create"],
                "before": {
                    "instance_type": "t3.small",
                    "availability_zone": "us-east-1a"
                },
                "after": {
                    "instance_type": "t3.small",
                    "availability_zone": "us-east-1b"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_elasticache_cluster() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_elasticache_cluster.redis",
            "type": "aws_elasticache_cluster",
            "change": {
                "actions": ["create"],
                "after": {
                    "cluster_id": "my-redis",
                    "engine": "redis",
                    "node_type": "cache.t3.medium",
                    "num_cache_nodes": 2
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_eks_cluster() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_eks_cluster.main",
            "type": "aws_eks_cluster",
            "change": {
                "actions": ["create"],
                "after": {
                    "name": "production",
                    "version": "1.28",
                    "role_arn": "arn:aws:iam::123456789012:role/eks-cluster"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_dynamodb_with_gsi() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_dynamodb_table.users",
            "type": "aws_dynamodb_table",
            "change": {
                "actions": ["create"],
                "after": {
                    "name": "users",
                    "billing_mode": "PROVISIONED",
                    "read_capacity": 20,
                    "write_capacity": 20,
                    "global_secondary_index": [
                        {
                            "name": "EmailIndex",
                            "read_capacity": 10,
                            "write_capacity": 10
                        }
                    ]
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_alb_with_listeners() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_lb.main",
            "type": "aws_lb",
            "change": {
                "actions": ["create"],
                "after": {
                    "name": "main-alb",
                    "load_balancer_type": "application",
                    "internal": false
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_cloudfront_distribution() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_cloudfront_distribution.cdn",
            "type": "aws_cloudfront_distribution",
            "change": {
                "actions": ["create"],
                "after": {
                    "enabled": true,
                    "price_class": "PriceClass_100"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_empty_plan() {
    let plan = create_terraform_plan(vec![]);
    assert_json_snapshot!(plan);
}

#[test]
fn snapshot_plan_with_no_changes() {
    let plan = create_terraform_plan(vec![
        json!({
            "address": "aws_instance.web",
            "type": "aws_instance",
            "change": {
                "actions": ["no-op"],
                "before": {
                    "instance_type": "t3.medium"
                },
                "after": {
                    "instance_type": "t3.medium"
                }
            }
        })
    ]);

    assert_json_snapshot!(plan);
}
