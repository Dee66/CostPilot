// Error model enforcement tests

use costpilot::engines::shared::error_model::{CostPilotError, ErrorCategory, map_category_to_id};

#[test]
fn test_stable_error_ids_are_consistent() {
    let error1 = CostPilotError::parse_error("test");
    let error2 = CostPilotError::parse_error("different message");
    
    // Same error type should have same ID
    assert_eq!(error1.id, error2.id);
    assert_eq!(error1.id, "E_PARSE");
}

#[test]
fn test_error_ids_follow_naming_convention() {
    let errors = vec![
        CostPilotError::parse_error("test"),
        CostPilotError::validation_error("test"),
        CostPilotError::io_error("test"),
        CostPilotError::config_error("test"),
        CostPilotError::prediction_error("test"),
    ];
    
    for error in errors {
        assert!(error.id.starts_with("E_"), "Error ID should start with E_: {}", error.id);
        assert!(error.id.chars().all(|c| c.is_uppercase() || c == '_'), 
            "Error ID should be uppercase with underscores: {}", error.id);
    }
}

#[test]
fn test_error_category_mapping_is_stable() {
    let categories = vec![
        ErrorCategory::InvalidInput,
        ErrorCategory::ParseError,
        ErrorCategory::PredictionError,
        ErrorCategory::PolicyViolation,
        ErrorCategory::SLOBreach,
        ErrorCategory::ValidationError,
        ErrorCategory::IoError,
        ErrorCategory::NotFound,
    ];
    
    for category in categories {
        let id = map_category_to_id(&category);
        assert!(id.starts_with("E_"), "Category mapping should produce E_ prefix");
        
        // Same category should always map to same ID
        let id2 = map_category_to_id(&category);
        assert_eq!(id, id2, "Category mapping should be stable");
    }
}

#[test]
fn test_all_categories_have_mappings() {
    let categories = vec![
        ErrorCategory::InvalidInput,
        ErrorCategory::ParseError,
        ErrorCategory::PredictionError,
        ErrorCategory::PolicyViolation,
        ErrorCategory::SLOBreach,
        ErrorCategory::DriftDetected,
        ErrorCategory::InternalError,
        ErrorCategory::ConfigError,
        ErrorCategory::FileSystemError,
        ErrorCategory::Timeout,
        ErrorCategory::CircuitBreaker,
        ErrorCategory::ValidationError,
        ErrorCategory::IoError,
        ErrorCategory::NotFound,
        ErrorCategory::SecurityViolation,
    ];
    
    for category in categories {
        let id = map_category_to_id(&category);
        assert!(!id.is_empty(), "Category {:?} should have a non-empty ID mapping", category);
    }
}

#[test]
fn test_remediation_hint_generation() {
    let errors = vec![
        CostPilotError::parse_error("invalid syntax"),
        CostPilotError::validation_error("missing field"),
        CostPilotError::io_error("file not found"),
        CostPilotError::config_error("invalid config"),
    ];
    
    for error in errors {
        let hint = error.generate_hint();
        assert!(!hint.is_empty(), "Every error should generate a non-empty hint");
        assert!(hint.len() > 10, "Hint should be descriptive: {}", hint);
    }
}

#[test]
fn test_remediation_hints_are_actionable() {
    let error = CostPilotError::parse_error("invalid JSON");
    let hint = error.generate_hint();
    
    // Hints should contain actionable verbs
    let actionable_verbs = ["check", "verify", "review", "ensure", "try", "use"];
    let hint_lower = hint.to_lowercase();
    
    assert!(actionable_verbs.iter().any(|verb| hint_lower.contains(verb)),
        "Hint should contain actionable verbs: {}", hint);
}

#[test]
fn test_error_with_explicit_hint() {
    let error = CostPilotError::parse_error("test")
        .with_hint("Custom remediation hint");
    
    assert_eq!(error.hint, Some("Custom remediation hint".to_string()));
    assert_eq!(error.generate_hint(), "Custom remediation hint");
}

#[test]
fn test_error_with_context() {
    let context = serde_json::json!({
        "file": "plan.json",
        "line": 42
    });
    
    let error = CostPilotError::parse_error("syntax error")
        .with_context(context.clone());
    
    assert_eq!(error.context, Some(context));
}

#[test]
fn test_machine_readable_format() {
    let error = CostPilotError::parse_error("test error");
    let json = error.to_machine_format();
    
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed["id"], "E_PARSE");
    assert_eq!(parsed["message"], "test error");
    assert_eq!(parsed["category"], "ParseError");
}

#[test]
fn test_machine_format_includes_hint() {
    let error = CostPilotError::parse_error("test")
        .with_hint("fix it");
    
    let json = error.to_machine_format();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed["hint"], "fix it");
}

#[test]
fn test_machine_format_includes_context() {
    let context = serde_json::json!({"file": "test.json"});
    let error = CostPilotError::parse_error("test")
        .with_context(context.clone());
    
    let json = error.to_machine_format();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed["context"]["file"], "test.json");
}

#[test]
fn test_policy_violation_error_has_stable_id() {
    let error1 = CostPilotError::policy_violation("MAX_COST", "exceeded");
    let error2 = CostPilotError::policy_violation("MAX_COST", "different message");
    
    // Same policy should generate same ID
    assert_eq!(error1.id, error2.id);
    assert_eq!(error1.id, "E_POLICY_MAX_COST");
}

#[test]
fn test_slo_breach_error_has_stable_id() {
    let error1 = CostPilotError::slo_breach("monthly budget", "exceeded");
    let error2 = CostPilotError::slo_breach("monthly budget", "different message");
    
    // Same SLO should generate same ID
    assert_eq!(error1.id, error2.id);
    assert!(error1.id.starts_with("E_SLO_"));
}

#[test]
fn test_error_display_format() {
    let error = CostPilotError::parse_error("syntax error")
        .with_hint("check JSON format");
    
    let display = format!("{}", error);
    
    assert!(display.contains("[E_PARSE]"));
    assert!(display.contains("syntax error"));
    assert!(display.contains("Hint:"));
    assert!(display.contains("check JSON format"));
}

#[test]
fn test_error_without_hint_still_generates_one() {
    let error = CostPilotError::new("E_TEST", ErrorCategory::InternalError, "test");
    
    // Should generate default hint based on category
    let hint = error.generate_hint();
    assert!(!hint.is_empty());
    assert!(hint.contains("internal"));
}
