// Root cause analysis

use crate::engines::explain::anti_patterns::AntiPattern;
use crate::engines::shared::models::{Detection, RegressionType, ResourceChange};
use serde::{Deserialize, Serialize};

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub primary_cause: String,
    pub contributing_factors: Vec<String>,
    pub category: RootCauseCategory,
    pub confidence: f64,
}

/// Root cause categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RootCauseCategory {
    ConfigurationChange,
    ScalingDecision,
    ProvisioningChoice,
    MissingOptimization,
    AntiPattern,
    Unknown,
}

impl RootCauseAnalysis {
    /// Analyze root cause of cost regression
    pub fn analyze(
        change: &ResourceChange,
        detection: &Detection,
        anti_patterns: &[AntiPattern],
    ) -> Self {
        // If anti-patterns detected, they're likely the root cause
        if !anti_patterns.is_empty() {
            return Self::from_anti_patterns(anti_patterns, change);
        }

        // Otherwise, analyze based on regression type
        match detection.regression_type {
            RegressionType::Configuration => Self::analyze_configuration(change),
            RegressionType::Scaling => Self::analyze_scaling(change),
            RegressionType::Provisioning => Self::analyze_provisioning(change),
            RegressionType::TrafficInferred | RegressionType::Traffic => {
                Self::analyze_traffic(change)
            }
            RegressionType::IndirectCost | RegressionType::Indirect => {
                Self::analyze_indirect(change)
            }
        }
    }

    /// Root cause from anti-patterns
    fn from_anti_patterns(patterns: &[AntiPattern], _change: &ResourceChange) -> Self {
        let primary = &patterns[0];
        let mut contributing_factors = Vec::new();

        // Add all pattern descriptions
        for pattern in patterns {
            contributing_factors.push(pattern.description.clone());
        }

        RootCauseAnalysis {
            primary_cause: format!("{}: {}", primary.pattern_name, primary.description),
            contributing_factors,
            category: RootCauseCategory::AntiPattern,
            confidence: 0.9, // High confidence when patterns detected
        }
    }

    /// Analyze configuration changes
    fn analyze_configuration(change: &ResourceChange) -> Self {
        let mut contributing_factors = Vec::new();
        let mut primary_cause = "Configuration change detected".to_string();

        if let Some(new_config) = &change.new_config {
            // Analyze specific configuration changes
            match change.resource_type.as_str() {
                "aws_instance" => {
                    if let Some(instance_type) = new_config.get("instance_type") {
                        primary_cause = format!("EC2 instance type changed to {}", instance_type);
                        contributing_factors
                            .push("Instance type directly affects hourly compute cost".to_string());
                    }
                }
                "aws_rds_instance" => {
                    if let Some(instance_class) = new_config.get("instance_class") {
                        primary_cause = format!("RDS instance class changed to {}", instance_class);
                        contributing_factors.push(
                            "Database instance class determines compute and memory costs"
                                .to_string(),
                        );
                    }
                }
                "aws_dynamodb_table" => {
                    if let Some(billing_mode) = new_config.get("billing_mode") {
                        primary_cause = format!("DynamoDB billing mode set to {}", billing_mode);
                        contributing_factors.push("Billing mode (provisioned vs on-demand) significantly impacts cost structure".to_string());
                    }
                }
                "aws_lambda_function" => {
                    if let Some(memory) = new_config.get("memory_size") {
                        primary_cause = format!("Lambda memory configured to {} MB", memory);
                        contributing_factors.push(
                            "Memory allocation affects both execution speed and cost".to_string(),
                        );
                    }
                }
                _ => {
                    contributing_factors
                        .push("Resource configuration change affects pricing".to_string());
                }
            }
        }

        RootCauseAnalysis {
            primary_cause,
            contributing_factors,
            category: RootCauseCategory::ConfigurationChange,
            confidence: 0.85,
        }
    }

