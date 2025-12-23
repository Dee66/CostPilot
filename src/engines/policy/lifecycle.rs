// Policy lifecycle state machine and transitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Policy lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyState {
    /// Draft - being written/edited
    Draft,
    /// Pending review - submitted for approval
    Review,
    /// Approved - ready to activate
    Approved,
    /// Active - currently enforced
    Active,
    /// Deprecated - marked for removal, still active
    Deprecated,
    /// Archived - no longer active
    Archived,
}

impl PolicyState {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PolicyState::Draft => "Policy is being drafted or edited",
            PolicyState::Review => "Policy is pending approval from reviewers",
            PolicyState::Approved => "Policy has been approved and is ready to activate",
            PolicyState::Active => "Policy is currently enforced in production",
            PolicyState::Deprecated => "Policy is marked for removal but still active",
            PolicyState::Archived => "Policy is no longer active and archived",
        }
    }

    /// Check if state allows editing
    pub fn is_editable(&self) -> bool {
        matches!(self, PolicyState::Draft)
    }

    /// Check if state is enforceable
    pub fn is_enforceable(&self) -> bool {
        matches!(self, PolicyState::Active | PolicyState::Deprecated)
    }

    /// Check if state requires approval
    pub fn requires_approval(&self) -> bool {
        matches!(self, PolicyState::Review)
    }

    /// Get valid next states from current state
    pub fn valid_transitions(&self) -> Vec<PolicyState> {
        match self {
            PolicyState::Draft => vec![PolicyState::Review, PolicyState::Archived],
            PolicyState::Review => vec![
                PolicyState::Draft,
                PolicyState::Approved,
                PolicyState::Archived,
            ],
            PolicyState::Approved => vec![PolicyState::Active, PolicyState::Archived],
            PolicyState::Active => vec![PolicyState::Deprecated, PolicyState::Archived],
            PolicyState::Deprecated => vec![PolicyState::Archived, PolicyState::Active],
            PolicyState::Archived => vec![], // No transitions from archived
        }
    }

    /// Check if transition is valid
    pub fn can_transition_to(&self, target: PolicyState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

/// Policy lifecycle manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLifecycle {
    /// Policy ID
    pub policy_id: String,

    /// Current state
    pub current_state: PolicyState,

    /// State history
    pub state_history: Vec<StateTransition>,

    /// Approval requirements
    pub approval_config: ApprovalConfig,

    /// Current approvals (only valid in Review state)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pending_approvals: Vec<ApprovalRequest>,
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// From state
    pub from_state: PolicyState,

    /// To state
    pub to_state: PolicyState,

    /// Who performed the transition
    pub actor: String,

    /// When the transition occurred
    pub timestamp: String,

    /// Reason for transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Approval IDs if transition required approvals
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub approval_ids: Vec<String>,
}

/// Approval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalConfig {
    /// Minimum number of approvals required
    pub min_approvals: usize,

    /// Required approver roles
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required_roles: Vec<String>,

    /// Allowed approvers (emails or IDs)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_approvers: Vec<String>,

    /// Auto-approve from these roles
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub auto_approve_roles: Vec<String>,

    /// Review expiration (days)
    #[serde(default = "default_review_expiration")]
    pub review_expiration_days: u32,
}

fn default_review_expiration() -> u32 {
    7 // 7 days default
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            min_approvals: 1,
            required_roles: vec!["policy-approver".to_string()],
            allowed_approvers: Vec::new(),
            auto_approve_roles: Vec::new(),
            review_expiration_days: 7,
        }
    }
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique approval ID
    pub id: String,

    /// Approver identifier
    pub approver: String,

    /// Approver role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Approval status
    pub status: ApprovalStatus,

    /// When requested
    pub requested_at: String,

    /// When responded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responded_at: Option<String>,

    /// Comment from approver
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    /// Pending approval
    Pending,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Expired (no response within time limit)
    Expired,
}

