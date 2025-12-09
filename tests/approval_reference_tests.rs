// Tests for approval reference enforcement

use costpilot::engines::policy::approval_workflow::{
    ApprovalWorkflowManager, WorkflowError,
};
use costpilot::engines::policy::lifecycle::ApprovalConfig;

fn create_test_manager() -> ApprovalWorkflowManager {
    let config = ApprovalConfig {
        min_approvals: 1,
        required_approver_roles: vec![],
        approval_timeout_hours: 48,
        auto_approve_minor: false,
    };
    
    let mut manager = ApprovalWorkflowManager::with_config(config);
    manager.assign_role("approver".to_string(), vec!["alice@example.com".to_string()]);
    manager
}

#[test]
fn test_approval_requires_reference() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    // Approve with valid reference
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "APPR-2024-001".to_string(),
        Some("Approved".to_string()),
    );
    
    assert!(result.is_ok());
    let approval_result = result.unwrap();
    assert!(approval_result.approved);
}

#[test]
fn test_missing_approval_reference_rejected() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    // Try to approve with empty reference
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "".to_string(),
        Some("Approved".to_string()),
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        WorkflowError::MissingApprovalReference => {},
        e => panic!("Expected MissingApprovalReference error, got {:?}", e),
    }
}

#[test]
fn test_invalid_approval_reference_rejected() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    // Try to approve with invalid reference (too short, no dash/hash)
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "xyz".to_string(),
        Some("Approved".to_string()),
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        WorkflowError::InvalidApprovalReference { reference } => {
            assert_eq!(reference, "xyz");
        },
        e => panic!("Expected InvalidApprovalReference error, got {:?}", e),
    }
}

#[test]
fn test_jira_ticket_format_accepted() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "PROJ-123".to_string(),
        None,
    );
    
    assert!(result.is_ok());
    let reference = manager.get_approval_reference("PROJ-123").unwrap();
    assert_eq!(reference.reference_id, "PROJ-123");
    assert_eq!(reference.policy_id, "test-policy");
}

#[test]
fn test_github_pr_format_accepted() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "#456".to_string(),
        None,
    );
    
    assert!(result.is_ok());
    let reference = manager.get_approval_reference("#456").unwrap();
    assert_eq!(reference.reference_id, "#456");
}

#[test]
fn test_servicenow_ticket_format_accepted() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "INC0012345".to_string(),
        None,
    );
    
    assert!(result.is_ok());
    let reference = manager.get_approval_reference("INC0012345").unwrap();
    assert_eq!(reference.reference_id, "INC0012345");
}

#[test]
fn test_custom_reference_format_accepted() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "APPR-2024-Q1-001".to_string(),
        None,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_approval_reference_stored_with_metadata() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "PROJ-789".to_string(),
        Some("Cost optimization approved".to_string()),
    ).unwrap();
    
    let reference = manager.get_approval_reference("PROJ-789").unwrap();
    assert_eq!(reference.reference_id, "PROJ-789");
    assert_eq!(reference.policy_id, "test-policy");
    assert_eq!(reference.approver, "alice@example.com");
    assert_eq!(reference.comment, Some("Cost optimization approved".to_string()));
    assert!(!reference.approved_at.is_empty());
}

#[test]
fn test_list_policy_references() {
    let mut manager = create_test_manager();
    manager.register_policy("policy-1".to_string(), None).unwrap();
    manager.register_policy("policy-2".to_string(), None).unwrap();
    
    manager.submit_for_approval("policy-1", "author@example.com".to_string()).unwrap();
    manager.submit_for_approval("policy-2", "author@example.com".to_string()).unwrap();
    
    manager.approve("policy-1", "alice@example.com".to_string(), "REF-001".to_string(), None).unwrap();
    manager.approve("policy-2", "alice@example.com".to_string(), "REF-002".to_string(), None).unwrap();
    
    let refs = manager.list_policy_references("policy-1");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].reference_id, "REF-001");
    
    let refs = manager.list_policy_references("policy-2");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].reference_id, "REF-002");
}

#[test]
fn test_whitespace_only_reference_rejected() {
    let mut manager = create_test_manager();
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    let result = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "   ".to_string(),
        None,
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        WorkflowError::MissingApprovalReference => {},
        e => panic!("Expected MissingApprovalReference error, got {:?}", e),
    }
}

#[test]
fn test_multiple_approvals_with_different_references() {
    let config = ApprovalConfig {
        min_approvals: 2,
        required_approver_roles: vec![],
        approval_timeout_hours: 48,
        auto_approve_minor: false,
    };
    
    let mut manager = ApprovalWorkflowManager::with_config(config);
    manager.assign_role("approver".to_string(), vec![
        "alice@example.com".to_string(),
        "bob@example.com".to_string(),
    ]);
    
    manager.register_policy("test-policy".to_string(), None).unwrap();
    manager.submit_for_approval("test-policy", "author@example.com".to_string()).unwrap();
    
    // First approval
    let result1 = manager.approve(
        "test-policy",
        "alice@example.com".to_string(),
        "APPR-001".to_string(),
        None,
    ).unwrap();
    assert!(!result1.sufficient_approvals);
    assert_eq!(result1.remaining_approvals, 1);
    
    // Second approval with different reference
    let result2 = manager.approve(
        "test-policy",
        "bob@example.com".to_string(),
        "APPR-002".to_string(),
        None,
    ).unwrap();
    assert!(result2.sufficient_approvals);
    assert_eq!(result2.remaining_approvals, 0);
    
    // Both references should be stored
    let refs = manager.list_policy_references("test-policy");
    assert_eq!(refs.len(), 2);
}

#[test]
fn test_approval_reference_required_for_flagged_policies() {
    // This test simulates the full workflow when a policy is flagged (RequireApproval action)
    let mut manager = create_test_manager();
    manager.register_policy("flagged-policy".to_string(), None).unwrap();
    manager.submit_for_approval("flagged-policy", "author@example.com".to_string()).unwrap();
    
    // Attempt approval without reference should fail
    let result = manager.approve(
        "flagged-policy",
        "alice@example.com".to_string(),
        "".to_string(),
        Some("Emergency approval".to_string()),
    );
    
    assert!(result.is_err());
    
    // With proper reference should succeed
    let result = manager.approve(
        "flagged-policy",
        "alice@example.com".to_string(),
        "EMERGENCY-001".to_string(),
        Some("Emergency approval".to_string()),
    );
    
    assert!(result.is_ok());
    let reference = manager.get_approval_reference("EMERGENCY-001").unwrap();
    assert_eq!(reference.policy_id, "flagged-policy");
}
