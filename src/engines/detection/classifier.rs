// Classifier: smell/risk/explosion + regression type

use crate::engines::shared::models::{ChangeAction, RegressionType, ResourceChange};
use serde_json::Value;

/// Classify the type of cost regression
pub struct RegressionClassifier;

impl RegressionClassifier {
    /// Classify a resource change into a regression type
    pub fn classify(change: &ResourceChange) -> RegressionType {
        // Configuration changes
        if Self::is_configuration_change(change) {
            return RegressionType::Configuration;
        }

        // Provisioning changes (new resources or major changes)
        if change.action == ChangeAction::Create || change.action == ChangeAction::Replace {
            return RegressionType::Provisioning;
        }

        // Scaling changes
        if Self::is_scaling_change(change) {
            return RegressionType::Scaling;
        }

        // Traffic-inferred changes
        if Self::is_traffic_change(change) {
            return RegressionType::TrafficInferred;
        }

        // Default to indirect cost
        RegressionType::IndirectCost
    }

    /// Check if this is a configuration change
    fn is_configuration_change(change: &ResourceChange) -> bool {
        if change.action != ChangeAction::Update {
            return false;
        }

        // Check for configuration-type changes
        let old = change.old_config.as_ref();
        let new = change.new_config.as_ref();

        if let (Some(old_val), Some(new_val)) = (old, new) {
            // Billing mode changes
            if Self::field_changed(old_val, new_val, "billing_mode") {
                return true;
            }

            // Lifecycle rules
            if Self::field_changed(old_val, new_val, "lifecycle_rule") {
                return true;
            }

            // Encryption changes
            if Self::field_changed(old_val, new_val, "encryption") {
                return true;
            }

            // Storage class changes
            if Self::field_changed(old_val, new_val, "storage_class") {
                return true;
            }
        }

        false
    }

    /// Check if this is a scaling change
    fn is_scaling_change(change: &ResourceChange) -> bool {
        if change.action != ChangeAction::Update {
            return false;
        }

        let old = change.old_config.as_ref();
        let new = change.new_config.as_ref();

        if let (Some(old_val), Some(new_val)) = (old, new) {
            // Instance count changes
            if Self::numeric_field_increased(old_val, new_val, "count") {
                return true;
            }

            // Autoscaling changes
            if Self::numeric_field_increased(old_val, new_val, "desired_capacity")
                || Self::numeric_field_increased(old_val, new_val, "max_size")
            {
                return true;
            }

            // Lambda concurrency
            if Self::numeric_field_increased(old_val, new_val, "reserved_concurrent_executions") {
                return true;
            }

            // Replica count
            if Self::numeric_field_increased(old_val, new_val, "replica_count")
                || Self::numeric_field_increased(old_val, new_val, "number_of_replicas")
            {
                return true;
            }
        }

        false
    }

    /// Check if this is a traffic change
    fn is_traffic_change(change: &ResourceChange) -> bool {
        // Check resource types that are traffic-sensitive
        matches!(
            change.resource_type.as_str(),
            "aws_nat_gateway" | "aws_lb" | "aws_alb" | "aws_cloudfront_distribution"
        )
    }

    /// Helper: Check if a field changed between old and new
    fn field_changed(old: &Value, new: &Value, field: &str) -> bool {
        if let (Some(old_obj), Some(new_obj)) = (old.as_object(), new.as_object()) {
            let old_field = old_obj.get(field);
            let new_field = new_obj.get(field);

            old_field != new_field && new_field.is_some()
        } else {
            false
        }
    }

    /// Helper: Check if a numeric field increased
    fn numeric_field_increased(old: &Value, new: &Value, field: &str) -> bool {
        if let (Some(old_obj), Some(new_obj)) = (old.as_object(), new.as_object()) {
            if let (Some(old_num), Some(new_num)) = (
                old_obj.get(field).and_then(|v| v.as_f64()),
                new_obj.get(field).and_then(|v| v.as_f64()),
            ) {
                return new_num > old_num;
            }
        }
        false
    }
}

/// Convenience function for classification
pub fn classify_regression(change: &ResourceChange) -> RegressionType {
    RegressionClassifier::classify(change)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ResourceChange, ChangeAction};
    use serde_json::json;

    #[test]
    fn test_provisioning_classification() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.micro"}))
            .build();

        assert_eq!(
            RegressionClassifier::classify(&change),
            RegressionType::Provisioning
        );
    }

    #[test]
    fn test_scaling_classification() {
        let change = ResourceChange::builder()
            .resource_id("aws_autoscaling_group.test")
            .resource_type("aws_autoscaling_group")
            .action(ChangeAction::Update)
            .old_config(json!({"desired_capacity": 2}))
            .new_config(json!({"desired_capacity": 5}))
            .build();

        assert_eq!(
            RegressionClassifier::classify(&change),
            RegressionType::Scaling
        );
    }
}
