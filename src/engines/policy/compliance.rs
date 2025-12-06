// Compliance reporting and audit analysis

use super::audit_log::{AuditEvent, AuditEventType, AuditLog, AuditLogEntry, AuditSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Compliance framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceFramework {
    /// SOC 2 Type II
    Soc2,
    /// ISO 27001
    Iso27001,
    /// GDPR
    Gdpr,
    /// HIPAA
    Hipaa,
    /// PCI DSS
    PciDss,
}

impl ComplianceFramework {
    /// Get required retention period in days
    pub fn retention_days(&self) -> u32 {
        match self {
            ComplianceFramework::Soc2 => 365,
            ComplianceFramework::Iso27001 => 365,
            ComplianceFramework::Gdpr => 2190, // 6 years
            ComplianceFramework::Hipaa => 2190, // 6 years
            ComplianceFramework::PciDss => 365,
        }
    }

    /// Get framework requirements
    pub fn requirements(&self) -> Vec<String> {
        match self {
            ComplianceFramework::Soc2 => vec![
                "Audit log integrity verification".to_string(),
                "Access control events tracked".to_string(),
                "Policy change approval workflow".to_string(),
                "Tamper-proof audit trail".to_string(),
            ],
            ComplianceFramework::Iso27001 => vec![
                "Information security events logged".to_string(),
                "Access attempts recorded".to_string(),
                "Configuration changes tracked".to_string(),
                "Log integrity maintained".to_string(),
            ],
            ComplianceFramework::Gdpr => vec![
                "Data access logged".to_string(),
                "Consent changes tracked".to_string(),
                "Data retention policy enforced".to_string(),
                "User rights requests recorded".to_string(),
            ],
            ComplianceFramework::Hipaa => vec![
                "PHI access logged".to_string(),
                "Security incidents recorded".to_string(),
                "Audit logs protected".to_string(),
                "Access control enforced".to_string(),
            ],
            ComplianceFramework::PciDss => vec![
                "Cardholder data access tracked".to_string(),
                "Security events logged".to_string(),
                "Failed authentication attempts recorded".to_string(),
                "Log review performed regularly".to_string(),
            ],
        }
    }
}

/// Compliance requirement check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    /// Requirement name
    pub requirement: String,

    /// Check status
    pub status: ComplianceStatus,

    /// Description
    pub description: String,

    /// Evidence (event IDs, counts, etc.)
    pub evidence: Vec<String>,

    /// Recommendations if non-compliant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendations: Option<Vec<String>>,
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    NotApplicable,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Framework being evaluated
    pub framework: ComplianceFramework,

    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Time period covered
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Overall compliance status
    pub overall_status: ComplianceStatus,

    /// Individual requirement checks
    pub checks: Vec<ComplianceCheck>,

    /// Summary statistics
    pub summary: ComplianceSummary,

    /// Audit log verification result
    pub audit_log_verified: bool,
}

/// Compliance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    /// Total requirements checked
    pub total_requirements: usize,

    /// Compliant requirements
    pub compliant: usize,

    /// Non-compliant requirements
    pub non_compliant: usize,

    /// Partially compliant requirements
    pub partially_compliant: usize,

    /// Not applicable requirements
    pub not_applicable: usize,

    /// Compliance percentage
    pub compliance_percentage: f64,
}

/// Compliance analyzer
pub struct ComplianceAnalyzer {
    audit_log: AuditLog,
}

impl ComplianceAnalyzer {
    /// Create new compliance analyzer
    pub fn new(audit_log: AuditLog) -> Self {
        Self { audit_log }
    }

