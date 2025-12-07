# Enterprise Policy Lifecycle & Approval Flow

Complete governance system for managing policy lifecycle, multi-party approvals, versioning, and audit trails.

## Overview

The Enterprise Policy Lifecycle system provides enterprise-grade governance for cost policies through:

- **State Machine**: Six-state lifecycle (Draft → Review → Approved → Active → Deprecated → Archived)
- **Approval Workflows**: Multi-party approval with role-based access control
- **Version Control**: Full history tracking with semantic versioning and rollback
- **Audit Trail**: Complete change history with timestamps and actor attribution
- **CLI Integration**: Rich command-line interface for all lifecycle operations

## Architecture

### Components

**1. Policy Lifecycle** (`lifecycle.rs` - 440 lines, 12 tests)
- State machine with 6 states and validated transitions
- Approval tracking and validation
- Review expiration detection
- Lifecycle summary generation

**2. Approval Workflow Manager** (`approval_workflow.rs` - 440 lines, 11 tests)
- Multi-policy workflow management
- Role-based approver assignment
- Authorization validation
- Workflow statistics and reporting

**3. Policy History** (`policy_history.rs` - 450 lines, 10 tests)
- Semantic versioning (major.minor.patch)
- Content checksumming (SHA-256)
- Version diffing
- Rollback capability
- Tag-based version retrieval

**4. CLI Commands** (`policy_lifecycle.rs` - 550 lines)
- 8 comprehensive commands
- Multiple output formats (text, JSON)
- Rich colored terminal output

## Policy Lifecycle States

### State Diagram

```
Draft → Review → Approved → Active → Deprecated → Archived
  ↓       ↓         ↓                      ↓          
  └───────┴─────────┴──────────────────────┴────→ Archived
```

### State Descriptions

| State | Description | Editable | Enforceable | Next States |
|-------|-------------|----------|-------------|-------------|
| **Draft** | Being written/edited | ✅ Yes | ❌ No | Review, Archived |
| **Review** | Pending approval | ❌ No | ❌ No | Draft, Approved, Archived |
| **Approved** | Ready to activate | ❌ No | ❌ No | Active, Archived |
| **Active** | Currently enforced | ❌ No | ✅ Yes | Deprecated, Archived |
| **Deprecated** | Marked for removal | ❌ No | ✅ Yes | Archived, Active |
| **Archived** | No longer active | ❌ No | ❌ No | None (terminal) |

## Approval Workflow

### Configuration

```rust
use costpilot::engines::policy::{ApprovalConfig, ApprovalWorkflowManager};

// Create approval configuration
let config = ApprovalConfig {
    min_approvals: 2,
    required_roles: vec!["policy-approver".to_string()],
    allowed_approvers: vec![
        "alice@example.com".to_string(),
        "bob@example.com".to_string(),
    ],
    auto_approve_roles: vec!["admin".to_string()],
    review_expiration_days: 7,
};

// Create workflow manager
let mut manager = ApprovalWorkflowManager::with_config(config);

// Assign approvers to roles
manager.assign_role(
    "policy-approver".to_string(),
    vec!["alice@example.com".to_string(), "bob@example.com".to_string()],
);
```

### Approval Process

```rust
// 1. Register policy
manager.register_policy("cost-limit-policy".to_string(), None)?;

// 2. Submit for approval
let approvers = manager.submit_for_approval(
    "cost-limit-policy",
    "author@example.com".to_string(),
)?;

// 3. Record approvals
manager.approve(
    "cost-limit-policy",
    "alice@example.com".to_string(),
    Some("Looks good to me".to_string()),
)?;

manager.approve(
    "cost-limit-policy",
    "bob@example.com".to_string(),
    Some("Approved".to_string()),
)?;

// 4. Activate policy
let lifecycle = manager.get_lifecycle_mut("cost-limit-policy").unwrap();
lifecycle.transition(
    PolicyState::Approved,
    "alice@example.com".to_string(),
    None,
)?;

manager.activate_policy("cost-limit-policy", "admin@example.com".to_string())?;
```

## Version Control

### Creating Versions

```rust
use costpilot::engines::policy::{PolicyContent, PolicyHistory};
use serde_json::json;

// Create initial version
let content = PolicyContent {
    id: "nat-gateway-limit".to_string(),
    name: "NAT Gateway Limit Policy".to_string(),
    description: "Limit NAT gateways to reduce costs".to_string(),
    rules: json!({
        "resource_type": "aws_nat_gateway",
        "max_count": 2
    }),
    config: HashMap::new(),
};

let mut history = PolicyHistory::new(
    "nat-gateway-limit".to_string(),
    content,
    "author@example.com".to_string(),
);

// Add patch version
let updated_content = PolicyContent {
    id: "nat-gateway-limit".to_string(),
    name: "NAT Gateway Limit Policy".to_string(),
    description: "Updated description".to_string(),
    rules: json!({
        "resource_type": "aws_nat_gateway",
        "max_count": 3
    }),
    config: HashMap::new(),
};

let new_version = history.add_version(
    updated_content,
    "author@example.com".to_string(),
    "Increased limit to 3".to_string(),
    false, // Not a major version
)?;

assert_eq!(new_version, "1.0.1");
```

