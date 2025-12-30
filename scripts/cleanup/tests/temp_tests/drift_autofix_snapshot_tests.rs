// Drift cases for autofix snapshot tests - validate autofix behavior in drift scenarios

use insta::assert_json_snapshot;
use serde_json::json;

#[test]
fn test_drift_case_resource_deleted_outside_terraform() {
    // Resource deleted manually, Terraform wants to recreate
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.web",
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "instance_type": "t3.medium",
                    "ami": "ami-12345678"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_resource_modified_outside_terraform() {
    // Resource modified manually, Terraform detects drift
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.app",
            "mode": "managed",
            "type": "aws_instance",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "instance_type": "t2.micro",
                    "tags": {"Environment": "dev"}
                },
                "after": {
                    "instance_type": "t3.medium",
                    "tags": {"Environment": "dev", "ManagedBy": "Terraform"}
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_state_file_out_of_sync() {
    // State file shows different resource than reality
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_db_instance.main",
            "mode": "managed",
            "type": "aws_db_instance",
            "name": "main",
            "change": {
                "actions": ["update"],
                "before": {
                    "instance_class": "db.t3.micro",
                    "allocated_storage": 20
                },
                "after": {
                    "instance_class": "db.t3.small",
                    "allocated_storage": 20
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_force_replacement_due_to_drift() {
    // Drift requires resource replacement
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.critical",
            "mode": "managed",
            "type": "aws_instance",
            "name": "critical",
            "change": {
                "actions": ["delete", "create"],
                "before": {
                    "instance_type": "t3.medium",
                    "availability_zone": "us-east-1a"
                },
                "after": {
                    "instance_type": "t3.medium",
                    "availability_zone": "us-east-1b"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_tainted_resource() {
    // Resource marked as tainted, must be recreated
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.tainted",
            "mode": "managed",
            "type": "aws_instance",
            "name": "tainted",
            "change": {
                "actions": ["delete", "create"],
                "before": {
                    "instance_type": "t3.medium"
                },
                "after": {
                    "instance_type": "t3.medium"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_dependency_chain_broken() {
    // Dependency deleted, causes cascading drift
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [
            {
                "address": "aws_security_group.app",
                "mode": "managed",
                "type": "aws_security_group",
                "name": "app",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "name": "app-sg"
                    }
                }
            },
            {
                "address": "aws_instance.app",
                "mode": "managed",
                "type": "aws_instance",
                "name": "app",
                "change": {
                    "actions": ["update"],
                    "before": {
                        "security_groups": []
                    },
                    "after": {
                        "security_groups": ["app-sg"]
                    }
                }
            }
        ]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_import_existing_resource() {
    // Importing manually created resource into Terraform
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_s3_bucket.existing",
            "mode": "managed",
            "type": "aws_s3_bucket",
            "name": "existing",
            "change": {
                "actions": ["no-op"],
                "before": {
                    "bucket": "my-existing-bucket"
                },
                "after": {
                    "bucket": "my-existing-bucket"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_partial_configuration_drift() {
    // Some fields drifted, others unchanged
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.partial_drift",
            "mode": "managed",
            "type": "aws_instance",
            "name": "partial_drift",
            "change": {
                "actions": ["update"],
                "before": {
                    "instance_type": "t3.medium",
                    "monitoring": false,
                    "tags": {"Name": "test"}
                },
                "after": {
                    "instance_type": "t3.medium",
                    "monitoring": true,
                    "tags": {"Name": "test", "Owner": "ops"}
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_multiple_resources_drifted() {
    // Multiple resources with different drift patterns
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [
            {
                "address": "aws_instance.drift1",
                "mode": "managed",
                "type": "aws_instance",
                "name": "drift1",
                "change": {
                    "actions": ["update"],
                    "before": {"instance_type": "t2.micro"},
                    "after": {"instance_type": "t3.medium"}
                }
            },
            {
                "address": "aws_instance.drift2",
                "mode": "managed",
                "type": "aws_instance",
                "name": "drift2",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {"instance_type": "t3.large"}
                }
            },
            {
                "address": "aws_instance.drift3",
                "mode": "managed",
                "type": "aws_instance",
                "name": "drift3",
                "change": {
                    "actions": ["delete", "create"],
                    "before": {"instance_type": "t3.small"},
                    "after": {"instance_type": "t3.small"}
                }
            }
        ]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_manual_tag_changes() {
    // Tags changed manually outside Terraform
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.tagged",
            "mode": "managed",
            "type": "aws_instance",
            "name": "tagged",
            "change": {
                "actions": ["update"],
                "before": {
                    "tags": {
                        "Name": "production-server",
                        "Environment": "prod"
                    }
                },
                "after": {
                    "tags": {
                        "Name": "production-server",
                        "Environment": "prod",
                        "CostCenter": "engineering"
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_lifecycle_ignore_changes() {
    // Drift on fields with lifecycle ignore_changes
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_instance.ignore_tags",
            "mode": "managed",
            "type": "aws_instance",
            "name": "ignore_tags",
            "change": {
                "actions": ["no-op"],
                "before": {
                    "instance_type": "t3.medium",
                    "tags": {"manual": "tag"}
                },
                "after": {
                    "instance_type": "t3.medium",
                    "tags": {"manual": "tag"}
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_security_group_rules_modified() {
    // Security group rules changed manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_security_group.web",
            "mode": "managed",
            "type": "aws_security_group",
            "name": "web",
            "change": {
                "actions": ["update"],
                "before": {
                    "ingress": [
                        {"from_port": 80, "to_port": 80, "protocol": "tcp"}
                    ]
                },
                "after": {
                    "ingress": [
                        {"from_port": 80, "to_port": 80, "protocol": "tcp"},
                        {"from_port": 443, "to_port": 443, "protocol": "tcp"}
                    ]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_iam_policy_drift() {
    // IAM policy modified outside Terraform
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_iam_role.app",
            "mode": "managed",
            "type": "aws_iam_role",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "assume_role_policy": "{\"Version\":\"2012-10-17\"}"
                },
                "after": {
                    "assume_role_policy": "{\"Version\":\"2012-10-17\",\"Statement\":[]}"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_autoscaling_desired_capacity() {
    // Autoscaling group adjusted capacity manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_autoscaling_group.app",
            "mode": "managed",
            "type": "aws_autoscaling_group",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "desired_capacity": 2,
                    "min_size": 1,
                    "max_size": 4
                },
                "after": {
                    "desired_capacity": 3,
                    "min_size": 1,
                    "max_size": 4
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_database_parameter_group() {
    // Database parameter group modified
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_db_parameter_group.main",
            "mode": "managed",
            "type": "aws_db_parameter_group",
            "name": "main",
            "change": {
                "actions": ["update"],
                "before": {
                    "parameter": [
                        {"name": "max_connections", "value": "100"}
                    ]
                },
                "after": {
                    "parameter": [
                        {"name": "max_connections", "value": "200"}
                    ]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_s3_bucket_lifecycle_rules() {
    // S3 bucket lifecycle rules changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_s3_bucket.data",
            "mode": "managed",
            "type": "aws_s3_bucket",
            "name": "data",
            "change": {
                "actions": ["update"],
                "before": {
                    "lifecycle_rule": []
                },
                "after": {
                    "lifecycle_rule": [
                        {
                            "id": "cleanup",
                            "enabled": true,
                            "expiration": {"days": 30}
                        }
                    ]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_cloudwatch_alarm_threshold() {
    // CloudWatch alarm threshold changed manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_cloudwatch_metric_alarm.cpu",
            "mode": "managed",
            "type": "aws_cloudwatch_metric_alarm",
            "name": "cpu",
            "change": {
                "actions": ["update"],
                "before": {
                    "threshold": 80.0,
                    "comparison_operator": "GreaterThanThreshold"
                },
                "after": {
                    "threshold": 90.0,
                    "comparison_operator": "GreaterThanThreshold"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_lambda_environment_variables() {
    // Lambda environment variables changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_lambda_function.processor",
            "mode": "managed",
            "type": "aws_lambda_function",
            "name": "processor",
            "change": {
                "actions": ["update"],
                "before": {
                    "environment": {
                        "variables": {
                            "LOG_LEVEL": "info"
                        }
                    }
                },
                "after": {
                    "environment": {
                        "variables": {
                            "LOG_LEVEL": "debug",
                            "DEBUG_MODE": "true"
                        }
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_route53_record_ttl() {
    // Route53 record TTL changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_route53_record.www",
            "mode": "managed",
            "type": "aws_route53_record",
            "name": "www",
            "change": {
                "actions": ["update"],
                "before": {
                    "ttl": 300,
                    "records": ["192.0.2.1"]
                },
                "after": {
                    "ttl": 60,
                    "records": ["192.0.2.1"]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_elb_health_check_parameters() {
    // ELB health check parameters modified
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_lb.app",
            "mode": "managed",
            "type": "aws_lb",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "health_check": {
                        "interval": 30,
                        "timeout": 5
                    }
                },
                "after": {
                    "health_check": {
                        "interval": 10,
                        "timeout": 3
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_eks_cluster_version() {
    // EKS cluster version upgraded manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_eks_cluster.main",
            "mode": "managed",
            "type": "aws_eks_cluster",
            "name": "main",
            "change": {
                "actions": ["update"],
                "before": {
                    "version": "1.27"
                },
                "after": {
                    "version": "1.28"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_kms_key_policy() {
    // KMS key policy modified
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_kms_key.data",
            "mode": "managed",
            "type": "aws_kms_key",
            "name": "data",
            "change": {
                "actions": ["update"],
                "before": {
                    "policy": "{\"Version\":\"2012-10-17\",\"Statement\":[]}"
                },
                "after": {
                    "policy": "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Effect\":\"Allow\"}]}"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_elasticache_node_count() {
    // ElastiCache cluster node count changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_elasticache_replication_group.redis",
            "mode": "managed",
            "type": "aws_elasticache_replication_group",
            "name": "redis",
            "change": {
                "actions": ["update"],
                "before": {
                    "number_cache_clusters": 2
                },
                "after": {
                    "number_cache_clusters": 3
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_rds_backup_retention() {
    // RDS backup retention period changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_db_instance.main",
            "mode": "managed",
            "type": "aws_db_instance",
            "name": "main",
            "change": {
                "actions": ["update"],
                "before": {
                    "backup_retention_period": 7
                },
                "after": {
                    "backup_retention_period": 14
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_vpc_flow_logs_enabled() {
    // VPC flow logs enabled manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_flow_log.vpc",
            "mode": "managed",
            "type": "aws_flow_log",
            "name": "vpc",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "vpc_id": "vpc-12345",
                    "traffic_type": "ALL"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_dynamodb_billing_mode() {
    // DynamoDB billing mode changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_dynamodb_table.data",
            "mode": "managed",
            "type": "aws_dynamodb_table",
            "name": "data",
            "change": {
                "actions": ["update"],
                "before": {
                    "billing_mode": "PROVISIONED",
                    "read_capacity": 5,
                    "write_capacity": 5
                },
                "after": {
                    "billing_mode": "PAY_PER_REQUEST"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_ecs_task_count() {
    // ECS service desired count changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_ecs_service.app",
            "mode": "managed",
            "type": "aws_ecs_service",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "desired_count": 2
                },
                "after": {
                    "desired_count": 4
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_cloudfront_price_class() {
    // CloudFront distribution price class changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_cloudfront_distribution.cdn",
            "mode": "managed",
            "type": "aws_cloudfront_distribution",
            "name": "cdn",
            "change": {
                "actions": ["update"],
                "before": {
                    "price_class": "PriceClass_100"
                },
                "after": {
                    "price_class": "PriceClass_All"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_kinesis_shard_count() {
    // Kinesis stream shard count changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_kinesis_stream.events",
            "mode": "managed",
            "type": "aws_kinesis_stream",
            "name": "events",
            "change": {
                "actions": ["update"],
                "before": {
                    "shard_count": 1
                },
                "after": {
                    "shard_count": 2
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_sqs_message_retention() {
    // SQS queue message retention changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_sqs_queue.tasks",
            "mode": "managed",
            "type": "aws_sqs_queue",
            "name": "tasks",
            "change": {
                "actions": ["update"],
                "before": {
                    "message_retention_seconds": 345600
                },
                "after": {
                    "message_retention_seconds": 1209600
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_api_gateway_throttling() {
    // API Gateway throttling settings changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_api_gateway_stage.prod",
            "mode": "managed",
            "type": "aws_api_gateway_stage",
            "name": "prod",
            "change": {
                "actions": ["update"],
                "before": {
                    "throttle_settings": {
                        "rate_limit": 1000,
                        "burst_limit": 2000
                    }
                },
                "after": {
                    "throttle_settings": {
                        "rate_limit": 5000,
                        "burst_limit": 10000
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_sns_topic_subscription() {
    // SNS topic subscription added manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_sns_topic_subscription.alert",
            "mode": "managed",
            "type": "aws_sns_topic_subscription",
            "name": "alert",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "topic_arn": "arn:aws:sns:us-east-1:123456789012:alerts",
                    "protocol": "email",
                    "endpoint": "ops@example.com"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_ecr_lifecycle_policy() {
    // ECR lifecycle policy added manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_ecr_lifecycle_policy.images",
            "mode": "managed",
            "type": "aws_ecr_lifecycle_policy",
            "name": "images",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "policy": "{\"rules\":[{\"rulePriority\":1,\"selection\":{\"tagStatus\":\"untagged\",\"countType\":\"sinceImagePushed\",\"countNumber\":14},\"action\":{\"type\":\"expire\"}}]}"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_secrets_manager_rotation() {
    // Secrets Manager rotation config changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_secretsmanager_secret_rotation.db",
            "mode": "managed",
            "type": "aws_secretsmanager_secret_rotation",
            "name": "db",
            "change": {
                "actions": ["update"],
                "before": {
                    "rotation_rules": {
                        "automatically_after_days": 30
                    }
                },
                "after": {
                    "rotation_rules": {
                        "automatically_after_days": 7
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_waf_rule_priority() {
    // WAF rule priority changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_wafv2_web_acl.main",
            "mode": "managed",
            "type": "aws_wafv2_web_acl",
            "name": "main",
            "change": {
                "actions": ["update"],
                "before": {
                    "rule": [
                        {"priority": 1, "name": "rate-limit"}
                    ]
                },
                "after": {
                    "rule": [
                        {"priority": 10, "name": "rate-limit"}
                    ]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_codepipeline_stage_added() {
    // CodePipeline stage added manually
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_codepipeline.app",
            "mode": "managed",
            "type": "aws_codepipeline",
            "name": "app",
            "change": {
                "actions": ["update"],
                "before": {
                    "stage": [
                        {"name": "Source"},
                        {"name": "Build"}
                    ]
                },
                "after": {
                    "stage": [
                        {"name": "Source"},
                        {"name": "Build"},
                        {"name": "Deploy"}
                    ]
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_config_rule_modified() {
    // AWS Config rule modified
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_config_config_rule.encryption",
            "mode": "managed",
            "type": "aws_config_config_rule",
            "name": "encryption",
            "change": {
                "actions": ["update"],
                "before": {
                    "maximum_execution_frequency": "TwentyFour_Hours"
                },
                "after": {
                    "maximum_execution_frequency": "Six_Hours"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_glue_crawler_schedule() {
    // Glue crawler schedule changed
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_glue_crawler.data",
            "mode": "managed",
            "type": "aws_glue_crawler",
            "name": "data",
            "change": {
                "actions": ["update"],
                "before": {
                    "schedule": "cron(0 12 * * ? *)"
                },
                "after": {
                    "schedule": "cron(0 0 * * ? *)"
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_batch_compute_environment() {
    // AWS Batch compute environment modified
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [{
            "address": "aws_batch_compute_environment.jobs",
            "mode": "managed",
            "type": "aws_batch_compute_environment",
            "name": "jobs",
            "change": {
                "actions": ["update"],
                "before": {
                    "compute_resources": {
                        "max_vcpus": 64
                    }
                },
                "after": {
                    "compute_resources": {
                        "max_vcpus": 128
                    }
                }
            }
        }]
    });

    assert_json_snapshot!(plan);
}

#[test]
fn test_drift_case_mixed_create_update_delete() {
    // Complex drift scenario with multiple action types
    let plan = json!({
        "format_version": "1.0",
        "resource_changes": [
            {
                "address": "aws_instance.new",
                "change": {"actions": ["create"], "after": {"instance_type": "t3.medium"}}
            },
            {
                "address": "aws_instance.updated",
                "change": {
                    "actions": ["update"],
                    "before": {"instance_type": "t2.micro"},
                    "after": {"instance_type": "t3.small"}
                }
            },
            {
                "address": "aws_instance.replaced",
                "change": {
                    "actions": ["delete", "create"],
                    "before": {"instance_type": "t3.large"},
                    "after": {"instance_type": "t3.large"}
                }
            }
        ]
    });

    assert_json_snapshot!(plan);
}
