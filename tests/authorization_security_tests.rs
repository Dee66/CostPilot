use costpilot::edition::{EditionContext, EditionMode, Capabilities, require_premium, UpgradeRequired};
use costpilot::edition::EditionCapabilities;

/// Comprehensive authorization security tests
/// Covers privilege escalation, unauthorized access, role conflicts, permission inheritance, access control lists
#[cfg(test)]
mod authorization_security_tests {
    use super::*;

    #[test]
    fn test_privilege_escalation_prevention_free_to_premium() {
        // Test that free edition cannot escalate privileges to access premium features
        let free_edition = EditionContext::free();

        // Verify free edition is correctly identified
        assert!(!free_edition.is_premium());
        assert!(free_edition.is_free());

        // Test that premium features are blocked
        let premium_features = vec![
            ("Autofix", "allow_autofix"),
            ("Trend Analysis", "allow_trend"),
            ("Deep Mapping", "allow_mapping_deep"),
            ("SLO Enforcement", "allow_slo_enforce"),
            ("Policy Enforcement", "allow_policy_enforce"),
            ("Advanced Explain", "allow_explain_full"),
        ];

        for (feature_name, capability_field) in premium_features {
            // Test require_premium function
            let result = require_premium(&free_edition, feature_name);
            assert!(result.is_err(), "Free edition should not allow {}", feature_name);

            if let Err(e) = result {
                let upgrade_error = e.downcast_ref::<UpgradeRequired>();
                assert!(upgrade_error.is_some(), "Should get UpgradeRequired error for {}", feature_name);
                assert_eq!(upgrade_error.unwrap().feature, feature_name);
            }

            // Test capabilities struct
            let caps = free_edition.capabilities();
            match capability_field {
                "allow_autofix" => assert!(!caps.autofix_allowed, "Free edition should not allow autofix"),
                "allow_trend" => assert!(!caps.trend_allowed, "Free edition should not allow trend"),
                "allow_mapping_deep" => assert!(!caps.deep_map_allowed, "Free edition should not allow deep mapping"),
                "allow_slo_enforce" => assert!(!caps.enforce_slo_allowed, "Free edition should not allow SLO enforcement"),
                "allow_policy_enforce" => assert!(!caps.enforce_policy_allowed, "Free edition should not allow policy enforcement"),
                "allow_explain_full" => assert!(!caps.explain_advanced_allowed, "Free edition should not allow advanced explain"),
                _ => panic!("Unknown capability field: {}", capability_field),
            }
        }
    }

    #[test]
    fn test_unauthorized_access_attempts_blocked() {
        // Test various unauthorized access attempts are properly blocked
        let free_edition = EditionContext::free();

        // Test require_pro method fails for free edition
        let result = free_edition.require_pro("Test Feature");
        assert!(result.is_err(), "require_pro should fail for free edition");

        // Test that Capabilities::from_edition correctly restricts free users
        let caps = Capabilities::from_edition(&free_edition);
        assert!(!caps.allow_predict, "Free edition should not allow prediction");
        assert!(!caps.allow_explain_full, "Free edition should not allow full explain");
        assert!(!caps.allow_autofix, "Free edition should not allow autofix");
        assert!(!caps.allow_mapping_deep, "Free edition should not allow deep mapping");
        assert!(!caps.allow_trend, "Free edition should not allow trend");
        assert!(!caps.allow_policy_enforce, "Free edition should not allow policy enforcement");
        assert!(!caps.allow_slo_enforce, "Free edition should not allow SLO enforcement");

        // Test that all capabilities are false for free edition
        assert!(!caps.allow_predict && !caps.allow_explain_full && !caps.allow_autofix &&
                !caps.allow_mapping_deep && !caps.allow_trend && !caps.allow_policy_enforce &&
                !caps.allow_slo_enforce, "All capabilities should be false for free edition");
    }

