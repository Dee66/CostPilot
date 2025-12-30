// Integration tests for expired exemptions CI blocking

use costpilot::engines::policy::{
    check_exemptions_for_ci, ExemptionsFile, PolicyExemption, EXIT_EXEMPTION_EXPIRED, EXIT_SUCCESS,
};

fn create_test_exemption(id: &str, expires_at: &str) -> PolicyExemption {
    PolicyExemption {
        id: id.to_string(),
        policy_name: "NAT_GATEWAY_LIMIT".to_string(),
        resource_pattern: "module.vpc.*".to_string(),
        justification: "Required for production".to_string(),
        expires_at: expires_at.to_string(),
        approved_by: "ops@example.com".to_string(),
        created_at: "2025-01-01T00:00:00Z".to_string(),
        ticket_ref: Some("INFRA-123".to_string()),
    }
}

#[test]
fn test_ci_passes_with_all_active_exemptions() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2026-12-31"),
            create_test_exemption("EXE-002", "2027-06-30"),
            create_test_exemption("EXE-003", "2028-01-01"),
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.total_exemptions, 3);
    assert_eq!(result.active_exemptions, 3);
    assert_eq!(result.expired_exemptions, 0);
    assert_eq!(result.invalid_exemptions, 0);
    assert!(result.should_pass());
    assert_eq!(result.exit_code(), EXIT_SUCCESS);
}

#[test]
fn test_ci_blocked_with_single_expired_exemption() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2024-01-01"), // Expired
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.expired_exemptions, 1);
    assert!(!result.should_pass());
    assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED);

    // Verify expired details
    assert_eq!(result.expired_details.len(), 1);
    assert_eq!(result.expired_details[0].id, "EXE-001");
    assert_eq!(result.expired_details[0].expired_on, "2024-01-01");
}

#[test]
fn test_ci_blocked_with_multiple_expired_exemptions() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2023-12-31"),
            create_test_exemption("EXE-002", "2024-06-15"),
            create_test_exemption("EXE-003", "2024-11-30"),
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.expired_exemptions, 3);
    assert!(!result.should_pass());
    assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED);
    assert_eq!(result.expired_details.len(), 3);
}

#[test]
fn test_ci_blocked_with_mixed_active_and_expired() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2026-12-31"), // Active
            create_test_exemption("EXE-002", "2024-01-01"), // Expired
            create_test_exemption("EXE-003", "2027-06-30"), // Active
            create_test_exemption("EXE-004", "2024-06-15"), // Expired
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.total_exemptions, 4);
    assert_eq!(result.active_exemptions, 2);
    assert_eq!(result.expired_exemptions, 2);
    assert!(!result.should_pass());
    assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED);
}

#[test]
fn test_ci_summary_includes_expired_details() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2024-01-01"),
            create_test_exemption("EXE-002", "2024-06-15"),
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();
    let summary = result.summary();

    // Verify summary contains key information
    assert!(summary.contains("Total exemptions: 2"));
    assert!(summary.contains("Expired: 2"));
    assert!(summary.contains("Expired exemptions (blocking CI)"));
    assert!(summary.contains("EXE-001"));
    assert!(summary.contains("EXE-002"));
    assert!(summary.contains("2024-01-01"));
    assert!(summary.contains("2024-06-15"));
}

#[test]
fn test_ci_passes_with_empty_exemptions_file() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.total_exemptions, 0);
    assert!(result.should_pass());
    assert_eq!(result.exit_code(), EXIT_SUCCESS);
}

#[test]
fn test_expired_exemption_detail_has_correct_fields() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![PolicyExemption {
            id: "EXE-999".to_string(),
            policy_name: "EC2_INSTANCE_TYPE".to_string(),
            resource_pattern: "module.app.instance[0]".to_string(),
            justification: "Legacy hardware".to_string(),
            expires_at: "2024-03-15".to_string(),
            approved_by: "dev@example.com".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ticket_ref: Some("DEV-456".to_string()),
        }],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.expired_details.len(), 1);
    let detail = &result.expired_details[0];
    assert_eq!(detail.id, "EXE-999");
    assert_eq!(detail.policy_name, "EC2_INSTANCE_TYPE");
    assert_eq!(detail.resource_pattern, "module.app.instance[0]");
    assert_eq!(detail.expired_on, "2024-03-15");
}

#[test]
fn test_ci_tracks_expiring_soon_separately() {
    use chrono::Utc;

    // Create exemption expiring in 15 days
    let today = Utc::now().date_naive();
    let expiring_date = today + chrono::Duration::days(15);
    let expiring_str = expiring_date.format("%Y-%m-%d").to_string();

    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", &expiring_str),
            create_test_exemption("EXE-002", "2026-12-31"), // Active
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.total_exemptions, 2);
    assert_eq!(result.expiring_soon, 1);
    assert_eq!(result.expired_exemptions, 0);
    assert!(result.should_pass()); // Expiring soon doesn't block CI
    assert_eq!(result.exit_code(), EXIT_SUCCESS);
}

#[test]
fn test_exit_code_priority_expired_over_invalid() {
    // If there are both expired and invalid, expired takes priority
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![
            create_test_exemption("EXE-001", "2024-01-01"), // Expired
            PolicyExemption {
                id: "".to_string(), // Invalid: empty ID
                policy_name: "TEST".to_string(),
                resource_pattern: "test.*".to_string(),
                justification: "Test".to_string(),
                expires_at: "2026-12-31".to_string(),
                approved_by: "test@example.com".to_string(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                ticket_ref: None,
            },
        ],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    assert_eq!(result.expired_exemptions, 1);
    assert_eq!(result.invalid_exemptions, 1);
    assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED); // Expired has priority
}

#[test]
fn test_expired_on_exact_date() {
    use chrono::Utc;

    // Exemption expires today
    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![create_test_exemption("EXE-001", &today_str)],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();

    // Today counts as expiring soon (within threshold)
    assert_eq!(result.expiring_soon, 1);
    assert_eq!(result.expired_exemptions, 0);
    assert!(result.should_pass());
}

#[test]
fn test_summary_shows_zero_counts() {
    let exemptions_file = ExemptionsFile {
        version: "1.0".to_string(),
        exemptions: vec![create_test_exemption("EXE-001", "2026-12-31")],
        metadata: None,
    };

    let result = check_exemptions_for_ci(&exemptions_file).unwrap();
    let summary = result.summary();

    assert!(summary.contains("Expired: 0"));
    assert!(summary.contains("Invalid: 0"));
    assert!(!summary.contains("Expired exemptions (blocking CI)")); // Should not show section if no expired
}
