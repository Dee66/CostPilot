// Confidence calculation for predictions

use crate::engines::shared::models::ResourceChange;

/// Calculate confidence score for a prediction (0.0 - 1.0)
pub fn calculate_confidence(
    change: &ResourceChange,
    cold_start_used: bool,
    resource_type: &str,
) -> f64 {
    let mut confidence = 1.0;

    // Reduce confidence if cold start was used
    if cold_start_used {
        confidence *= 0.6;
    }

    // Reduce confidence for resources with unknown/computed values
    if has_unknown_values(change) {
        confidence *= 0.75;
    }

    // Adjust confidence based on resource type predictability
    confidence *= get_resource_predictability(resource_type);

    // Reduce confidence for complex module structures
    if let Some(module_path) = &change.module_path {
        let depth = module_path.split('.').count();
        if depth > 3 {
            confidence *= 0.9; // Nested modules are less predictable
        }
    }

    // Ensure confidence is within bounds
    confidence.clamp(0.0, 1.0)
}

/// Check if resource has unknown or computed values
fn has_unknown_values(change: &ResourceChange) -> bool {
    if let Some(config) = &change.new_config {
        // Check for null values which indicate unknown/computed
        if config.is_null() {
            return true;
        }

        // Check for nested null values
        if let Some(obj) = config.as_object() {
            for value in obj.values() {
                if value.is_null() {
                    return true;
                }
            }
        }
    }

    false
}

/// Get predictability score for resource type
fn get_resource_predictability(resource_type: &str) -> f64 {
    match resource_type {
        // High predictability (well-defined pricing)
        "aws_instance" |
        "aws_rds_instance" |
        "aws_nat_gateway" |
        "aws_lb" |
        "aws_alb" => 0.95,

        // Medium-high predictability
        "aws_dynamodb_table" |
        "aws_elasticache_cluster" |
        "aws_elasticsearch_domain" => 0.85,

        // Medium predictability (usage-dependent)
        "aws_lambda_function" |
        "aws_s3_bucket" => 0.70,

        // Lower predictability (complex pricing)
        "aws_ecs_service" |
        "aws_eks_cluster" |
        "aws_cloudfront_distribution" => 0.60,

        // Very low predictability (data transfer heavy)
        "aws_vpc_endpoint" |
        "aws_api_gateway_rest_api" => 0.50,

        // Default for unknown types
        _ => 0.65,
    }
}

/// Calculate confidence interval width
pub fn calculate_interval_width(confidence: f64, base_interval: f64) -> f64 {
    // Wider intervals for lower confidence
    base_interval * (2.0 - confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::ChangeAction;
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_confidence_calculation() {
        let change = ResourceChange {
            resource_id: "aws_instance.test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": "t3.micro"})),
            tags: HashMap::new(),
        };

        let confidence = calculate_confidence(&change, false, "aws_instance");
        assert!(confidence > 0.9, "EC2 without cold start should have high confidence");

        let confidence_with_cold_start = calculate_confidence(&change, true, "aws_instance");
        assert!(confidence_with_cold_start < confidence, "Cold start should reduce confidence");
    }

    #[test]
    fn test_resource_predictability() {
        assert_eq!(get_resource_predictability("aws_instance"), 0.95);
        assert_eq!(get_resource_predictability("aws_lambda_function"), 0.70);
        assert!(get_resource_predictability("unknown_type") < 0.7);
    }

    #[test]
    fn test_has_unknown_values() {
        let change_with_null = ResourceChange {
            resource_id: "test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": null})),
            tags: HashMap::new(),
        };

        assert!(has_unknown_values(&change_with_null));

        let change_without_null = ResourceChange {
            resource_id: "test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": "t3.micro"})),
            tags: HashMap::new(),
        };

        assert!(!has_unknown_values(&change_without_null));
    }

    #[test]
    fn test_interval_width() {
        let width_high_confidence = calculate_interval_width(0.9, 0.25);
        let width_low_confidence = calculate_interval_width(0.5, 0.25);

        assert!(width_low_confidence > width_high_confidence);
    }
}
