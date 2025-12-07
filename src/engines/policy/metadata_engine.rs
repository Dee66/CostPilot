use super::policy_metadata::*;
use super::policy_repository::*;
use super::policy_types::*;
use super::zero_network::*;
use crate::engines::detection::ResourceChange;
use crate::engines::prediction::CostEstimate;
use std::collections::HashMap;

/// Enhanced policy engine with full metadata support
/// 
/// All policy evaluation is zero-network and deterministic.
pub struct MetadataPolicyEngine {
    /// Policy repository with metadata
    repository: PolicyRepository<PolicyRule>,
    
    /// Legacy policy config for backward compatibility
    legacy_config: Option<PolicyConfig>,
}

/// A policy rule that can be evaluated
#[derive(Debug, Clone)]
pub enum PolicyRule {
    BudgetLimit {
        monthly_limit: f64,
        warning_threshold: f64,
    },
    ModuleBudget {
        module_name: String,
        monthly_limit: f64,
    },
    ResourceLimit {
        resource_type: String,
        max_count: usize,
    },
    TagRequired {
        resource_type: String,
        required_tags: Vec<String>,
    },
}

impl MetadataPolicyEngine {
    /// Create a new metadata policy engine
    pub fn new() -> Self {
        Self {
            repository: PolicyRepository::new(),
            legacy_config: None,
        }
    }
    
    /// Create from legacy policy config for backward compatibility
    pub fn from_legacy_config(config: PolicyConfig) -> Self {
        let mut engine = Self::new();
        engine.legacy_config = Some(config.clone());
        
        // Convert legacy config to metadata policies
        engine.import_legacy_config(&config);
        
        engine
    }
    
    /// Import legacy config into metadata repository
    fn import_legacy_config(&mut self, config: &PolicyConfig) {
        // Import global budget
        if let Some(global) = &config.budgets.global {
            let metadata = PolicyMetadata::new(
                "global-budget".to_string(),
                "Global Monthly Budget".to_string(),
                "Enforces global monthly spending limit across all resources".to_string(),
                PolicyCategory::Budget,
                Severity::Critical,
                "system".to_string(),
                "system".to_string(),
            );
            
            let rule = PolicyRule::BudgetLimit {
                monthly_limit: global.monthly_limit,
                warning_threshold: global.warning_threshold,
            };
            
            let mut policy = PolicyWithMetadata::new(metadata, rule);
            policy.metadata.activate();
            policy.metadata.add_tag("budget".to_string());
            policy.metadata.add_tag("global".to_string());
            
            let _ = self.repository.add(policy);
        }
        
        // Import module budgets
        for (idx, module) in config.budgets.modules.iter().enumerate() {
            let metadata = PolicyMetadata::new(
                format!("module-budget-{}", idx),
                format!("Module Budget: {}", module.name),
                format!("Enforces monthly spending limit for module {}", module.name),
                PolicyCategory::Budget,
                Severity::Error,
                "system".to_string(),
                "system".to_string(),
            );
            
            let rule = PolicyRule::ModuleBudget {
                module_name: module.name.clone(),
                monthly_limit: module.monthly_limit,
            };
            
            let mut policy = PolicyWithMetadata::new(metadata, rule);
            policy.metadata.activate();
            policy.metadata.add_tag("budget".to_string());
            policy.metadata.add_tag("module".to_string());
            policy.metadata.add_tag(module.name.clone());
            
            let _ = self.repository.add(policy);
        }
        
        // Import NAT gateway limit
        if let Some(nat) = &config.resources.nat_gateways {
            let metadata = PolicyMetadata::new(
                "nat-gateway-limit".to_string(),
                "NAT Gateway Limit".to_string(),
                format!("Limits NAT gateways to {} per region", nat.max_count),
                PolicyCategory::Resource,
                Severity::Warning,
                "system".to_string(),
                "system".to_string(),
            );
            
            let rule = PolicyRule::ResourceLimit {
                resource_type: "aws_nat_gateway".to_string(),
                max_count: nat.max_count,
            };
            
            let mut policy = PolicyWithMetadata::new(metadata, rule);
            policy.metadata.activate();
            policy.metadata.add_tag("resource".to_string());
            policy.metadata.add_tag("nat-gateway".to_string());
            
            let _ = self.repository.add(policy);
        }
        
        // Import S3 lifecycle requirement
        if config.resources.s3_lifecycle_required {
            let metadata = PolicyMetadata::new(
                "s3-lifecycle-required".to_string(),
                "S3 Lifecycle Required".to_string(),
                "All S3 buckets must have lifecycle policies configured".to_string(),
                PolicyCategory::Governance,
                Severity::Warning,
                "system".to_string(),
                "system".to_string(),
            );
            
            let rule = PolicyRule::TagRequired {
                resource_type: "aws_s3_bucket".to_string(),
                required_tags: vec!["lifecycle".to_string()],
            };
            
            let mut policy = PolicyWithMetadata::new(metadata, rule);
            policy.metadata.activate();
            policy.metadata.add_tag("s3".to_string());
            policy.metadata.add_tag("lifecycle".to_string());
            
            let _ = self.repository.add(policy);
        }
    }
    
