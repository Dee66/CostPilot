// Immutable audit log system with cryptographic chain

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

/// Audit event type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Policy lifecycle state transition
    PolicyStateChange,
    /// Policy approval or rejection
    PolicyApproval,
    /// Policy version created
    PolicyVersionCreated,
    /// Policy content modified
    PolicyContentModified,
    /// Policy activated for enforcement
    PolicyActivated,
    /// Policy deprecated
    PolicyDeprecated,
    /// Policy archived
    PolicyArchived,
    /// Exemption created
    ExemptionCreated,
    /// Exemption expired
    ExemptionExpired,
    /// Exemption revoked
    ExemptionRevoked,
    /// SLO violation detected
    SloViolation,
    /// SLO burn alert triggered
    SloBurnAlert,
    /// Configuration changed
    ConfigurationChange,
    /// Access granted
    AccessGranted,
    /// Access denied
    AccessDenied,
    /// User login
    UserLogin,
    /// User logout
    UserLogout,
}

impl AuditEventType {
    /// Get severity level for event type
    pub fn severity(&self) -> AuditSeverity {
        match self {
            AuditEventType::PolicyActivated
            | AuditEventType::PolicyDeprecated
            | AuditEventType::PolicyArchived => AuditSeverity::High,
            AuditEventType::PolicyApproval
            | AuditEventType::PolicyVersionCreated
            | AuditEventType::PolicyContentModified
            | AuditEventType::ExemptionCreated => AuditSeverity::Medium,
            AuditEventType::SloViolation | AuditEventType::SloBurnAlert => AuditSeverity::Critical,
            AuditEventType::AccessDenied => AuditSeverity::High,
            _ => AuditSeverity::Low,
        }
    }

    /// Check if event requires retention
    pub fn requires_long_retention(&self) -> bool {
        matches!(
            self,
            AuditEventType::PolicyActivated
                | AuditEventType::PolicyApproval
                | AuditEventType::AccessDenied
                | AuditEventType::SloViolation
        )
    }
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,

    /// Event type
    pub event_type: AuditEventType,

    /// When event occurred
    pub timestamp: DateTime<Utc>,

    /// Actor who initiated the event
    pub actor: String,

    /// Resource affected (policy ID, exemption ID, etc.)
    pub resource_id: String,

    /// Resource type
    pub resource_type: String,

    /// Event severity
    pub severity: AuditSeverity,

    /// Event description
    pub description: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Previous value (for changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,

    /// New value (for changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,

    /// IP address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Result of the action
    pub success: bool,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(
        event_type: AuditEventType,
        actor: String,
        resource_id: String,
        resource_type: String,
        description: String,
    ) -> Self {
        let id = Self::generate_event_id(&event_type, &resource_id);
        let severity = event_type.severity();

        Self {
            id,
            event_type,
            timestamp: Utc::now(),
            actor,
            resource_id,
            resource_type,
            severity,
            description,
            metadata: HashMap::new(),
            old_value: None,
            new_value: None,
            ip_address: None,
            user_agent: None,
            success: true,
            error_message: None,
        }
    }

    /// Generate unique event ID
    fn generate_event_id(event_type: &AuditEventType, resource_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let now = Utc::now();
        let mut hasher = DefaultHasher::new();
        format!("{:?}", event_type).hash(&mut hasher);
        resource_id.hash(&mut hasher);
        now.timestamp_millis().hash(&mut hasher);
        let hash = hasher.finish();

        format!("audit_{:016x}", hash)
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set old/new values for change events
    pub fn with_change(mut self, old_value: String, new_value: String) -> Self {
        self.old_value = Some(old_value);
        self.new_value = Some(new_value);
        self
    }

    /// Mark event as failed
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// Calculate event hash for chain verification
    pub fn calculate_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        let hash = Sha256::digest(json.as_bytes());
        format!("{:x}", hash)
    }
}

/// Immutable audit log entry in blockchain-style chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Sequence number in chain
    pub sequence: u64,

    /// Audit event
    pub event: AuditEvent,

    /// Hash of this entry
    pub hash: String,

    /// Hash of previous entry (creates chain)
    pub previous_hash: String,

    /// Signature of entry (SHA-256 HMAC)
    pub signature: String,
}

impl AuditLogEntry {
    /// Create new log entry
    pub fn new(sequence: u64, event: AuditEvent, previous_hash: String) -> Self {
        let event_hash = event.calculate_hash();
        let hash = Self::calculate_entry_hash(sequence, &event_hash, &previous_hash);
        let signature = Self::calculate_signature(&hash);

        Self {
            sequence,
            event,
            hash,
            previous_hash,
            signature,
        }
    }

