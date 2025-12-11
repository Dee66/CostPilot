use std::time::Duration;
//! Comprehensive exemption workflow tests
//!
//! Tests exemption lifecycle, approval workflows, expiration handling,
//! reference validation, and CI/CD integration.

use chrono::{Duration, Utc};

#[test]
fn test_exemption_creation_requires_all_fields() {
    let incomplete_exemption = ExemptionBuilder::new()
        .rule_id("budget-limit")
        .resource_pattern("aws_instance.*")
        // Missing: expires_at, reason, approved_by, approval_reference
        .build();
    
    assert!(incomplete_exemption.is_err());
    assert!(incomplete_exemption.unwrap_err().to_string().contains("required fields"));
}

#[test]
fn test_exemption_with_valid_reference() {
    let exemption = ExemptionBuilder::new()
        .rule_id("budget-limit")
        .resource_pattern("aws_instance.web")
        .expires_at(Utc::now() + Duration::days(30))
        .reason("Temporary spike for migration")
        .approved_by("tech-lead@company.com")
        .approval_reference("JIRA-12345")
        .build();
    
    assert!(exemption.is_ok());
    let ex = exemption.unwrap();
    assert_eq!(ex.approval_reference, "JIRA-12345");
}

#[test]
fn test_exemption_reference_validation_formats() {
    // Test various valid reference formats
    let jira_ref = validate_reference("JIRA-12345");
    let github_ref = validate_reference("GH-ISSUE-456");
    let url_ref = validate_reference("https://tickets.company.com/ticket/789");
    let slack_ref = validate_reference("SLACK-THREAD-2024-01-15");
    
    assert!(jira_ref.is_ok());
    assert!(github_ref.is_ok());
    assert!(url_ref.is_ok());
    assert!(slack_ref.is_ok());
}

#[test]
fn test_exemption_reference_validation_rejects_invalid() {
    let empty_ref = validate_reference("");
    let short_ref = validate_reference("AB");
    let spaces_only = validate_reference("   ");
    
    assert!(empty_ref.is_err());
    assert!(short_ref.is_err());
    assert!(spaces_only.is_err());
}

#[test]
fn test_exemption_expiration_exact_date() {
    let expires_tomorrow = Utc::now() + Duration::days(1);
    let expires_yesterday = Utc::now() - Duration::days(1);
    
    let active_exemption = mock_exemption_expiring_at(expires_tomorrow);
    let expired_exemption = mock_exemption_expiring_at(expires_yesterday);
    
    assert!(!is_expired(&active_exemption));
    assert!(is_expired(&expired_exemption));
}

#[test]
fn test_exemption_expiring_soon_warning() {
    let expires_in_5_days = Utc::now() + Duration::days(5);
    let expires_in_15_days = Utc::now() + Duration::days(15);
    
    let soon_exemption = mock_exemption_expiring_at(expires_in_5_days);
    let later_exemption = mock_exemption_expiring_at(expires_in_15_days);
    
    assert!(is_expiring_soon(&soon_exemption, 7)); // Within 7 days
    assert!(!is_expiring_soon(&later_exemption, 7));
}

#[test]
fn test_exemption_pattern_matching_exact() {
    let exemption = mock_exemption_with_pattern("aws_instance.web");
    
    assert!(matches_pattern(&exemption, "aws_instance.web"));
    assert!(!matches_pattern(&exemption, "aws_instance.api"));
}

#[test]
fn test_exemption_pattern_matching_wildcard() {
    let exemption = mock_exemption_with_pattern("aws_instance.*");
    
    assert!(matches_pattern(&exemption, "aws_instance.web"));
    assert!(matches_pattern(&exemption, "aws_instance.api"));
    assert!(matches_pattern(&exemption, "aws_instance.worker"));
    assert!(!matches_pattern(&exemption, "aws_rds_instance.db"));
}

