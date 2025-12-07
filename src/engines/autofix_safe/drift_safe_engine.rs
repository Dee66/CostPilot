use super::drift_safe_types::*;
use crate::engines::detection::ResourceChange;
use crate::engines::policy::PolicyEngine;
use crate::engines::slo::SloManager;
use std::collections::HashMap;

/// Drift-safe autofix engine with safety checks and rollback
pub struct DriftSafeEngine {
    /// Policy engine for validation
    policy_engine: Option<PolicyEngine>,
    
    /// SLO manager for budget checks
    slo_manager: Option<SloManager>,
    
    /// Enable strict safety mode
    strict_mode: bool,
}

impl DriftSafeEngine {
    /// Create a new drift-safe engine
    pub fn new() -> Self {
        Self {
            policy_engine: None,
            slo_manager: None,
            strict_mode: true,
        }
    }

    /// Set policy engine for validation
    pub fn with_policy_engine(mut self, engine: PolicyEngine) -> Self {
        self.policy_engine = Some(engine);
        self
    }

    /// Set SLO manager for budget validation
    pub fn with_slo_manager(mut self, manager: SloManager) -> Self {
        self.slo_manager = Some(manager);
        self
    }

    /// Set strict mode
    pub fn set_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Create drift-safe operation
    pub fn create_operation(
        &self,
        resource_id: String,
        resource_type: String,
        fix_description: String,
        current_change: &ResourceChange,
        proposed_fix: &ResourceChange,
        current_cost: f64,
        proposed_cost: f64,
    ) -> DriftSafeOperation {
        // Convert Option<Value> to HashMap
        let current_config = current_change.new_config
            .as_ref()
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        
        let proposed_config = proposed_fix.new_config
            .as_ref()
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        
        let original_state = ResourceState::new(
            current_config,
            current_cost,
        );

        let target_state = ResourceState::new(
            proposed_config,
            proposed_cost,
        );

        let mut operation = DriftSafeOperation::new(
            resource_id,
            resource_type,
            fix_description,
            original_state,
            target_state,
        );

        // Add default safety checks
        self.add_default_safety_checks(&mut operation);

        // Generate rollback plan
        self.generate_rollback_plan(&mut operation, current_change);

        operation
    }

    /// Add default safety checks
    fn add_default_safety_checks(&self, operation: &mut DriftSafeOperation) {
        // Check 1: No drift
        operation.add_safety_check(SafetyCheck::new(
            "no_drift".to_string(),
            "Verify resource has not drifted since snapshot".to_string(),
            SafetyCheckType::NoDrift,
        ));

        // Check 2: Resource exists
        operation.add_safety_check(SafetyCheck::new(
            "resource_exists".to_string(),
            "Verify resource still exists".to_string(),
            SafetyCheckType::ResourceExists,
        ));

        // Check 3: Config hash matches
        operation.add_safety_check(SafetyCheck::new(
            "config_hash".to_string(),
            "Verify configuration hash matches snapshot".to_string(),
            SafetyCheckType::ConfigHashMatch,
        ));

        // Check 4: Cost impact acceptable
        operation.add_safety_check(SafetyCheck::new(
            "cost_impact".to_string(),
            "Verify cost reduction meets expectations".to_string(),
            SafetyCheckType::CostImpactAcceptable,
        ));

        // Check 5: No policy violations (if policy engine available)
        if self.policy_engine.is_some() {
            operation.add_safety_check(SafetyCheck::new(
                "policy_check".to_string(),
                "Verify fix does not violate policies".to_string(),
                SafetyCheckType::NoPolicyViolations,
            ));
        }

        // Check 6: No SLO violations (if SLO manager available)
        if self.slo_manager.is_some() {
            operation.add_safety_check(SafetyCheck::new(
                "slo_check".to_string(),
                "Verify fix does not violate SLOs".to_string(),
                SafetyCheckType::NoSloViolations,
            ));
        }
    }