impl PolicyLifecycle {
    /// Create new policy lifecycle in Draft state
    pub fn new(policy_id: String) -> Self {
        Self {
            policy_id,
            current_state: PolicyState::Draft,
            state_history: Vec::new(),
            approval_config: ApprovalConfig::default(),
            pending_approvals: Vec::new(),
        }
    }

    /// Create with custom approval config
    pub fn with_approval_config(policy_id: String, config: ApprovalConfig) -> Self {
        Self {
            policy_id,
            current_state: PolicyState::Draft,
            state_history: Vec::new(),
            approval_config: config,
            pending_approvals: Vec::new(),
        }
    }

    /// Transition to a new state
    pub fn transition(
        &mut self,
        target_state: PolicyState,
        actor: String,
        reason: Option<String>,
    ) -> Result<(), LifecycleError> {
        // Check if transition is valid
        if !self.current_state.can_transition_to(target_state) {
            return Err(LifecycleError::InvalidTransition {
                from: self.current_state,
                to: target_state,
            });
        }

        // Check approval requirements for certain transitions
        if target_state == PolicyState::Approved
            && self.current_state == PolicyState::Review
            && !self.has_sufficient_approvals()
        {
            return Err(LifecycleError::InsufficientApprovals {
                required: self.approval_config.min_approvals,
                received: self.count_approvals(),
            });
        }

        // Record transition
        let transition = StateTransition {
            from_state: self.current_state,
            to_state: target_state,
            actor,
            timestamp: Utc::now().to_rfc3339(),
            reason,
            approval_ids: self.get_approval_ids(),
        };

        self.state_history.push(transition);
        self.current_state = target_state;

        // Clear pending approvals if leaving Review state
        if target_state != PolicyState::Review {
            self.pending_approvals.clear();
        }

        Ok(())
    }

    /// Submit policy for review
    pub fn submit_for_review(
        &mut self,
        actor: String,
        approvers: Vec<String>,
    ) -> Result<(), LifecycleError> {
        if self.current_state != PolicyState::Draft {
            return Err(LifecycleError::InvalidState {
                operation: "submit_for_review".to_string(),
                current: self.current_state,
            });
        }

        // Create approval requests
        self.pending_approvals = approvers
            .into_iter()
            .enumerate()
            .map(|(i, approver)| ApprovalRequest {
                id: format!("{}-approval-{}", self.policy_id, i),
                approver,
                role: None,
                status: ApprovalStatus::Pending,
                requested_at: Utc::now().to_rfc3339(),
                responded_at: None,
                comment: None,
            })
            .collect();

        self.transition(
            PolicyState::Review,
            actor,
            Some("Submitted for approval".to_string()),
        )
    }

    /// Record an approval
    pub fn record_approval(
        &mut self,
        approver: String,
        approved: bool,
        comment: Option<String>,
    ) -> Result<(), LifecycleError> {
        if self.current_state != PolicyState::Review {
            return Err(LifecycleError::InvalidState {
                operation: "record_approval".to_string(),
                current: self.current_state,
            });
        }

        // Find pending approval for this approver
        let approval = self
            .pending_approvals
            .iter_mut()
            .find(|a| a.approver == approver && a.status == ApprovalStatus::Pending)
            .ok_or_else(|| LifecycleError::ApprovalNotFound {
                approver: approver.clone(),
            })?;

        approval.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        approval.responded_at = Some(Utc::now().to_rfc3339());
        approval.comment = comment;

        Ok(())
    }

    /// Check if policy has sufficient approvals
    pub fn has_sufficient_approvals(&self) -> bool {
        self.count_approvals() >= self.approval_config.min_approvals
    }

    /// Count approved approvals
    pub fn count_approvals(&self) -> usize {
        self.pending_approvals
            .iter()
            .filter(|a| a.status == ApprovalStatus::Approved)
            .count()
    }

    /// Check if any approval was rejected
    pub fn has_rejections(&self) -> bool {
        self.pending_approvals
            .iter()
            .any(|a| a.status == ApprovalStatus::Rejected)
    }

    /// Get approval IDs for transition record
    fn get_approval_ids(&self) -> Vec<String> {
        self.pending_approvals
            .iter()
            .filter(|a| a.status == ApprovalStatus::Approved)
            .map(|a| a.id.clone())
            .collect()
    }