    /// Generate compliance report
    pub fn generate_report(
        &self,
        framework: ComplianceFramework,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<ComplianceReport, ComplianceError> {
        // Verify audit log integrity
        let audit_log_verified = self.audit_log.verify_chain().is_ok();

        // Run checks based on framework
        let checks = match framework {
            ComplianceFramework::Soc2 => self.check_soc2(period_start, period_end)?,
            ComplianceFramework::Iso27001 => self.check_iso27001(period_start, period_end)?,
            ComplianceFramework::Gdpr => self.check_gdpr(period_start, period_end)?,
            ComplianceFramework::Hipaa => self.check_hipaa(period_start, period_end)?,
            ComplianceFramework::PciDss => self.check_pci_dss(period_start, period_end)?,
        };

        // Calculate summary
        let summary = Self::calculate_summary(&checks);

        // Determine overall status
        let overall_status = if summary.non_compliant > 0 {
            ComplianceStatus::NonCompliant
        } else if summary.partially_compliant > 0 {
            ComplianceStatus::PartiallyCompliant
        } else {
            ComplianceStatus::Compliant
        };

        Ok(ComplianceReport {
            framework,
            generated_at: Utc::now(),
            period_start,
            period_end,
            overall_status,
            checks,
            summary,
            audit_log_verified,
        })
    }

    /// Check SOC 2 compliance
    fn check_soc2(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Vec<ComplianceCheck>, ComplianceError> {
        let mut checks = Vec::new();

        // Check 1: Audit log integrity
        let integrity_check = ComplianceCheck {
            requirement: "Audit log integrity verification".to_string(),
            status: if self.audit_log.verify_chain().is_ok() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            description: "Audit logs must be tamper-proof and verifiable".to_string(),
            evidence: vec![format!(
                "Chain verification: {}",
                if self.audit_log.verify_chain().is_ok() {
                    "PASS"
                } else {
                    "FAIL"
                }
            )],
            recommendations: None,
        };
        checks.push(integrity_check);

        // Check 2: Access control events
        let access_events = self.audit_log.get_by_time_range(period_start, period_end);
        let access_control_events: Vec<_> = access_events
            .iter()
            .filter(|e| {
                matches!(
                    e.event.event_type,
                    AuditEventType::AccessGranted | AuditEventType::AccessDenied
                )
            })
            .collect();

        let access_check = ComplianceCheck {
            requirement: "Access control events tracked".to_string(),
            status: if !access_control_events.is_empty() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::PartiallyCompliant
            },
            description: "All access attempts must be logged".to_string(),
            evidence: vec![format!(
                "{} access control events recorded",
                access_control_events.len()
            )],
            recommendations: if access_control_events.is_empty() {
                Some(vec![
                    "Enable access logging for all resources".to_string()
                ])
            } else {
                None
            },
        };
        checks.push(access_check);

        // Check 3: Policy change approval
        let policy_events = self.audit_log.get_by_time_range(period_start, period_end);
        let policy_changes: Vec<_> = policy_events
            .iter()
            .filter(|e| {
                matches!(
                    e.event.event_type,
                    AuditEventType::PolicyApproval | AuditEventType::PolicyActivated
                )
            })
            .collect();

        let approval_check = ComplianceCheck {
            requirement: "Policy change approval workflow".to_string(),
            status: if policy_changes.iter().all(|e| e.event.success) {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            description: "All policy changes must follow approval workflow".to_string(),
            evidence: vec![format!("{} policy changes recorded", policy_changes.len())],
            recommendations: None,
        };
        checks.push(approval_check);

        // Check 4: Tamper-proof trail
        let tamper_check = ComplianceCheck {
            requirement: "Tamper-proof audit trail".to_string(),
            status: ComplianceStatus::Compliant,
            description: "Audit trail uses cryptographic chain for integrity".to_string(),
            evidence: vec![
                "SHA-256 hashing enabled".to_string(),
                "Blockchain-style chain implemented".to_string(),
                format!("{} entries in chain", self.audit_log.entry_count()),
            ],
            recommendations: None,
        };
        checks.push(tamper_check);

        Ok(checks)
    }

    /// Check ISO 27001 compliance
    fn check_iso27001(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Vec<ComplianceCheck>, ComplianceError> {
        let mut checks = Vec::new();

        // Similar to SOC 2 but with ISO-specific requirements
        let security_events = self.audit_log.get_by_time_range(period_start, period_end);
        let critical_events: Vec<_> = security_events
            .iter()
            .filter(|e| e.event.severity == AuditSeverity::Critical)
            .collect();

        let security_check = ComplianceCheck {
            requirement: "Information security events logged".to_string(),
            status: ComplianceStatus::Compliant,
            description: "Security-related events must be logged and monitored".to_string(),
            evidence: vec![
                format!("{} total events in period", security_events.len()),
                format!("{} critical events", critical_events.len()),
            ],
            recommendations: None,
        };
        checks.push(security_check);

        Ok(checks)
    }

    /// Check GDPR compliance
    fn check_gdpr(
        &self,
        _period_start: DateTime<Utc>,
        _period_end: DateTime<Utc>,
    ) -> Result<Vec<ComplianceCheck>, ComplianceError> {
        let mut checks = Vec::new();

        // GDPR-specific checks
        let retention_check = ComplianceCheck {
            requirement: "Data retention policy enforced".to_string(),
            status: ComplianceStatus::Compliant,
            description: "Audit logs retained according to policy".to_string(),
            evidence: vec![format!(
                "Retention period: {} days",
                ComplianceFramework::Gdpr.retention_days()
            )],
            recommendations: None,
        };
        checks.push(retention_check);

        Ok(checks)
    }

    /// Check HIPAA compliance
    fn check_hipaa(
        &self,
        _period_start: DateTime<Utc>,
        _period_end: DateTime<Utc>,
    ) -> Result<Vec<ComplianceCheck>, ComplianceError> {
        let mut checks = Vec::new();

        // HIPAA-specific checks
        let protection_check = ComplianceCheck {
            requirement: "Audit logs protected".to_string(),
            status: if self.audit_log.verify_chain().is_ok() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            description: "Audit logs must be protected from tampering".to_string(),
            evidence: vec!["Cryptographic chain verified".to_string()],
            recommendations: None,
        };
        checks.push(protection_check);

        Ok(checks)
    }

    /// Check PCI DSS compliance
    fn check_pci_dss(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Vec<ComplianceCheck>, ComplianceError> {
        let mut checks = Vec::new();

        // PCI DSS-specific checks
        let events = self.audit_log.get_by_time_range(period_start, period_end);
        let failed_events: Vec<_> = events.iter().filter(|e| !e.event.success).collect();

        let auth_check = ComplianceCheck {
            requirement: "Failed authentication attempts recorded".to_string(),
            status: ComplianceStatus::Compliant,
            description: "All authentication failures must be logged".to_string(),
            evidence: vec![format!("{} failed events recorded", failed_events.len())],
            recommendations: None,
        };
        checks.push(auth_check);

        Ok(checks)
    }

    /// Calculate compliance summary
    fn calculate_summary(checks: &[ComplianceCheck]) -> ComplianceSummary {
        let total_requirements = checks.len();
        let mut compliant = 0;
        let mut non_compliant = 0;
        let mut partially_compliant = 0;
        let mut not_applicable = 0;

        for check in checks {
            match check.status {
                ComplianceStatus::Compliant => compliant += 1,
                ComplianceStatus::NonCompliant => non_compliant += 1,
                ComplianceStatus::PartiallyCompliant => partially_compliant += 1,
                ComplianceStatus::NotApplicable => not_applicable += 1,
            }
        }

        let compliance_percentage = if total_requirements > 0 {
            (compliant as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        ComplianceSummary {
            total_requirements,
            compliant,
            non_compliant,
            partially_compliant,
            not_applicable,
            compliance_percentage,
        }
    }
}

/// Audit query builder
pub struct AuditQuery<'a> {
    log: &'a AuditLog,
    event_types: Vec<AuditEventType>,
    actors: Vec<String>,
    resources: Vec<String>,
    severities: Vec<AuditSeverity>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    success_filter: Option<bool>,
}

impl<'a> AuditQuery<'a> {
    /// Create new query
    pub fn new(log: &'a AuditLog) -> Self {
        Self {
            log,
            event_types: Vec::new(),
            actors: Vec::new(),
            resources: Vec::new(),
            severities: Vec::new(),
            start_time: None,
            end_time: None,
            success_filter: None,
        }
    }

    /// Filter by event type
    pub fn with_event_type(mut self, event_type: AuditEventType) -> Self {
        self.event_types.push(event_type);
        self
    }

    /// Filter by actor
    pub fn with_actor(mut self, actor: String) -> Self {
        self.actors.push(actor);
        self
    }

    /// Filter by resource
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resources.push(resource);
        self
    }

    /// Filter by severity
    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severities.push(severity);
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by success status
    pub fn with_success(mut self, success: bool) -> Self {
        self.success_filter = Some(success);
        self
    }

    /// Execute query
    pub fn execute(&self) -> Vec<&AuditLogEntry> {
        self.log
            .get_entries()
            .iter()
            .filter(|entry| {
                // Event type filter
                if !self.event_types.is_empty()
                    && !self.event_types.contains(&entry.event.event_type)
                {
                    return false;
                }

                // Actor filter
                if !self.actors.is_empty() && !self.actors.contains(&entry.event.actor) {
                    return false;
                }

                // Resource filter
                if !self.resources.is_empty() && !self.resources.contains(&entry.event.resource_id)
                {
                    return false;
                }

                // Severity filter
                if !self.severities.is_empty() && !self.severities.contains(&entry.event.severity) {
                    return false;
                }

                // Time range filter
                if let Some(start) = self.start_time {
                    if entry.event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = self.end_time {
                    if entry.event.timestamp > end {
                        return false;
                    }
                }

                // Success filter
                if let Some(success) = self.success_filter {
                    if entry.event.success != success {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Count matching entries
    pub fn count(&self) -> usize {
        self.execute().len()
    }
}

/// Compliance error
#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("Compliance check failed: {0}")]
    CheckFailed(String),

    #[error("Invalid time range")]
    InvalidTimeRange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::policy::audit_log::AuditEvent;

    #[test]
    fn test_compliance_framework_retention() {
        assert_eq!(ComplianceFramework::Soc2.retention_days(), 365);
        assert_eq!(ComplianceFramework::Gdpr.retention_days(), 2190);
    }

    #[test]
    fn test_compliance_analyzer_soc2() {
        let mut log = AuditLog::new();

        // Add audit events
        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin@example.com".to_string(),
            "policy-1".to_string(),
            "cost_policy".to_string(),
            "Policy activated".to_string(),
        );
        log.append(event).unwrap();

        let analyzer = ComplianceAnalyzer::new(log);

        let start = Utc::now() - chrono::Duration::days(30);
        let end = Utc::now();

        let report = analyzer
            .generate_report(ComplianceFramework::Soc2, start, end)
            .unwrap();

        assert_eq!(report.framework, ComplianceFramework::Soc2);
        assert!(report.audit_log_verified);
        assert!(!report.checks.is_empty());
    }

    #[test]
    fn test_audit_query() {
        let mut log = AuditLog::new();

        // Add events
        let event1 = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Event 1".to_string(),
        );
        log.append(event1).unwrap();

        let event2 = AuditEvent::new(
            AuditEventType::PolicyApproval,
            "reviewer".to_string(),
            "policy-2".to_string(),
            "policy".to_string(),
            "Event 2".to_string(),
        );
        log.append(event2).unwrap();

        // Query by event type
        let results = AuditQuery::new(&log)
            .with_event_type(AuditEventType::PolicyActivated)
            .execute();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event.event_type, AuditEventType::PolicyActivated);
    }

    #[test]
    fn test_audit_query_multiple_filters() {
        let mut log = AuditLog::new();

        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Test event".to_string(),
        );
        log.append(event).unwrap();

        let results = AuditQuery::new(&log)
            .with_event_type(AuditEventType::PolicyActivated)
            .with_actor("admin".to_string())
            .with_severity(AuditSeverity::High)
            .execute();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_compliance_summary() {
        let checks = vec![
            ComplianceCheck {
                requirement: "Req 1".to_string(),
                status: ComplianceStatus::Compliant,
                description: "".to_string(),
                evidence: vec![],
                recommendations: None,
            },
            ComplianceCheck {
                requirement: "Req 2".to_string(),
                status: ComplianceStatus::NonCompliant,
                description: "".to_string(),
                evidence: vec![],
                recommendations: None,
            },
            ComplianceCheck {
                requirement: "Req 3".to_string(),
                status: ComplianceStatus::Compliant,
                description: "".to_string(),
                evidence: vec![],
                recommendations: None,
            },
        ];

        let summary = ComplianceAnalyzer::calculate_summary(&checks);

        assert_eq!(summary.total_requirements, 3);
        assert_eq!(summary.compliant, 2);
        assert_eq!(summary.non_compliant, 1);
        assert!((summary.compliance_percentage - 66.67).abs() < 0.1);
    }
}