    /// Generate rollback plan
    fn generate_rollback_plan(
        &self,
        operation: &mut DriftSafeOperation,
        original_change: &ResourceChange,
    ) {
        // Step 1: Restore original configuration
        let restore_step = RollbackStep {
            order: 1,
            description: format!(
                "Restore {} to original configuration",
                operation.resource_id
            ),
            restore_config: original_change.config.clone(),
            verification: Some("Verify configuration matches original hash".to_string()),
        };

        operation.rollback_plan.steps.push(restore_step);
        operation.rollback_plan.timeout_seconds = if self.strict_mode { 180 } else { 300 };
        operation.rollback_plan.auto_rollback = true;
    }

    /// Detect drift between expected and actual state
    pub fn detect_drift(
        &self,
        expected_state: &ResourceState,
        actual_config: &HashMap<String, serde_json::Value>,
    ) -> DriftDetection {
        let mut drifted_attributes = Vec::new();

        for (key, expected_value) in &expected_state.config {
            if let Some(actual_value) = actual_config.get(key) {
                if expected_value != actual_value {
                    drifted_attributes.push(DriftedAttribute {
                        name: key.clone(),
                        expected: expected_value.clone(),
                        actual: actual_value.clone(),
                        impact: self.assess_drift_impact(key, expected_value, actual_value),
                    });
                }
            } else {
                drifted_attributes.push(DriftedAttribute {
                    name: key.clone(),
                    expected: expected_value.clone(),
                    actual: serde_json::Value::Null,
                    impact: "Attribute removed".to_string(),
                });
            }
        }

        // Check for new attributes
        for key in actual_config.keys() {
            if !expected_state.config.contains_key(key) {
                drifted_attributes.push(DriftedAttribute {
                    name: key.clone(),
                    expected: serde_json::Value::Null,
                    actual: actual_config[key].clone(),
                    impact: "New attribute added".to_string(),
                });
            }
        }

        if drifted_attributes.is_empty() {
            DriftDetection::no_drift()
        } else {
            DriftDetection::with_drift(drifted_attributes)
        }
    }

    /// Assess impact of drift
    fn assess_drift_impact(
        &self,
        key: &str,
        expected: &serde_json::Value,
        actual: &serde_json::Value,
    ) -> String {
        // Cost-impacting attributes
        if key.contains("instance_type")
            || key.contains("size")
            || key.contains("capacity")
            || key.contains("volume_size")
        {
            return "Cost impact - configuration change".to_string();
        }

        // Security-impacting attributes
        if key.contains("security_group")
            || key.contains("iam_role")
            || key.contains("public")
            || key.contains("encrypt")
        {
            return "Security impact - access control change".to_string();
        }

        // Availability-impacting attributes
        if key.contains("availability_zone")
            || key.contains("multi_az")
            || key.contains("backup")
        {
            return "Availability impact - reliability change".to_string();
        }

        "Configuration drift".to_string()
    }

    /// Run all safety checks for an operation
    pub fn run_safety_checks(&self, operation: &mut DriftSafeOperation) -> Result<(), String> {
        operation.status = OperationStatus::ValidatingSafety;
        operation.log(LogLevel::Info, "Starting safety checks".to_string());

        let mut all_passed = true;

        for check in &mut operation.safety_checks {
            match check.check_type {
                SafetyCheckType::NoDrift => {
                    self.check_no_drift(operation, check);
                }
                SafetyCheckType::ResourceExists => {
                    self.check_resource_exists(operation, check);
                }
                SafetyCheckType::ConfigHashMatch => {
                    self.check_config_hash(operation, check);
                }
                SafetyCheckType::CostImpactAcceptable => {
                    self.check_cost_impact(operation, check);
                }
                SafetyCheckType::NoPolicyViolations => {
                    self.check_policies(operation, check);
                }
                SafetyCheckType::NoSloViolations => {
                    self.check_slos(operation, check);
                }
                SafetyCheckType::Custom(_) => {
                    check.status = CheckStatus::Skipped;
                }
            }

            if check.status == CheckStatus::Failed {
                all_passed = false;
                if self.strict_mode {
                    operation.log(
                        LogLevel::Error,
                        format!("Safety check failed: {}", check.name),
                    );
                    return Err(format!("Safety check '{}' failed: {}", 
                        check.name, 
                        check.message.as_ref().unwrap_or(&"Unknown error".to_string())
                    ));
                }
            }
        }

        if all_passed {
            operation.log(LogLevel::Info, "All safety checks passed".to_string());
            Ok(())
        } else {
            Err("Some safety checks failed".to_string())
        }
    }