#[test]
fn test_exemption_pattern_matching_regex() {
    let exemption = mock_exemption_with_pattern(r"aws_instance\.(prod|staging)-.*");
    
    assert!(matches_pattern(&exemption, "aws_instance.prod-web-01"));
    assert!(matches_pattern(&exemption, "aws_instance.staging-api-02"));
    assert!(!matches_pattern(&exemption, "aws_instance.dev-test-01"));
}

#[test]
fn test_exemption_applies_to_violation() {
    let exemption = mock_exemption_for_rule("budget-limit", "aws_instance.web");
    let matching_violation = mock_violation("budget-limit", "aws_instance.web");
    let different_rule_violation = mock_violation("tagging-required", "aws_instance.web");
    let different_resource_violation = mock_violation("budget-limit", "aws_instance.api");
    
    assert!(exemption_applies(&exemption, &matching_violation));
    assert!(!exemption_applies(&exemption, &different_rule_violation));
    assert!(!exemption_applies(&exemption, &different_resource_violation));
}

#[test]
fn test_exemption_approval_workflow_single_approver() {
    let exemption_request = mock_exemption_request();
    let approval = mock_approval("tech-lead@company.com", "JIRA-123");
    
    let result = process_approval(&exemption_request, &approval);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, "approved");
}

#[test]
fn test_exemption_approval_workflow_multiple_approvers() {
    let exemption_request = mock_exemption_request_requiring_n_approvals(2);
    let approval1 = mock_approval("tech-lead@company.com", "JIRA-123");
    let approval2 = mock_approval("eng-manager@company.com", "JIRA-123");
    
    let after_first = process_approval(&exemption_request, &approval1);
    assert!(after_first.is_ok());
    assert_eq!(after_first.as_ref().unwrap().status, "pending"); // Still needs 1 more
    
    let after_second = process_approval(&after_first.unwrap(), &approval2);
    assert!(after_second.is_ok());
    assert_eq!(after_second.unwrap().status, "approved");
}

#[test]
fn test_exemption_approval_requires_reference_match() {
    let exemption_request = mock_exemption_request_with_reference("JIRA-123");
    let matching_approval = mock_approval("tech-lead@company.com", "JIRA-123");
    let mismatched_approval = mock_approval("tech-lead@company.com", "JIRA-456");
    
    let match_result = process_approval(&exemption_request, &matching_approval);
    let mismatch_result = process_approval(&exemption_request, &mismatched_approval);
    
    assert!(match_result.is_ok());
    assert!(mismatch_result.is_err());
    assert!(mismatch_result.unwrap_err().to_string().contains("reference mismatch"));
}

#[test]
fn test_exemption_renewal_extends_expiration() {
    let original_exemption = mock_exemption_expiring_at(Utc::now() + Duration::days(5));
    let renewal_days = 30;
    
    let renewed = renew_exemption(&original_exemption, renewal_days);
    
    assert!(renewed.is_ok());
    let renewed_ex = renewed.unwrap();
    assert!(renewed_ex.expires_at > original_exemption.expires_at);
    assert_eq!(
        (renewed_ex.expires_at - Utc::now()).num_days(),
        renewal_days as i64
    );
}

#[test]
fn test_exemption_revocation() {
    let active_exemption = mock_active_exemption();
    let revocation_reason = "Policy changed - exemption no longer needed";
    
    let revoked = revoke_exemption(&active_exemption, revocation_reason);
    
    assert!(revoked.is_ok());
    let revoked_ex = revoked.unwrap();
    assert_eq!(revoked_ex.status, "revoked");
    assert!(revoked_ex.revoked_at.is_some());
    assert_eq!(revoked_ex.revocation_reason.unwrap(), revocation_reason);
}

#[test]
fn test_exemption_audit_log_created() {
    let exemption = mock_exemption();
    
    let audit_log = get_exemption_audit_log(&exemption);
    
    assert!(audit_log.is_ok());
    let log = audit_log.unwrap();
    assert!(!log.events.is_empty());
    assert!(log.events.iter().any(|e| e.event_type == "exemption_created"));
}

