# Audit Logs & Tamper-Proofing

Enterprise-grade immutable audit logging with cryptographic chain verification and compliance reporting.

## Overview

The Audit & Compliance system provides tamper-proof audit trails using blockchain-style cryptographic chains. Every action is recorded in an immutable log that can be cryptographically verified, ensuring complete accountability and compliance with regulatory frameworks.

## Architecture

### Components

**1. Audit Log (`audit_log.rs` - 890 lines, 15 tests)**
- Immutable event recording with cryptographic hashing
- Blockchain-style chain linking entries
- Event type classification and severity levels
- SHA-256 hashing for integrity verification
- Multi-format export (JSON, NDJSON, CSV)

**2. Compliance Reporting (`compliance.rs` - 600 lines, 7 tests)**
- Multi-framework support (SOC 2, ISO 27001, GDPR, HIPAA, PCI DSS)
- Automated compliance checking
- Evidence collection and recommendations
- Advanced query builder for audit analysis

**3. CLI Commands (`audit.rs` - 420 lines)**
- 6 comprehensive commands
- Multi-format output (JSON, text)
- Rich colored terminal display

## Cryptographic Chain

### How It Works

Each audit log entry contains:
- **Event Hash**: SHA-256 hash of the event data
- **Previous Hash**: Hash of the previous entry (creates chain)
- **Entry Hash**: Combined hash of sequence + event hash + previous hash
- **Signature**: SHA-256 HMAC for tamper detection

```
Genesis → Entry 0 → Entry 1 → Entry 2 → ... → Entry N
   ↓         ↓         ↓         ↓              ↓
 Hash 0 → Hash 1 → Hash 2 → Hash 3 → ... → Hash N
```

Any modification to any entry breaks the chain, making tampering immediately detectable.

### Verification Algorithm

```rust
fn verify_chain(log: &AuditLog) -> Result<(), AuditLogError> {
    let mut previous_hash = log.genesis_hash.clone();
    
    for (i, entry) in log.entries.iter().enumerate() {
        // Verify sequence
        assert_eq!(entry.sequence, i as u64);
        
        // Verify previous hash matches
        assert_eq!(entry.previous_hash, previous_hash);
        
        // Verify event hash
        let event_hash = entry.event.calculate_hash();
        
        // Verify entry hash
        let calculated_hash = calculate_entry_hash(
            entry.sequence,
            &event_hash,
            &entry.previous_hash
        );
        assert_eq!(entry.hash, calculated_hash);
        
        // Verify signature
        let calculated_signature = calculate_signature(&entry.hash);
        assert_eq!(entry.signature, calculated_signature);
        
        previous_hash = entry.hash.clone();
    }
    
    Ok(())
}
```

## Event Types

### Policy Events
- **PolicyStateChange**: Lifecycle state transitions
- **PolicyApproval**: Approval or rejection decisions
- **PolicyVersionCreated**: New version created
- **PolicyContentModified**: Policy content changed
- **PolicyActivated**: Policy enabled for enforcement
- **PolicyDeprecated**: Policy marked for removal
- **PolicyArchived**: Policy permanently archived

### Exemption Events
- **ExemptionCreated**: New exemption granted
- **ExemptionExpired**: Exemption reached expiration
- **ExemptionRevoked**: Exemption manually revoked

### SLO Events
- **SloViolation**: SLO threshold exceeded
- **SloBurnAlert**: Burn rate alert triggered

### Security Events
- **ConfigurationChange**: System configuration modified
- **AccessGranted**: Access permission granted
- **AccessDenied**: Access attempt denied
- **UserLogin**: User authentication success
- **UserLogout**: User session ended

## Severity Levels

| Severity | Color | Use Case | Examples |
|----------|-------|----------|----------|
| **Critical** | 🔴 Red | Security incidents, SLO violations | SLO breach, unauthorized access |
| **High** | 🟠 Orange | Policy changes, access denials | Policy activation, access denied |
| **Medium** | 🟡 Yellow | Approvals, version changes | Policy approval, new version |
| **Low** | ⚫ Gray | Routine operations | Configuration view, status check |

## Usage Examples

### Recording Events Programmatically

```rust
use costpilot::engines::policy::{AuditLog, AuditEvent, AuditEventType};

// Create audit log
let mut log = AuditLog::new();

// Create event
let event = AuditEvent::new(
    AuditEventType::PolicyActivated,
    "admin@example.com".to_string(),
    "nat-gateway-limit".to_string(),
    "cost_policy".to_string(),
    "Policy activated for production".to_string(),
)
.with_metadata("environment".to_string(), "production".to_string())
.with_change(
    "status: draft".to_string(),
    "status: active".to_string(),
);

// Append to log
let sequence = log.append(event)?;
println!("Event recorded at sequence {}", sequence);

// Verify chain integrity
log.verify_chain()?;
```

### Querying Audit Log