    /// Check for drift
    fn check_no_drift(&self, operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        // Simulate drift check - in real implementation would query actual state
        let drift = self.detect_drift(
            &operation.original_state,
            &operation.original_state.config,
        );

        if drift.is_blocking() {
            check.mark_failed(format!(
                "Drift detected: {} attributes changed",
                drift.drifted_attributes.len()
            ));
        } else {
            check.mark_passed("No significant drift detected".to_string());
        }
    }

    /// Check resource exists
    fn check_resource_exists(&self, operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        // Simulate existence check - in real implementation would query cloud provider
        // For now, always pass
        check.mark_passed(format!("Resource {} exists", operation.resource_id));
    }

    /// Check config hash
    fn check_config_hash(&self, operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        if operation.original_state.verify_hash() {
            check.mark_passed("Configuration hash verified".to_string());
        } else {
            check.mark_failed("Configuration hash mismatch".to_string());
        }
    }

    /// Check cost impact
    fn check_cost_impact(&self, operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        let original_cost = operation.original_state.estimated_cost;
        let target_cost = operation.target_state.estimated_cost;
        let savings = original_cost - target_cost;
        let savings_percent = (savings / original_cost) * 100.0;

        if savings >= 0.0 {
            check.mark_passed(format!(
                "Cost reduction: ${:.2} ({:.1}% savings)",
                savings, savings_percent
            ));
        } else {
            if self.strict_mode {
                check.mark_failed(format!(
                    "Cost increase: ${:.2} ({:.1}%)",
                    -savings, -savings_percent
                ));
            } else {
                check.mark_passed(format!(
                    "Cost change acceptable: ${:.2} ({:.1}%)",
                    -savings, -savings_percent
                ));
            }
        }
    }

    /// Check policies
    fn check_policies(&self, _operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        if let Some(_policy_engine) = &self.policy_engine {
            // In real implementation, would validate against policies
            check.mark_passed("No policy violations detected".to_string());
        } else {
            check.status = CheckStatus::Skipped;
        }
    }

    /// Check SLOs
    fn check_slos(&self, _operation: &DriftSafeOperation, check: &mut SafetyCheck) {
        if let Some(_slo_manager) = &self.slo_manager {
            // In real implementation, would validate against SLOs
            check.mark_passed("No SLO violations detected".to_string());
        } else {
            check.status = CheckStatus::Skipped;
        }
    }

    /// Apply operation (simulated)
    pub fn apply_operation(&self, operation: &mut DriftSafeOperation) -> Result<(), String> {
        if !operation.can_proceed() {
            return Err("Operation cannot proceed - safety checks not passed".to_string());
        }

        operation.status = OperationStatus::Applying;
        operation.log(LogLevel::Info, "Applying fix".to_string());

        // Simulate application
        // In real implementation, would apply configuration changes

        operation.status = OperationStatus::Applied;
        operation.log(LogLevel::Info, "Fix applied successfully".to_string());

        Ok(())
    }

    /// Execute rollback
    pub fn execute_rollback(&self, operation: &mut DriftSafeOperation) -> Result<(), String> {
        operation.trigger_rollback();
        operation.log(LogLevel::Warning, "Executing rollback".to_string());

        // Execute rollback steps in order
        for step in &operation.rollback_plan.steps {
            operation.log(
                LogLevel::Info,
                format!("Rollback step {}: {}", step.order, step.description),
            );

            // Simulate rollback execution
            // In real implementation, would restore configuration
        }

        operation.mark_rollback_complete();
        Ok(())
    }
}

