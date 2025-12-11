/// Tests for zero-network enforcement
///
/// These tests verify that policy evaluation never makes network calls
/// and can run safely in WASM/sandboxed environments.

#[cfg(test)]
mod zero_network_tests {
    use costpilot::engines::shared::models::{ResourceChange, ChangeAction, CostEstimate, CostImpact, Severity};
    use costpilot::engines::policy::{*, Severity as PolicySeverity};
    use costpilot::edition::EditionContext;
    use chrono;
    use serde_json::json;
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

        let engine = PolicyEngine::new(config, &EditionContext::free());
        let token = ZeroNetworkToken::new();

        let cost = CostEstimate::builder()
            .prediction_interval_low(450.0)
            .prediction_interval_high(550.0)
            .heuristic_reference("monthly".to_string())
            .confidence_score(0.9)
            .build();

        // Test zero-network evaluation
        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed);
    }

    #[test]
    fn test_metadata_engine_zero_network() {
        let mut engine = MetadataPolicyEngine::new();

        // Add a budget policy
        let policy = PolicyWithMetadata {
            metadata: PolicyMetadata {
                id: "test_budget".to_string(),
                name: "Test Budget Policy".to_string(),
                description: "Test policy for zero-network".to_string(),
                category: PolicyCategory::Budget,
                severity: PolicySeverity::Error,
                status: PolicyStatus::Active,
                version: "1.0.0".to_string(),
                ownership: PolicyOwnership {
                    author: "test".to_string(),
                    owner: "test".to_string(),
                    team: None,
                    contact: None,
                    reviewers: vec![],
                },
                lifecycle: MetadataLifecycle {
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    effective_from: None,
                    effective_until: None,
                    deprecation: None,
                    revisions: vec![],
                },
                tags: Default::default(),
                links: Default::default(),
                metrics: Default::default(),
                custom: Default::default(),
            },
            spec: MetadataPolicyRule::BudgetLimit {
                monthly_limit: 1000.0,
                warning_threshold: 0.8,
            },
        };

        engine.add_policy(policy);

        let token = ZeroNetworkToken::new();
        let cost = CostEstimate::builder()
            .prediction_interval_low(450.0)
            .prediction_interval_high(550.0)
            .heuristic_reference("monthly".to_string())
            .confidence_score(0.9)
            .build();

        // Test zero-network evaluation
        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.violations.is_empty());
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

        let engine = PolicyEngine::new(config, &EditionContext::free());
        let token = ZeroNetworkToken::new();

        // Cost that exceeds limit
        let cost = CostEstimate::builder()
            .prediction_interval_low(1400.0)
            .prediction_interval_high(1600.0)
            .monthly_cost(1500.0)
            .confidence_score(0.9)
            .heuristic_reference("monthly".to_string())
            .build();

        let result = engine.evaluate_zero_network(&[], &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(!policy_result.passed);
        assert_eq!(policy_result.violations.len(), 1);
    }

    #[test]
    fn test_zero_network_with_resource_changes() {
        let config = PolicyConfig {
            version: "1.0.0".to_string(),
            budgets: BudgetPolicies::default(),
            resources: ResourcePolicies {
                nat_gateways: Some(NatGatewayPolicy { 
                    max_count: 2,
                    require_justification: false,
                }),
                ..Default::default()
            },
            slos: vec![],
            enforcement: EnforcementConfig::default(),
        };

        let engine = PolicyEngine::new(config, &EditionContext::free());
        let token = ZeroNetworkToken::new();

        // Create resource changes
        let changes = vec![
            ResourceChange::builder()
                .resource_type("aws_nat_gateway")
                .resource_id("nat_1")
                .action(ChangeAction::Create)
                .cost_impact(CostImpact {
                    delta: 45.0,
                    confidence: 0.9,
                    heuristic_source: None,
                })
                .new_config(json!({"subnet_id": "subnet-123"}))
                .build(),
            ResourceChange::builder()
                .resource_type("aws_nat_gateway")
                .resource_id("nat_2")
                .action(ChangeAction::Create)
                .cost_impact(CostImpact {
                    delta: 45.0,
                    confidence: 0.9,
                    heuristic_source: None,
                })
                .new_config(json!({"subnet_id": "subnet-456"}))
                .build(),
        ];

        let cost = CostEstimate::builder()
            .prediction_interval_low(85.0)
            .prediction_interval_high(95.0)
            .monthly_cost(90.0)
            .confidence_score(0.9)
            .heuristic_reference("monthly".to_string())
            .build();

        // Test zero-network evaluation with resources
        let result = engine.evaluate_zero_network(&changes, &cost, token);
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed); // 2 NAT gateways is within limit
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

            let engine = PolicyEngine::new(config, &EditionContext::free());
            let cost = CostEstimate::builder()
                .prediction_interval_low(450.0)
                .prediction_interval_high(550.0)
                .monthly_cost(500.0)
                .confidence_score(0.9)
                .heuristic_reference("monthly".to_string())
                .build();

            engine.evaluate_zero_network(&[], &cost, token)
        });

        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed);
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

        let engine = PolicyEngine::new(config, &EditionContext::free());
        let token = ZeroNetworkToken::new();

        let cost = CostEstimate::builder()
            .prediction_interval_low(1150.0)
            .prediction_interval_high(1250.0)
            .monthly_cost(1200.0)
            .confidence_score(0.9)
            .heuristic_reference("monthly".to_string())
            .build();

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
        assert_eq!(r1.passed, r2.passed);
        assert_eq!(r2.passed, r3.passed);
        assert_eq!(r1.violations.len(), r2.violations.len());
        assert_eq!(r2.violations.len(), r3.violations.len());
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

        let enforced_engine = ZeroNetworkEnforced::new(PolicyEngine::new(config, &EditionContext::free()));

        let cost = CostEstimate::builder()
            .prediction_interval_low(450.0)
            .prediction_interval_high(550.0)
            .monthly_cost(500.0)
            .confidence_score(0.9)
            .heuristic_reference("monthly".to_string())
            .build();

        let result = enforced_engine.inner().evaluate_zero_network(&[], &cost, enforced_engine.token());
        assert!(result.is_ok());
        let policy_result = result.unwrap();
        assert!(policy_result.passed);
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