    #[test]
    fn test_role_conflicts_prevented() {
        // Test that role conflicts are prevented (free vs premium capabilities)
        let free_edition = EditionContext::free();

        // Test that free edition cannot have premium capabilities
        let caps = free_edition.capabilities();
        let _premium_caps = EditionCapabilities {
            autofix_allowed: true,
            trend_allowed: true,
            deep_map_allowed: true,
            enforce_slo_allowed: true,
            enforce_policy_allowed: true,
            explain_advanced_allowed: true,
        };

        // Ensure free edition capabilities conflict with premium expectations
        assert!(!caps.autofix_allowed, "Free edition should conflict with premium autofix capability");
        assert!(!caps.trend_allowed, "Free edition should conflict with premium trend capability");
        assert!(!caps.deep_map_allowed, "Free edition should conflict with premium deep mapping capability");
        assert!(!caps.enforce_slo_allowed, "Free edition should conflict with premium SLO enforcement capability");
        assert!(!caps.enforce_policy_allowed, "Free edition should conflict with premium policy enforcement capability");
        assert!(!caps.explain_advanced_allowed, "Free edition should conflict with premium advanced explain capability");

        // Test that attempting to use premium features in free mode fails consistently
        let test_features = vec!["Autofix", "Trend", "Deep Mapping", "SLO Enforcement", "Policy Enforcement"];
        for feature in test_features {
            let result = require_premium(&free_edition, feature);
            assert!(result.is_err(), "Role conflict: {} should be blocked in free edition", feature);
        }
    }

    #[test]
    fn test_permission_inheritance_consistency() {
        // Test that permissions are consistently inherited across the system
        let free_edition = EditionContext::free();

        // Test that EditionContext and Capabilities are consistent
        let context_caps = free_edition.capabilities();
        let direct_caps = Capabilities::from_edition(&free_edition);

        // All capabilities should match between different access methods
        assert_eq!(context_caps.autofix_allowed, direct_caps.allow_autofix,
                  "Permission inheritance should be consistent for autofix");
        assert_eq!(context_caps.trend_allowed, direct_caps.allow_trend,
                  "Permission inheritance should be consistent for trend");
        assert_eq!(context_caps.deep_map_allowed, direct_caps.allow_mapping_deep,
                  "Permission inheritance should be consistent for deep mapping");
        assert_eq!(context_caps.enforce_slo_allowed, direct_caps.allow_slo_enforce,
                  "Permission inheritance should be consistent for SLO enforcement");
        assert_eq!(context_caps.enforce_policy_allowed, direct_caps.allow_policy_enforce,
                  "Permission inheritance should be consistent for policy enforcement");
        assert_eq!(context_caps.explain_advanced_allowed, direct_caps.allow_explain_full,
                  "Permission inheritance should be consistent for advanced explain");

        // Test that edition mode correctly determines inheritance
        assert_eq!(free_edition.mode, EditionMode::Free, "Edition mode should be Free");
        assert!(!free_edition.is_premium(), "Free edition should not be premium");

        // Test that free edition inheritance blocks all premium features
        assert!(!context_caps.autofix_allowed && !context_caps.trend_allowed &&
                !context_caps.deep_map_allowed && !context_caps.enforce_slo_allowed &&
                !context_caps.enforce_policy_allowed && !context_caps.explain_advanced_allowed,
                "Permission inheritance should block all premium features in free edition");
    }

    #[test]
    fn test_access_control_list_enforcement() {
        // Test that access control lists (capabilities) are properly enforced
        let free_edition = EditionContext::free();

        // Define the access control list for free edition
        let expected_acl = vec![
            ("autofix", false),
            ("trend", false),
            ("deep_mapping", false),
            ("slo_enforcement", false),
            ("policy_enforcement", false),
            ("advanced_explain", false),
        ];

        let caps = free_edition.capabilities();

        // Verify each item in the access control list is enforced
        for (feature, should_be_allowed) in expected_acl {
            let is_allowed = match feature {
                "autofix" => caps.autofix_allowed,
                "trend" => caps.trend_allowed,
                "deep_mapping" => caps.deep_map_allowed,
                "slo_enforcement" => caps.enforce_slo_allowed,
                "policy_enforcement" => caps.enforce_policy_allowed,
                "advanced_explain" => caps.explain_advanced_allowed,
                _ => panic!("Unknown feature in ACL: {}", feature),
            };

            assert_eq!(is_allowed, should_be_allowed,
                      "ACL enforcement failed for {}: expected {}, got {}",
                      feature, should_be_allowed, is_allowed);
        }

        // Test that ACL enforcement prevents feature access
        let acl_violations = vec![
            ("Autofix", caps.autofix_allowed),
            ("Trend Analysis", caps.trend_allowed),
            ("Deep Mapping", caps.deep_map_allowed),
            ("SLO Enforcement", caps.enforce_slo_allowed),
            ("Policy Enforcement", caps.enforce_policy_allowed),
            ("Advanced Explain", caps.explain_advanced_allowed),
        ];

        for (feature_name, is_allowed) in acl_violations {
            assert!(!is_allowed, "ACL should prevent {} access in free edition", feature_name);

            // Verify that attempting to use blocked features fails
            let result = require_premium(&free_edition, feature_name);
            assert!(result.is_err(), "ACL should block {} feature access", feature_name);
        }
    }