    /// Calculate entry hash
    fn calculate_entry_hash(sequence: u64, event_hash: &str, previous_hash: &str) -> String {
        let combined = format!("{}:{}:{}", sequence, event_hash, previous_hash);
        let hash = Sha256::digest(combined.as_bytes());
        format!("{:x}", hash)
    }

    /// Calculate signature (SHA-256 HMAC simulation)
    fn calculate_signature(hash: &str) -> String {
        // In production, use proper HMAC with secret key
        // For determinism, we use hash-based signature
        let signature_input = format!("COSTPILOT_AUDIT:{}", hash);
        let sig = Sha256::digest(signature_input.as_bytes());
        format!("{:x}", sig)
    }

    /// Verify entry integrity
    pub fn verify(&self, previous_hash: &str) -> bool {
        // Verify previous hash matches
        if self.previous_hash != previous_hash {
            return false;
        }

        // Verify event hash
        let event_hash = self.event.calculate_hash();

        // Verify entry hash
        let calculated_hash =
            Self::calculate_entry_hash(self.sequence, &event_hash, &self.previous_hash);
        if self.hash != calculated_hash {
            return false;
        }

        // Verify signature
        let calculated_signature = Self::calculate_signature(&self.hash);
        if self.signature != calculated_signature {
            return false;
        }

        true
    }
}

/// Audit log chain manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Log entries (immutable chain)
    entries: Vec<AuditLogEntry>,

    /// Genesis hash (start of chain)
    pub genesis_hash: String,
}

impl AuditLog {
    /// Create new audit log
    pub fn new() -> Self {
        let genesis_hash = Self::calculate_genesis_hash();

        Self {
            entries: Vec::new(),
            genesis_hash,
        }
    }

    /// Calculate genesis hash
    fn calculate_genesis_hash() -> String {
        let genesis = "COSTPILOT_AUDIT_LOG_GENESIS_2025";
        let hash = Sha256::digest(genesis.as_bytes());
        format!("{:x}", hash)
    }

    /// Get previous hash for next entry
    fn get_previous_hash(&self) -> String {
        self.entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| self.genesis_hash.clone())
    }

    /// Append event to log
    pub fn append(&mut self, event: AuditEvent) -> Result<u64, AuditLogError> {
        let sequence = self.entries.len() as u64;
        let previous_hash = self.get_previous_hash();

        let entry = AuditLogEntry::new(sequence, event, previous_hash);

        // Verify entry before appending
        let verify_hash = if sequence == 0 {
            &self.genesis_hash
        } else {
            &self.entries[sequence as usize - 1].hash
        };

        if !entry.verify(verify_hash) {
            return Err(AuditLogError::InvalidEntry);
        }

        self.entries.push(entry);
        Ok(sequence)
    }

    /// Verify entire chain integrity
    pub fn verify_chain(&self) -> Result<(), AuditLogError> {
        let mut previous_hash = self.genesis_hash.clone();

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.sequence != i as u64 {
                return Err(AuditLogError::BrokenChain(format!(
                    "Sequence mismatch at index {}: expected {}, got {}",
                    i, i, entry.sequence
                )));
            }

            if !entry.verify(&previous_hash) {
                return Err(AuditLogError::BrokenChain(format!(
                    "Entry {} failed verification",
                    entry.sequence
                )));
            }

            previous_hash = entry.hash.clone();
        }

        Ok(())
    }

    /// Get all entries
    pub fn get_entries(&self) -> &[AuditLogEntry] {
        &self.entries
    }

    /// Get entries by event type
    pub fn get_by_event_type(&self, event_type: AuditEventType) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.event_type == event_type)
            .collect()
    }

    /// Get entries by resource
    pub fn get_by_resource(&self, resource_id: &str) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.resource_id == resource_id)
            .collect()
    }

    /// Get entries by actor
    pub fn get_by_actor(&self, actor: &str) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.actor == actor)
            .collect()
    }

    /// Get entries by severity
    pub fn get_by_severity(&self, severity: AuditSeverity) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.severity == severity)
            .collect()
    }

    /// Get entries in time range
    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.timestamp >= start && e.event.timestamp <= end)
            .collect()
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get last entry
    pub fn last_entry(&self) -> Option<&AuditLogEntry> {
        self.entries.last()
    }

    /// Export to NDJSON format
    pub fn export_ndjson(&self) -> Result<String, AuditLogError> {
        let mut output = String::new();
        for entry in &self.entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| AuditLogError::SerializationError(e.to_string()))?;
            output.push_str(&json);
            output.push('\n');
        }
        Ok(output)
    }

    /// Export to CSV format
    pub fn export_csv(&self) -> Result<String, AuditLogError> {
        let mut output = String::new();

        // CSV header
        output.push_str("sequence,timestamp,event_type,actor,resource_id,resource_type,severity,description,success,hash\n");

        // CSV rows
        for entry in &self.entries {
            let row = format!(
                "{},{},{:?},{},{},{},{:?},{},{},{}\n",
                entry.sequence,
                entry.event.timestamp.to_rfc3339(),
                entry.event.event_type,
                Self::csv_escape(&entry.event.actor),
                Self::csv_escape(&entry.event.resource_id),
                Self::csv_escape(&entry.event.resource_type),
                entry.event.severity,
                Self::csv_escape(&entry.event.description),
                entry.event.success,
                entry.hash
            );
            output.push_str(&row);
        }

        Ok(output)
    }

    /// CSV escape helper
    fn csv_escape(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total events
    pub total_events: usize,

    /// Events by type
    pub events_by_type: HashMap<String, usize>,

    /// Events by severity
    pub events_by_severity: HashMap<String, usize>,

    /// Unique actors
    pub unique_actors: usize,

    /// Unique resources
    pub unique_resources: usize,

    /// Failed events
    pub failed_events: usize,

    /// Events in last 24 hours
    pub events_last_24h: usize,

    /// Events in last 7 days
    pub events_last_7d: usize,

    /// Events in last 30 days
    pub events_last_30d: usize,

    /// Chain verified
    pub chain_verified: bool,
}