    /// Check if review has expired
    pub fn is_review_expired(&self) -> bool {
        if self.current_state != PolicyState::Review {
            return false;
        }

        if let Some(last_transition) = self.state_history.last() {
            if let Ok(requested) = DateTime::parse_from_rfc3339(&last_transition.timestamp) {
                let now = Utc::now();
                let days_elapsed =
                    (now.signed_duration_since(requested.with_timezone(&Utc))).num_days();
                return days_elapsed > self.approval_config.review_expiration_days as i64;
            }
        }

        false
    }

    /// Activate an approved policy
    pub fn activate(&mut self, actor: String) -> Result<(), LifecycleError> {
        if self.current_state != PolicyState::Approved {
            return Err(LifecycleError::InvalidState {
                operation: "activate".to_string(),
                current: self.current_state,
            });
        }

        self.transition(
            PolicyState::Active,
            actor,
            Some("Policy activated".to_string()),
        )
    }

    /// Deprecate an active policy
    pub fn deprecate(&mut self, actor: String, reason: String) -> Result<(), LifecycleError> {
        if self.current_state != PolicyState::Active {
            return Err(LifecycleError::InvalidState {
                operation: "deprecate".to_string(),
                current: self.current_state,
            });
        }

        self.transition(PolicyState::Deprecated, actor, Some(reason))
    }

    /// Archive a policy
    pub fn archive(&mut self, actor: String, reason: String) -> Result<(), LifecycleError> {
        self.transition(PolicyState::Archived, actor, Some(reason))
    }

    /// Get lifecycle summary
    pub fn summary(&self) -> LifecycleSummary {
        LifecycleSummary {
            policy_id: self.policy_id.clone(),
            current_state: self.current_state,
            state_description: self.current_state.description().to_string(),
            is_editable: self.current_state.is_editable(),
            is_enforceable: self.current_state.is_enforceable(),
            requires_approval: self.current_state.requires_approval(),
            approvals_received: self.count_approvals(),
            approvals_required: self.approval_config.min_approvals,
            has_rejections: self.has_rejections(),
            is_expired: self.is_review_expired(),
            transition_count: self.state_history.len(),
            last_transition: self.state_history.last().map(|t| t.timestamp.clone()),
        }
    }
}

/// Lifecycle summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSummary {
    pub policy_id: String,
    pub current_state: PolicyState,
    pub state_description: String,
    pub is_editable: bool,
    pub is_enforceable: bool,
    pub requires_approval: bool,
    pub approvals_received: usize,
    pub approvals_required: usize,
    pub has_rejections: bool,
    pub is_expired: bool,
    pub transition_count: usize,
    pub last_transition: Option<String>,
}