```rust
use costpilot::engines::policy::{AuditQuery, AuditEventType, AuditSeverity};
use chrono::{Duration, Utc};

let now = Utc::now();
let week_ago = now - Duration::days(7);

// Build complex query
let results = AuditQuery::new(&log)
    .with_event_type(AuditEventType::PolicyActivated)
    .with_severity(AuditSeverity::High)
    .with_time_range(week_ago, now)
    .with_success(true)
    .execute();

println!("Found {} matching events", results.len());
```

## CLI Commands

### View Audit Log

```bash
# View all entries
costpilot audit view

# View last 10 entries
costpilot audit view -n 10

# Filter by event type
costpilot audit view --event-type policy_activated

# Filter by actor
costpilot audit view --actor admin@example.com

# Filter by severity
costpilot audit view --severity critical

# Combine filters
costpilot audit view --event-type policy_approval --actor admin@example.com -n 5
```

**Output Example:**
```
📋 Audit Log Entries
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ✅ Sequence #0
  Type: PolicyActivated
  Severity: high
  Timestamp: 2025-12-06T10:30:00Z
  Actor: admin@example.com
  Resource: nat-gateway-limit (cost_policy)
  Description: Policy activated for production
  ← Old: status: approved
  → New: status: active
  Hash: a1b2c3d4e5f6g7h8

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total entries: 1
```

### Verify Audit Log Integrity

```bash
costpilot audit verify
```

**Success Output:**
```
🔐 Verifying Audit Log Integrity
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Audit log verification: PASS

The audit log chain is intact and tamper-proof.
Total entries verified: 125
Genesis hash: 8f3e2a1b0c9d7e6f
Last entry hash: 4a5b6c7d8e9f0a1b
```

**Failure Output:**
```
🔐 Verifying Audit Log Integrity
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ Audit log verification: FAIL

⚠ Integrity violation detected!
Error: Entry 23 failed verification

The audit log chain has been tampered with or corrupted.
This indicates a serious security issue.
```

### Generate Compliance Report

```bash
# SOC 2 compliance (last 30 days)
costpilot audit compliance --framework SOC2

# ISO 27001 compliance (last 90 days)
costpilot audit compliance --framework ISO27001 --days 90

# GDPR compliance
costpilot audit compliance --framework GDPR --days 180
```

**Output Example:**
```
📊 SOC2 Compliance Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Overall Status: ✅ COMPLIANT
Report Period: 2025-11-06 to 2025-12-06
Audit Log Verified: ✅ Yes

Summary
  Total Requirements: 4
  ✅ Compliant: 4
  ❌ Non-Compliant: 0
  ⚠ Partially Compliant: 0
  Compliance Rate: 100.0%

Requirement Checks
  ✅ Audit log integrity verification
     Audit logs must be tamper-proof and verifiable
     Evidence:
       • Chain verification: PASS

  ✅ Access control events tracked
     All access attempts must be logged
     Evidence:
       • 15 access control events recorded

  ✅ Policy change approval workflow
     All policy changes must follow approval workflow
     Evidence:
       • 8 policy changes recorded

  ✅ Tamper-proof audit trail
     Audit trail uses cryptographic chain for integrity
     Evidence:
       • SHA-256 hashing enabled
       • Blockchain-style chain implemented
       • 125 entries in chain

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Report generated at: 2025-12-06T14:30:00Z
```

### Export Audit Log

```bash
# Export to NDJSON (newline-delimited JSON)
costpilot audit export --format ndjson --output audit_log.ndjson

# Export to CSV
costpilot audit export --format csv --output audit_log.csv

# Export to JSON
costpilot audit export --format json --output audit_log.json
```

### Audit Log Statistics

```bash
costpilot audit stats
```

**Output:**
```
📊 Audit Log Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Overview
  Total Events: 125
  Unique Actors: 8
  Unique Resources: 15
  Failed Events: 3
  Chain Verified: ✅ Yes

Time-Based Activity
  Last 24 hours: 12
  Last 7 days: 45
  Last 30 days: 125

Events by Type
  PolicyActivated                 32
  PolicyApproval                  28
  PolicyStateChange               20
  PolicyVersionCreated            15
  ConfigurationChange             12
  SloViolation                    8
  AccessDenied                    5
  AccessGranted                   5

Events by Severity
  Critical                        8
  High                            37
  Medium                          43
  Low                             37
```

### Record Manual Event

```bash
costpilot audit record \
  --event-type configuration_change \
  --actor admin@example.com \
  --resource-id system-config \
  --resource-type configuration \
  --description "Updated SLO thresholds"
```

**Output:**
```
✅ Audit event recorded

Sequence: #125
Total entries: 126
```

## Compliance Frameworks

### SOC 2 Type II

**Requirements:**
- Audit log integrity verification
- Access control events tracked
- Policy change approval workflow
- Tamper-proof audit trail

**Retention:** 365 days

### ISO 27001

**Requirements:**
- Information security events logged
- Access attempts recorded
- Configuration changes tracked
- Log integrity maintained

**Retention:** 365 days