impl AuditLog {
    /// Get audit statistics
    pub fn get_statistics(&self) -> Result<AuditStatistics, AuditLogError> {
        use std::collections::HashSet;

        let chain_verified = self.verify_chain().is_ok();

        let now = Utc::now();
        let day_ago = now - chrono::Duration::hours(24);
        let week_ago = now - chrono::Duration::days(7);
        let month_ago = now - chrono::Duration::days(30);

        let mut events_by_type: HashMap<String, usize> = HashMap::new();
        let mut events_by_severity: HashMap<String, usize> = HashMap::new();
        let mut actors: HashSet<String> = HashSet::new();
        let mut resources: HashSet<String> = HashSet::new();
        let mut failed_events = 0;
        let mut events_last_24h = 0;
        let mut events_last_7d = 0;
        let mut events_last_30d = 0;

        for entry in &self.entries {
            // Count by type
            *events_by_type
                .entry(format!("{:?}", entry.event.event_type))
                .or_insert(0) += 1;

            // Count by severity
            *events_by_severity
                .entry(format!("{:?}", entry.event.severity))
                .or_insert(0) += 1;

            // Track actors and resources
            actors.insert(entry.event.actor.clone());
            resources.insert(entry.event.resource_id.clone());

            // Count failures
            if !entry.event.success {
                failed_events += 1;
            }

            // Time-based counts
            if entry.event.timestamp >= day_ago {
                events_last_24h += 1;
            }
            if entry.event.timestamp >= week_ago {
                events_last_7d += 1;
            }
            if entry.event.timestamp >= month_ago {
                events_last_30d += 1;
            }
        }

        Ok(AuditStatistics {
            total_events: self.entries.len(),
            events_by_type,
            events_by_severity,
            unique_actors: actors.len(),
            unique_resources: resources.len(),
            failed_events,
            events_last_24h,
            events_last_7d,
            events_last_30d,
            chain_verified,
        })
    }
}

/// Audit log errors
#[derive(Debug, Error)]
pub enum AuditLogError {
    #[error("Invalid audit entry")]
    InvalidEntry,

    #[error("Broken audit chain: {0}")]
    BrokenChain(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin@example.com".to_string(),
            "policy-123".to_string(),
            "cost_policy".to_string(),
            "Policy activated for production".to_string(),
        );