    /// Get the policy repository
    pub fn repository(&self) -> &PolicyRepository<PolicyRule> {
        &self.repository
    }
    
    /// Get mutable policy repository
    pub fn repository_mut(&mut self) -> &mut PolicyRepository<PolicyRule> {
        &mut self.repository
    }
    
    /// Add a policy to the engine
    pub fn add_policy(&mut self, policy: PolicyWithMetadata<PolicyRule>) -> Result<(), String> {
        self.repository.add(policy)
    }
    
    /// Activate a policy by ID
    pub fn activate_policy(&mut self, id: &str) -> Result<(), String> {
        if let Some(policy) = self.repository.get_mut(id) {
            policy.metadata.activate();
            Ok(())
        } else {
            Err(format!("Policy '{}' not found", id))
        }
    }
    
    /// Disable a policy by ID
    pub fn disable_policy(&mut self, id: &str) -> Result<(), String> {
        if let Some(policy) = self.repository.get_mut(id) {
            policy.metadata.disable();
            Ok(())
        } else {
            Err(format!("Policy '{}' not found", id))
        }
    }
    
    /// Evaluate all enforceable policies
    pub fn evaluate(
        &mut self,
        changes: &[ResourceChange],
        total_cost: &CostEstimate,
    ) -> MetadataPolicyResult {
        let mut result = MetadataPolicyResult::new();
        
        // Get all policies that should be enforced
        let enforceable = self.repository.get_enforceable();
        
        for policy in enforceable {
            let policy_id = policy.metadata.id.clone();
            let violations = self.evaluate_policy(policy, changes, total_cost);
            
            // Record evaluation in metrics
            if let Some(policy_mut) = self.repository.get_mut(&policy_id) {
                policy_mut.metadata.metrics.record_evaluation(!violations.is_empty());
            }
            
            // Add violations to result
            for violation in violations {
                result.add_violation(violation);
            }
        }
        
        result
    }
    
    /// Evaluate with explicit zero-network guarantee
    pub fn evaluate_zero_network(
        &mut self,
        changes: &[ResourceChange],
        total_cost: &CostEstimate,
        token: ZeroNetworkToken,
    ) -> Result<MetadataPolicyResult, ZeroNetworkViolation> {
        token.validate()?;
        Ok(self.evaluate(changes, total_cost))
    }
    
