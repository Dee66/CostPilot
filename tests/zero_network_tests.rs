/// Tests for zero-network enforcement
/// 
/// These tests verify that policy evaluation never makes network calls
/// and can run safely in WASM/sandboxed environments.

#[cfg(test)]
mod zero_network_tests {
    use crate::engines::detection::{ChangeType, ResourceChange};
    use crate::engines::policy::*;
    use crate::engines::prediction::CostEstimate;
    use std::collections::HashMap;

    #[test]
    fn test_policy_engine_zero_network() {
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
        let token = ZeroNetworkToken::new();

        let cost = CostEstimate {
            monthly: 500.0,
            yearly: 6000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        // Test zero-network evaluation
        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed());
    }

    #[test]
    fn test_metadata_engine_zero_network() {
        let mut engine = MetadataPolicyEngine::new();

        // Add a budget policy
        let policy = PolicyWithMetadata {
            metadata: PolicyMetadata {
                id: "test_budget".to_string(),
                name: "Test Budget Policy".to_string(),
                description: Some("Test policy for zero-network".to_string()),
                category: PolicyCategory::Budget,
                severity: PolicySeverity::Error,
                status: PolicyStatus::Active,
                ..Default::default()
            },
            spec: PolicyRule::BudgetLimit {
                monthly_limit: 1000.0,
                warning_threshold: 0.8,
            },
        };

        engine.add_policy(policy);

        let token = ZeroNetworkToken::new();
        let cost = CostEstimate {
            monthly: 500.0,
            yearly: 6000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        // Test zero-network evaluation
        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(!policy_result.has_violations());
    }

    #[test]
    fn test_zero_network_with_violations() {
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
        let token = ZeroNetworkToken::new();

        // Cost that exceeds limit
        let cost = CostEstimate {
            monthly: 1500.0,
            yearly: 18000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(!policy_result.passed());
        assert_eq!(policy_result.violations().len(), 1);
    }

    #[test]
    fn test_zero_network_with_resource_changes() {
        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies::default(),
            resources: ResourcePolicies {
                nat_gateways: Some(NatGatewayPolicy { max_count: 2 }),
                ..Default::default()
            },
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let engine = PolicyEngine::new(config);
        let token = ZeroNetworkToken::new();

        // Create resource changes
        let changes = vec![
            ResourceChange {
                resource_type: "aws_nat_gateway".to_string(),
                resource_id: "nat_1".to_string(),
                change_type: ChangeType::Create,
                cost_impact: crate::engines::prediction::CostImpact {
                    monthly_change: 45.0,
                    confidence: 0.9,
                },
                old_config: serde_json::Value::Null,
                new_config: serde_json::json!({"subnet_id": "subnet-123"}),
            },
            ResourceChange {
                resource_type: "aws_nat_gateway".to_string(),
                resource_id: "nat_2".to_string(),
                change_type: ChangeType::Create,
                cost_impact: crate::engines::prediction::CostImpact {
                    monthly_change: 45.0,
                    confidence: 0.9,
                },
                old_config: serde_json::Value::Null,
                new_config: serde_json::json!({"subnet_id": "subnet-456"}),
            },
        ];

        let cost = CostEstimate {
            monthly: 90.0,
            yearly: 1080.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        // Test zero-network evaluation with resources
        let result = engine.evaluate_zero_network(&changes, &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed()); // 2 NAT gateways is within limit
    }

    #[test]
    fn test_zero_network_runtime() {
        let runtime = ZeroNetworkRuntime::new();
        assert!(runtime.verify_environment().is_ok());

        // Execute policy evaluation in zero-network runtime
        let result = runtime.execute(|token| {
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
                monthly: 500.0,
                yearly: 6000.0,
                one_time: 0.0,
                breakdown: HashMap::new(),
            };

            engine.evaluate_zero_network(&[], &cost, token)
        });

        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed());
    }

    #[test]
    fn test_zero_network_determinism() {
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
        let token = ZeroNetworkToken::new();

        let cost = CostEstimate {
            monthly: 1200.0,
            yearly: 14400.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        // Run evaluation multiple times - should be deterministic
        let result1 = engine.evaluate_zero_network(&[], &cost, token);
        let result2 = engine.evaluate_zero_network(&[], &cost, token);
        let result3 = engine.evaluate_zero_network(&[], &cost, token);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let r1 = result1.unwrap();
        let r2 = result2.unwrap();
        let r3 = result3.unwrap();

        // All results should be identical
        assert_eq!(r1.passed(), r2.passed());
        assert_eq!(r2.passed(), r3.passed());
        assert_eq!(r1.violations().len(), r2.violations().len());
        assert_eq!(r2.violations().len(), r3.violations().len());
    }

    #[test]
    fn test_zero_network_enforced_wrapper() {
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

        let enforced_engine = ZeroNetworkEnforced::new(PolicyEngine::new(config));

        let cost = CostEstimate {
            monthly: 500.0,
            yearly: 6000.0,
            one_time: 0.0,
            breakdown: HashMap::new(),
        };

        // Use the enforced wrapper
        let result = enforced_engine.with_zero_network(|engine, token| {
            engine.evaluate_zero_network(&[], &cost, token)
        });

        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed());
    }

    #[test]
    fn test_dependency_validation() {
        // These should be allowed
        assert!(ZeroNetworkValidator::is_allowed_dependency("serde"));
        assert!(ZeroNetworkValidator::is_allowed_dependency("serde_json"));
        assert!(ZeroNetworkValidator::is_allowed_dependency("serde_yaml"));
        assert!(ZeroNetworkValidator::is_allowed_dependency("chrono"));
        assert!(ZeroNetworkValidator::is_allowed_dependency("thiserror"));

        // These should be disallowed (network-capable)
        assert!(!ZeroNetworkValidator::is_allowed_dependency("reqwest"));
        assert!(!ZeroNetworkValidator::is_allowed_dependency("ureq"));
        assert!(!ZeroNetworkValidator::is_allowed_dependency("hyper"));
        assert!(!ZeroNetworkValidator::is_allowed_dependency("tokio::net"));
        assert!(!ZeroNetworkValidator::is_allowed_dependency("rusoto_core"));
        assert!(!ZeroNetworkValidator::is_allowed_dependency("aws-sdk-s3"));
    }

    #[test]
    fn test_deterministic_operations() {
        // Pure operations should be allowed
        assert!(ZeroNetworkValidator::ensure_deterministic("calculate_cost").is_ok());
        assert!(ZeroNetworkValidator::ensure_deterministic("evaluate_policy").is_ok());
        assert!(ZeroNetworkValidator::ensure_deterministic("parse_json").is_ok());

        // Non-deterministic operations should fail
        assert!(ZeroNetworkValidator::ensure_deterministic("SystemTime::now()").is_err());
        assert!(ZeroNetworkValidator::ensure_deterministic("rand::thread_rng()").is_err());
        assert!(ZeroNetworkValidator::ensure_deterministic("thread::sleep(1)").is_err());
    }
}