### Version Diffing

```rust
let diff = history.diff("1.0.0", "1.0.1")?;

if diff.rules_changed {
    println!("Rules were modified");
}

println!("Summary: {}", diff.summary());
// Output: "Changed: description, rules"
```

### Rollback

```rust
// Rollback to previous version
let rollback_version = history.rollback(
    "1.0.0".to_string(),
    "admin@example.com".to_string(),
    "Revert problematic changes".to_string(),
)?;

assert_eq!(rollback_version, "1.0.2");
```

## CLI Usage

### Submit Policy for Approval

```bash
costpilot policy submit \
  --policy .costpilot/policies/nat-gateway-limit.yml \
  --approvers alice@example.com,bob@example.com
```

**Output:**
```
📝 Submitting policy for approval...

✅ Policy submitted for approval

Policy ID: nat-gateway-limit
Status: Review

Approval requests sent to:
  • alice@example.com
  • bob@example.com

Next steps:
  1. Wait for approvers to review
  2. Check status: costpilot policy status nat-gateway-limit
  3. Activate when approved: costpilot policy activate nat-gateway-limit
```

### Approve a Policy

```bash
costpilot policy approve nat-gateway-limit \
  --approver alice@example.com \
  --comment "Looks good to me"
```

**Output:**
```
✅ Approving policy 'nat-gateway-limit'...

✅ Approval recorded

Policy ID: nat-gateway-limit
Approver: alice@example.com
Comment: Looks good to me

🎉 Policy has sufficient approvals!

Next step: costpilot policy activate nat-gateway-limit
```

### Reject a Policy

```bash
costpilot policy reject nat-gateway-limit \
  --approver bob@example.com \
  --reason "Limit is too high, should be 2 not 3"
```

**Output:**
```
❌ Rejecting policy 'nat-gateway-limit'...

❌ Policy rejected

Policy ID: nat-gateway-limit
Reviewer: bob@example.com
Reason: Limit is too high, should be 2 not 3

The policy has been sent back to draft status.
Author should address feedback and resubmit.
```

### Activate an Approved Policy

```bash
costpilot policy activate nat-gateway-limit --actor admin@example.com
```

**Output:**
```
🚀 Activating policy 'nat-gateway-limit'...

✅ Policy activated successfully!

Policy ID: nat-gateway-limit
Status: Active
Activated by: admin@example.com

The policy is now enforced in all evaluations.
```

### Deprecate an Active Policy

```bash
costpilot policy deprecate nat-gateway-limit \
  --actor admin@example.com \
  --reason "Replaced by v2 policy"
```

**Output:**
```
⚠️  Deprecating policy 'nat-gateway-limit'...

⚠️  Policy deprecated

Policy ID: nat-gateway-limit
Status: Deprecated
Reason: Replaced by v2 policy

The policy is still active but marked for removal.
Plan migration to replacement policy.
```

### Check Policy Status

```bash
costpilot policy status nat-gateway-limit
```

**Output:**
```
📊 Policy status for 'nat-gateway-limit'...

Policy Lifecycle Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Policy ID: nat-gateway-limit
Status: Active
Description: Policy is currently enforced in production

Permissions:
  Editable: ❌ No
  Enforceable: ✅ Yes

History:
  Transitions: 4
  Last change: 2024-12-06T10:30:00Z
```

### View Policy History

```bash
costpilot policy history nat-gateway-limit
```

**Output:**
```
📚 Policy history for 'nat-gateway-limit'...

Policy Version History
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Version 1.0.0
  Author: author@example.com
  Created: 2024-11-01T10:00:00Z
  Description: Initial policy version
  Tags: initial, production

  Version 1.0.1
  Author: author@example.com
  Created: 2024-11-15T14:30:00Z
  Description: Increased limit to 3

→ Version 1.0.2
  Author: admin@example.com
  Created: 2024-12-01T09:15:00Z
  Description: Rollback to 1.0.0: Revert problematic changes

Current version: 1.0.2
Total versions: 3
```

### Compare Policy Versions

```bash
costpilot policy diff nat-gateway-limit --from 1.0.0 --to 1.0.1
```

**Output:**
```
🔍 Comparing versions 1.0.0 → 1.0.1...

Version Diff
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

From: 1.0.0 (author@example.com)
To:   1.0.1 (author@example.com)

Changes:
  ✓ Description
  ✓ Rules

Summary: Changed: description, rules

Checksums:
  From: a1b2c3d4e5f6g7h8
  To:   x1y2z3w4v5u6t7s8
```