#[test]
fn test_exemption_audit_log_tracks_changes() {
    let mut exemption = mock_exemption();
    
    exemption = renew_exemption(&exemption, 30).unwrap();
    exemption = revoke_exemption(&exemption, "Test").unwrap();
    
    let audit_log = get_exemption_audit_log(&exemption);
    
    assert!(audit_log.is_ok());
    let log = audit_log.unwrap();
    assert!(log.events.iter().any(|e| e.event_type == "exemption_renewed"));
    assert!(log.events.iter().any(|e| e.event_type == "exemption_revoked"));
}

#[test]
fn test_exemption_ci_check_passes_all_active() {
    let exemptions = vec![
        mock_exemption_expiring_at(Utc::now() + Duration::days(30)),
        mock_exemption_expiring_at(Utc::now() + Duration::days(15)),
        mock_exemption_expiring_at(Utc::now() + Duration::days(60)),
    ];
    
    let ci_result = run_exemption_ci_check(&exemptions);
    
    assert!(ci_result.passed);
    assert_eq!(ci_result.expired_count, 0);
    assert_eq!(ci_result.exit_code, 0);
}

#[test]
fn test_exemption_ci_check_fails_with_expired() {
    let exemptions = vec![
        mock_exemption_expiring_at(Utc::now() + Duration::days(30)),
        mock_exemption_expiring_at(Utc::now() - Duration::days(1)), // Expired
        mock_exemption_expiring_at(Utc::now() - Duration::days(5)), // Expired
    ];
    
    let ci_result = run_exemption_ci_check(&exemptions);
    
    assert!(!ci_result.passed);
    assert_eq!(ci_result.expired_count, 2);
    assert_eq!(ci_result.exit_code, 1);
}

#[test]
fn test_exemption_ci_check_warns_expiring_soon() {
    let exemptions = vec![
        mock_exemption_expiring_at(Utc::now() + Duration::days(3)), // Expiring soon
        mock_exemption_expiring_at(Utc::now() + Duration::days(30)),
    ];
    
    let ci_result = run_exemption_ci_check(&exemptions);
    
    assert!(ci_result.passed); // Doesn't fail, but warns
    assert_eq!(ci_result.expiring_soon_count, 1);
    assert!(!ci_result.warnings.is_empty());
}

#[test]
fn test_exemption_serialization_roundtrip() {
    let original = mock_exemption();
    
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Exemption = serde_json::from_str(&json).unwrap();
    
    assert_eq!(original.rule_id, deserialized.rule_id);
    assert_eq!(original.resource_pattern, deserialized.resource_pattern);
    assert_eq!(original.approval_reference, deserialized.approval_reference);
}

#[test]
fn test_exemption_bulk_import_validation() {
    let exemptions_yaml = r#"
exemptions:
  - rule_id: budget-limit
    resource_pattern: aws_instance.web
    expires_at: 2025-12-31T23:59:59Z
    reason: Migration
    approved_by: tech-lead
    approval_reference: JIRA-123
  - rule_id: tagging-required
    resource_pattern: aws_rds_instance.*
    expires_at: 2025-06-30T23:59:59Z
    reason: Legacy resources
    approved_by: manager
    approval_reference: JIRA-456
"#;
    
    let import_result = import_exemptions_from_yaml(exemptions_yaml);
    
    assert!(import_result.is_ok());
    let exemptions = import_result.unwrap();
    assert_eq!(exemptions.len(), 2);
}

#[test]
fn test_exemption_bulk_import_rejects_invalid() {
    let invalid_yaml = r#"
exemptions:
  - rule_id: budget-limit
    resource_pattern: aws_instance.web
    # Missing required fields
"#;
    
    let import_result = import_exemptions_from_yaml(invalid_yaml);
    
    assert!(import_result.is_err());
}

// Mock helper functions

