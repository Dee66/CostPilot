/// Test fixtures for CostPilot tests
///
/// Provides pre-built test data for Terraform plans, CDK diffs, CloudFormation templates,
/// policies, and other resources used in testing.

use serde_json::json;

/// Load a fixture file by name from the fixtures directory
pub fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
}

/// Create a minimal valid Terraform plan JSON for testing
pub fn minimal_terraform_plan() -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": [],
        "configuration": {
            "root_module": {}
        }
    })
}

/// Create a Terraform plan with a single EC2 instance
pub fn terraform_plan_with_ec2(instance_type: &str) -> serde_json::Value {
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

/// Create a Terraform plan with RDS instance
pub fn terraform_plan_with_rds(instance_class: &str, engine: &str, storage_gb: i32) -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_db_instance.main",
            "mode": "managed",
            "type": "aws_db_instance",
            "name": "main",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "instance_class": instance_class,
                    "engine": engine,
                    "allocated_storage": storage_gb,
                    "storage_type": "gp3",
                    "multi_az": false
                }
            }
        }]
    })
}

/// Create a Terraform plan with NAT Gateway
pub fn terraform_plan_with_nat_gateway() -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_nat_gateway.main",
            "mode": "managed",
            "type": "aws_nat_gateway",
            "name": "main",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "subnet_id": "subnet-12345",
                    "allocation_id": "eipalloc-12345"
                }
            }
        }]
    })
}

/// Create a Terraform plan with Lambda function
pub fn terraform_plan_with_lambda(memory_mb: i32) -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_lambda_function.api",
            "mode": "managed",
            "type": "aws_lambda_function",
            "name": "api",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "function_name": "api-handler",
                    "runtime": "python3.11",
                    "memory_size": memory_mb,
                    "timeout": 30,
                    "handler": "index.handler"
                }
            }
        }]
    })
}

/// Create a Terraform plan with S3 bucket
pub fn terraform_plan_with_s3() -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_s3_bucket.storage",
            "mode": "managed",
            "type": "aws_s3_bucket",
            "name": "storage",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "bucket": "my-app-storage-bucket"
                }
            }
        }]
    })
}

/// Create a Terraform plan with DynamoDB table
pub fn terraform_plan_with_dynamodb(billing_mode: &str) -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_dynamodb_table.users",
            "mode": "managed",
            "type": "aws_dynamodb_table",
            "name": "users",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "name": "users-table",
                    "billing_mode": billing_mode,
                    "hash_key": "id",
                    "attribute": [{
                        "name": "id",
                        "type": "S"
                    }]
                }
            }
        }]
    })
}

/// Create a complex Terraform plan with multiple resources
pub fn terraform_plan_complex() -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web[0]",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "index": 0,
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-12345678"
                    }
                }
            },
            {
                "address": "aws_instance.web[1]",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "index": 1,
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-12345678"
                    }
                }
            },
            {
                "address": "aws_db_instance.main",
                "mode": "managed",
                "type": "aws_db_instance",
                "name": "main",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_class": "db.r5.large",
                        "engine": "mysql",
                        "allocated_storage": 100
                    }
                }
            },
            {
                "address": "aws_nat_gateway.main",
                "mode": "managed",
                "type": "aws_nat_gateway",
                "name": "main",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "subnet_id": "subnet-12345"
                    }
                }
            },
            {
                "address": "aws_lambda_function.api",
                "mode": "managed",
                "type": "aws_lambda_function",
                "name": "api",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "function_name": "api-handler",
                        "runtime": "python3.11",
                        "memory_size": 512
                    }
                }
            }
        ]
    })
}

/// Create a minimal policy file
pub fn minimal_policy() -> serde_json::Value {
    json!({
        "id": "test-policy",
        "name": "Test Policy",
        "version": "1.0.0",
        "rules": []
    })
}

/// Create a policy with NAT gateway limit rule
pub fn policy_with_nat_limit() -> serde_json::Value {
    json!({
        "id": "nat-limit-policy",
        "name": "NAT Gateway Limit Policy",
        "version": "1.0.0",
        "rules": [{
            "id": "limit-nat-gateways",
            "description": "Limit NAT gateways per module",
            "severity": "High",
            "conditions": [{
                "type": "ResourceCount",
                "resource_type": "aws_nat_gateway",
                "operator": "GreaterThan",
                "value": 2
            }],
            "actions": ["Block"]
        }]
    })
}

/// Create a policy with budget rule
pub fn policy_with_budget_rule(max_cost: f64) -> serde_json::Value {
    json!({
        "id": "budget-policy",
        "name": "Budget Policy",
        "version": "1.0.0",
        "rules": [{
            "id": "enforce-budget",
            "description": "Enforce monthly cost budget",
            "severity": "Critical",
            "conditions": [{
                "type": "MonthlyCost",
                "operator": "GreaterThan",
                "value": max_cost
            }],
            "actions": ["Block"]
        }]
    })
}

/// Create a baseline file
pub fn baseline_with_module(module_path: &str, expected_cost: f64) -> serde_json::Value {
    json!({
        "version": "1.0.0",
        "baselines": [{
            "module_path": module_path,
            "expected_monthly_cost": expected_cost,
            "last_updated": "2025-12-06T00:00:00Z",
            "justification": "Initial baseline"
        }]
    })
}

/// Create an SLO file
pub fn slo_monthly_cost(max_cost: f64) -> serde_json::Value {
    json!({
        "version": "1.0.0",
        "slos": [{
            "id": "monthly-cost-limit",
            "name": "Monthly Cost SLO",
            "type": "MonthlyCost",
            "threshold": max_cost,
            "enforcement": "Block"
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_terraform_plan() {
        let plan = minimal_terraform_plan();
        assert_eq!(plan["format_version"], "1.1");
        assert!(plan["resource_changes"].is_array());
    }

    #[test]
    fn test_terraform_plan_with_ec2() {
        let plan = terraform_plan_with_ec2("t3.medium");
        let resources = plan["resource_changes"].as_array().unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0]["type"], "aws_instance");
    }

    #[test]
    fn test_complex_plan_has_multiple_resources() {
        let plan = terraform_plan_complex();
        let resources = plan["resource_changes"].as_array().unwrap();
        assert_eq!(resources.len(), 5);
    }
}
