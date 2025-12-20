# Policy Metadata System

## Overview

The Policy Metadata System provides comprehensive policy management and governance capabilities for CostPilot. It extends the basic policy engine with rich metadata, lifecycle management, ownership tracking, metrics collection, and organizational features.

## Architecture

### Components

1. **policy_metadata.rs** (658 lines)
   - Core metadata types and structures
   - Policy lifecycle management
   - Ownership and responsibility tracking
   - Metrics collection and reporting
   - 18 comprehensive unit tests

2. **policy_repository.rs** (579 lines)
   - Policy storage and retrieval
   - Filtering and querying capabilities
   - Statistics and analytics
   - Bulk operations
   - 16 unit tests

3. **metadata_engine.rs** (525 lines)
   - Enhanced policy evaluation with metadata
   - Automatic metrics tracking
   - Integration with legacy policy system
   - Repository management
   - 8 unit tests

**Total**: 1,762 lines code + 42 tests

## Key Features

✅ **Rich Metadata** - ID, name, description, category, severity, status, version
✅ **Ownership Tracking** - Author, owner, team, contact, reviewers
✅ **Lifecycle Management** - Creation, updates, effective dates, deprecation
✅ **Status Management** - Draft, Active, Disabled, Deprecated, Archived
✅ **Severity Levels** - Info, Warning, Error, Critical with blocking logic
✅ **Categories** - Budget, Resource, Security, Governance, Performance, SLO
✅ **Tagging System** - Flexible tags for search and organization
✅ **Documentation Links** - Runbooks, tickets, related resources
✅ **Metrics Tracking** - Evaluations, violations, exemptions, rates
✅ **Version History** - Complete revision tracking
✅ **Deprecation Support** - Planned obsolescence with replacements
✅ **Repository Management** - Filtering, search, bulk operations, statistics

## Metadata Structure

### PolicyMetadata

Complete metadata for a policy:

```rust
pub struct PolicyMetadata {
    pub id: String,                    // Unique identifier
    pub name: String,                  // Human-readable name
    pub description: String,           // Detailed purpose
    pub category: PolicyCategory,      // Organizational category
    pub severity: Severity,            // Violation severity
    pub status: PolicyStatus,          // Current lifecycle status
    pub version: String,               // Version number
    pub ownership: PolicyOwnership,    // Who owns/created
    pub lifecycle: PolicyLifecycle,    // Dates and history
    pub tags: HashSet<String>,         // Search tags
    pub links: PolicyLinks,            // Documentation
    pub metrics: PolicyMetrics,        // Execution stats
    pub custom: HashMap<String, String>, // Custom fields
}
```

### Categories

Policies are organized into categories:

- **Budget**: Cost limits and spending controls
- **Resource**: Resource count and type restrictions
- **Security**: Security and compliance rules
- **Governance**: Tagging and naming conventions
- **Performance**: Performance and optimization
- **SLO**: Service Level Objectives
- **Environment**: Environment-specific policies
- **Custom(String)**: Custom categories

### Severity Levels

Four severity levels with automatic blocking logic:

| Severity | Score | Blocking | Use Case |
|----------|-------|----------|----------|
| Info | 1 | No | Informational only |
| Warning | 2 | No | Should be reviewed |
| Error | 3 | Yes | Must be fixed |
| Critical | 4 | Yes | Immediate attention |

```rust
// Check if severity blocks deployment
if policy.metadata.severity.is_blocking() {
    // Error or Critical - blocks deployment
}
```

### Status Lifecycle

Policies move through status states:

```
Draft → Active → Disabled
         ↓
    Deprecated → Archived
```

**Draft**: Policy created but not enforced
**Active**: Policy is enforced
**Disabled**: Temporarily disabled
**Deprecated**: Marked for removal, has replacement
**Archived**: Historical record only

### Ownership

Track responsibility and accountability:

```rust
pub struct PolicyOwnership {
    pub author: String,           // Who created it
    pub owner: String,            // Who maintains it
    pub team: Option<String>,     // Team responsible
    pub contact: Option<String>,  // Contact email
    pub reviewers: Vec<String>,   // Who approved it
}
```

### Lifecycle Management

Comprehensive lifecycle tracking:

```rust
pub struct PolicyLifecycle {
    pub created_at: DateTime<Utc>,              // Creation time
    pub updated_at: DateTime<Utc>,              // Last modification
    pub effective_from: Option<DateTime<Utc>>,  // Start date
    pub effective_until: Option<DateTime<Utc>>, // End date
    pub deprecation: Option<DeprecationInfo>,   // If deprecated
    pub revisions: Vec<PolicyRevision>,         // Change history
}
```

