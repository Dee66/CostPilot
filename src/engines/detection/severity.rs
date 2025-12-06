// Severity calculation

use crate::engines::shared::models::{ResourceChange, Severity, RegressionType};

/// Calculate severity score (0-100) for a resource change
pub fn calculate_severity_score(
    change: &ResourceChange,
    cost_delta: f64,
    regression_type: &RegressionType,
    confidence: f64,
) -> u32 {
    let mut score = 0.0;

    // Magnitude component (45%)
    let magnitude_score = calculate_magnitude_score(cost_delta);
    score += magnitude_score * 0.45;

    // Confidence component (25%)
    score += confidence * 25.0;

    // Resource type importance (20%)
    let importance_score = calculate_resource_importance(&change.resource_type);
    score += importance_score * 0.20;

    // Blast radius (10%)
    let blast_radius_score = calculate_blast_radius(change);
    score += blast_radius_score * 0.10;

    // Ensure score is within bounds
    score.clamp(0.0, 100.0) as u32
}

/// Calculate magnitude score based on cost delta
fn calculate_magnitude_score(cost_delta: f64) -> f64 {
    let abs_delta = cost_delta.abs();

    if abs_delta < 10.0 {
        10.0
    } else if abs_delta < 50.0 {
        30.0
    } else if abs_delta < 200.0 {
        50.0
    } else if abs_delta < 500.0 {
        70.0
    } else if abs_delta < 1000.0 {
        85.0
    } else {
        100.0
    }
}

/// Calculate resource type importance score
fn calculate_resource_importance(resource_type: &str) -> f64 {
    // High-cost resources get higher importance
    match resource_type {
        // High importance (expensive resources)
        "aws_rds_cluster" | 
        "aws_rds_instance" |
        "aws_elasticache_cluster" |
        "aws_elasticsearch_domain" |
        "aws_eks_cluster" => 100.0,

        // Medium-high importance
        "aws_instance" |
        "aws_nat_gateway" |
        "aws_lb" |
        "aws_alb" => 75.0,

        // Medium importance
        "aws_dynamodb_table" |
        "aws_lambda_function" |
        "aws_s3_bucket" => 50.0,

        // Lower importance
        "aws_cloudwatch_log_group" |
        "aws_security_group" |
        "aws_iam_role" => 25.0,

        // Default
        _ => 40.0,
    }
}

/// Calculate blast radius score based on module and dependencies
fn calculate_blast_radius(change: &ResourceChange) -> f64 {
    let mut score = 50.0; // Base score

    // Root module changes have higher blast radius
    if change.module_path.is_none() || change.module_path.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        score += 30.0;
    }

    // Shared resources have higher blast radius
    if let Some(name) = change.resource_id.split('.').last() {
        if name.contains("shared") || name.contains("common") {
            score += 20.0;
        }
    }

    score.min(100.0)
}

/// Convert severity score to severity level
pub fn score_to_severity(score: u32) -> Severity {
    match score {
        0..=25 => Severity::Low,
        26..=50 => Severity::Medium,
        51..=75 => Severity::High,
        _ => Severity::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::ChangeAction;
    use std::collections::HashMap;

    #[test]
    fn test_severity_calculation() {
        let change = ResourceChange {
            resource_id: "aws_rds_instance.prod".to_string(),
            resource_type: "aws_rds_instance".to_string(),
            action: ChangeAction::Update,
            module_path: None,
            old_config: None,
            new_config: None,
            tags: HashMap::new(),
        };

        let score = calculate_severity_score(
            &change,
            500.0, // High cost delta
            &RegressionType::Scaling,
            0.8, // High confidence
        );

        assert!(score > 50, "High-cost RDS change should have high severity");
    }

    #[test]
    fn test_score_to_severity() {
        assert_eq!(score_to_severity(10), Severity::Low);
        assert_eq!(score_to_severity(40), Severity::Medium);
        assert_eq!(score_to_severity(60), Severity::High);
        assert_eq!(score_to_severity(90), Severity::Critical);
    }
}
