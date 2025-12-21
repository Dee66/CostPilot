use costpilot::engines::prediction::{PredictionEngine, prediction_engine::CostHeuristics};
use costpilot::engines::shared::models::{ResourceChange, ChangeAction};
use std::collections::HashMap;

/// Manual ground-truth test cases with hand-computed expected costs
/// These serve as regression tests against manually verified calculations
#[cfg(test)]
mod ground_truth_tests {
    use super::*;

    /// Test case: Single t2.micro EC2 instance
    /// Manual calculation: Free edition uses static cost of $150/month for aws_instance
    #[test]
    fn test_single_ec2_instance_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = ResourceChange {
            resource_id: "test-ec2-single".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({
                "instance_type": "t2.micro",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        };

        let estimates = engine.predict(&[change]).unwrap();
        assert_eq!(estimates.len(), 1, "Expected exactly one estimate");

        let estimate = &estimates[0];
        assert_eq!(estimate.resource_id, "test-ec2-single");

        // Free edition static cost for aws_instance is $150/month
        assert!((estimate.monthly_cost - 150.0).abs() < 0.01,
            "aws_instance cost {} should be $150.00 in free edition", estimate.monthly_cost);

        assert!(estimate.confidence_score >= 0.0,
            "Confidence score should be defined, got {}", estimate.confidence_score);
    }

    /// Test case: Single RDS db.t2.micro instance
    /// Manual calculation: Free edition uses static cost of $0/month for non-EC2 resources
    #[test]
    fn test_single_rds_instance_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = ResourceChange {
            resource_id: "test-rds-single".to_string(),
            resource_type: "aws_db_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({
                "instance_class": "db.t2.micro",
                "engine": "mysql"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        };

        let estimates = engine.predict(&[change]).unwrap();
        assert_eq!(estimates.len(), 1, "Expected exactly one estimate");

        let estimate = &estimates[0];
        assert_eq!(estimate.resource_id, "test-rds-single");

        // Free edition static cost for non-EC2 resources is $0/month
        assert!((estimate.monthly_cost - 0.0).abs() < 0.01,
            "aws_db_instance cost {} should be $0.00 in free edition", estimate.monthly_cost);
    }