/// Lifecycle errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum LifecycleError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: PolicyState, to: PolicyState },

    #[error("Operation '{operation}' not allowed in state {current:?}")]
    InvalidState {
        operation: String,
        current: PolicyState,
    },

    #[error("Insufficient approvals: required {required}, received {received}")]
    InsufficientApprovals { required: usize, received: usize },

    #[error("Approval not found for approver: {approver}")]
    ApprovalNotFound { approver: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_state_transitions() {
        assert!(PolicyState::Draft.can_transition_to(PolicyState::Review));
        assert!(PolicyState::Review.can_transition_to(PolicyState::Approved));
        assert!(PolicyState::Approved.can_transition_to(PolicyState::Active));
        assert!(PolicyState::Active.can_transition_to(PolicyState::Deprecated));

        // Invalid transitions
        assert!(!PolicyState::Draft.can_transition_to(PolicyState::Active));
        assert!(!PolicyState::Archived.can_transition_to(PolicyState::Active));
    }

    #[test]
    fn test_lifecycle_creation() {
        let lifecycle = PolicyLifecycle::new("test-policy".to_string());
        assert_eq!(lifecycle.current_state, PolicyState::Draft);
        assert!(lifecycle.state_history.is_empty());
    }

    #[test]
    fn test_submit_for_review() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        let approvers = vec![
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        ];

        let result = lifecycle.submit_for_review("author@example.com".to_string(), approvers);
        assert!(result.is_ok());
        assert_eq!(lifecycle.current_state, PolicyState::Review);
        assert_eq!(lifecycle.pending_approvals.len(), 2);
    }

    #[test]
    fn test_record_approval() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        lifecycle.approval_config.min_approvals = 2;

        let approvers = vec![
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        ];
        lifecycle
            .submit_for_review("author@example.com".to_string(), approvers)
            .unwrap();

        // First approval
        lifecycle
            .record_approval(
                "alice@example.com".to_string(),
                true,
                Some("Looks good".to_string()),
            )
            .unwrap();
        assert_eq!(lifecycle.count_approvals(), 1);
        assert!(!lifecycle.has_sufficient_approvals());

        // Second approval
        lifecycle
            .record_approval(
                "bob@example.com".to_string(),
                true,
                Some("Approved".to_string()),
            )
            .unwrap();
        assert_eq!(lifecycle.count_approvals(), 2);
        assert!(lifecycle.has_sufficient_approvals());
    }

    #[test]
    fn test_approval_rejection() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        let approvers = vec!["alice@example.com".to_string()];
        lifecycle
            .submit_for_review("author@example.com".to_string(), approvers)
            .unwrap();

        lifecycle
            .record_approval(
                "alice@example.com".to_string(),
                false,
                Some("Needs changes".to_string()),
            )
            .unwrap();

        assert!(lifecycle.has_rejections());
        assert!(!lifecycle.has_sufficient_approvals());
    }

    #[test]
    fn test_full_lifecycle() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        lifecycle.approval_config.min_approvals = 1;

        // Draft -> Review
        lifecycle
            .submit_for_review(
                "author@example.com".to_string(),
                vec!["approver@example.com".to_string()],
            )
            .unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Review);

        // Record approval
        lifecycle
            .record_approval("approver@example.com".to_string(), true, None)
            .unwrap();

        // Review -> Approved
        lifecycle
            .transition(
                PolicyState::Approved,
                "approver@example.com".to_string(),
                Some("Policy approved".to_string()),
            )
            .unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Approved);

        // Approved -> Active
        lifecycle.activate("admin@example.com".to_string()).unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Active);

        // Active -> Deprecated
        lifecycle
            .deprecate(
                "admin@example.com".to_string(),
                "Replaced by v2".to_string(),
            )
            .unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Deprecated);

        // Deprecated -> Archived
        lifecycle
            .archive("admin@example.com".to_string(), "End of life".to_string())
            .unwrap();
        assert_eq!(lifecycle.current_state, PolicyState::Archived);

        // Verify history
        assert_eq!(lifecycle.state_history.len(), 5);
    }

    #[test]
    fn test_insufficient_approvals_blocks_approval() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        lifecycle.approval_config.min_approvals = 2;

        lifecycle
            .submit_for_review(
                "author@example.com".to_string(),
                vec!["alice@example.com".to_string()],
            )
            .unwrap();

        lifecycle
            .record_approval("alice@example.com".to_string(), true, None)
            .unwrap();

        // Try to approve without sufficient approvals
        let result =
            lifecycle.transition(PolicyState::Approved, "alice@example.com".to_string(), None);

        assert!(result.is_err());
        assert_eq!(lifecycle.current_state, PolicyState::Review);
    }

    #[test]
    fn test_lifecycle_summary() {
        let mut lifecycle = PolicyLifecycle::new("test-policy".to_string());
        lifecycle
            .submit_for_review(
                "author@example.com".to_string(),
                vec!["approver@example.com".to_string()],
            )
            .unwrap();

        let summary = lifecycle.summary();
        assert_eq!(summary.current_state, PolicyState::Review);
        assert!(summary.requires_approval);
        assert!(!summary.is_editable);
        assert!(!summary.is_enforceable);
        assert_eq!(summary.approvals_required, 1);
        assert_eq!(summary.approvals_received, 0);
    }
}