fn mock_exemption_expiring_at(expires_at: chrono::DateTime<chrono::Utc>) -> Exemption {
    Exemption {
        rule_id: "budget-limit".to_string(),
        resource_pattern: "aws_instance.web".to_string(),
        expires_at,
        reason: "Test exemption".to_string(),
        approved_by: "tech-lead".to_string(),
        approval_reference: "JIRA-123".to_string(),
        status: "active".to_string(),
        revoked_at: None,
        revocation_reason: None,
    }
}

fn mock_exemption_with_pattern(pattern: &str) -> Exemption {
    let mut ex = mock_exemption();
    ex.resource_pattern = pattern.to_string();
    ex
}

fn mock_exemption_for_rule(rule_id: &str, resource_pattern: &str) -> Exemption {
    let mut ex = mock_exemption();
    ex.rule_id = rule_id.to_string();
    ex.resource_pattern = resource_pattern.to_string();
    ex
}

fn mock_exemption() -> Exemption {
    mock_exemption_expiring_at(Utc::now() + Duration::days(30))
}

fn mock_active_exemption() -> Exemption {
    mock_exemption()
}

fn mock_violation(rule_id: &str, resource: &str) -> Violation {
    Violation {
        rule_id: rule_id.to_string(),
        resource_address: resource.to_string(),
    }
}

fn mock_exemption_request() -> ExemptionRequest {
    ExemptionRequest {
        rule_id: "budget-limit".to_string(),
        resource_pattern: "aws_instance.web".to_string(),
        reason: "Test".to_string(),
        requested_by: "dev@company.com".to_string(),
        approval_reference: "JIRA-123".to_string(),
        required_approvals: 1,
        approvals: vec![],
        status: "pending".to_string(),
    }
}

fn mock_exemption_request_requiring_n_approvals(n: usize) -> ExemptionRequest {
    let mut req = mock_exemption_request();
    req.required_approvals = n;
    req
}

fn mock_exemption_request_with_reference(reference: &str) -> ExemptionRequest {
    let mut req = mock_exemption_request();
    req.approval_reference = reference.to_string();
    req
}

fn mock_approval(approver: &str, reference: &str) -> Approval {
    Approval {
        approver: approver.to_string(),
        timestamp: Utc::now(),
        reference: reference.to_string(),
    }
}

// Stub implementations

fn validate_reference(reference: &str) -> Result<(), String> {
    if reference.trim().is_empty() || reference.len() < 3 {
        Err("Invalid reference".to_string())
    } else {
        Ok(())
    }
}

fn is_expired(exemption: &Exemption) -> bool {
    exemption.expires_at < Utc::now()
}

fn is_expiring_soon(exemption: &Exemption, days: i64) -> bool {
    let threshold = Utc::now() + Duration::days(days);
    exemption.expires_at < threshold && exemption.expires_at > Utc::now()
}

fn matches_pattern(exemption: &Exemption, resource: &str) -> bool {
    let pattern = exemption.resource_pattern.replace("*", ".*");
    regex::Regex::new(&pattern).unwrap().is_match(resource)
}

fn exemption_applies(exemption: &Exemption, violation: &Violation) -> bool {
    exemption.rule_id == violation.rule_id 
        && matches_pattern(exemption, &violation.resource_address)
}

fn process_approval(request: &ExemptionRequest, approval: &Approval) -> Result<ExemptionRequest, String> {
    if request.approval_reference != approval.reference {
        return Err("Approval reference mismatch".to_string());
    }
    
    let mut updated = request.clone();
    updated.approvals.push(approval.clone());
    
    if updated.approvals.len() >= updated.required_approvals {
        updated.status = "approved".to_string();
    }
    
    Ok(updated)
}

fn renew_exemption(exemption: &Exemption, days: u32) -> Result<Exemption, String> {
    let mut renewed = exemption.clone();
    renewed.expires_at = Utc::now() + Duration::days(days as i64);
    Ok(renewed)
}

fn revoke_exemption(exemption: &Exemption, reason: &str) -> Result<Exemption, String> {
    let mut revoked = exemption.clone();
    revoked.status = "revoked".to_string();
    revoked.revoked_at = Some(Utc::now());
    revoked.revocation_reason = Some(reason.to_string());
    Ok(revoked)
}

