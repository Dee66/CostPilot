# Drift-Safe Autofix (Beta)

## Overview

The Drift-Safe Autofix system provides automated cost optimization with comprehensive safety checks, drift detection, and automatic rollback capabilities. It ensures that automated fixes are applied only when safe, with full ability to revert changes if issues occur.

## Architecture

### Components

1. **drift_safe_types.rs** (672 lines)
   - Core data structures for safe operations
   - `DriftSafeOperation`: Complete operation with state tracking
   - `ResourceState`: Configuration snapshots with hashing
   - `SafetyCheck`: Individual safety validation
   - `RollbackPlan`: Automatic rollback steps
   - `DriftDetection`: Drift analysis results
   - `LogEntry`: Execution tracking
   - 11 comprehensive unit tests

2. **drift_safe_engine.rs** (492 lines)
   - Drift-safe autofix orchestration
   - `DriftSafeEngine`: Main engine with safety validation
   - Safety check execution (6 types)
   - Drift detection and impact assessment
   - Rollback execution
   - Policy and SLO integration
   - 10 unit tests for validation and execution

3. **mod.rs** (11 lines)
   - Module exports and public API

## Key Features

✅ **State Snapshots**: Configuration and cost snapshots before/after changes  
✅ **Drift Detection**: Identify configuration changes since snapshot  
✅ **6 Safety Checks**: No drift, resource exists, config hash, cost impact, policies, SLOs  
✅ **Automatic Rollback**: Revert to original state on failure  
✅ **Execution Logging**: Complete audit trail of operations  
✅ **Policy Integration**: Validate fixes against policy rules  
✅ **SLO Integration**: Ensure fixes don't violate budget limits  
✅ **Configuration Hashing**: Verify config integrity

## Safety Checks

### 1. No Drift Check
Verifies resource hasn't changed since the state snapshot was taken.

**Severity**: Blocks operation if major drift detected (>5 attributes changed)

### 2. Resource Exists Check
Confirms the target resource still exists in the infrastructure.

**Severity**: Blocks operation if resource missing

### 3. Config Hash Match
Verifies configuration hash matches the snapshot to detect tampering.

**Severity**: Blocks operation if hash mismatch

### 4. Cost Impact Acceptable
Validates that the fix provides expected cost reduction.

**Severity**: Blocks in strict mode if cost increases instead of decreases

### 5. No Policy Violations
Checks that the proposed fix doesn't violate any active policies.

**Severity**: Blocks operation if policy violations detected

### 6. No SLO Violations
Ensures the fix doesn't cause SLO budget violations.

**Severity**: Blocks operation if SLO violations detected

## Operation States

```
Pending → ValidatingSafety → Applying → Applied
                ↓                ↓
              Failed ← ─────────┘
                ↓
          RollingBack → RolledBack
                ↓
        RollbackFailed (manual intervention)
```

### State Descriptions

- **Pending**: Operation created, awaiting safety checks
- **ValidatingSafety**: Running all safety checks
- **Applying**: Applying the fix to infrastructure
- **Applied**: Successfully applied
- **Failed**: Application failed
- **RollingBack**: Executing rollback plan
- **RolledBack**: Successfully reverted to original state
- **RollbackFailed**: Rollback failed, manual intervention needed

## Drift Detection

### Drift Severity Levels

- **Minor** (1-2 attributes): Operation can proceed with warning
- **Moderate** (3-5 attributes): Caution advised, review recommended
- **Major** (6-10 attributes): Operation blocked by default
- **Critical** (>10 attributes): Operation blocked, immediate review needed

### Drift Impact Assessment

The engine automatically assesses the impact of detected drift:

**Cost Impact**: Changes to instance_type, size, capacity, volume_size  
**Security Impact**: Changes to security_groups, IAM roles, encryption, public access  
**Availability Impact**: Changes to availability_zone, multi_az, backup settings  

## Usage

### Creating a Drift-Safe Operation

```rust
use costpilot::engines::autofix_safe::DriftSafeEngine;

// Create engine
let engine = DriftSafeEngine::new()
    .with_policy_engine(policy_engine)
    .with_slo_manager(slo_manager)
    .set_strict_mode(true);

// Create operation from resource changes
let operation = engine.create_operation(
    "aws_instance.web_server".to_string(),
    "aws_instance".to_string(),
    "Downsize from t3.xlarge to t3.large".to_string(),
    &current_change,
    &proposed_fix,
);

println!("Created operation: {}", operation.id);
```

