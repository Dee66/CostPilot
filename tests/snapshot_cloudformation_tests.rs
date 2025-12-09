// Snapshot tests for CloudFormation variants

use insta::assert_json_snapshot;
use serde_json::json;

#[test]
fn snapshot_cfn_ec2_instance_creation() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "WebServer": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "InstanceType": "t3.medium",
                    "ImageId": "ami-0c55b159cbfafe1f0",
                    "Tags": [
                        {
                            "Key": "Name",
                            "Value": "WebServer"
                        }
                    ]
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_rds_postgres_instance() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Database": {
                "Type": "AWS::RDS::DBInstance",
                "Properties": {
                    "Engine": "postgres",
                    "EngineVersion": "14.6",
                    "DBInstanceClass": "db.t3.medium",
                    "AllocatedStorage": "100",
                    "StorageType": "gp3",
                    "MasterUsername": "admin",
                    "MasterUserPassword": { "Ref": "DBPassword" }
                }
            }
        },
        "Parameters": {
            "DBPassword": {
                "Type": "String",
                "NoEcho": true
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_lambda_with_api_gateway() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "ApiHandler": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "Runtime": "python3.11",
                    "Handler": "index.handler",
                    "MemorySize": 512,
                    "Timeout": 30,
                    "Code": {
                        "ZipFile": "def handler(event, context): return {'statusCode': 200}"
                    }
                }
            },
            "RestApi": {
                "Type": "AWS::ApiGateway::RestApi",
                "Properties": {
                    "Name": "MyApi",
                    "EndpointConfiguration": {
                        "Types": ["REGIONAL"]
                    }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_dynamodb_table_with_gsi() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "UsersTable": {
                "Type": "AWS::DynamoDB::Table",
                "Properties": {
                    "TableName": "Users",
                    "BillingMode": "PROVISIONED",
                    "AttributeDefinitions": [
                        {
                            "AttributeName": "id",
                            "AttributeType": "S"
                        },
                        {
                            "AttributeName": "email",
                            "AttributeType": "S"
                        }
                    ],
                    "KeySchema": [
                        {
                            "AttributeName": "id",
                            "KeyType": "HASH"
                        }
                    ],
                    "ProvisionedThroughput": {
                        "ReadCapacityUnits": 20,
                        "WriteCapacityUnits": 20
                    },
                    "GlobalSecondaryIndexes": [
                        {
                            "IndexName": "EmailIndex",
                            "KeySchema": [
                                {
                                    "AttributeName": "email",
                                    "KeyType": "HASH"
                                }
                            ],
                            "Projection": {
                                "ProjectionType": "ALL"
                            },
                            "ProvisionedThroughput": {
                                "ReadCapacityUnits": 10,
                                "WriteCapacityUnits": 10
                            }
                        }
                    ]
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_s3_bucket_with_lifecycle() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
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
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_nat_gateway_with_eip() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "NatGatewayEIP": {
                "Type": "AWS::EC2::EIP",
                "Properties": {
                    "Domain": "vpc"
                }
            },
            "NatGateway": {
                "Type": "AWS::EC2::NatGateway",
                "Properties": {
                    "AllocationId": { "Fn::GetAtt": ["NatGatewayEIP", "AllocationId"] },
                    "SubnetId": { "Ref": "PublicSubnet" }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_elasticache_redis_cluster() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "RedisCluster": {
                "Type": "AWS::ElastiCache::CacheCluster",
                "Properties": {
                    "Engine": "redis",
                    "CacheNodeType": "cache.t3.medium",
                    "NumCacheNodes": 2,
                    "Port": 6379,
                    "VpcSecurityGroupIds": [
                        { "Ref": "RedisSecurityGroup" }
                    ]
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_alb_with_target_group() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "LoadBalancer": {
                "Type": "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "Properties": {
                    "Name": "main-alb",
                    "Type": "application",
                    "Scheme": "internet-facing",
                    "Subnets": [
                        { "Ref": "PublicSubnet1" },
                        { "Ref": "PublicSubnet2" }
                    ]
                }
            },
            "TargetGroup": {
                "Type": "AWS::ElasticLoadBalancingV2::TargetGroup",
                "Properties": {
                    "Port": 80,
                    "Protocol": "HTTP",
                    "VpcId": { "Ref": "VPC" },
                    "HealthCheckPath": "/health"
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_ecs_fargate_service() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "TaskDefinition": {
                "Type": "AWS::ECS::TaskDefinition",
                "Properties": {
                    "Family": "app-task",
                    "RequiresCompatibilities": ["FARGATE"],
                    "NetworkMode": "awsvpc",
                    "Cpu": "512",
                    "Memory": "1024",
                    "ContainerDefinitions": [
                        {
                            "Name": "app",
                            "Image": "nginx:latest",
                            "Memory": 512
                        }
                    ]
                }
            },
            "Service": {
                "Type": "AWS::ECS::Service",
                "Properties": {
                    "Cluster": { "Ref": "ECSCluster" },
                    "DesiredCount": 3,
                    "LaunchType": "FARGATE",
                    "TaskDefinition": { "Ref": "TaskDefinition" }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_cloudfront_distribution() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "Distribution": {
                "Type": "AWS::CloudFront::Distribution",
                "Properties": {
                    "DistributionConfig": {
                        "Enabled": true,
                        "PriceClass": "PriceClass_100",
                        "Origins": [
                            {
                                "DomainName": { "Fn::GetAtt": ["Bucket", "DomainName"] },
                                "Id": "S3Origin",
                                "S3OriginConfig": {}
                            }
                        ],
                        "DefaultCacheBehavior": {
                            "TargetOriginId": "S3Origin",
                            "ViewerProtocolPolicy": "redirect-to-https",
                            "ForwardedValues": {
                                "QueryString": false
                            }
                        }
                    }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_eks_cluster() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "EKSCluster": {
                "Type": "AWS::EKS::Cluster",
                "Properties": {
                    "Name": "production",
                    "Version": "1.28",
                    "RoleArn": { "Fn::GetAtt": ["ClusterRole", "Arn"] },
                    "ResourcesVpcConfig": {
                        "SubnetIds": [
                            { "Ref": "Subnet1" },
                            { "Ref": "Subnet2" }
                        ]
                    }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_vpc_with_subnets() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "VPC": {
                "Type": "AWS::EC2::VPC",
                "Properties": {
                    "CidrBlock": "10.0.0.0/16",
                    "EnableDnsHostnames": true,
                    "EnableDnsSupport": true
                }
            },
            "PublicSubnet": {
                "Type": "AWS::EC2::Subnet",
                "Properties": {
                    "VpcId": { "Ref": "VPC" },
                    "CidrBlock": "10.0.1.0/24",
                    "AvailabilityZone": { "Fn::Select": [0, { "Fn::GetAZs": "" }] }
                }
            },
            "PrivateSubnet": {
                "Type": "AWS::EC2::Subnet",
                "Properties": {
                    "VpcId": { "Ref": "VPC" },
                    "CidrBlock": "10.0.2.0/24",
                    "AvailabilityZone": { "Fn::Select": [1, { "Fn::GetAZs": "" }] }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_kinesis_stream() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "EventStream": {
                "Type": "AWS::Kinesis::Stream",
                "Properties": {
                    "Name": "event-stream",
                    "ShardCount": 2,
                    "RetentionPeriodHours": 168
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_sqs_queue_with_dlq() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "DeadLetterQueue": {
                "Type": "AWS::SQS::Queue",
                "Properties": {
                    "QueueName": "events-dlq",
                    "MessageRetentionPeriod": 1209600
                }
            },
            "MainQueue": {
                "Type": "AWS::SQS::Queue",
                "Properties": {
                    "QueueName": "events",
                    "VisibilityTimeout": 300,
                    "RedrivePolicy": {
                        "deadLetterTargetArn": { "Fn::GetAtt": ["DeadLetterQueue", "Arn"] },
                        "maxReceiveCount": 3
                    }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_nested_stack() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {
            "NetworkStack": {
                "Type": "AWS::CloudFormation::Stack",
                "Properties": {
                    "TemplateURL": "https://s3.amazonaws.com/bucket/network.yaml",
                    "Parameters": {
                        "VpcCidr": "10.0.0.0/16"
                    }
                }
            },
            "ApplicationStack": {
                "Type": "AWS::CloudFormation::Stack",
                "Properties": {
                    "TemplateURL": "https://s3.amazonaws.com/bucket/app.yaml",
                    "Parameters": {
                        "VpcId": { "Fn::GetAtt": ["NetworkStack", "Outputs.VpcId"] }
                    }
                }
            }
        }
    });

    assert_json_snapshot!(template);
}

#[test]
fn snapshot_cfn_empty_template() {
    let template = json!({
        "AWSTemplateFormatVersion": "2010-09-09",
        "Resources": {}
    });

    assert_json_snapshot!(template);
}