### Deprecation

When policies are phased out:

```rust
pub struct DeprecationInfo {
    pub deprecated_at: DateTime<Utc>,
    pub reason: String,
    pub replacement_policy_id: Option<String>,  // New policy
    pub migration_guide: Option<String>,        // How to migrate
}
```

### Metrics

Automatic tracking of policy effectiveness:

```rust
pub struct PolicyMetrics {
    pub evaluation_count: u64,              // Times evaluated
    pub violation_count: u64,               // Times violated
    pub exemption_count: u64,               // Exemptions granted
    pub last_evaluated: Option<DateTime<Utc>>,
    pub last_violation: Option<DateTime<Utc>>,
    pub violation_rate: Option<f64>,        // Auto-calculated
}
```

## Usage

### Creating a Policy with Metadata

```rust
use costpilot::engines::policy::*;

// Create metadata
let metadata = PolicyMetadata::new(
    "global-budget".to_string(),
    "Global Monthly Budget".to_string(),
    "Enforces org-wide monthly spending limit".to_string(),
    PolicyCategory::Budget,
    Severity::Critical,
    "alice".to_string(),    // author
    "finops-team".to_string(),  // owner
);

// Create policy rule
let rule = PolicyRule::BudgetLimit {
    monthly_limit: 6000.0,
    warning_threshold: 0.8,
};

// Combine into policy
let policy = PolicyWithMetadata::new(metadata, rule);
```

### Activating a Policy

```rust
let mut metadata = PolicyMetadata::new(/* ... */);

// Policy starts as Draft
assert_eq!(metadata.status, PolicyStatus::Draft);

// Activate to enforce
metadata.activate();
assert_eq!(metadata.status, PolicyStatus::Active);

// Disable temporarily
metadata.disable();

// Deprecate when replacing
metadata.deprecate(
    "Replaced by new-policy".to_string(),
    Some("new-policy-id".to_string()),
);
```

### Adding Tags and Links

```rust
// Add tags for organization
metadata.add_tag("production".to_string());
metadata.add_tag("critical".to_string());
metadata.add_tags(vec![
    "finops".to_string(),
    "compliance".to_string(),
]);

// Add documentation links
metadata.links.documentation = Some(
    "https://wiki.company.com/policies/global-budget".to_string()
);
metadata.links.runbook = Some(
    "https://runbooks.company.com/budget-exceeded".to_string()
);
```

### Tracking Revisions

```rust
// Record a policy change
metadata.add_revision(
    "bob".to_string(),
    "Increased limit from $5000 to $6000 based on Q2 growth".to_string(),
);

// Version history is maintained
for revision in &metadata.lifecycle.revisions {
    println!("{}: {} by {}",
        revision.version,
        revision.changes,
        revision.author
    );
}
```

## Policy Repository

### Creating and Managing Repository

```rust
use costpilot::engines::policy::*;

// Create repository
let mut repo = PolicyRepository::new();

// Add policies
repo.add(policy1)?;
repo.add(policy2)?;
repo.add(policy3)?;

// Get by ID
let policy = repo.get("global-budget");

// Count policies
println!("Total policies: {}", repo.count());
```

### Filtering and Querying

```rust
// Get all active, enforceable policies
let active = repo.get_enforceable();

// Get by status
let drafts = repo.get_by_status(&PolicyStatus::Draft);
let active = repo.get_by_status(&PolicyStatus::Active);

// Get by category
let budget_policies = repo.get_by_category(&PolicyCategory::Budget);
let security_policies = repo.get_by_category(&PolicyCategory::Security);

// Get by severity
let critical = repo.get_by_severity(&Severity::Critical);
let high_severity = repo.get_by_min_severity(&Severity::Error);

// Get blocking policies
let blocking = repo.get_blocking();

// Get by tag
let production = repo.get_by_tag("production");

// Get by owner or team
let my_policies = repo.get_by_owner("alice");
let team_policies = repo.get_by_team("finops");

// Search by name/description
let results = repo.search("budget");
```

### Statistics and Analytics

```rust
// Get comprehensive statistics
let stats = repo.statistics();

println!("Total: {}", stats.total_policies);
println!("Active: {}", stats.by_status[&PolicyStatus::Active]);
println!("Violations: {}", stats.total_violations);
println!("Rate: {:.2}%", stats.overall_violation_rate().unwrap() * 100.0);

// Get high violation policies
let problematic = repo.get_high_violation_policies(0.5); // >50% violation rate

// Get never evaluated policies
let unused = repo.get_never_evaluated();
```

### Bulk Operations