    /// Analyze scaling changes
    fn analyze_scaling(change: &ResourceChange) -> Self {
        let mut contributing_factors = Vec::new();
        let primary_cause = format!("Scaling change in {}", change.resource_type);

        contributing_factors.push("Resource capacity increased to handle higher load".to_string());
        contributing_factors.push("Scaling directly multiplies per-unit costs".to_string());

        RootCauseAnalysis {
            primary_cause,
            contributing_factors,
            category: RootCauseCategory::ScalingDecision,
            confidence: 0.80,
        }
    }

    /// Analyze provisioning changes
    fn analyze_provisioning(change: &ResourceChange) -> Self {
        let mut contributing_factors = Vec::new();
        let primary_cause = format!("New {} provisioned", change.resource_type);

        contributing_factors.push("New resource adds recurring infrastructure costs".to_string());

        if change.resource_type.contains("nat_gateway") {
            contributing_factors
                .push("NAT Gateways have high fixed costs ($32.85/month)".to_string());
        } else if change.resource_type.contains("lb") || change.resource_type.contains("alb") {
            contributing_factors
                .push("Load balancers incur hourly charges plus LCU costs".to_string());
        }

        RootCauseAnalysis {
            primary_cause,
            contributing_factors,
            category: RootCauseCategory::ProvisioningChoice,
            confidence: 0.90,
        }
    }

    /// Analyze traffic changes
    fn analyze_traffic(change: &ResourceChange) -> Self {
        let contributing_factors = vec![
            "Traffic-dependent resources scale costs with usage".to_string(),
            "Unexpected traffic spikes can cause significant cost increases".to_string(),
        ];

        RootCauseAnalysis {
            primary_cause: format!("Traffic-dependent resource {} added", change.resource_type),
            contributing_factors,
            category: RootCauseCategory::ProvisioningChoice,
            confidence: 0.70,
        }
    }

    /// Analyze indirect changes
    fn analyze_indirect(change: &ResourceChange) -> Self {
        let contributing_factors = vec![
            "Indirect cost impact through related resources".to_string(),
            "May affect data transfer, API calls, or storage patterns".to_string(),
        ];

        RootCauseAnalysis {
            primary_cause: format!(
                "Change to {} may have indirect cost effects",
                change.resource_type
            ),
            contributing_factors,
            category: RootCauseCategory::Unknown,
            confidence: 0.60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ResourceChange, ChangeAction, Detection, Severity, RegressionType};
    use serde_json::json;

    #[test]
    fn test_configuration_analysis() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "m5.large"}))
            .build();

        let detection = Detection::builder()
            .resource_id("aws_instance.test")
            .regression_type(RegressionType::Configuration)
            .severity(Severity::Medium)
            .rule_id("test")
            .build();

        let analysis = RootCauseAnalysis::analyze(&change, &detection, &[]);

        assert!(analysis.primary_cause.contains("instance type"));
        assert!(matches!(
            analysis.category,
            RootCauseCategory::ConfigurationChange
        ));
        assert!(analysis.confidence > 0.8);
    }

    #[test]
    fn test_anti_pattern_root_cause() {
        let change = ResourceChange::builder()
            .resource_id("aws_nat_gateway.test")
            .resource_type("aws_nat_gateway")
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let detection = Detection::builder()
            .resource_id("aws_nat_gateway.test")
            .regression_type(RegressionType::Provisioning)
            .severity(Severity::High)
            .rule_id("test")
            .build();

        let pattern = AntiPattern {
            pattern_id: "TEST".to_string(),
            pattern_name: "Test Pattern".to_string(),
            description: "Test description".to_string(),
            severity: "HIGH".to_string(),
            detected_in: "test".to_string(),
            evidence: vec![],
            suggested_fix: None,
            cost_impact: None,
        };

        let analysis = RootCauseAnalysis::analyze(&change, &detection, &[pattern]);

        assert!(matches!(analysis.category, RootCauseCategory::AntiPattern));
        assert!(analysis.confidence > 0.85);
    }
}