    /// Test case: Two resources - manual aggregation verification
    /// EC2 aws_instance ($150) + RDS aws_db_instance ($0) = $150 total
    #[test]
    fn test_two_resources_aggregation_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let changes = vec![
            ResourceChange {
                resource_id: "test-ec2-multi".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({
                    "instance_type": "t2.micro",
                    "ami": "ami-12345"
                })),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
            },
            ResourceChange {
                resource_id: "test-rds-multi".to_string(),
                resource_type: "aws_db_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({
                    "instance_class": "db.t2.micro",
                    "engine": "mysql"
                })),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
            },
        ];

        let estimates = engine.predict(&changes).unwrap();
        assert_eq!(estimates.len(), 2, "Expected exactly two estimates");

        let total_cost: f64 = estimates.iter().map(|e| e.monthly_cost).sum();

        // Combined cost should be exactly $150 ($150 + $0)
        assert!((total_cost - 150.0).abs() < 0.01,
            "Total cost {} should be exactly $150.00 ($150 + $0)", total_cost);

        // Verify individual costs are correct
        let ec2_estimate = estimates.iter().find(|e| e.resource_id == "test-ec2-multi").unwrap();
        let rds_estimate = estimates.iter().find(|e| e.resource_id == "test-rds-multi").unwrap();

        assert!((ec2_estimate.monthly_cost - 150.0).abs() < 0.01,
            "EC2 cost should be $150.00, got {}", ec2_estimate.monthly_cost);
        assert!((rds_estimate.monthly_cost - 0.0).abs() < 0.01,
            "RDS cost should be $0.00, got {}", rds_estimate.monthly_cost);
    }

    /// Test case: Boundary case - small instance (t2.nano)
    /// Manual calculation: Free edition uses same static cost of $150 for all aws_instance types
    #[test]
    fn test_boundary_small_instance_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = ResourceChange {
            resource_id: "test-ec2-nano".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({
                "instance_type": "t2.nano",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        };

        let estimates = engine.predict(&[change]).unwrap();
        assert_eq!(estimates.len(), 1, "Expected exactly one estimate");

        let estimate = &estimates[0];

        // Free edition static cost for aws_instance is $150/month regardless of instance type
        assert!((estimate.monthly_cost - 150.0).abs() < 0.01,
            "t2.nano cost {} should be $150.00 in free edition (static for all instances)", estimate.monthly_cost);

        // Should be positive
        assert!(estimate.monthly_cost > 0.0, "Cost should be positive");
    }

    /// Test case: Boundary case - large instance (m5.24xlarge)
    /// Manual calculation: Free edition uses same static cost of $150 for all aws_instance types
    #[test]
    fn test_boundary_large_instance_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = ResourceChange {
            resource_id: "test-ec2-large".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({
                "instance_type": "m5.24xlarge",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        };

        let estimates = engine.predict(&[change]).unwrap();
        assert_eq!(estimates.len(), 1, "Expected exactly one estimate");

        let estimate = &estimates[0];

        // Free edition static cost for aws_instance is $150/month regardless of instance type
        assert!((estimate.monthly_cost - 150.0).abs() < 0.01,
            "m5.24xlarge cost {} should be $150.00 in free edition (static for all instances)", estimate.monthly_cost);

        // Should be positive
        assert!(estimate.monthly_cost > 0.0, "Cost should be positive");
    }

    /// Test case: Zero-cost resource (should not crash)
    /// Manual verification: Some resources might legitimately cost $0
    #[test]
    fn test_zero_cost_boundary_ground_truth() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = ResourceChange {
            resource_id: "test-zero-cost".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({
                "instance_type": "t2.micro",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: Some(0.0), // Explicitly set to zero
            config: None,
            cost_impact: None,
        };

        let estimates = engine.predict(&[change]).unwrap();
        assert_eq!(estimates.len(), 1, "Expected exactly one estimate");

        let estimate = &estimates[0];

        // Should handle zero cost gracefully (not crash)
        assert!(estimate.monthly_cost >= 0.0, "Cost should not be negative, got {}", estimate.monthly_cost);

        // Even with explicit zero, the engine might still estimate based on instance type
        // This is acceptable behavior - the test ensures no crashes
    }

    /// Test case: Manual calculation fixture
    /// This represents a human-verifiable calculation that can be checked by hand
    /// Free edition: 2 x aws_instance = 2 x $150 = $300 total
    #[test]
    fn test_manual_calculation_fixture() {
        // Fixture: 2 x t2.small instances
        // Manual calc: Free edition static = $150/instance → 2 x $150 = $300/month total
        let mut engine = PredictionEngine::new().unwrap();
        let changes = vec![
            ResourceChange {
                resource_id: "fixture-instance-1".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({
                    "instance_type": "t2.small"
                })),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
            },
            ResourceChange {
                resource_id: "fixture-instance-2".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({
                    "instance_type": "t2.small"
                })),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
            },
        ];

        let estimates = engine.predict(&changes).unwrap();
        assert_eq!(estimates.len(), 2, "Expected exactly two estimates");

        let total_cost: f64 = estimates.iter().map(|e| e.monthly_cost).sum();

        // Expected: $300/month for 2 x $150 instances
        assert!((total_cost - 300.0).abs() < 0.01,
            "2 x aws_instance total cost {} should be exactly $300.00 (2 x $150)", total_cost);

        // Each instance should cost exactly $150
        let cost1 = estimates[0].monthly_cost;
        let cost2 = estimates[1].monthly_cost;
        assert!((cost1 - 150.0).abs() < 0.01, "First instance cost should be $150.00, got {}", cost1);
        assert!((cost2 - 150.0).abs() < 0.01, "Second instance cost should be $150.00, got {}", cost2);
    }
}
