// Approval workflow manager for policy lifecycle

use super::lifecycle::{ApprovalConfig, ApprovalStatus, PolicyLifecycle, PolicyState};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Approval reference required for flagged policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReference {
    /// Unique approval reference ID (e.g., "APPR-2024-001")
    pub reference_id: String,

    /// Policy ID requiring approval
    pub policy_id: String,

    /// Approver who granted approval
    pub approver: String,

    /// Timestamp when approval was granted
    pub approved_at: String,

    /// Optional approval comment/justification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Expiration timestamp (if temporary approval)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Manages approval workflows across multiple policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflowManager {
    /// Active workflows by policy ID
    workflows: HashMap<String, PolicyLifecycle>,

    /// Global approval configuration
    default_config: ApprovalConfig,

    /// Role-based approver assignments
    role_assignments: HashMap<String, Vec<String>>,

    /// Stored approval references
    approval_references: HashMap<String, ApprovalReference>,
}

impl ApprovalWorkflowManager {
    /// Create new workflow manager
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            default_config: ApprovalConfig::default(),
            role_assignments: HashMap::new(),
            approval_references: HashMap::new(),
        }
    }

    /// Create with custom default config
    pub fn with_config(config: ApprovalConfig) -> Self {
        Self {
            workflows: HashMap::new(),
            default_config: config,
            role_assignments: HashMap::new(),
            approval_references: HashMap::new(),
        }
    }

    /// Register a new policy workflow
    pub fn register_policy(
        &mut self,
        policy_id: String,
        approval_config: Option<ApprovalConfig>,
    ) -> Result<(), WorkflowError> {
        if self.workflows.contains_key(&policy_id) {
            return Err(WorkflowError::PolicyAlreadyExists { policy_id });
        }

        let config = approval_config.unwrap_or_else(|| self.default_config.clone());
        let lifecycle = PolicyLifecycle::with_approval_config(policy_id.clone(), config);
        self.workflows.insert(policy_id, lifecycle);

        Ok(())
    }

    /// Get policy lifecycle
    pub fn get_lifecycle(&self, policy_id: &str) -> Option<&PolicyLifecycle> {
        self.workflows.get(policy_id)
    }

    /// Get mutable policy lifecycle
    pub fn get_lifecycle_mut(&mut self, policy_id: &str) -> Option<&mut PolicyLifecycle> {
        self.workflows.get_mut(policy_id)
    }

    /// Submit policy for approval
    pub fn submit_for_approval(
        &mut self,
        policy_id: &str,
        submitter: String,
    ) -> Result<Vec<String>, WorkflowError> {
        // Get required approvers based on policy's approval config
        let approvers = {
            let lifecycle =
                self.workflows
                    .get(policy_id)
                    .ok_or_else(|| WorkflowError::PolicyNotFound {
                        policy_id: policy_id.to_string(),
                    })?;
            self.resolve_approvers(&lifecycle.approval_config)?
        };

        // Now mutate the lifecycle
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .submit_for_review(submitter, approvers.clone())
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })?;

        Ok(approvers)
    }

    /// Approve a policy with approval reference
    pub fn approve(
        &mut self,
        policy_id: &str,
        approver: String,
        approval_reference: String,
        comment: Option<String>,
    ) -> Result<ApprovalResult, WorkflowError> {
        // Validate approval reference format
        Self::validate_approval_reference(&approval_reference)?;

        // Verify approver is authorized (read-only check)
        {
            let lifecycle =
                self.workflows
                    .get(policy_id)
                    .ok_or_else(|| WorkflowError::PolicyNotFound {
                        policy_id: policy_id.to_string(),
                    })?;

            if !self.is_authorized_approver(&approver, &lifecycle.approval_config) {
                return Err(WorkflowError::UnauthorizedApprover {
                    approver: approver.clone(),
                });
            }
        }

        // Now mutate the lifecycle
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .record_approval(approver.clone(), true, comment.clone())
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })?;

        // Store approval reference
        let reference = ApprovalReference {
            reference_id: approval_reference.clone(),
            policy_id: policy_id.to_string(),
            approver: approver.clone(),
            approved_at: Utc::now().to_rfc3339(),
            comment,
            expires_at: None,
        };
        self.approval_references
            .insert(approval_reference, reference);

        // Check if we have sufficient approvals to auto-transition
        let sufficient = lifecycle.has_sufficient_approvals();
        let result = ApprovalResult {
            approved: true,
            approver,
            sufficient_approvals: sufficient,
            remaining_approvals: lifecycle
                .approval_config
                .min_approvals
                .saturating_sub(lifecycle.count_approvals()),
        };

        Ok(result)
    }

    /// Validate approval reference format
    /// Accepts: JIRA tickets (PROJ-123), GitHub PR (#456), ServiceNow (INC0012345), custom (APPR-2024-001)
    fn validate_approval_reference(reference: &str) -> Result<(), WorkflowError> {
        if reference.trim().is_empty() {
            return Err(WorkflowError::MissingApprovalReference);
        }

        // Check for valid patterns
        let valid = reference.contains('-') // PROJ-123, APPR-2024-001, INC0012345
            || reference.starts_with('#') // GitHub PR #456
            || reference.len() >= 5; // Minimum length for custom refs

        if !valid {
            return Err(WorkflowError::InvalidApprovalReference {
                reference: reference.to_string(),
            });
        }

        Ok(())
    }

    /// Get approval reference by ID
    pub fn get_approval_reference(&self, reference_id: &str) -> Option<&ApprovalReference> {
        self.approval_references.get(reference_id)
    }

    /// List approval references for a policy
    pub fn list_policy_references(&self, policy_id: &str) -> Vec<&ApprovalReference> {
        self.approval_references
            .values()
            .filter(|r| r.policy_id == policy_id)
            .collect()
    }

    /// Reject a policy
    pub fn reject(
        &mut self,
        policy_id: &str,
        approver: String,
        reason: String,
    ) -> Result<ApprovalResult, WorkflowError> {
        // Verify approver is authorized (read-only check)
        {
            let lifecycle =
                self.workflows
                    .get(policy_id)
                    .ok_or_else(|| WorkflowError::PolicyNotFound {
                        policy_id: policy_id.to_string(),
                    })?;

            if !self.is_authorized_approver(&approver, &lifecycle.approval_config) {
                return Err(WorkflowError::UnauthorizedApprover {
                    approver: approver.clone(),
                });
            }
        }

        // Now mutate the lifecycle
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .record_approval(approver.clone(), false, Some(reason))
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })?;

        let result = ApprovalResult {
            approved: false,
            approver,
            sufficient_approvals: false,
            remaining_approvals: 0,
        };

        Ok(result)
    }

    /// Activate an approved policy
    pub fn activate_policy(&mut self, policy_id: &str, actor: String) -> Result<(), WorkflowError> {
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .activate(actor)
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })
    }

    /// Deprecate an active policy
    pub fn deprecate_policy(
        &mut self,
        policy_id: &str,
        actor: String,
        reason: String,
    ) -> Result<(), WorkflowError> {
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .deprecate(actor, reason)
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })
    }

    /// Archive a policy
    pub fn archive_policy(
        &mut self,
        policy_id: &str,
        actor: String,
        reason: String,
    ) -> Result<(), WorkflowError> {
        let lifecycle =
            self.workflows
                .get_mut(policy_id)
                .ok_or_else(|| WorkflowError::PolicyNotFound {
                    policy_id: policy_id.to_string(),
                })?;

        lifecycle
            .archive(actor, reason)
            .map_err(|e| WorkflowError::LifecycleError {
                policy_id: policy_id.to_string(),
                error: e.to_string(),
            })
    }

    /// Assign approvers to a role
    pub fn assign_role(&mut self, role: String, approvers: Vec<String>) {
        self.role_assignments.insert(role, approvers);
    }

    /// Resolve approvers from approval config
    fn resolve_approvers(&self, config: &ApprovalConfig) -> Result<Vec<String>, WorkflowError> {
        let mut approvers = Vec::new();

        // Add explicitly allowed approvers
        approvers.extend(config.allowed_approvers.clone());

        // Add approvers from required roles
        for role in &config.required_roles {
            if let Some(role_approvers) = self.role_assignments.get(role) {
                approvers.extend(role_approvers.clone());
            } else {
                return Err(WorkflowError::RoleNotFound { role: role.clone() });
            }
        }

        if approvers.is_empty() {
            return Err(WorkflowError::NoApproversFound);
        }

        // Deduplicate
        approvers.sort();
        approvers.dedup();

        Ok(approvers)
    }

    /// Check if approver is authorized
    fn is_authorized_approver(&self, approver: &str, config: &ApprovalConfig) -> bool {
        // Check explicit list
        if config.allowed_approvers.contains(&approver.to_string()) {
            return true;
        }

        // Check role assignments
        for role in &config.required_roles {
            if let Some(role_approvers) = self.role_assignments.get(role) {
                if role_approvers.contains(&approver.to_string()) {
                    return true;
                }
            }
        }

        false
    }

    /// Get all policies in a specific state
    pub fn get_policies_by_state(&self, state: PolicyState) -> Vec<String> {
        self.workflows
            .iter()
            .filter(|(_, lifecycle)| lifecycle.current_state == state)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all policies pending approval
    pub fn get_pending_approvals(&self) -> Vec<PendingApproval> {
        self.workflows
            .iter()
            .filter(|(_, lifecycle)| lifecycle.current_state == PolicyState::Review)
            .map(|(id, lifecycle)| PendingApproval {
                policy_id: id.clone(),
                approvals_received: lifecycle.count_approvals(),
                approvals_required: lifecycle.approval_config.min_approvals,
                has_rejections: lifecycle.has_rejections(),
                is_expired: lifecycle.is_review_expired(),
                pending_approvers: lifecycle
                    .pending_approvals
                    .iter()
                    .filter(|a| a.status == ApprovalStatus::Pending)
                    .map(|a| a.approver.clone())
                    .collect(),
            })
            .collect()
    }

    /// Get approvals pending for a specific approver
    pub fn get_approvals_for_approver(&self, approver: &str) -> Vec<String> {
        self.workflows
            .iter()
            .filter(|(_, lifecycle)| {
                lifecycle.current_state == PolicyState::Review
                    && lifecycle
                        .pending_approvals
                        .iter()
                        .any(|a| a.approver == approver && a.status == ApprovalStatus::Pending)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check and mark expired approvals
    pub fn process_expired_approvals(&mut self) -> Vec<String> {
        let mut expired = Vec::new();

        for (policy_id, lifecycle) in self.workflows.iter_mut() {
            if lifecycle.is_review_expired() {
                // Mark all pending approvals as expired
                for approval in lifecycle.pending_approvals.iter_mut() {
                    if approval.status == ApprovalStatus::Pending {
                        approval.status = ApprovalStatus::Expired;
                    }
                }
                expired.push(policy_id.clone());
            }
        }

        expired
    }

    /// Get workflow statistics
    pub fn get_statistics(&self) -> WorkflowStatistics {
        let mut stats = WorkflowStatistics::default();

        for lifecycle in self.workflows.values() {
            stats.total_policies += 1;

            match lifecycle.current_state {
                PolicyState::Draft => stats.draft_count += 1,
                PolicyState::Review => stats.review_count += 1,
                PolicyState::Approved => stats.approved_count += 1,
                PolicyState::Active => stats.active_count += 1,
                PolicyState::Deprecated => stats.deprecated_count += 1,
                PolicyState::Archived => stats.archived_count += 1,
            }

            if lifecycle.is_review_expired() {
                stats.expired_reviews += 1;
            }
        }

        stats
    }
}

impl Default for ApprovalWorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of an approval action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    /// Whether approved or rejected
    pub approved: bool,

    /// Who performed the action
    pub approver: String,

    /// Whether sufficient approvals have been collected
    pub sufficient_approvals: bool,

    /// How many more approvals are needed
    pub remaining_approvals: usize,
}

/// Pending approval info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub policy_id: String,
    pub approvals_received: usize,
    pub approvals_required: usize,
    pub has_rejections: bool,
    pub is_expired: bool,
    pub pending_approvers: Vec<String>,
}

/// Workflow statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowStatistics {
    pub total_policies: usize,
    pub draft_count: usize,
    pub review_count: usize,
    pub approved_count: usize,
    pub active_count: usize,
    pub deprecated_count: usize,
    pub archived_count: usize,
    pub expired_reviews: usize,
}