    #[test]
    fn test_capability_boundary_enforcement() {
        // Test that capability boundaries are strictly enforced
        let free_edition = EditionContext::free();

        // Test boundary: free edition should have zero premium capabilities
        let caps = free_edition.capabilities();
        let premium_capability_count = [
            caps.autofix_allowed,
            caps.trend_allowed,
            caps.deep_map_allowed,
            caps.enforce_slo_allowed,
            caps.enforce_policy_allowed,
            caps.explain_advanced_allowed,
        ].iter().filter(|&&x| x).count();

        assert_eq!(premium_capability_count, 0,
                  "Free edition should have zero premium capabilities, found {}", premium_capability_count);

        // Test boundary: any attempt to access premium features should fail
        let boundary_tests = vec![
            ("Autofix", "core engine feature"),
            ("Trend", "analysis feature"),
            ("Deep Mapping", "mapping feature"),
            ("SLO Enforcement", "enforcement feature"),
            ("Policy Enforcement", "policy feature"),
            ("Advanced Explain", "explanation feature"),
        ];

        for (feature, category) in boundary_tests {
            let result = require_premium(&free_edition, feature);
            assert!(result.is_err(),
                   "Capability boundary should prevent {} ({}) access", feature, category);

            // Verify error type
            if let Err(e) = result {
                assert!(e.downcast_ref::<UpgradeRequired>().is_some(),
                       "Boundary violation should return UpgradeRequired error for {}", feature);
            }
        }
    }

    #[test]
    fn test_authorization_state_consistency() {
        // Test that authorization state remains consistent across operations
        let free_edition = EditionContext::free();

        // Test multiple sequential authorization checks
        let test_features = vec!["Test Feature 0", "Test Feature 1", "Test Feature 2",
                                "Test Feature 3", "Test Feature 4", "Test Feature 5",
                                "Test Feature 6", "Test Feature 7", "Test Feature 8", "Test Feature 9"];

        for (i, feature) in test_features.iter().enumerate() {
            // State should remain consistent
            assert!(free_edition.is_free(), "Authorization state should remain free (check {})", i);
            assert!(!free_edition.is_premium(), "Authorization state should not become premium (check {})", i);

            // Capabilities should remain consistent
            let caps = free_edition.capabilities();
            assert!(!caps.autofix_allowed, "Capabilities should remain restricted (check {})", i);

            // Authorization checks should consistently fail
            let result = require_premium(&free_edition, feature);
            assert!(result.is_err(), "Authorization should consistently fail (check {})", i);
        }

        // Test that state doesn't change after multiple operations
        let final_caps = free_edition.capabilities();
        assert!(!final_caps.autofix_allowed, "Final state should remain restricted");
        assert!(!final_caps.trend_allowed, "Final state should remain restricted");
    }

    #[test]
    fn test_cross_feature_authorization_isolation() {
        // Test that authorization failures in one feature don't affect others
        let free_edition = EditionContext::free();

        // Test that failing to access one feature doesn't prevent checking others
        let features = vec!["Feature A", "Feature B", "Feature C"];

        for feature in features {
            let result = require_premium(&free_edition, feature);
            assert!(result.is_err(), "Each feature should be independently blocked: {}", feature);

            // Verify other features are still properly checked
            let other_result = require_premium(&free_edition, "Other Feature");
            assert!(other_result.is_err(), "Cross-feature isolation failed for: {}", feature);
        }

        // Test that capability checks are isolated
        let caps1 = free_edition.capabilities();
        let caps2 = free_edition.capabilities();

        assert_eq!(caps1.autofix_allowed, caps2.autofix_allowed,
                  "Capability checks should be isolated and consistent");
    }
}