```rust
// Activate multiple policies
repo.activate_policies(&[
    "policy-1".to_string(),
    "policy-2".to_string(),
    "policy-3".to_string(),
])?;

// Disable multiple policies
repo.disable_policies(&[
    "policy-4".to_string(),
    "policy-5".to_string(),
])?;

// Archive old deprecated policies
let archived = repo.archive_deprecated(90); // older than 90 days
println!("Archived {} old policies", archived);
```

## Metadata Policy Engine

### Creating the Engine

```rust
use costpilot::engines::policy::*;

// Create new engine
let mut engine = MetadataPolicyEngine::new();

// Or from legacy config (backward compatible)
let legacy_config = PolicyConfig { /* ... */ };
let engine = MetadataPolicyEngine::from_legacy_config(legacy_config);
```

### Adding and Managing Policies

```rust
// Add a policy
let policy = PolicyWithMetadata::new(metadata, rule);
engine.add_policy(policy)?;

// Activate by ID
engine.activate_policy("global-budget")?;

// Disable by ID
engine.disable_policy("old-policy")?;

// Access repository directly
let repo = engine.repository();
let all_policies = repo.all();
```

### Evaluating Policies

```rust
// Evaluate all enforceable policies
let result = engine.evaluate(&changes, &total_cost);

// Check for violations
if result.has_violations() {
    println!("Found {} violations", result.violations.len());
}

// Check for blocking violations
if result.has_blocking_violations() {
    let blocking = result.blocking_violations();
    for violation in blocking {
        println!("BLOCKING: {} - {}",
            violation.policy_name,
            violation.message
        );
    }
}

// Filter by severity
let critical = result.by_severity(&Severity::Critical);
let warnings = result.by_severity(&Severity::Warning);

// Filter by category
let budget_violations = result.by_category(&PolicyCategory::Budget);
```

### Metrics and Reporting

```rust
// Metrics are automatically tracked
let result = engine.evaluate(&changes, &cost);

// Get updated metrics
let policy = engine.repository().get("global-budget").unwrap();
println!("Evaluations: {}", policy.metadata.metrics.evaluation_count);
println!("Violations: {}", policy.metadata.metrics.violation_count);
println!("Rate: {:.2}%",
    policy.metadata.metrics.violation_rate.unwrap() * 100.0
);

// Get engine-wide statistics
let stats = engine.statistics();
println!("{}", stats.format());

// Find problematic policies
let high_violation = engine.high_violation_policies(0.3); // >30% rate
```

## Example Policy JSON

See `examples/policy_with_metadata.json` for complete examples showing:

- **Global Budget**: Critical severity, active, tracked metrics
- **Module Budget**: Error severity, VPC-specific
- **Resource Limits**: Warning severity, NAT gateway restrictions
- **Governance**: S3 lifecycle requirements
- **Deprecated Policy**: Shows complete deprecation workflow

Each example includes:
- Complete metadata with ownership
- Revision history
- Tags and links
- Real metrics
- Custom fields

## Integration with Existing Systems

### Backward Compatibility

The metadata system maintains full backward compatibility:

```rust
// Legacy system still works
let legacy_engine = PolicyEngine::new(config);
let result = legacy_engine.evaluate(&changes, &cost);

// New metadata system
let metadata_engine = MetadataPolicyEngine::from_legacy_config(config);
let result = metadata_engine.evaluate(&changes, &cost);
```

### Migration Path

Existing policies automatically get metadata:

```rust
// Load legacy config
let config = PolicyLoader::load_from_file(".costpilot/policy.yml")?;

// Convert to metadata system
let engine = MetadataPolicyEngine::from_legacy_config(config);

// All policies now have metadata
let repo = engine.repository();
for policy in repo.all() {
    println!("{}: {} ({})",
        policy.metadata.id,
        policy.metadata.name,
        policy.metadata.status
    );
}
```

### Exemption System

Metadata policies work with existing exemptions:

```rust
// Exemptions still apply
let exemptions = ExemptionsFile::load(".costpilot/exemptions.yml")?;

// Policy violations can be exempted
// (Exemption system operates at evaluation level)
```

### SLO Integration

Metadata policies complement SLO system:

```rust
// Policies for immediate blocking
let policy_result = policy_engine.evaluate(&changes, &cost);

// SLOs for budget tracking
let slo_result = slo_manager.evaluate_snapshot(&snapshot);

// Combined governance
if policy_result.has_blocking_violations() {
    // Policy block
} else if slo_manager.should_block_deployment(&slo_result) {
    // SLO block
}
```

## Best Practices

### Policy Metadata