fn get_exemption_audit_log(_exemption: &Exemption) -> Result<AuditLog, String> {
    Ok(AuditLog {
        events: vec![
            AuditEvent { event_type: "exemption_created".to_string() },
        ],
    })
}

fn run_exemption_ci_check(exemptions: &[Exemption]) -> CICheckResult {
    let expired: Vec<_> = exemptions.iter().filter(|e| is_expired(e)).collect();
    let expiring_soon: Vec<_> = exemptions.iter().filter(|e| is_expiring_soon(e, 7)).collect();
    
    CICheckResult {
        passed: expired.is_empty(),
        expired_count: expired.len(),
        expiring_soon_count: expiring_soon.len(),
        warnings: expiring_soon.iter().map(|e| format!("Exemption {} expiring soon", e.rule_id)).collect(),
        exit_code: if expired.is_empty() { 0 } else { 1 },
    }
}

fn import_exemptions_from_yaml(yaml: &str) -> Result<Vec<Exemption>, String> {
    serde_yaml::from_str::<ExemptionsFile>(yaml)
        .map(|f| f.exemptions)
        .map_err(|e| e.to_string())
}

// Type definitions

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Exemption {
    rule_id: String,
    resource_pattern: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    reason: String,
    approved_by: String,
    approval_reference: String,
    status: String,
    revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    revocation_reason: Option<String>,
}

struct ExemptionBuilder {
    rule_id: Option<String>,
    resource_pattern: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    reason: Option<String>,
    approved_by: Option<String>,
    approval_reference: Option<String>,
}

impl ExemptionBuilder {
    fn new() -> Self {
        ExemptionBuilder {
            rule_id: None,
            resource_pattern: None,
            expires_at: None,
            reason: None,
            approved_by: None,
            approval_reference: None,
        }
    }
    
    fn rule_id(mut self, rule_id: &str) -> Self {
        self.rule_id = Some(rule_id.to_string());
        self
    }
    
    fn resource_pattern(mut self, pattern: &str) -> Self {
        self.resource_pattern = Some(pattern.to_string());
        self
    }
    
    fn expires_at(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
    
    fn approved_by(mut self, approved_by: &str) -> Self {
        self.approved_by = Some(approved_by.to_string());
        self
    }
    
    fn approval_reference(mut self, reference: &str) -> Self {
        self.approval_reference = Some(reference.to_string());
        self
    }
    
    fn build(self) -> Result<Exemption, String> {
        Ok(Exemption {
            rule_id: self.rule_id.ok_or("rule_id required")?,
            resource_pattern: self.resource_pattern.ok_or("resource_pattern required")?,
            expires_at: self.expires_at.ok_or("expires_at required")?,
            reason: self.reason.ok_or("reason required")?,
            approved_by: self.approved_by.ok_or("approved_by required")?,
            approval_reference: self.approval_reference.ok_or("approval_reference required")?,
            status: "active".to_string(),
            revoked_at: None,
            revocation_reason: None,
        })
    }
}

struct Violation {
    rule_id: String,
    resource_address: String,
}

#[derive(Clone)]
struct ExemptionRequest {
    rule_id: String,
    resource_pattern: String,
    reason: String,
    requested_by: String,
    approval_reference: String,
    required_approvals: usize,
    approvals: Vec<Approval>,
    status: String,
}

#[derive(Clone)]
struct Approval {
    approver: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    reference: String,
}

struct AuditLog {
    events: Vec<AuditEvent>,
}

struct AuditEvent {
    event_type: String,
}

struct CICheckResult {
    passed: bool,
    expired_count: usize,
    expiring_soon_count: usize,
    warnings: Vec<String>,
    exit_code: i32,
}

#[derive(serde::Deserialize)]
struct ExemptionsFile {
    exemptions: Vec<Exemption>,
}