/// Workflow errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum WorkflowError {
    #[error("Policy not found: {policy_id}")]
    PolicyNotFound { policy_id: String },

    #[error("Policy already exists: {policy_id}")]
    PolicyAlreadyExists { policy_id: String },

    #[error("Role not found: {role}")]
    RoleNotFound { role: String },

    #[error("No approvers found for policy")]
    NoApproversFound,

    #[error("Approver not authorized: {approver}")]
    UnauthorizedApprover { approver: String },

    #[error("Approval reference is required but missing")]
    MissingApprovalReference,

    #[error("Invalid approval reference format: {reference}")]
    InvalidApprovalReference { reference: String },

    #[error("Lifecycle error for policy {policy_id}: {error}")]
    LifecycleError { policy_id: String, error: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> ApprovalWorkflowManager {
        let mut manager = ApprovalWorkflowManager::new();

        // Setup roles
        manager.assign_role(
            "policy-approver".to_string(),
            vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
        );

        manager
    }

    #[test]
    fn test_register_policy() {
        let mut manager = create_test_manager();

        let result = manager.register_policy("test-policy".to_string(), None);
        assert!(result.is_ok());

        let lifecycle = manager.get_lifecycle("test-policy");
        assert!(lifecycle.is_some());
        assert_eq!(lifecycle.unwrap().current_state, PolicyState::Draft);
    }

    #[test]
    fn test_submit_for_approval() {
        let mut manager = create_test_manager();
        manager
            .register_policy("test-policy".to_string(), None)
            .unwrap();

        let approvers = manager
            .submit_for_approval("test-policy", "author@example.com".to_string())
            .unwrap();

        assert_eq!(approvers.len(), 2);
        assert!(approvers.contains(&"alice@example.com".to_string()));

        let lifecycle = manager.get_lifecycle("test-policy").unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Review);
    }

    #[test]
    fn test_approve_workflow() {
        let mut manager = create_test_manager();
        manager
            .register_policy("test-policy".to_string(), None)
            .unwrap();
        manager
            .submit_for_approval("test-policy", "author@example.com".to_string())
            .unwrap();

        let result = manager
            .approve(
                "test-policy",
                "alice@example.com".to_string(),
                "APPR-2024-001".to_string(),
                Some("Looks good".to_string()),
            )
            .unwrap();

        assert!(result.approved);
        assert!(result.sufficient_approvals);
        assert_eq!(result.remaining_approvals, 0);

        // Verify approval reference was stored
        let reference = manager.get_approval_reference("APPR-2024-001").unwrap();
        assert_eq!(reference.policy_id, "test-policy");
        assert_eq!(reference.approver, "alice@example.com");
    }

    #[test]
    fn test_reject_workflow() {
        let mut manager = create_test_manager();
        manager
            .register_policy("test-policy".to_string(), None)
            .unwrap();
        manager
            .submit_for_approval("test-policy", "author@example.com".to_string())
            .unwrap();

        let result = manager
            .reject(
                "test-policy",
                "alice@example.com".to_string(),
                "Needs changes".to_string(),
            )
            .unwrap();

        assert!(!result.approved);
        assert!(!result.sufficient_approvals);
    }

    #[test]
    fn test_unauthorized_approver() {
        let mut manager = create_test_manager();
        manager
            .register_policy("test-policy".to_string(), None)
            .unwrap();
        manager
            .submit_for_approval("test-policy", "author@example.com".to_string())
            .unwrap();

        let result = manager.approve(
            "test-policy",
            "unauthorized@example.com".to_string(),
            "REF-001".to_string(),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_policies_by_state() {
        let mut manager = create_test_manager();
        manager
            .register_policy("policy-1".to_string(), None)
            .unwrap();
        manager
            .register_policy("policy-2".to_string(), None)
            .unwrap();
        manager
            .submit_for_approval("policy-1", "author@example.com".to_string())
            .unwrap();

        let drafts = manager.get_policies_by_state(PolicyState::Draft);
        assert_eq!(drafts.len(), 1);
        assert!(drafts.contains(&"policy-2".to_string()));

        let reviews = manager.get_policies_by_state(PolicyState::Review);
        assert_eq!(reviews.len(), 1);
        assert!(reviews.contains(&"policy-1".to_string()));
    }

    #[test]
    fn test_get_pending_approvals() {
        let mut manager = create_test_manager();
        manager
            .register_policy("policy-1".to_string(), None)
            .unwrap();
        manager
            .register_policy("policy-2".to_string(), None)
            .unwrap();

        manager
            .submit_for_approval("policy-1", "author@example.com".to_string())
            .unwrap();
        manager
            .submit_for_approval("policy-2", "author@example.com".to_string())
            .unwrap();

        let pending = manager.get_pending_approvals();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_get_approvals_for_approver() {
        let mut manager = create_test_manager();
        manager
            .register_policy("policy-1".to_string(), None)
            .unwrap();
        manager
            .register_policy("policy-2".to_string(), None)
            .unwrap();

        manager
            .submit_for_approval("policy-1", "author@example.com".to_string())
            .unwrap();
        manager
            .submit_for_approval("policy-2", "author@example.com".to_string())
            .unwrap();

        let approvals = manager.get_approvals_for_approver("alice@example.com");
        assert_eq!(approvals.len(), 2);
    }

    #[test]
    fn test_workflow_statistics() {
        let mut manager = create_test_manager();
        manager
            .register_policy("policy-1".to_string(), None)
            .unwrap();
        manager
            .register_policy("policy-2".to_string(), None)
            .unwrap();
        manager
            .submit_for_approval("policy-1", "author@example.com".to_string())
            .unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_policies, 2);
        assert_eq!(stats.draft_count, 1);
        assert_eq!(stats.review_count, 1);
    }

    #[test]
    fn test_full_approval_workflow() {
        let mut manager = create_test_manager();
        manager
            .register_policy("test-policy".to_string(), None)
            .unwrap();

        // Submit
        manager
            .submit_for_approval("test-policy", "author@example.com".to_string())
            .unwrap();

        // Approve with reference
        manager
            .approve(
                "test-policy",
                "alice@example.com".to_string(),
                "PROJ-456".to_string(),
                None,
            )
            .unwrap();

        // Transition to approved (would normally be done after sufficient approvals)
        let lifecycle = manager.get_lifecycle_mut("test-policy").unwrap();
        lifecycle
            .transition(PolicyState::Approved, "alice@example.com".to_string(), None)
            .unwrap();

        // Activate
        manager
            .activate_policy("test-policy", "admin@example.com".to_string())
            .unwrap();

        let lifecycle = manager.get_lifecycle("test-policy").unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Active);
    }
}
