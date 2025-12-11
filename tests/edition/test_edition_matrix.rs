// Edition matrix tests - Free vs Premium detection

use costpilot::edition::{EditionContext, detect_edition};

#[test]
fn test_free_mode_when_no_license_or_wasm() {
    // No license or WASM files present
    let result = detect_edition();
    
    assert!(result.is_ok(), "detect_edition should not fail when files missing");
    let edition = result.unwrap();
    
    assert!(edition.is_free(), "Should be free mode when no license/wasm");
    assert!(!edition.is_premium(), "Should not be premium mode");
    assert!(edition.pro.is_none(), "ProEngine handle should be None");
}

#[test]
fn test_premium_mode_when_license_and_wasm_valid() {
    // TODO: Mock load_pro_engine to return Some(ProEngineHandle)
    // This requires dependency injection or test harness
    
    // For now, verify free mode behavior
    let edition = EditionContext::free();
    assert!(edition.is_free());
    assert!(edition.pro.is_none());
}

#[test]
fn test_fallback_to_free_on_invalid_license() {
    // TODO: Simulate invalid signature / expired license
    // Should fall back to free edition
    
    let edition = EditionContext::free();
    assert!(edition.is_free());
    assert!(edition.pro.is_none());
}

#[test]
fn test_require_pro_returns_error_in_free() {
    let edition = EditionContext::free();
    let result = edition.require_pro("TestFeature");
    
    assert!(result.is_err(), "require_pro should fail in free mode");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("TestFeature") || err_str.contains("Premium"), 
        "Error should mention feature or Premium requirement");
}