    /// Evaluate a single policy
    fn evaluate_policy(
        &self,
        policy: &PolicyWithMetadata<PolicyRule>,
        changes: &[ResourceChange],
        total_cost: &CostEstimate,
    ) -> Vec<MetadataPolicyViolation> {
        let mut violations = Vec::new();
        
        match &policy.spec {
            PolicyRule::BudgetLimit { monthly_limit, warning_threshold } => {
                let monthly_cost = total_cost.monthly_cost;
                
                if monthly_cost > *monthly_limit {
                    violations.push(MetadataPolicyViolation {
                        policy_id: policy.metadata.id.clone(),
                        policy_name: policy.metadata.name.clone(),
                        severity: policy.metadata.severity.clone(),
                        category: policy.metadata.category.clone(),
                        resource_id: "global".to_string(),
                        message: format!(
                            "Monthly cost ${:.2} exceeds limit ${:.2}",
                            monthly_cost, monthly_limit
                        ),
                        actual_value: format!("${:.2}", monthly_cost),
                        expected_value: format!("<= ${:.2}", monthly_limit),
                        blocking: policy.is_blocking(),
                    });
                } else if monthly_cost > monthly_limit * warning_threshold {
                    violations.push(MetadataPolicyViolation {
                        policy_id: policy.metadata.id.clone(),
                        policy_name: policy.metadata.name.clone(),
                        severity: Severity::Warning,
                        category: policy.metadata.category.clone(),
                        resource_id: "global".to_string(),
                        message: format!(
                            "Monthly cost ${:.2} is at {:.0}% of limit",
                            monthly_cost,
                            (monthly_cost / monthly_limit) * 100.0
                        ),
                        actual_value: format!("${:.2}", monthly_cost),
                        expected_value: format!("<= ${:.2}", monthly_limit * warning_threshold),
                        blocking: false,
                    });
                }
            }
            
            PolicyRule::ModuleBudget { module_name, monthly_limit } => {
                // Calculate module cost from changes
                let module_cost: f64 = changes
                    .iter()
                    .filter(|c| c.resource_id.starts_with(&format!("module.{}", module_name)))
                    .map(|c| c.cost_impact.monthly_change)
                    .sum();
                
                if module_cost > *monthly_limit {
                    violations.push(MetadataPolicyViolation {
                        policy_id: policy.metadata.id.clone(),
                        policy_name: policy.metadata.name.clone(),
                        severity: policy.metadata.severity.clone(),
                        category: policy.metadata.category.clone(),
                        resource_id: module_name.clone(),
                        message: format!(
                            "Module {} cost ${:.2} exceeds limit ${:.2}",
                            module_name, module_cost, monthly_limit
                        ),
                        actual_value: format!("${:.2}", module_cost),
                        expected_value: format!("<= ${:.2}", monthly_limit),
                        blocking: policy.is_blocking(),
                    });
                }
            }
            
            PolicyRule::ResourceLimit { resource_type, max_count } => {
                let count = changes
                    .iter()
                    .filter(|c| c.resource_type == *resource_type)
                    .count();
                
                if count > *max_count {
                    violations.push(MetadataPolicyViolation {
                        policy_id: policy.metadata.id.clone(),
                        policy_name: policy.metadata.name.clone(),
                        severity: policy.metadata.severity.clone(),
                        category: policy.metadata.category.clone(),
                        resource_id: resource_type.clone(),
                        message: format!(
                            "{} count {} exceeds limit {}",
                            resource_type, count, max_count
                        ),
                        actual_value: count.to_string(),
                        expected_value: format!("<= {}", max_count),
                        blocking: policy.is_blocking(),
                    });
                }
            }
            
            PolicyRule::TagRequired { resource_type, required_tags } => {
                // Check for missing tags in resources
                for change in changes {
                    if change.resource_type == *resource_type {
                        // In a real implementation, check actual tags
                        // For now, this is a placeholder
                    }
                }
            }
        }
        
        violations
    }
    
    /// Get policy statistics
    pub fn statistics(&self) -> RepositoryStatistics {
        self.repository.statistics()
    }
    
    /// Get policies with high violation rates
    pub fn high_violation_policies(&self, threshold: f64) -> Vec<&PolicyWithMetadata<PolicyRule>> {
        self.repository.get_high_violation_policies(threshold)
    }
}

impl Default for MetadataPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy evaluation result with metadata
#[derive(Debug, Clone)]
pub struct MetadataPolicyResult {
    pub violations: Vec<MetadataPolicyViolation>,
    pub warnings: Vec<String>,
}

impl MetadataPolicyResult {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_violation(&mut self, violation: MetadataPolicyViolation) {
        self.violations.push(violation);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
    
    pub fn has_blocking_violations(&self) -> bool {
        self.violations.iter().any(|v| v.blocking)
    }
    
    pub fn blocking_violations(&self) -> Vec<&MetadataPolicyViolation> {
        self.violations.iter().filter(|v| v.blocking).collect()
    }
    
    pub fn by_severity(&self, severity: &Severity) -> Vec<&MetadataPolicyViolation> {
        self.violations.iter().filter(|v| &v.severity == severity).collect()
    }
    
