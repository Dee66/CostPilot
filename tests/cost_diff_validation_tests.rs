// Cost diff validation tests

use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange, Severity};
use serde_json::json;

#[test]
fn test_cost_diff_never_negative_for_resource_additions() {
    let mut engine = PredictionEngine::new().unwrap();

    let before = vec![];
    let after = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Create)
        .new_config(json!({"instance_type": "t3.medium"}))
        .monthly_cost(50.0)
        .build()];

    let before_cost = engine.predict_total_cost(&before).unwrap();
    let after_cost = engine.predict_total_cost(&after).unwrap();

    let delta = after_cost.monthly - before_cost.monthly;

    // Adding resources should never result in negative cost delta
    assert!(
        delta >= 0.0,
        "Delta should not be negative when adding resources: {}",
        delta
    );
}

#[test]
fn test_cost_diff_negative_only_for_deletions() {
    let mut engine = PredictionEngine::new().unwrap();

    let before = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Delete)
        .old_config(json!({"instance_type": "t3.xlarge"}))
        .monthly_cost(140.0)
        .build()];
    let after = vec![];

    let before_cost = engine.predict_total_cost(&before).unwrap();
    let after_cost = engine.predict_total_cost(&after).unwrap();

    let delta = after_cost.monthly - before_cost.monthly;

    // Removing resources should result in negative delta (cost savings)
    assert!(
        delta <= 0.0,
        "Delta should be negative or zero when removing resources: {}",
        delta
    );
}

#[test]
fn test_cost_diff_zero_for_no_changes() {
    let mut engine = PredictionEngine::new().unwrap();

    let before = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::NoOp)
        .old_config(json!({"instance_type": "t3.medium"}))
        .new_config(json!({"instance_type": "t3.medium"}))
        .monthly_cost(70.0)
        .build()];
    let after = before.clone();

    let before_cost = engine.predict_total_cost(&before).unwrap();
    let after_cost = engine.predict_total_cost(&after).unwrap();

    let delta = after_cost.monthly - before_cost.monthly;

    assert_eq!(delta, 0.0, "Delta should be zero for no-op changes");
}

#[test]
fn test_cost_diff_positive_for_upgrades() {
    let mut engine = PredictionEngine::new().unwrap();

    let before = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Update)
        .old_config(json!({"instance_type": "t3.small"}))
        .new_config(json!({"instance_type": "t3.xlarge"}))
        .monthly_cost(35.0)
        .build()];
    let after = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Update)
        .old_config(json!({"instance_type": "t3.small"}))
        .new_config(json!({"instance_type": "t3.xlarge"}))
        .monthly_cost(140.0)
        .build()];

    let before_cost = engine.predict_total_cost(&before).unwrap();
    let after_cost = engine.predict_total_cost(&after).unwrap();

    let delta = after_cost.monthly - before_cost.monthly;

    // Upgrading should increase cost
    assert!(
        delta > 0.0,
        "Delta should be positive for upgrades: {}",
        delta
    );
}

#[test]
fn test_cost_diff_negative_for_downgrades() {
    let mut engine = PredictionEngine::new().unwrap();

    let before = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Update)
        .old_config(json!({"instance_type": "t3.xlarge"}))
        .new_config(json!({"instance_type": "t3.small"}))
        .monthly_cost(140.0)
        .build()];
    let after = vec![ResourceChange::builder()
        .resource_id("aws_instance.web")
        .resource_type("aws_instance")
        .action(ChangeAction::Update)
        .old_config(json!({"instance_type": "t3.xlarge"}))
        .new_config(json!({"instance_type": "t3.small"}))
        .monthly_cost(35.0)
        .build()];

    let before_cost = engine.predict_total_cost(&before).unwrap();
    let after_cost = engine.predict_total_cost(&after).unwrap();

    let delta = after_cost.monthly - before_cost.monthly;

    // Downgrading should decrease cost
    assert!(
        delta < 0.0,
        "Delta should be negative for downgrades: {}",
        delta
    );
}

#[test]
fn test_cost_diff_consistent_with_individual_predictions() {
    let mut engine = PredictionEngine::new().unwrap();

    let resources = vec![
        ResourceChange::builder()
            .resource_id("aws_instance.web1")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.medium"}))
            .monthly_cost(70.0)
            .build(),
        ResourceChange::builder()
            .resource_id("aws_instance.web2")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.small"}))
            .monthly_cost(35.0)
            .build(),
    ];

    let total_cost = engine.predict_total_cost(&resources).unwrap();

    let individual_sum: f64 = resources
        .iter()
        .map(|r| r.monthly_cost.unwrap_or(0.0))
        .sum();

    // Total should approximately match sum of individual costs
    assert!(
        (total_cost.monthly - individual_sum).abs() < 1.0,
        "Total cost ({}) should match sum of individual costs ({})",
        total_cost.monthly,
        individual_sum
    );
}
