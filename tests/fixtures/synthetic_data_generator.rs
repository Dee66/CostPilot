use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Synthetic data generator for test fixtures
pub struct SyntheticDataGenerator {
    faker: Faker,
}

impl SyntheticDataGenerator {
    pub fn new() -> Self {
        Self {
            faker: Faker::new(),
        }
    }

    /// Generate synthetic Terraform plan data
    pub fn generate_terraform_plan(&self, resource_type: &str) -> serde_json::Value {
        match resource_type {
            "aws_instance" => self.generate_ec2_instance(),
            "aws_rds_instance" => self.generate_rds_instance(),
            "aws_s3_bucket" => self.generate_s3_bucket(),
            _ => self.generate_generic_resource(resource_type),
        }
    }

    /// Generate synthetic cost policy
    pub fn generate_cost_policy(&self) -> serde_yaml::Value {
        serde_yaml::from_str(&format!(r#"
name: synthetic-cost-policy-{}
description: "Synthetic cost policy for testing"
version: "1.0"
rules:
  - name: budget-limit
    condition: "monthly_cost > {}"
    action: alert
    severity: high
  - name: instance-optimization
    condition: "instance_type in ['t2.micro', 't2.small']"
    action: recommend
    recommendation: "Consider upgrading to t3 instances for better performance"
metadata:
  created_by: synthetic-data-generator
  created_at: "{}"
  environment: test
"#, self.faker.uuid(), self.faker.number(100..1000), self.faker.date())).unwrap()
    }

    fn generate_ec2_instance(&self) -> serde_json::Value {
        serde_json::json!({
            "resource": {
                "aws_instance": {
                    "test_instance": {
                        "instance_type": self.faker.random_element(&["t3.micro", "t3.small", "t3.medium"]),
                        "ami": self.faker.uuid(),
                        "tags": {
                            "Name": self.faker.words(2),
                            "Environment": "test"
                        }
                    }
                }
            }
        })
    }

    fn generate_rds_instance(&self) -> serde_json::Value {
        serde_json::json!({
            "resource": {
                "aws_db_instance": {
                    "test_db": {
                        "instance_class": self.faker.random_element(&["db.t3.micro", "db.t3.small"]),
                        "engine": "mysql",
                        "engine_version": "8.0",
                        "allocated_storage": self.faker.number(20..100)
                    }
                }
            }
        })
    }

    fn generate_s3_bucket(&self) -> serde_json::Value {
        serde_json::json!({
            "resource": {
                "aws_s3_bucket": {
                    "test_bucket": {
                        "bucket": format!("synthetic-bucket-{}", self.faker.uuid()),
                        "tags": {
                            "Environment": "test"
                        }
                    }
                }
            }
        })
    }

    fn generate_generic_resource(&self, resource_type: &str) -> serde_json::Value {
        serde_json::json!({
            "resource": {
                resource_type: {
                    "synthetic_resource": {
                        "name": format!("synthetic-{}", self.faker.words(1)),
                        "description": self.faker.sentence()
                    }
                }
            }
        })
    }
}

/// Simple faker implementation for synthetic data
struct Faker {
    // In a real implementation, this would use a proper faker library
}

impl Faker {
    fn new() -> Self {
        Self {}
    }

    fn uuid(&self) -> String {
        "550e8400-e29b-41d4-a716-446655440000".to_string()
    }

    fn words(&self, count: usize) -> String {
        ["synthetic", "test", "data", "sample", "example"][..count].join("-")
    }

    fn sentence(&self) -> String {
        "This is a synthetic test data entry.".to_string()
    }

    fn number(&self, range: std::ops::Range<i32>) -> i32 {
        42 // Fixed for reproducibility
    }

    fn date(&self) -> String {
        "2024-01-01T00:00:00Z".to_string()
    }

    fn random_element<T: Clone>(&self, elements: &[T]) -> T {
        elements[0].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_data_generation() {
        let generator = SyntheticDataGenerator::new();

        let tf_plan = generator.generate_terraform_plan("aws_instance");
        assert!(tf_plan.is_object());

        let policy = generator.generate_cost_policy();
        assert!(policy.is_mapping());
    }
}