### JSON Output for CI/CD

All commands support `--format json`:

```bash
costpilot policy status nat-gateway-limit --format json
```

```json
{
  "policy_id": "nat-gateway-limit",
  "current_state": "active",
  "state_description": "Policy is currently enforced in production",
  "is_editable": false,
  "is_enforceable": true,
  "requires_approval": false,
  "approvals_received": 2,
  "approvals_required": 2,
  "has_rejections": false,
  "is_expired": false,
  "transition_count": 4,
  "last_transition": "2024-12-06T10:30:00Z"
}
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Policy Approval Workflow

on:
  pull_request:
    paths:
      - '.costpilot/policies/**'

jobs:
  policy-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install CostPilot
        run: cargo install costpilot
      
      - name: Detect Changed Policies
        id: policies
        run: |
          git diff --name-only origin/main... | \
            grep '.costpilot/policies/' | \
            tee policies.txt
      
      - name: Submit Policies for Approval
        run: |
          for policy in $(cat policies.txt); do
            costpilot policy submit \
              --policy $policy \
              --approvers policy-team@company.com,finops-lead@company.com
          done
      
      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '📝 Policies submitted for approval. Please review and approve.'
            });
```

### Approval Webhook

```yaml
name: Policy Approval

on:
  issue_comment:
    types: [created]

jobs:
  approve:
    if: contains(github.event.comment.body, '/approve-policy')
    runs-on: ubuntu-latest
    steps:
      - name: Extract Policy ID
        id: extract
        run: |
          POLICY_ID=$(echo "${{ github.event.comment.body }}" | grep -oP '/approve-policy \K\S+')
          echo "policy_id=$POLICY_ID" >> $GITHUB_OUTPUT
      
      - name: Approve Policy
        run: |
          costpilot policy approve ${{ steps.extract.outputs.policy_id }} \
            --approver ${{ github.actor }}@github.com \
            --comment "Approved via GitHub comment"
```

## Best Practices

### 1. Approval Configuration

**Minimum Requirements:**
- At least 2 approvers for production policies
- Separate roles for different policy categories (budget, security, compliance)
- Expiration period: 7 days for standard policies, 3 days for urgent

**Example:**
```rust
let config = ApprovalConfig {
    min_approvals: 2,
    required_roles: vec!["policy-lead".to_string(), "finops-manager".to_string()],
    review_expiration_days: 7,
    ..Default::default()
};
```

### 2. Version Management

**Semantic Versioning:**
- **Major (X.0.0)**: Breaking changes, complete policy rewrites
- **Minor (1.X.0)**: New rules or significant modifications
- **Patch (1.0.X)**: Bug fixes, threshold adjustments

**Tagging:**
```rust
history.tag_version("1.0.0", "production".to_string())?;
history.tag_version("1.1.0", "staging".to_string())?;
history.tag_version("2.0.0", "production".to_string())?;
```

### 3. Audit Trail

**Track Everything:**
- All state transitions with actor and timestamp
- Approval decisions with comments
- Version changes with change descriptions
- Rejections with detailed reasoning

**Query Audit Trail:**
```rust
let lifecycle = manager.get_lifecycle("policy-id").unwrap();
for transition in &lifecycle.state_history {
    println!("{} → {} by {} at {}",
        transition.from_state,
        transition.to_state,
        transition.actor,
        transition.timestamp
    );
}
```

### 4. Delegation & Ownership

**Role-Based Structure:**
```rust
// Team-specific roles
manager.assign_role("budget-policy-approver", vec!["finops-lead".to_string()]);
manager.assign_role("security-policy-approver", vec!["security-lead".to_string()]);
manager.assign_role("compliance-policy-approver", vec!["compliance-officer".to_string()]);

// Separate approval configs by category
let budget_config = ApprovalConfig {
    min_approvals: 2,
    required_roles: vec!["budget-policy-approver".to_string()],
    ..Default::default()
};
```

## Error Handling

### Common Errors

**Invalid Transition:**
```
Error: Invalid state transition from Draft to Active
```
**Solution:** Follow proper workflow: Draft → Review → Approved → Active

**Insufficient Approvals:**
```
Error: Insufficient approvals: required 2, received 1
```
**Solution:** Wait for additional approvals before transitioning to Approved

**Unauthorized Approver:**
```
Error: Approver not authorized: john@example.com
```
**Solution:** Ensure approver is in required_roles or allowed_approvers list

**No Changes:**
```
Error: No changes detected in policy content
```
**Solution:** Modify policy content before creating new version

## Related Documentation

- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Core policy evaluation
- [POLICY_METADATA.md](POLICY_METADATA.md) - Metadata system
- [CLI.md](CLI.md) - Full CLI reference

## License

Part of CostPilot - Enterprise cost management suite