        assert_eq!(event.event_type, AuditEventType::PolicyActivated);
        assert_eq!(event.actor, "admin@example.com");
        assert_eq!(event.resource_id, "policy-123");
        assert_eq!(event.severity, AuditSeverity::High);
        assert!(event.success);
    }

    #[test]
    fn test_audit_event_hash() {
        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin@example.com".to_string(),
            "policy-123".to_string(),
            "cost_policy".to_string(),
            "Policy activated".to_string(),
        );

        let hash1 = event.calculate_hash();
        let hash2 = event.calculate_hash();

        // Hash should be deterministic
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_audit_log_chain() {
        let mut log = AuditLog::new();

        // Add first event
        let event1 = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin@example.com".to_string(),
            "policy-1".to_string(),
            "cost_policy".to_string(),
            "First policy".to_string(),
        );
        let seq1 = log.append(event1).unwrap();
        assert_eq!(seq1, 0);

        // Add second event
        let event2 = AuditEvent::new(
            AuditEventType::PolicyApproval,
            "reviewer@example.com".to_string(),
            "policy-2".to_string(),
            "cost_policy".to_string(),
            "Second policy".to_string(),
        );
        let seq2 = log.append(event2).unwrap();
        assert_eq!(seq2, 1);

        // Verify chain
        assert!(log.verify_chain().is_ok());
        assert_eq!(log.entry_count(), 2);
    }

    #[test]
    fn test_chain_integrity() {
        let mut log = AuditLog::new();

        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::PolicyActivated,
                format!("user-{}", i),
                format!("policy-{}", i),
                "cost_policy".to_string(),
                format!("Event {}", i),
            );
            log.append(event).unwrap();
        }

        // Chain should be valid
        assert!(log.verify_chain().is_ok());

        // Manually corrupt an entry
        if let Some(entry) = log.entries.get_mut(2) {
            entry.hash = "corrupted_hash".to_string();
        }

        // Chain verification should fail
        assert!(log.verify_chain().is_err());
    }

    #[test]
    fn test_query_by_event_type() {
        let mut log = AuditLog::new();

        let event1 = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Activated".to_string(),
        );
        log.append(event1).unwrap();

        let event2 = AuditEvent::new(
            AuditEventType::PolicyApproval,
            "reviewer".to_string(),
            "policy-2".to_string(),
            "policy".to_string(),
            "Approved".to_string(),
        );
        log.append(event2).unwrap();

        let event3 = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-3".to_string(),
            "policy".to_string(),
            "Activated".to_string(),
        );
        log.append(event3).unwrap();

        let activated = log.get_by_event_type(AuditEventType::PolicyActivated);
        assert_eq!(activated.len(), 2);
    }

    #[test]
    fn test_query_by_actor() {
        let mut log = AuditLog::new();

        let event1 = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin@example.com".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Event 1".to_string(),
        );
        log.append(event1).unwrap();

        let event2 = AuditEvent::new(
            AuditEventType::PolicyApproval,
            "reviewer@example.com".to_string(),
            "policy-2".to_string(),
            "policy".to_string(),
            "Event 2".to_string(),
        );
        log.append(event2).unwrap();

        let admin_events = log.get_by_actor("admin@example.com");
        assert_eq!(admin_events.len(), 1);
        assert_eq!(admin_events[0].event.actor, "admin@example.com");
    }

    #[test]
    fn test_export_ndjson() {
        let mut log = AuditLog::new();

        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Test event".to_string(),
        );
        log.append(event).unwrap();

        let ndjson = log.export_ndjson().unwrap();
        assert!(ndjson.contains("\"sequence\":0"));
        assert!(ndjson.contains("\"actor\":\"admin\""));
        assert!(ndjson.ends_with('\n'));
    }

    #[test]
    fn test_export_csv() {
        let mut log = AuditLog::new();

        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Test event".to_string(),
        );
        log.append(event).unwrap();

        let csv = log.export_csv().unwrap();
        assert!(csv.contains("sequence,timestamp,event_type"));
        assert!(csv.contains("0,"));
        assert!(csv.contains("admin"));
    }

    #[test]
    fn test_statistics() {
        let mut log = AuditLog::new();

        // Add various events
        for i in 0..10 {
            let event = AuditEvent::new(
                if i % 2 == 0 {
                    AuditEventType::PolicyActivated
                } else {
                    AuditEventType::PolicyApproval
                },
                format!("user-{}", i % 3),
                format!("policy-{}", i),
                "policy".to_string(),
                format!("Event {}", i),
            );
            log.append(event).unwrap();
        }

        let stats = log.get_statistics().unwrap();
        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.unique_actors, 3);
        assert_eq!(stats.unique_resources, 10);
        assert!(stats.chain_verified);
    }

    #[test]
    fn test_event_severity() {
        assert_eq!(
            AuditEventType::PolicyActivated.severity(),
            AuditSeverity::High
        );
        assert_eq!(
            AuditEventType::SloViolation.severity(),
            AuditSeverity::Critical
        );
        assert_eq!(
            AuditEventType::PolicyApproval.severity(),
            AuditSeverity::Medium
        );
    }

    #[test]
    fn test_failed_event() {
        let event = AuditEvent::new(
            AuditEventType::PolicyActivated,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Failed activation".to_string(),
        )
        .with_error("Insufficient permissions".to_string());

        assert!(!event.success);
        assert_eq!(
            event.error_message,
            Some("Insufficient permissions".to_string())
        );
    }

    #[test]
    fn test_event_with_change() {
        let event = AuditEvent::new(
            AuditEventType::PolicyContentModified,
            "admin".to_string(),
            "policy-1".to_string(),
            "policy".to_string(),
            "Updated limit".to_string(),
        )
        .with_change("max_cost: 100".to_string(), "max_cost: 200".to_string());

        assert_eq!(event.old_value, Some("max_cost: 100".to_string()));
        assert_eq!(event.new_value, Some("max_cost: 200".to_string()));
    }
}