### GDPR

**Requirements:**
- Data access logged
- Consent changes tracked
- Data retention policy enforced
- User rights requests recorded

**Retention:** 2190 days (6 years)

### HIPAA

**Requirements:**
- PHI access logged
- Security incidents recorded
- Audit logs protected
- Access control enforced

**Retention:** 2190 days (6 years)

### PCI DSS

**Requirements:**
- Cardholder data access tracked
- Security events logged
- Failed authentication attempts recorded
- Log review performed regularly

**Retention:** 365 days

## Integration with Policy Lifecycle

The audit log automatically captures policy lifecycle events:

```rust
use costpilot::engines::policy::{
    AuditLog, AuditEvent, AuditEventType,
    PolicyLifecycle, PolicyState,
};

let mut log = AuditLog::new();
let mut lifecycle = PolicyLifecycle::new("cost-policy".to_string());

// Transition policy state
lifecycle.transition(
    PolicyState::Review,
    "author@example.com".to_string(),
    Some("Ready for review".to_string()),
)?;

// Record audit event
let event = AuditEvent::new(
    AuditEventType::PolicyStateChange,
    "author@example.com".to_string(),
    "cost-policy".to_string(),
    "cost_policy".to_string(),
    "Policy transitioned from Draft to Review".to_string(),
)
.with_change("Draft".to_string(), "Review".to_string());

log.append(event)?;
```

## Best Practices

### 1. Regular Verification

Run integrity verification regularly:

```bash
# Daily verification (recommended)
costpilot audit verify

# Or as part of CI/CD
if ! costpilot audit verify --format json | jq -e '.verified'; then
  echo "Audit log integrity violation!"
  exit 1
fi
```

### 2. Compliance Reporting

Generate compliance reports for auditors:

```bash
# Monthly SOC 2 report
costpilot audit compliance --framework SOC2 --days 30 > soc2_report_$(date +%Y%m).txt

# Quarterly ISO 27001 report
costpilot audit compliance --framework ISO27001 --days 90 > iso27001_report_Q$(date +%q).txt
```

### 3. Export for Long-Term Storage

Export audit logs for regulatory retention:

```bash
# Annual export
costpilot audit export --format ndjson --output audit_$(date +%Y).ndjson

# Compress and archive
gzip audit_$(date +%Y).ndjson
mv audit_$(date +%Y).ndjson.gz /archive/compliance/
```

### 4. Event Enrichment

Add metadata to events for better analysis:

```rust
let event = AuditEvent::new(...)
    .with_metadata("environment".to_string(), "production".to_string())
    .with_metadata("region".to_string(), "us-east-1".to_string())
    .with_metadata("ticket_id".to_string(), "CHANGE-12345".to_string())
    .with_ip("192.168.1.100".to_string());
```

### 5. Query Optimization

Use filters to narrow searches:

```rust
// Efficient: Filter at query level
let results = AuditQuery::new(&log)
    .with_event_type(AuditEventType::PolicyActivated)
    .with_time_range(start, end)
    .execute();

// Inefficient: Filter after loading all
let all_entries = log.get_entries();
let results: Vec<_> = all_entries.iter()
    .filter(|e| e.event.event_type == AuditEventType::PolicyActivated)
    .collect();
```

## Security Guarantees

### Immutability

Once recorded, audit entries cannot be modified without breaking the cryptographic chain:

```rust
// Append-only operations
log.append(event)?; // ✅ Allowed

// No mutation methods exist
log.entries[0].event.description = "Modified"; // ❌ Compilation error (private field)
```

### Tamper Detection

Any modification is immediately detectable:

```
Original: Hash(Entry 5) = abc123
Modified: Hash(Entry 5) = xyz789
Entry 6 expects previous_hash = abc123
Entry 6 has previous_hash = abc123
Entry 6 hash verification = FAIL ❌
```

### Chain Verification

```rust
match log.verify_chain() {
    Ok(_) => println!("✅ Intact"),
    Err(AuditLogError::BrokenChain(msg)) => {
        eprintln!("❌ Tampered: {}", msg);
        // Alert security team
        // Investigate incident
    }
}
```

## Performance Characteristics

- **Append**: O(1) - constant time
- **Verify Chain**: O(n) - linear in entry count
- **Query by Event Type**: O(n) - full scan
- **Query by Time Range**: O(n) - full scan (can optimize with indexing)
- **Export**: O(n) - linear serialization

**Memory Usage:**
- Per entry: ~500 bytes (average)
- 10,000 entries: ~5 MB
- 100,000 entries: ~50 MB

## Related Documentation

- [POLICY_LIFECYCLE.md](POLICY_LIFECYCLE.md) - Policy governance and approval
- [EXEMPTION_WORKFLOW.md](EXEMPTION_WORKFLOW.md) - Exemption management
- [SLO_BURN_ALERTS.md](SLO_BURN_ALERTS.md) - SLO monitoring

## License

Part of CostPilot - Enterprise cost management suite
