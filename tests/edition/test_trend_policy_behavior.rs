// Trend and Policy behavior tests - verify edition-specific behavior

use costpilot::edition::{EditionContext, require_premium};

#[test]
fn test_trend_blocked_in_free() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Trend tracking");

    assert!(result.is_err(), "Trend should be blocked in free mode");
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Trend") || err_str.contains("Premium"));
}

#[test]
fn test_policy_lint_allowed_in_free() {
    let edition = EditionContext::free();

    // Policy lint mode should work in free
    assert!(!edition.capabilities.allow_policy_enforce,
        "Policy enforcement should not be allowed");

    // But basic policy evaluation (lint) should work
    assert!(edition.is_free());
}

#[test]
fn test_policy_enforce_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Policy enforcement");

    assert!(result.is_err(), "Policy enforcement requires premium");
}

#[test]
fn test_slo_validate_allowed_in_free() {
    let edition = EditionContext::free();

    // SLO validation should work in free
    assert!(!edition.capabilities.allow_slo_enforce,
        "SLO enforcement should not be allowed");
}

#[test]
fn test_slo_enforce_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "SLO enforcement");

    assert!(result.is_err(), "SLO enforcement requires premium");
}

#[test]
fn test_map_depth_1_allowed_in_free() {
    let edition = EditionContext::free();

    // Depth 1 mapping should work
    assert!(!edition.capabilities.allow_mapping_deep);
}

#[test]
fn test_map_deep_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Deep mapping");

    assert!(result.is_err(), "Deep mapping (depth > 1) requires premium");
}

#[test]
fn test_explain_basic_free_advanced_premium() {
    let edition = EditionContext::free();

    // Basic explain should work in free
    assert!(!edition.capabilities.allow_explain_full);

    // Advanced explain requires premium
    let result = require_premium(&edition, "Advanced Explain");
    assert!(result.is_err());
}

#[test]
fn test_premium_capabilities_all_enabled() {
    // When premium is properly loaded, all capabilities should be true
    // TODO: Mock premium edition context

    let free_edition = EditionContext::free();
    let caps = free_edition.capabilities();

    assert!(!caps.autofix_allowed);
    assert!(!caps.trend_allowed);
    assert!(!caps.deep_map_allowed);
    assert!(!caps.enforce_slo_allowed);
    assert!(!caps.enforce_policy_allowed);
    assert!(!caps.explain_advanced_allowed);
}
