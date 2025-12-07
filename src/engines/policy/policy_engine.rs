use super::exemption_types::ExemptionsFile;
use super::exemption_validator::ExemptionValidator;
use super::policy_types::*;
use super::zero_network::*;
use crate::engines::detection::ResourceChange;
use crate::engines::prediction::CostEstimate;
use crate::engines::shared::models::ChangeAction;
use std::collections::HashMap;

/// Policy evaluation engine with exemption support
/// 
/// This engine guarantees zero-network evaluation through the ZeroNetworkSafe trait.
/// All policy evaluation is deterministic and happens entirely locally.
pub struct PolicyEngine {
    config: PolicyConfig,
    exemptions: Option<ExemptionsFile>,
    exemption_validator: ExemptionValidator,
}

impl PolicyEngine {
    /// Create a new policy engine with configuration
    pub fn new(config: PolicyConfig) -> Self {
        Self {
            config,
            exemptions: None,
            exemption_validator: ExemptionValidator::new(),
        }
    }

    /// Create a new policy engine with configuration and exemptions
    pub fn with_exemptions(config: PolicyConfig, exemptions: ExemptionsFile) -> Self {
        Self {
            config,
            exemptions: Some(exemptions),
            exemption_validator: ExemptionValidator::new(),
        }
    }

    /// Evaluate policies against resource changes and cost estimates
    pub fn evaluate(
        &self,
        changes: &[ResourceChange],
        total_cost: &CostEstimate,
    ) -> PolicyResult {
        let mut result = PolicyResult::new();

        // Evaluate budget policies
        self.evaluate_budgets(total_cost, &mut result);

        // Evaluate resource policies
        self.evaluate_resources(changes, &mut result);

        result
    }
    
    /// Evaluate policies with explicit zero-network guarantee
    /// 
    /// This method requires a ZeroNetworkToken, proving at compile time
    /// that no network calls will be made during evaluation.
    pub fn evaluate_zero_network(
        &self,
        changes: &[ResourceChange],
        total_cost: &CostEstimate,
        token: ZeroNetworkToken,
    ) -> Result<PolicyResult, ZeroNetworkViolation> {
        token.validate()?;
        Ok(self.evaluate(changes, total_cost))
    }

    /// Check if a violation is exempted
    fn is_violation_exempted(&self, policy_name: &str, resource_id: &str) -> bool {
        if let Some(exemptions) = &self.exemptions {
            let matches = self.exemption_validator.find_exemptions(
                exemptions,
                policy_name,
                resource_id,
            );
            !matches.is_empty()
        } else {
            false
        }
    }

    /// Evaluate budget policies
    fn evaluate_budgets(&self, cost: &CostEstimate, result: &mut PolicyResult) {
        // Check global budget
        if let Some(global) = &self.config.budgets.global {
            let monthly_cost = cost.monthly;

            // Check if exceeds limit
            if monthly_cost > global.monthly_limit {
                // Check for exemption
                if !self.is_violation_exempted("global_budget", "global") {
                    result.add_violation(PolicyViolation {
                        policy_name: "global_budget".to_string(),
                        severity: "CRITICAL".to_string(),
                        resource_id: "global".to_string(),
                        message: format!(
                            "Monthly cost ${:.2} exceeds global limit ${:.2}",
                            monthly_cost, global.monthly_limit
                        ),
                        actual_value: format!("${:.2}", monthly_cost),
                        expected_value: format!("<= ${:.2}", global.monthly_limit),
                    });
                }
            }
            // Check warning threshold
            else if monthly_cost > global.monthly_limit * global.warning_threshold {
                result.add_warning(format!(
                    "Monthly cost ${:.2} is at {:.0}% of global limit ${:.2}",
                    monthly_cost,
                    (monthly_cost / global.monthly_limit) * 100.0,
                    global.monthly_limit
                ));
            }
        }

        // Check module budgets (simplified - assumes module info in tags)
        for module_budget in &self.config.budgets.modules {
            // In a real implementation, we'd track per-module costs
            // For MVP, this is a placeholder for the structure
            result.add_warning(format!(
                "Module budget checking for '{}' not yet implemented (requires module tagging)",
                module_budget.name
            ));
        }
    }

