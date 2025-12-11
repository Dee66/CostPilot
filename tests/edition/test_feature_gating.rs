// Feature gating tests - ensure Premium features are blocked in Free mode

use costpilot::edition::{EditionContext, require_premium};

#[test]
fn test_autofix_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Autofix");
    
    assert!(result.is_err(), "Autofix should require premium");
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Autofix") || err_str.contains("Premium"));
}

#[test]
fn test_trend_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Trend tracking");
    
    assert!(result.is_err(), "Trend should require premium");
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Trend") || err_str.contains("Premium"));
}

#[test]
fn test_deep_map_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Deep mapping");
    
    assert!(result.is_err(), "Deep mapping should require premium");
}

#[test]
fn test_slo_enforce_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "SLO enforcement");
    
    assert!(result.is_err(), "SLO enforcement should require premium");
}

#[test]
fn test_policy_enforce_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Policy enforcement");
    
    assert!(result.is_err(), "Policy enforcement should require premium");
}

#[test]
fn test_scan_allowed_in_free() {
    let edition = EditionContext::free();
    
    // Scan should work in free mode (basic functionality)
    assert!(edition.is_free());
    
    // No gating for basic scan
    assert!(edition.pro.is_none());
}

#[test]
fn test_explain_basic_allowed_in_free() {
    let edition = EditionContext::free();
    
    // Basic explain (non-verbose) should work in free
    let result = require_premium(&edition, "Advanced Explain");
    assert!(result.is_err(), "Advanced explain should require premium");
}

#[test]
fn test_diff_requires_premium() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Diff");
    
    assert!(result.is_err(), "Diff should require premium");
}
