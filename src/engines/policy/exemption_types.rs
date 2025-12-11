use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a policy exemption with expiration and justification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyExemption {
    /// Unique identifier for the exemption
    pub id: String,

    /// The policy rule being exempted (e.g., "NAT_GATEWAY_LIMIT")
    pub policy_name: String,

    /// Resource pattern or specific resource ID this exemption applies to
    /// Supports wildcards: "module.vpc.*" or specific: "module.vpc.nat_gateway[0]"
    pub resource_pattern: String,

    /// Human-readable justification for the exemption
    pub justification: String,

    /// ISO 8601 date when exemption expires (YYYY-MM-DD)
    pub expires_at: String,

    /// Who approved this exemption (email or username)
    pub approved_by: String,

    /// ISO 8601 timestamp when exemption was created
    pub created_at: String,

    /// Optional reference to ticket/issue (e.g., JIRA-123)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_ref: Option<String>,
}

/// Result of exemption validation
#[derive(Debug, Clone, PartialEq)]
pub enum ExemptionStatus {
    /// Exemption is valid and active
    Active,

    /// Exemption has expired
    Expired { expired_on: String },

    /// Exemption is about to expire (within warning threshold)
    ExpiringSoon { expires_in_days: u32 },

    /// Exemption has invalid format or missing required fields
    Invalid { reason: String },
}

/// Configuration for exemption validation behavior
#[derive(Debug, Clone)]
pub struct ExemptionConfig {
    /// Days before expiration to start warning (default: 30)
    pub warning_threshold_days: u32,

    /// Whether to enforce expiration checks (default: true)
    pub enforce_expiration: bool,

    /// Maximum allowed exemption duration in days (default: 365)
    pub max_duration_days: u32,
}

impl Default for ExemptionConfig {
    fn default() -> Self {
        Self {
            warning_threshold_days: 30,
            enforce_expiration: true,
            max_duration_days: 365,
        }
    }
}

/// Container for all exemptions loaded from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionsFile {
    /// Schema version for future compatibility
    pub version: String,

    /// List of active exemptions
    pub exemptions: Vec<PolicyExemption>,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ExemptionMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionMetadata {
    /// Last time exemptions file was reviewed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reviewed: Option<String>,

    /// Team or organization owning these exemptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl fmt::Display for ExemptionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExemptionStatus::Active => write!(f, "Active"),
            ExemptionStatus::Expired { expired_on } => {
                write!(f, "Expired on {}", expired_on)
            }
            ExemptionStatus::ExpiringSoon { expires_in_days } => {
                write!(f, "Expiring in {} days", expires_in_days)
            }
            ExemptionStatus::Invalid { reason } => {
                write!(f, "Invalid: {}", reason)
            }
        }
    }
}

impl PolicyExemption {
    /// Check if this exemption matches a given policy and resource
    pub fn matches(&self, policy_name: &str, resource_id: &str) -> bool {
        if self.policy_name != policy_name {
            return false;
        }

        // Exact match
        if self.resource_pattern == resource_id {
            return true;
        }

        // Wildcard matching: "module.vpc.*" matches "module.vpc.nat_gateway[0]"
        if self.resource_pattern.ends_with('*') {
            let prefix = self.resource_pattern.trim_end_matches('*');
            return resource_id.starts_with(prefix);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exemption_exact_match() {
        let exemption = PolicyExemption {
            id: "EXE-001".to_string(),
            policy_name: "NAT_GATEWAY_LIMIT".to_string(),
            resource_pattern: "module.vpc.nat_gateway[0]".to_string(),
            justification: "Required for production".to_string(),
            expires_at: "2026-01-01".to_string(),
            approved_by: "ops@example.com".to_string(),
            created_at: "2025-12-01T00:00:00Z".to_string(),
            ticket_ref: None,
        };

        assert!(exemption.matches("NAT_GATEWAY_LIMIT", "module.vpc.nat_gateway[0]"));
        assert!(!exemption.matches("NAT_GATEWAY_LIMIT", "module.vpc.nat_gateway[1]"));
        assert!(!exemption.matches("EC2_INSTANCE_TYPE", "module.vpc.nat_gateway[0]"));
    }

    #[test]
    fn test_exemption_wildcard_match() {
        let exemption = PolicyExemption {
            id: "EXE-002".to_string(),
            policy_name: "EC2_INSTANCE_TYPE".to_string(),
            resource_pattern: "module.app.*".to_string(),
            justification: "Legacy instances".to_string(),
            expires_at: "2026-03-01".to_string(),
            approved_by: "dev@example.com".to_string(),
            created_at: "2025-12-01T00:00:00Z".to_string(),
            ticket_ref: Some("JIRA-456".to_string()),
        };

        assert!(exemption.matches("EC2_INSTANCE_TYPE", "module.app.instance[0]"));
        assert!(exemption.matches("EC2_INSTANCE_TYPE", "module.app.instance[1]"));
        assert!(exemption.matches("EC2_INSTANCE_TYPE", "module.app.web_server"));
        assert!(!exemption.matches("EC2_INSTANCE_TYPE", "module.vpc.instance[0]"));
    }

    #[test]
    fn test_exemption_status_display() {
        assert_eq!(ExemptionStatus::Active.to_string(), "Active");
        assert_eq!(
            ExemptionStatus::Expired {
                expired_on: "2025-11-01".to_string()
            }
            .to_string(),
            "Expired on 2025-11-01"
        );
        assert_eq!(
            ExemptionStatus::ExpiringSoon {
                expires_in_days: 15
            }
            .to_string(),
            "Expiring in 15 days"
        );
        assert_eq!(
            ExemptionStatus::Invalid {
                reason: "Missing approval".to_string()
            }
            .to_string(),
            "Invalid: Missing approval"
        );
    }

    #[test]
    fn test_exemption_config_defaults() {
        let config = ExemptionConfig::default();
        assert_eq!(config.warning_threshold_days, 30);
        assert!(config.enforce_expiration);
        assert_eq!(config.max_duration_days, 365);
    }
}
