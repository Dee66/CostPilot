// Free edition heuristics - minimal static cost rules

use serde::{Deserialize, Serialize};

/// Free edition heuristics with static rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeHeuristics {
    pub rules: Vec<FreeRule>,
}

/// Simple cost rule for a resource type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeRule {
    pub resource_type: String,
    pub base_cost: f64,
}

impl FreeHeuristics {
    /// Load hardcoded minimal heuristics for Free edition
    pub fn load_free_heuristics() -> Self {
        Self {
            rules: vec![
                // Compute
                FreeRule {
                    resource_type: "aws_instance".to_string(),
                    base_cost: 150.0, // Free edition static cost for EC2 instances
                },
                FreeRule {
                    resource_type: "aws_lambda_function".to_string(),
                    base_cost: 5.0, // ~$5/month baseline
                },
                FreeRule {
                    resource_type: "aws_ecs_service".to_string(),
                    base_cost: 40.0,
                },
                FreeRule {
                    resource_type: "aws_eks_cluster".to_string(),
                    base_cost: 73.0, // $0.10/hour
                },
                // Storage
                FreeRule {
                    resource_type: "aws_s3_bucket".to_string(),
                    base_cost: 2.3, // ~100GB at $0.023/GB
                },
                FreeRule {
                    resource_type: "aws_ebs_volume".to_string(),
                    base_cost: 8.0, // ~100GB gp3
                },
                FreeRule {
                    resource_type: "aws_efs_file_system".to_string(),
                    base_cost: 30.0,
                },
                // Database
                FreeRule {
                    resource_type: "aws_db_instance".to_string(),
                    base_cost: 0.0, // Free edition static cost for RDS instances
                },
                FreeRule {
                    resource_type: "aws_dynamodb_table".to_string(),
                    base_cost: 10.0,
                },
                FreeRule {
                    resource_type: "aws_rds_cluster".to_string(),
                    base_cost: 150.0,
                },
                // Networking
                FreeRule {
                    resource_type: "aws_nat_gateway".to_string(),
                    base_cost: 32.85, // $0.045/hour
                },
                FreeRule {
                    resource_type: "aws_lb".to_string(),
                    base_cost: 16.43, // ALB baseline
                },
                FreeRule {
                    resource_type: "aws_alb".to_string(),
                    base_cost: 16.43,
                },
                FreeRule {
                    resource_type: "aws_elb".to_string(),
                    base_cost: 18.0,
                },
                // Default fallback
                FreeRule {
                    resource_type: "_default".to_string(),
                    base_cost: 10.0,
                },
            ],
        }
    }

    /// Get base cost for a resource type
    pub fn get_base_cost(&self, resource_type: &str) -> f64 {
        self.rules
            .iter()
            .find(|r| r.resource_type == resource_type)
            .or_else(|| self.rules.iter().find(|r| r.resource_type == "_default"))
            .map(|r| r.base_cost)
            .unwrap_or(10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_free_heuristics() {
        let heuristics = FreeHeuristics::load_free_heuristics();
        assert!(!heuristics.rules.is_empty());
    }

    #[test]
    fn test_get_base_cost() {
        let heuristics = FreeHeuristics::load_free_heuristics();
        assert_eq!(heuristics.get_base_cost("aws_instance"), 150.0);
        assert_eq!(heuristics.get_base_cost("aws_db_instance"), 0.0);
        assert_eq!(heuristics.get_base_cost("aws_s3_bucket"), 2.3);
        assert_eq!(heuristics.get_base_cost("unknown_type"), 10.0);
    }
}