    pub fn by_category(&self, category: &PolicyCategory) -> Vec<&MetadataPolicyViolation> {
        self.violations.iter().filter(|v| &v.category == category).collect()
    }
}

impl Default for MetadataPolicyResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy violation with metadata
#[derive(Debug, Clone)]
pub struct MetadataPolicyViolation {
    pub policy_id: String,
    pub policy_name: String,
    pub severity: Severity,
    pub category: PolicyCategory,
    pub resource_id: String,
    pub message: String,
    pub actual_value: String,
    pub expected_value: String,
    pub blocking: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_engine_new() {
        let engine = MetadataPolicyEngine::new();
        assert_eq!(engine.repository().count(), 0);
    }

    #[test]
    fn test_from_legacy_config() {
        let config = PolicyConfig {
            version: "1.0".to_string(),
            budgets: BudgetPolicies {
                global: Some(BudgetLimit {
                    monthly_limit: 1000.0,
                    warning_threshold: 0.8,
                }),
                modules: vec![],
            },
            resources: Default::default(),
            slos: vec![],
            enforcement: Default::default(),
        };
        
        let engine = MetadataPolicyEngine::from_legacy_config(config);
        assert_eq!(engine.repository().count(), 1);
        
        let policy = engine.repository().get("global-budget");
        assert!(policy.is_some());
        assert_eq!(policy.unwrap().metadata.status, PolicyStatus::Active);
    }

    #[test]
    fn test_add_and_activate_policy() {
        let mut engine = MetadataPolicyEngine::new();
        
        let metadata = PolicyMetadata::new(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            "A test".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );
        
        let rule = PolicyRule::BudgetLimit {
            monthly_limit: 5000.0,
            warning_threshold: 0.8,
        };
        
        let policy = PolicyWithMetadata::new(metadata, rule);
        engine.add_policy(policy).unwrap();
        
        assert_eq!(engine.repository().count(), 1);
        
        // Policy starts as draft, not enforced
        assert_eq!(engine.repository().get_enforceable().len(), 0);
        
        // Activate it
        engine.activate_policy("test-policy").unwrap();
        assert_eq!(engine.repository().get_enforceable().len(), 1);
    }

    #[test]
    fn test_evaluate_budget_policy() {
        let mut engine = MetadataPolicyEngine::new();
        
        let mut metadata = PolicyMetadata::new(
            "budget-test".to_string(),
            "Budget Test".to_string(),
            "Test budget".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );
        metadata.activate();
        
        let rule = PolicyRule::BudgetLimit {
            monthly_limit: 1000.0,
            warning_threshold: 0.8,
        };
        
        let policy = PolicyWithMetadata::new(metadata, rule);
        engine.add_policy(policy).unwrap();
        
        // Test with cost under limit
        let cost = CostEstimate {
            monthly: 500.0,
            yearly: 6000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };
        
        let result = engine.evaluate(&[], &cost);
        assert!(!result.has_violations());
        
        // Test with cost over limit
        let cost = CostEstimate {
            monthly: 1500.0,
            yearly: 18000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };
        
        let result = engine.evaluate(&[], &cost);
        assert!(result.has_violations());
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].blocking);
    }

    #[test]
    fn test_policy_metrics() {
        let mut engine = MetadataPolicyEngine::new();
        
        let mut metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );
        metadata.activate();
        
        let rule = PolicyRule::BudgetLimit {
            monthly_limit: 1000.0,
            warning_threshold: 0.8,
        };
        
        let policy = PolicyWithMetadata::new(metadata, rule);
        engine.add_policy(policy).unwrap();
        
        // Evaluate multiple times
        let cost_ok = CostEstimate {
            monthly: 500.0,
            yearly: 6000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };
        
        let cost_bad = CostEstimate {
            monthly: 1500.0,
            yearly: 18000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };
        
        engine.evaluate(&[], &cost_ok);
        engine.evaluate(&[], &cost_bad);
        engine.evaluate(&[], &cost_ok);
        
        let policy = engine.repository().get("test").unwrap();
        assert_eq!(policy.metadata.metrics.evaluation_count, 3);
        assert_eq!(policy.metadata.metrics.violation_count, 1);
    }
}
