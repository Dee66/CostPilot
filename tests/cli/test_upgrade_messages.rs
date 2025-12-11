// CLI upgrade messaging tests - verify user-facing error messages

use costpilot::edition::{EditionContext, require_premium, upgrade_message};

#[test]
fn test_autofix_upgrade_message_text() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Autofix");
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    
    assert!(err_msg.contains("Autofix"), "Should mention Autofix");
    assert!(err_msg.contains("Premium") || err_msg.contains("upgrade"), 
        "Should mention Premium or upgrade");
}

#[test]
fn test_trend_upgrade_message_text() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Trend tracking");
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    
    assert!(err_msg.contains("Trend"), "Should mention Trend");
}

#[test]
fn test_upgrade_message_contains_url() {
    let msg = upgrade_message("TestFeature");
    
    assert!(msg.contains("https://shieldcraft-ai.com/costpilot/upgrade"),
        "Upgrade message should contain upgrade URL");
    assert!(msg.contains("TestFeature"), "Should mention feature name");
}

#[test]
fn test_upgrade_message_contains_feature_list() {
    let msg = upgrade_message("Autofix");
    
    // Should list premium features
    let premium_features = [
        "Cost predictions",
        "Autofix",
        "Deep mapping",
        "Trend analysis",
    ];
    
    let mut found_count = 0;
    for feature in &premium_features {
        if msg.contains(feature) {
            found_count += 1;
        }
    }
    
    assert!(found_count >= 2, "Should list multiple premium features");
}

#[test]
fn test_diff_upgrade_message() {
    let edition = EditionContext::free();
    let result = require_premium(&edition, "Diff");
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Diff"));
}