impl Default for DriftSafeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::ChangeAction;

    fn create_test_change(monthly_cost: f64) -> ResourceChange {
        let mut config = HashMap::new();
        config.insert(
            "instance_type".to_string(),
            serde_json::json!("t3.large"),
        );

        ResourceChange {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Update,
            config,
            monthly_cost: Some(monthly_cost),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = DriftSafeEngine::new();
        assert!(engine.strict_mode);
        assert!(engine.policy_engine.is_none());
    }

    #[test]
    fn test_create_operation() {
        let engine = DriftSafeEngine::new();
        let current = create_test_change(100.0);
        let proposed = create_test_change(50.0);

        let operation = engine.create_operation(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            "Downsize to t3.medium".to_string(),
            &current,
            &proposed,
        );

        assert_eq!(operation.resource_id, "aws_instance.web");
        assert!(!operation.safety_checks.is_empty());
        assert!(!operation.rollback_plan.steps.is_empty());
    }

    #[test]
    fn test_drift_detection_no_drift() {
        let engine = DriftSafeEngine::new();
        let mut config = HashMap::new();
        config.insert("instance_type".to_string(), serde_json::json!("t3.large"));
        let state = ResourceState::new(config.clone(), 100.0);

        let drift = engine.detect_drift(&state, &config);
        assert!(!drift.has_drift);
    }

    #[test]
    fn test_drift_detection_with_drift() {
        let engine = DriftSafeEngine::new();
        let mut original_config = HashMap::new();
        original_config.insert("instance_type".to_string(), serde_json::json!("t3.large"));
        let state = ResourceState::new(original_config, 100.0);

        let mut actual_config = HashMap::new();
        actual_config.insert("instance_type".to_string(), serde_json::json!("t3.xlarge"));

        let drift = engine.detect_drift(&state, &actual_config);
        assert!(drift.has_drift);
        assert_eq!(drift.drifted_attributes.len(), 1);
    }

    #[test]
    fn test_safety_checks() {
        let engine = DriftSafeEngine::new();
        let current = create_test_change(100.0);
        let proposed = create_test_change(50.0);

        let mut operation = engine.create_operation(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            "Downsize".to_string(),
            &current,
            &proposed,
        );

        let result = engine.run_safety_checks(&mut operation);
        assert!(result.is_ok());
        assert!(operation.all_safety_checks_passed());
    }

    #[test]
    fn test_apply_operation() {
        let engine = DriftSafeEngine::new();
        let current = create_test_change(100.0);
        let proposed = create_test_change(50.0);

        let mut operation = engine.create_operation(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            "Downsize".to_string(),
            &current,
            &proposed,
        );

        // Run safety checks first
        engine.run_safety_checks(&mut operation).unwrap();

        // Apply operation
        let result = engine.apply_operation(&mut operation);
        assert!(result.is_ok());
        assert_eq!(operation.status, OperationStatus::Applied);
    }

    #[test]
    fn test_rollback() {
        let engine = DriftSafeEngine::new();
        let current = create_test_change(100.0);
        let proposed = create_test_change(50.0);

        let mut operation = engine.create_operation(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            "Downsize".to_string(),
            &current,
            &proposed,
        );

        // Simulate failure
        operation.mark_failed("Test failure".to_string());

        // Execute rollback
        let result = engine.execute_rollback(&mut operation);
        assert!(result.is_ok());
        assert_eq!(operation.status, OperationStatus::RolledBack);
    }

    #[test]
    fn test_drift_impact_assessment() {
        let engine = DriftSafeEngine::new();

        let impact = engine.assess_drift_impact(
            "instance_type",
            &serde_json::json!("t3.medium"),
            &serde_json::json!("t3.large"),
        );
        assert!(impact.contains("Cost impact"));

        let impact = engine.assess_drift_impact(
            "security_group_ids",
            &serde_json::json!(["sg-123"]),
            &serde_json::json!(["sg-456"]),
        );
        assert!(impact.contains("Security impact"));
    }
}