    /// Evaluate resource-specific policies
    fn evaluate_resources(&self, changes: &[ResourceChange], result: &mut PolicyResult) {
        // Track NAT gateway count
        let nat_gateway_count = changes
            .iter()
            .filter(|c| c.resource_type == "aws_nat_gateway" && c.action != ChangeAction::Delete)
            .count();

        // Check NAT gateway policy
        if let Some(nat_policy) = &self.config.resources.nat_gateways {
            if nat_gateway_count > nat_policy.max_count {
                if !self.is_violation_exempted("nat_gateway_limit", "nat_gateways") {
                    result.add_violation(PolicyViolation {
                        policy_name: "nat_gateway_limit".to_string(),
                        severity: "HIGH".to_string(),
                        resource_id: "nat_gateways".to_string(),
                        message: format!(
                            "NAT gateway count {} exceeds limit {}",
                            nat_gateway_count, nat_policy.max_count
                        ),
                        actual_value: nat_gateway_count.to_string(),
                        expected_value: format!("<= {}", nat_policy.max_count),
                    });
                }
            }
        }

        // Check EC2 instance policies
        if let Some(ec2_policy) = &self.config.resources.ec2_instances {
            for change in changes {
                if change.resource_type == "aws_instance" && change.action != ChangeAction::Delete {
                    if let Some(config) = &change.new_config {
                        // Check instance type family
                        if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
                            let family = instance_type.split('.').next().unwrap_or("");
                            
                            if !ec2_policy.allowed_families.is_empty()
                                && !ec2_policy.allowed_families.contains(&family.to_string())
                            {
                                if !self.is_violation_exempted("ec2_allowed_families", &change.resource_id) {
                                    result.add_violation(PolicyViolation {
                                        policy_name: "ec2_allowed_families".to_string(),
                                        severity: "MEDIUM".to_string(),
                                        resource_id: change.resource_id.clone(),
                                        message: format!(
                                            "EC2 instance family '{}' not in allowed list",
                                            family
                                        ),
                                        actual_value: family.to_string(),
                                        expected_value: format!("One of: {:?}", ec2_policy.allowed_families),
                                    });
                                }
                            }

                            // Check instance size
                            if let Some(max_size) = &ec2_policy.max_size {
                                let size = instance_type.split('.').nth(1).unwrap_or("");
                                if self.exceeds_size_limit(size, max_size) {
                                    if !self.is_violation_exempted("ec2_max_size", &change.resource_id) {
                                        result.add_violation(PolicyViolation {
                                            policy_name: "ec2_max_size".to_string(),
                                            severity: "MEDIUM".to_string(),
                                            resource_id: change.resource_id.clone(),
                                            message: format!(
                                                "EC2 instance size '{}' exceeds limit '{}'",
                                                size, max_size
                                            ),
                                            actual_value: size.to_string(),
                                            expected_value: format!("<= {}", max_size),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check S3 policies
        if let Some(s3_policy) = &self.config.resources.s3_buckets {
            if s3_policy.require_lifecycle_rules {
                for change in changes {
                    if change.resource_type == "aws_s3_bucket" && change.action != ChangeAction::Delete {
                        let has_lifecycle = change
                            .new_config
                            .as_ref()
                            .and_then(|c| c.get("lifecycle_rule"))
                            .is_some();

                        if !has_lifecycle {
                            if !self.is_violation_exempted("s3_lifecycle_required", &change.resource_id) {
                                result.add_violation(PolicyViolation {
                                    policy_name: "s3_lifecycle_required".to_string(),
                                    severity: "MEDIUM".to_string(),
                                    resource_id: change.resource_id.clone(),
                                    message: "S3 bucket missing lifecycle rules".to_string(),
                                    actual_value: "no lifecycle rules".to_string(),
                                    expected_value: "lifecycle_rule configured".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check Lambda policies
        if let Some(lambda_policy) = &self.config.resources.lambda_functions {
            if lambda_policy.require_concurrency_limit {
                for change in changes {
                    if change.resource_type == "aws_lambda_function" && change.action != ChangeAction::Delete {
                        let has_limit = change
                            .new_config
                            .as_ref()
                            .and_then(|c| c.get("reserved_concurrent_executions"))
                            .is_some();

                        if !has_limit {
                            if !self.is_violation_exempted("lambda_concurrency_required", &change.resource_id) {
                                result.add_violation(PolicyViolation {
                                    policy_name: "lambda_concurrency_required".to_string(),
                                    severity: "HIGH".to_string(),
                                    resource_id: change.resource_id.clone(),
                                    message: "Lambda function missing concurrency limit".to_string(),
                                    actual_value: "no concurrency limit".to_string(),
                                    expected_value: "reserved_concurrent_executions configured".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check DynamoDB policies
        if let Some(dynamo_policy) = &self.config.resources.dynamodb_tables {
            if dynamo_policy.prefer_provisioned {
                for change in changes {
                    if change.resource_type == "aws_dynamodb_table" && change.action != ChangeAction::Delete {
                        if let Some(config) = &change.new_config {
                            let billing_mode = config
                                .get("billing_mode")
                                .and_then(|v| v.as_str())
                                .unwrap_or("PROVISIONED");

                            if billing_mode == "PAY_PER_REQUEST" {
                                if !self.is_violation_exempted("dynamodb_prefer_provisioned", &change.resource_id) {
                                    result.add_violation(PolicyViolation {
                                        policy_name: "dynamodb_prefer_provisioned".to_string(),
                                        severity: "MEDIUM".to_string(),
                                        resource_id: change.resource_id.clone(),
                                        message: "DynamoDB table using PAY_PER_REQUEST billing".to_string(),
                                        actual_value: "PAY_PER_REQUEST".to_string(),
                                        expected_value: "PROVISIONED".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if instance size exceeds limit
    fn exceeds_size_limit(&self, size: &str, max_size: &str) -> bool {
        let size_order = ["nano", "micro", "small", "medium", "large", "xlarge", "2xlarge", "4xlarge", "8xlarge", "16xlarge", "24xlarge", "32xlarge"];
        
        let size_idx = size_order.iter().position(|&s| s == size);
        let max_idx = size_order.iter().position(|&s| s == max_size);

        match (size_idx, max_idx) {
            (Some(s), Some(m)) => s > m,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_budget_evaluation() {
        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies {
                global: Some(BudgetLimit {
                    monthly_limit: 1000.0,
                    warning_threshold: 0.8,
                }),
                modules: vec![],
            },
            resources: ResourcePolicies::default(),
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let engine = PolicyEngine::new(config);
        let cost = CostEstimate {
            hourly: 50.0,
            daily: 1200.0,
            monthly: 1500.0,
            confidence: 0.9,
        };

        let result = engine.evaluate(&[], &cost);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].policy_name, "global_budget");
    }

    #[test]
    fn test_nat_gateway_limit() {
        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies::default(),
            resources: ResourcePolicies {
                nat_gateways: Some(NatGatewayPolicy {
                    max_count: 2,
                    require_justification: true,
                }),
                ..Default::default()
            },
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let engine = PolicyEngine::new(config);
        let changes = vec![
            ResourceChange {
                resource_id: "nat1".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                change_type: "create".to_string(),
                old_config: None,
                new_config: Some(json!({})),
            },
            ResourceChange {
                resource_id: "nat2".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                change_type: "create".to_string(),
                old_config: None,
                new_config: Some(json!({})),
            },
            ResourceChange {
                resource_id: "nat3".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                change_type: "create".to_string(),
                old_config: None,
                new_config: Some(json!({})),
            },
        ];

        let cost = CostEstimate {
            hourly: 1.0,
            daily: 24.0,
            monthly: 720.0,
            confidence: 0.9,
        };

        let result = engine.evaluate(&changes, &cost);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.policy_name == "nat_gateway_limit"));
    }

    #[test]
    fn test_lambda_concurrency_required() {
        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies::default(),
            resources: ResourcePolicies {
                lambda_functions: Some(LambdaPolicy {
                    require_concurrency_limit: true,
                    max_memory_mb: None,
                }),
                ..Default::default()
            },
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let engine = PolicyEngine::new(config);
        let changes = vec![ResourceChange {
            resource_id: "lambda1".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            change_type: "create".to_string(),
            old_config: None,
            new_config: Some(json!({"memory_size": 128})),
        }];

        let cost = CostEstimate {
            hourly: 0.1,
            daily: 2.4,
            monthly: 72.0,
            confidence: 0.9,
        };

        let result = engine.evaluate(&changes, &cost);
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.policy_name == "lambda_concurrency_required"));
    }

    #[test]
    fn test_exemption_filters_violation() {
        use super::exemption_types::*;

        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies::default(),
            resources: ResourcePolicies {
                nat_gateways: Some(NatGatewayPolicy { max_count: 1 }),
                ..Default::default()
            },
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let exemptions = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![PolicyExemption {
                id: "EXE-001".to_string(),
                policy_name: "nat_gateway_limit".to_string(),
                resource_pattern: "nat_gateways".to_string(),
                justification: "Production requirement".to_string(),
                expires_at: "2026-12-31".to_string(),
                approved_by: "ops@example.com".to_string(),
                created_at: "2025-12-01T00:00:00Z".to_string(),
                ticket_ref: Some("JIRA-123".to_string()),
            }],
            metadata: None,
        };

        let engine = PolicyEngine::with_exemptions(config, exemptions);
        
        let changes = vec![
            ResourceChange {
                resource_id: "nat1".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                change_type: "create".to_string(),
                old_config: None,
                new_config: Some(json!({})),
            },
            ResourceChange {
                resource_id: "nat2".to_string(),
                resource_type: "aws_nat_gateway".to_string(),
                change_type: "create".to_string(),
                old_config: None,
                new_config: Some(json!({})),
            },
        ];

        let cost = CostEstimate {
            hourly: 1.0,
            daily: 24.0,
            monthly: 720.0,
            confidence: 0.9,
        };

        let result = engine.evaluate(&changes, &cost);
        // Should pass because exemption filters the violation
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }
}
