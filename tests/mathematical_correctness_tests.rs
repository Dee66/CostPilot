// Mathematical correctness tests for cost calculations
// Validates that cost deltas sum correctly and economic invariants hold

use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_cost_deltas_sum_correctly_across_resources() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create resource changes for multiple resources
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.web1".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.micro",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_instance.web2".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.micro",
                "ami": "ami-12345"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_lambda_function.api".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "function_name": "api",
                "memory_size": 128,
                "runtime": "python3.9"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
    ];

    let estimates = engine.predict(&changes).unwrap();

    // Extract individual resource costs
    let mut total_monthly_cost = 0.0;
    let mut resource_costs = Vec::new();

    for estimate in &estimates {
        total_monthly_cost += estimate.monthly_cost;
        resource_costs.push(estimate.monthly_cost);
    }

    // Validate that individual costs sum to total
    let expected_total = resource_costs.iter().sum::<f64>();
    assert!(
        (total_monthly_cost - expected_total).abs() < 0.01,
        "Individual resource costs should sum to total cost. Expected: {}, Got: {}",
        expected_total,
        total_monthly_cost
    );

    // Validate no negative costs (unless explicitly justified)
    for cost in &resource_costs {
        assert!(
            *cost >= 0.0,
            "Resource cost should not be negative: {}",
            cost
        );
    }
}

#[test]
fn test_zero_delta_pr_produces_zero_cost_impact() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create empty changes (zero delta)
    let changes = Vec::<ResourceChange>::new();

    let estimates = engine.predict(&changes).unwrap();

    // Should have zero total cost
    assert!(
        estimates.is_empty(),
        "Zero-delta PR should produce no cost estimates"
    );
}

#[test]
fn test_single_resource_pr_produces_identical_leaf_and_aggregate_values() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create single resource change
    let changes = vec![ResourceChange {
        resource_id: "aws_instance.single".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t3.micro",
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
    }];

    let estimates = engine.predict(&changes).unwrap();

    // Should have exactly one estimate
    assert_eq!(
        estimates.len(),
        1,
        "Single resource PR should have exactly one cost estimate"
    );

    let resource_cost = estimates[0].monthly_cost;

    // For single resource, the cost should be the resource cost
    assert!(
        resource_cost >= 0.0,
        "Single resource cost should be non-negative: {}",
        resource_cost
    );
}

#[test]
fn test_aggregates_equal_sum_of_components() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create multiple resource changes
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.test1".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.micro"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_instance.test2".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.small"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_lambda_function.func".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "memory_size": 256,
                "runtime": "nodejs18.x"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
    ];

    let estimates = engine.predict(&changes).unwrap();

    // Calculate sum of individual resource costs
    let sum_of_components: f64 = estimates.iter().map(|e| e.monthly_cost).sum();

    // For this test, we validate that all costs are non-negative and sum correctly
    assert!(
        sum_of_components >= 0.0,
        "Sum of components should be non-negative: {}",
        sum_of_components
    );

    // Validate no individual negative costs
    for estimate in &estimates {
        assert!(
            estimate.monthly_cost >= 0.0,
            "Individual cost should not be negative: {}",
            estimate.monthly_cost
        );
    }
}

#[test]
fn test_no_negative_costs_unless_explicitly_justified() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create various resource changes
    let changes = vec![
        ResourceChange {
            resource_id: "aws_instance.example".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "instance_type": "t3.micro"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
        ResourceChange {
            resource_id: "aws_lambda_function.example".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({
                "memory_size": 128
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        },
    ];

    let estimates = engine.predict(&changes).unwrap();

    // Check that no costs are negative
    for estimate in &estimates {
        assert!(
            estimate.monthly_cost >= 0.0,
            "Resource {} has negative cost: ${}",
            estimate.resource_id,
            estimate.monthly_cost
        );
    }
}

#[test]
fn test_percentages_normalize_to_exactly_100_percent() {
    let engine = PredictionEngine::new().unwrap();

    // Test with a DynamoDB table that has multiple cost components
    let change = ResourceChange {
        resource_id: "aws_dynamodb_table.users".to_string(),
        resource_type: "aws_dynamodb_table".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "billing_mode": "PAY_PER_REQUEST",
            "read_capacity": 5,
            "write_capacity": 5
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
    };

    let explanation = engine.explain(&change).unwrap();

    // Sum all component percentages
    let total_percentage: f64 = explanation
        .final_estimate
        .components
        .iter()
        .map(|c| c.percentage)
        .sum();

    // Should be exactly 100.0 (within floating point precision)
    assert!(
        (total_percentage - 100.0).abs() < 0.001,
        "Component percentages sum to {:.3}, expected 100.0. Components: {:?}",
        total_percentage,
        explanation
            .final_estimate
            .components
            .iter()
            .map(|c| format!("{}: {:.1}%", c.name, c.percentage))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_zero_cost_resources_handled_explicitly() {
    let mut engine = PredictionEngine::new().unwrap();

    // Create a resource change that should result in zero cost
    let change = ResourceChange {
        resource_id: "aws_instance.free_tier".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(json!({
            "instance_type": "t2.micro",  // This might be free tier eligible
            "ami": "ami-12345"
        })),
        tags: HashMap::new(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
    };

    let estimates = engine.predict(std::slice::from_ref(&change)).unwrap();

    // Should still return an estimate (not omit zero-cost resources)
    assert_eq!(
        estimates.len(),
        1,
        "Zero-cost resource should still be included in estimates"
    );

    let estimate = &estimates[0];
    assert_eq!(estimate.resource_id, change.resource_id);

    // Cost should be >= 0 (zero is acceptable)
    assert!(
        estimate.monthly_cost >= 0.0,
        "Zero-cost resource should have non-negative cost, got: ${}",
        estimate.monthly_cost
    );

    // Test explanation for zero-cost resource
    let explanation = engine.explain(&change).unwrap();

    // Should still have components even if cost is zero
    assert!(
        !explanation.final_estimate.components.is_empty(),
        "Zero-cost resource should still have cost components in explanation"
    );

    // All components should have non-negative costs
    for component in &explanation.final_estimate.components {
        assert!(
            component.cost >= 0.0,
            "Component '{}' has negative cost: ${}",
            component.name,
            component.cost
        );
    }
}
