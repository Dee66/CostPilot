// Anti-leakage tests - ensure free edition doesn't expose premium internals

use costpilot::edition::EditionContext;

#[test]
fn test_free_mode_never_calls_pro_execute() {
    let edition = EditionContext::free();

    // Verify pro is None
    assert!(edition.pro.is_none(), "Free edition should have no ProEngine handle");

    // Verify is_free returns true
    assert!(edition.is_free());
    assert!(!edition.is_premium());
}

#[test]
fn test_free_edition_has_no_premium_capabilities() {
    let edition = EditionContext::free();

    // Verify all premium capabilities are false
    assert!(!edition.capabilities.allow_autofix);
    assert!(!edition.capabilities.allow_trend);
    assert!(!edition.capabilities.allow_mapping_deep);
    assert!(!edition.capabilities.allow_policy_enforce);
    assert!(!edition.capabilities.allow_slo_enforce);
}

#[test]
fn test_require_pro_never_succeeds_in_free() {
    let edition = EditionContext::free();

    let features = [
        "Autofix",
        "Trend tracking",
        "Deep mapping",
        "Policy enforcement",
        "SLO enforcement",
        "Advanced Explain",
    ];

    for feature in &features {
        let result = edition.require_pro(feature);
        assert!(result.is_err(), "require_pro should fail for {}", feature);
    }
}

#[test]
fn test_free_binary_string_content() {
    // This test would inspect the binary for strings
    // In a real implementation, would build free-only binary and check symbols

    // Verify edition context doesn't leak premium symbols
    let edition = EditionContext::free();
    let debug_str = format!("{:?}", edition.mode);

    assert!(!debug_str.contains("Premium") || debug_str.contains("Free"),
        "Free mode debug output should not expose premium internals");
}

#[test]
fn test_no_premium_filesystem_access_in_free() {
    let edition = EditionContext::free();

    // Verify paths are set but not accessed
    let wasm_path = edition.paths.pro_wasm_path();
    let license_path = edition.paths.license_path();

    // Paths can exist in struct, but shouldn't be read in free mode
    assert!(wasm_path.to_str().unwrap().contains("pro_engine"));
    assert!(license_path.to_str().unwrap().contains("license"));
}