### Running Safety Checks

```rust
let mut operation = engine.create_operation(/* ... */);

match engine.run_safety_checks(&mut operation) {
    Ok(_) => {
        println!("✅ All safety checks passed");
        println!("Operation can proceed");
    }
    Err(e) => {
        eprintln!("❌ Safety checks failed: {}", e);
        for check in &operation.safety_checks {
            if check.status == CheckStatus::Failed {
                eprintln!("  - {}: {}", 
                    check.name, 
                    check.message.as_ref().unwrap()
                );
            }
        }
    }
}
```

### Applying the Fix

```rust
if operation.can_proceed() {
    match engine.apply_operation(&mut operation) {
        Ok(_) => {
            println!("✅ Fix applied successfully");
            println!("Cost reduced from ${:.2} to ${:.2}",
                operation.original_state.estimated_cost,
                operation.target_state.estimated_cost
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to apply fix: {}", e);
            // Automatic rollback triggered if auto_rollback is true
        }
    }
} else {
    eprintln!("❌ Operation cannot proceed - safety checks failed");
}
```

### Manual Rollback

```rust
// If something goes wrong, manually trigger rollback
if operation.status == OperationStatus::Failed {
    match engine.execute_rollback(&mut operation) {
        Ok(_) => {
            println!("✅ Rollback completed successfully");
            println!("Resource restored to original state");
        }
        Err(e) => {
            eprintln!("❌ Rollback failed: {}", e);
            eprintln!("Manual intervention required");
        }
    }
}
```

### Drift Detection

```rust
// Detect drift between expected and actual state
let drift = engine.detect_drift(
    &operation.original_state,
    &actual_current_config,
);

if drift.has_drift {
    println!("⚠️  Drift detected: {} attributes changed", 
        drift.drifted_attributes.len()
    );
    
    for attr in &drift.drifted_attributes {
        println!("  - {}: {:?} → {:?} ({})",
            attr.name,
            attr.expected,
            attr.actual,
            attr.impact
        );
    }
    
    if drift.is_blocking() {
        println!("❌ Drift severity is {}, operation blocked", 
            match drift.severity {
                DriftSeverity::Major => "MAJOR",
                DriftSeverity::Critical => "CRITICAL",
                _ => "UNKNOWN",
            }
        );
    }
}
```

## Rollback Plan

### Automatic Generation

Rollback plans are automatically generated when creating an operation:

```rust
pub struct RollbackPlan {
    pub steps: Vec<RollbackStep>,
    pub timeout_seconds: u32,
    pub auto_rollback: bool,
    pub status: RollbackStatus,
}
```

Each step includes:
- **Order**: Execution sequence
- **Description**: Human-readable explanation
- **Restore Config**: Configuration to apply
- **Verification**: Post-step validation

### Execution

Rollback is triggered automatically on failure (if `auto_rollback` is true) or manually:

```rust
// Automatic rollback on failure
operation.mark_failed("Application timeout".to_string());
// Rollback automatically triggered

// Manual rollback
engine.execute_rollback(&mut operation)?;
```

## Execution Logging

All operations are fully logged with timestamps and context:

```rust
// View execution log
for entry in &operation.execution_log {
    println!("[{}] {}: {}",
        entry.timestamp,
        match entry.level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRIT",
            LogLevel::Debug => "DEBUG",
        },
        entry.message
    );
}
```

Example log output:
```
[2024-01-15T14:30:00Z] INFO: Starting safety checks
[2024-01-15T14:30:01Z] INFO: All safety checks passed
[2024-01-15T14:30:02Z] INFO: Applying fix
[2024-01-15T14:30:05Z] INFO: Fix applied successfully
```

## Integration with Other Systems

### Policy Engine
Safety checks validate fixes against active policies:

```rust
let engine = DriftSafeEngine::new()
    .with_policy_engine(policy_engine);
```

### SLO Manager
Safety checks ensure fixes don't violate budget SLOs:

```rust
let engine = DriftSafeEngine::new()
    .with_slo_manager(slo_manager);
```

### Detection Engine
Drift-safe operations work with detection results:

```rust
let detections = detection_engine.detect(&changes)?;
for detection in detections {
    if let Some(fix) = generate_fix(&detection) {
        let operation = engine.create_operation(/* ... */);
        // Apply with safety checks
    }
}
```

## Configuration

### Strict Mode
In strict mode, any cost increase fails the cost_impact check:

```rust
let engine = DriftSafeEngine::new()
    .set_strict_mode(true);  // Default
```

Non-strict mode allows minor cost increases:

```rust
let engine = DriftSafeEngine::new()
    .set_strict_mode(false);
```

### Rollback Timeout
Configure maximum rollback execution time:

```rust
operation.rollback_plan.timeout_seconds = 300; // 5 minutes
```

Strict mode uses 180 seconds (3 minutes) by default.

### Auto-Rollback
Control whether rollback triggers automatically on failure:

```rust
operation.rollback_plan.auto_rollback = true; // Default
```

## Example Operation Lifecycle

```rust
// 1. Create operation
let mut operation = engine.create_operation(
    "aws_instance.web".to_string(),
    "aws_instance".to_string(),
    "Downsize to t3.large".to_string(),
    &current,
    &proposed,
);

// 2. Run safety checks
engine.run_safety_checks(&mut operation)?;

// 3. Apply fix
engine.apply_operation(&mut operation)?;

// 4. If failure occurs, rollback triggers automatically
// operation.status == OperationStatus::RolledBack

// 5. Review execution log
for entry in &operation.execution_log {
    println!("{:?}: {}", entry.level, entry.message);
}
```

## Safety Guarantees

1. **No Destructive Changes**: Original state always preserved
2. **Automatic Rollback**: Failures trigger immediate revert
3. **Drift Protection**: Operations blocked if infrastructure changed
4. **Policy Compliance**: Fixes validated against all policies
5. **Budget Protection**: SLO checks prevent budget violations
6. **Audit Trail**: Complete log of all actions
7. **Configuration Integrity**: Hash verification prevents tampering

## Limitations (Beta)

Current limitations that will be addressed in V1 release:

- **Simulated Application**: Actual infrastructure changes not yet implemented
- **No Cloud Provider Integration**: Requires external tool integration
- **No Parallel Operations**: Operations executed sequentially
- **Limited Drift Sources**: Only config-based drift detection
- **No Multi-Resource Operations**: One resource per operation

## Test Coverage

**drift_safe_types.rs**: 11 tests
- Operation creation and state tracking
- Safety check lifecycle (pending/passed/failed)
- Config hashing and verification
- Drift detection (no drift, minor, major, critical)
- Drift severity calculation
- Rollback triggering
- Execution logging

**drift_safe_engine.rs**: 10 tests
- Engine creation and configuration
- Operation creation with safety checks
- Drift detection (no drift, with drift)
- Safety check execution
- Operation application
- Rollback execution
- Drift impact assessment

**Total**: 21 tests

## Future Enhancements

### Planned for V1
- **Real Cloud Provider Integration**: AWS, Azure, GCP API integration
- **Parallel Operations**: Batch processing with dependency ordering
- **Multi-Resource Operations**: Atomic changes across multiple resources
- **Enhanced Drift Sources**: Cloud provider state, remote state backends
- **Canary Rollouts**: Gradual rollout with monitoring

### Planned for V2
- **Approval Workflow**: Multi-party approval for high-risk changes
- **Scheduled Operations**: Maintenance window scheduling
- **Progressive Rollback**: Partial rollback with validation
- **Machine Learning**: Predict operation success probability
- **Cost Forecasting**: Predict long-term impact of changes

## Related Documentation

- [AUTOFIX_ENGINE.md](AUTOFIX_ENGINE.md) - Basic autofix functionality
- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Policy validation
- [SLO_ENGINE.md](SLO_ENGINE.md) - SLO budget enforcement
- [examples/drift_safe_operation.json](../examples/drift_safe_operation.json) - Example operation

## API Reference

See inline documentation in source files:
- `src/engines/autofix_safe/drift_safe_types.rs`
- `src/engines/autofix_safe/drift_safe_engine.rs`