1. **Use Descriptive IDs**: `global-budget` not `policy-001`
2. **Write Clear Descriptions**: Explain why policy exists
3. **Choose Appropriate Severity**:
   - Info: FYI only
   - Warning: Should fix
   - Error: Must fix before deploy
   - Critical: Blocks immediately
4. **Set Ownership**: Always assign owner and team
5. **Tag Consistently**: Use standard tags (production, dev, critical, etc.)

### Lifecycle Management

1. **Start as Draft**: Test policies before activating
2. **Use Effective Dates**: Schedule policy activation
3. **Track Revisions**: Document all changes
4. **Deprecate Gracefully**: Provide replacement and migration guide
5. **Archive Old Policies**: Clean up after 90 days deprecated

### Organization

1. **Categorize Correctly**: Use standard categories
2. **Group Related Policies**: Use tags to create logical groups
3. **Link Documentation**: Always add runbook and docs
4. **Monitor Metrics**: Track violation rates
5. **Review Regularly**: Check unused or problematic policies

### Performance

1. **Limit Active Policies**: Archive unused policies
2. **Use Appropriate Severity**: Don't make everything Critical
3. **Batch Operations**: Use bulk activate/disable
4. **Filter Before Iterating**: Use repository queries
5. **Monitor Evaluation Rates**: Identify slow policies

## Test Coverage

**policy_metadata.rs**: 18 tests
- Severity ordering and blocking logic
- Status enforcement and lifecycle
- Lifecycle effective date checking
- Metadata creation and activation
- Deprecation workflow
- Tag management
- Metrics recording and rate calculation
- PolicyWithMetadata enforcement logic

**policy_repository.rs**: 16 tests
- Repository creation and basic CRUD
- Duplicate prevention
- Filtering by status, category, severity
- Minimum severity filtering
- Blocking policy retrieval
- Tag-based filtering
- Search functionality
- Statistics calculation
- Bulk activation/disabling

**metadata_engine.rs**: 8 tests
- Engine creation and initialization
- Legacy config import
- Policy addition and activation
- Budget policy evaluation
- Violation detection and blocking
- Metrics tracking during evaluation

**Total**: 42 comprehensive tests

## API Reference

### Core Types

- `PolicyMetadata` - Complete policy metadata
- `PolicyCategory` - Categorization enum
- `Severity` - Violation severity levels
- `PolicyStatus` - Lifecycle status
- `PolicyOwnership` - Ownership tracking
- `PolicyLifecycle` - Dates and history
- `PolicyMetrics` - Execution metrics
- `PolicyWithMetadata<T>` - Policy with metadata wrapper

### Repository

- `PolicyRepository<T>` - Policy storage and management
- `RepositoryStatistics` - Aggregate statistics

### Engine

- `MetadataPolicyEngine` - Enhanced policy evaluation
- `PolicyRule` - Policy rule specifications
- `MetadataPolicyResult` - Evaluation results
- `MetadataPolicyViolation` - Violation with metadata

## Future Enhancements

### Planned for V2

- **Policy Inheritance**: Child policies inherit from parents
- **Policy Templates**: Reusable policy templates
- **Approval Workflow**: Multi-party approval for activation
- **Scheduled Activation**: Cron-like scheduling
- **Dynamic Thresholds**: Adjust limits based on metrics
- **Machine Learning**: Predict policy effectiveness

### Planned for Enterprise

- **RBAC Integration**: Role-based policy management
- **Audit Trail**: Complete change history with diffs
- **Policy Compliance Reporting**: Automated compliance reports
- **Slack/Email Integration**: Notification on violations
- **Dashboard**: Web UI for policy management
- **GraphQL API**: Query policies via GraphQL

## Related Documentation

- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Basic policy engine
- [EXEMPTIONS.md](EXEMPTIONS.md) - Exemption system
- [SLO_ENGINE.md](SLO_ENGINE.md) - SLO enforcement
- [examples/policy_with_metadata.json](../examples/policy_with_metadata.json) - Example policies

## Migration Guide

### From Basic to Metadata System

```rust
// Before: Basic policy engine
let config = PolicyLoader::load_from_file(".costpilot/policy.yml")?;
let engine = PolicyEngine::new(config);

// After: Metadata policy engine
let config = PolicyLoader::load_from_file(".costpilot/policy.yml")?;
let mut engine = MetadataPolicyEngine::from_legacy_config(config);

// Now you can use metadata features
let stats = engine.statistics();
let active = engine.repository().get_enforceable();
```

### Adding Metadata to Existing Policies

1. Load legacy config
2. Create metadata engine
3. Access policies via repository
4. Enhance with tags, links, custom fields
5. Save to new format (JSON)

See migration script in `scripts/migrate_policies.py` (planned).
