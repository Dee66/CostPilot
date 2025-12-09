// Snapshot tests for CDK diff variants

use insta::assert_json_snapshot;
use serde_json::json;

/// Helper to create a basic CDK diff JSON
fn create_cdk_diff(resources: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "Resources": resources.into_iter().map(|r| {
            (r["LogicalId"].as_str().unwrap().to_string(), r)
        }).collect::<serde_json::Map<String, serde_json::Value>>()
    })
}

#[test]
fn snapshot_cdk_lambda_function_addition() {
    let diff = json!({
        "Resources": {
            "ApiHandler": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "Runtime": "nodejs18.x",
                    "Handler": "index.handler",
                    "MemorySize": 512,
                    "Timeout": 30
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_lambda_memory_update() {
    let diff = json!({
        "Resources": {
            "ApiHandler": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": {
                        "Old": 256,
                        "New": 1024
                    }
                },
                "ChangeType": "Update"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_dynamodb_table_addition() {
    let diff = json!({
        "Resources": {
            "UsersTable": {
                "Type": "AWS::DynamoDB::Table",
                "Properties": {
                    "BillingMode": "PAY_PER_REQUEST",
                    "AttributeDefinitions": [
                        {
                            "AttributeName": "id",
                            "AttributeType": "S"
                        }
                    ],
                    "KeySchema": [
                        {
                            "AttributeName": "id",
                            "KeyType": "HASH"
                        }
                    ]
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_dynamodb_billing_mode_change() {
    let diff = json!({
        "Resources": {
            "UsersTable": {
                "Type": "AWS::DynamoDB::Table",
                "Properties": {
                    "BillingMode": {
                        "Old": "PROVISIONED",
                        "New": "PAY_PER_REQUEST"
                    },
                    "ProvisionedThroughput": {
                        "Old": {
                            "ReadCapacityUnits": 5,
                            "WriteCapacityUnits": 5
                        },
                        "New": null
                    }
                },
                "ChangeType": "Update"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_api_gateway_addition() {
    let diff = json!({
        "Resources": {
            "RestApi": {
                "Type": "AWS::ApiGateway::RestApi",
                "Properties": {
                    "Name": "MyApi",
                    "EndpointConfiguration": {
                        "Types": ["REGIONAL"]
                    }
                },
                "ChangeType": "Addition"
            },
            "Deployment": {
                "Type": "AWS::ApiGateway::Deployment",
                "Properties": {
                    "RestApiId": { "Ref": "RestApi" },
                    "StageName": "prod"
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_rds_instance_addition() {
    let diff = json!({
        "Resources": {
            "Database": {
                "Type": "AWS::RDS::DBInstance",
                "Properties": {
                    "Engine": "postgres",
                    "EngineVersion": "14.6",
                    "DBInstanceClass": "db.t3.medium",
                    "AllocatedStorage": "100",
                    "StorageType": "gp3"
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_rds_instance_class_upgrade() {
    let diff = json!({
        "Resources": {
            "Database": {
                "Type": "AWS::RDS::DBInstance",
                "Properties": {
                    "DBInstanceClass": {
                        "Old": "db.t3.small",
                        "New": "db.r5.xlarge"
                    }
                },
                "ChangeType": "Update"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_s3_bucket_addition() {
    let diff = json!({
        "Resources": {
            "DataBucket": {
                "Type": "AWS::S3::Bucket",
                "Properties": {
                    "BucketName": "my-data-bucket",
                    "VersioningConfiguration": {
                        "Status": "Enabled"
                    },
                    "LifecycleConfiguration": {
                        "Rules": [
                            {
                                "Status": "Enabled",
                                "Transitions": [
                                    {
                                        "StorageClass": "GLACIER",
                                        "TransitionInDays": 90
                                    }
                                ]
                            }
                        ]
                    }
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_nat_gateway_addition() {
    let diff = json!({
        "Resources": {
            "NatGateway": {
                "Type": "AWS::EC2::NatGateway",
                "Properties": {
                    "SubnetId": { "Ref": "PublicSubnet" },
                    "AllocationId": { "Fn::GetAtt": ["EIP", "AllocationId"] }
                },
                "ChangeType": "Addition"
            },
            "EIP": {
                "Type": "AWS::EC2::EIP",
                "Properties": {
                    "Domain": "vpc"
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_ecs_service_addition() {
    let diff = json!({
        "Resources": {
            "FargateService": {
                "Type": "AWS::ECS::Service",
                "Properties": {
                    "Cluster": { "Ref": "EcsCluster" },
                    "DesiredCount": 3,
                    "LaunchType": "FARGATE",
                    "TaskDefinition": { "Ref": "TaskDef" }
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_ecs_task_cpu_memory_update() {
    let diff = json!({
        "Resources": {
            "TaskDef": {
                "Type": "AWS::ECS::TaskDefinition",
                "Properties": {
                    "Cpu": {
                        "Old": "256",
                        "New": "1024"
                    },
                    "Memory": {
                        "Old": "512",
                        "New": "2048"
                    }
                },
                "ChangeType": "Update"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_cloudfront_distribution_addition() {
    let diff = json!({
        "Resources": {
            "CdnDistribution": {
                "Type": "AWS::CloudFront::Distribution",
                "Properties": {
                    "DistributionConfig": {
                        "Enabled": true,
                        "PriceClass": "PriceClass_100",
                        "Origins": [
                            {
                                "DomainName": { "Fn::GetAtt": ["Bucket", "DomainName"] },
                                "Id": "S3Origin"
                            }
                        ]
                    }
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_elasticache_cluster_addition() {
    let diff = json!({
        "Resources": {
            "RedisCluster": {
                "Type": "AWS::ElastiCache::CacheCluster",
                "Properties": {
                    "Engine": "redis",
                    "CacheNodeType": "cache.t3.medium",
                    "NumCacheNodes": 2,
                    "Port": 6379
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_resource_removal() {
    let diff = json!({
        "Resources": {
            "OldFunction": {
                "Type": "AWS::Lambda::Function",
                "ChangeType": "Removal"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_resource_replacement() {
    let diff = json!({
        "Resources": {
            "Database": {
                "Type": "AWS::RDS::DBInstance",
                "Properties": {
                    "AvailabilityZone": {
                        "Old": "us-east-1a",
                        "New": "us-east-1b"
                    }
                },
                "ChangeType": "Replacement"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_multi_resource_stack() {
    let diff = json!({
        "Resources": {
            "Vpc": {
                "Type": "AWS::EC2::VPC",
                "Properties": {
                    "CidrBlock": "10.0.0.0/16"
                },
                "ChangeType": "Addition"
            },
            "PublicSubnet": {
                "Type": "AWS::EC2::Subnet",
                "Properties": {
                    "VpcId": { "Ref": "Vpc" },
                    "CidrBlock": "10.0.1.0/24"
                },
                "ChangeType": "Addition"
            },
            "Instance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "InstanceType": "t3.medium",
                    "SubnetId": { "Ref": "PublicSubnet" }
                },
                "ChangeType": "Addition"
            }
        }
    });

    assert_json_snapshot!(diff);
}

#[test]
fn snapshot_cdk_empty_diff() {
    let diff = json!({
        "Resources": {}
    });

    assert_json_snapshot!(diff);
}
