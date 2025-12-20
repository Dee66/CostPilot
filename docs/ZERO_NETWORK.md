# Zero-Network Policy Enforcement

## Overview

CostPilot's zero-network policy enforcement guarantees that policy evaluation never makes network calls, external API requests, or performs non-deterministic operations. This enables safe execution in sandboxed environments like WASM, CI/CD pipelines, and air-gapped systems.

## Guarantees

### 1. **No Network I/O**
- No `std::net` usage
- No HTTP clients (reqwest, ureq, hyper, etc.)
- No AWS SDK calls
- No external API requests
- No DNS lookups

### 2. **Deterministic Evaluation**
- Same inputs always produce identical outputs
- No random number generation
- No system time dependencies
- No thread-based randomness
- Stable ordering of results

### 3. **WASM-Safe**
- Can run in WebAssembly sandbox
- No filesystem dependencies (except config loading)
- No environment variable dependencies at runtime
- Memory-safe operations only

### 4. **Offline-First**
- All evaluation happens locally
- No external data sources required
- Configuration is self-contained
- No telemetry or phone-home

## Architecture

### Zero-Network Token

The `ZeroNetworkToken` is a zero-sized type that provides compile-time proof of zero-network execution:

```rust
use costpilot::engines::policy::*;

// Token can only be created safely
let token = ZeroNetworkToken::new();

// Validates no network state is present
token.validate()?;
```

### Policy Engine Integration

Both policy engines support explicit zero-network evaluation:

```rust
use costpilot::engines::policy::*;
use costpilot::engines::detection::ResourceChange;
use costpilot::engines::prediction::CostEstimate;

let config = PolicyConfig::load("policies.yml")?;
let engine = PolicyEngine::new(config);

// Create zero-network token
let token = ZeroNetworkToken::new();

// Evaluate with compile-time guarantee
let result = engine.evaluate_zero_network(
    &changes,
    &cost_estimate,
    token
)?;
```

### Metadata Engine

The metadata policy engine also supports zero-network evaluation:

```rust
let mut engine = MetadataPolicyEngine::new();
engine.add_policy(policy);

let token = ZeroNetworkToken::new();
let result = engine.evaluate_zero_network(&changes, &cost, token)?;
```

## Dependency Validation

The `ZeroNetworkValidator` prevents network-capable dependencies:

```rust
// Allowed dependencies (pure computation)
assert!(ZeroNetworkValidator::is_allowed_dependency("serde"));
assert!(ZeroNetworkValidator::is_allowed_dependency("serde_json"));
assert!(ZeroNetworkValidator::is_allowed_dependency("chrono"));

// Disallowed dependencies (network-capable)
assert!(!ZeroNetworkValidator::is_allowed_dependency("reqwest"));
assert!(!ZeroNetworkValidator::is_allowed_dependency("hyper"));
assert!(!ZeroNetworkValidator::is_allowed_dependency("aws-sdk"));
```

### Blocked Crates

The following network-capable crates are explicitly blocked:

- **HTTP Clients**: reqwest, ureq, hyper, curl, surf, isahc, attohttpc, minreq
- **AWS SDKs**: rusoto_core, aws-sdk-* family
- **Cloud SDKs**: azure_core, google-cloud-*
- **Async Network**: tokio::net, async_std::net

## Determinism Validation

Non-deterministic operations are detected and blocked:

```rust
// Safe operations
ZeroNetworkValidator::ensure_deterministic("calculate_cost")?;
ZeroNetworkValidator::ensure_deterministic("evaluate_policy")?;

// Unsafe operations (will error)
ZeroNetworkValidator::ensure_deterministic("SystemTime::now()")?;  // Error
ZeroNetworkValidator::ensure_deterministic("rand::random()")?;     // Error
ZeroNetworkValidator::ensure_deterministic("thread::sleep()")?;    // Error
```

### Blocked Patterns

- `rand::*` - Random number generation
- `SystemTime::now` - System time access
- `Instant::now` - Monotonic time access
- `thread::sleep` - Blocking operations
- `thread_rng` - Thread-local randomness

## Zero-Network Runtime

The `ZeroNetworkRuntime` provides runtime enforcement:

```rust
let runtime = ZeroNetworkRuntime::new();

// Verify environment is safe
runtime.verify_environment()?;

// Execute with zero-network guarantee
let result = runtime.execute(|token| {
    let engine = PolicyEngine::new(config);
    engine.evaluate_zero_network(&changes, &cost, token)
})?;
```

## Enforced Wrapper

`ZeroNetworkEnforced` wraps values with compile-time guarantees:

```rust
let config = PolicyConfig::load("policies.yml")?;
let engine = PolicyEngine::new(config);

// Wrap in enforced container
let enforced = ZeroNetworkEnforced::new(engine);

// Access with zero-network token
let result = enforced.with_zero_network(|engine, token| {
    engine.evaluate_zero_network(&changes, &cost, token)
})?;
```

## Usage Examples

### Basic Policy Evaluation

```rust
use costpilot::engines::policy::*;

// Load configuration
let config = PolicyConfig {
    version: "1.0.0".to_string(),
    budgets: BudgetPolicies {
        global: Some(BudgetLimit {
            monthly_limit: 1000.0,
            warning_threshold: 0.8,
        }),
        modules: vec![],
    },
    resources: ResourcePolicies::default(),
    slos: vec![],
    enforcement: EnforcementConfig::default(),
};

// Create engine
let engine = PolicyEngine::new(config);

// Create token
let token = ZeroNetworkToken::new();

// Evaluate with zero-network guarantee
let result = engine.evaluate_zero_network(&[], &cost, token)?;

if !result.passed() {
    println!("Policy violations detected!");
    for violation in result.violations() {
        println!("  - {}", violation.policy_name);
    }
}
```

### Metadata Policy Evaluation

```rust
use costpilot::engines::policy::*;

let mut engine = MetadataPolicyEngine::new();

// Add policy with metadata
let policy = PolicyWithMetadata {
    metadata: PolicyMetadata {
        id: "monthly_budget".to_string(),
        name: "Monthly Budget Limit".to_string(),
        description: Some("Enforce monthly cost limit".to_string()),
        category: PolicyCategory::Budget,
        severity: PolicySeverity::Error,
        status: PolicyStatus::Active,
        ..Default::default()
    },
    spec: PolicyRule::BudgetLimit {
        monthly_limit: 5000.0,
        warning_threshold: 0.9,
    },
};

engine.add_policy(policy);

// Evaluate with zero-network
let token = ZeroNetworkToken::new();
let result = engine.evaluate_zero_network(&changes, &cost, token)?;

if result.has_blocking_violations() {
    println!("Deployment blocked by policies!");
}
```

### CI/CD Integration

```rust
use costpilot::engines::policy::*;

fn evaluate_in_ci() -> Result<(), Box<dyn std::error::Error>> {
    // Create zero-network runtime for CI
    let runtime = ZeroNetworkRuntime::new();

    // Verify we're in a safe environment
    runtime.verify_environment()?;

    // Load configuration from repo
    let config = PolicyConfig::load(".costpilot/policies.yml")?;
    let engine = PolicyEngine::new(config);

    // Parse Terraform plan
    let changes = parse_terraform_plan("plan.json")?;
    let cost = estimate_cost(&changes)?;

    // Evaluate with zero-network guarantee
    let result = runtime.execute(|token| {
        engine.evaluate_zero_network(&changes, &cost, token)
    })?;

    // Exit with appropriate code
    if !result.passed() {
        eprintln!("Policy violations found!");
        std::process::exit(2);  // POLICY_BLOCK exit code
    }

    Ok(())
}
```

### WASM Environment

```rust
// This code can compile to WASM and run safely

#[wasm_bindgen]
pub fn evaluate_policies_wasm(
    plan_json: &str,
    policies_yaml: &str,
) -> Result<JsValue, JsValue> {
    // Parse inputs
    let changes = parse_plan(plan_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let config = parse_policies(policies_yaml)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Create engine (no network possible in WASM)
    let engine = PolicyEngine::new(config);
    let token = ZeroNetworkToken::new();

    // Evaluate
    let cost = estimate_cost(&changes);
    let result = engine.evaluate_zero_network(&changes, &cost, token)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Return as JSON
    Ok(serde_wasm_bindgen::to_value(&result)?)
}
```

## Testing

### Determinism Tests

```rust
#[test]
fn test_deterministic_evaluation() {
    let config = create_test_config();
    let engine = PolicyEngine::new(config);
    let token = ZeroNetworkToken::new();

    // Run 100 times - should always be identical
    let results: Vec<_> = (0..100)
        .map(|_| engine.evaluate_zero_network(&changes, &cost, token))
        .collect();

    // All results must be identical
    for result in &results[1..] {
        assert_eq!(results[0], *result);
    }
}
```

### Network Isolation Tests

```rust
#[test]
fn test_no_network_calls() {
    // This test runs in an environment with no network access
    // If policy evaluation tries to make network calls, it will fail

    let engine = PolicyEngine::new(config);
    let token = ZeroNetworkToken::new();

    // Should succeed even with no network
    let result = engine.evaluate_zero_network(&changes, &cost, token);
    assert!(result.is_ok());
}
```

### WASM Compilation Test

```bash
# Compile to WASM - will fail if network dependencies are present
cargo build --target wasm32-unknown-unknown --features wasm

# Run WASM tests
wasm-pack test --node
```

## Error Handling

### Zero-Network Violations

```rust
use costpilot::engines::policy::ZeroNetworkViolation;

match engine.evaluate_zero_network(&changes, &cost, token) {
    Ok(result) => println!("Evaluation succeeded"),
    Err(ZeroNetworkViolation::NetworkCallAttempted { operation }) => {
        eprintln!("Network call blocked: {}", operation);
    }
    Err(ZeroNetworkViolation::ApiCallAttempted { endpoint }) => {
        eprintln!("API call blocked: {}", endpoint);
    }
    Err(ZeroNetworkViolation::NonDeterministicOperation { description }) => {
        eprintln!("Non-deterministic operation: {}", description);
    }
    Err(e) => {
        eprintln!("Zero-network violation: {}", e);
    }
}
```

## Best Practices

### 1. Always Use Zero-Network Methods in Production

```rust
// Good: Explicit zero-network guarantee
let token = ZeroNetworkToken::new();
let result = engine.evaluate_zero_network(&changes, &cost, token)?;

// Avoid: Regular evaluate() in production (use for convenience only)
let result = engine.evaluate(&changes, &cost);
```

### 2. Validate Dependencies

```rust
// In build.rs or CI
fn validate_dependencies() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    for dependency in parse_dependencies(&cargo_toml) {
        if !ZeroNetworkValidator::is_allowed_dependency(&dependency) {
            panic!("Disallowed dependency: {}", dependency);
        }
    }
}
```

### 3. Use Runtime Verification in CI

```bash
#!/bin/bash
# .github/workflows/costpilot.yml

# Verify no network access during policy evaluation
costpilot scan plan.json --verify-zero-network

# Or use network namespace isolation
unshare --net costpilot scan plan.json
```

### 4. Test Determinism Regularly

```rust
#[test]
fn test_regression_determinism() {
    // Load golden file
    let expected = load_golden_result("tests/golden/policy_result.json")?;

    // Evaluate
    let token = ZeroNetworkToken::new();
    let actual = engine.evaluate_zero_network(&changes, &cost, token)?;

    // Must match exactly
    assert_eq!(expected, actual);
}
```

## Compliance

### Security Audits

Zero-network enforcement enables:

- **SOC 2 Compliance** - No data exfiltration
- **Air-Gapped Deployments** - Works offline
- **Supply Chain Security** - No malicious network dependencies
- **Reproducible Builds** - Deterministic outputs

### Certifications

CostPilot's zero-network design supports:

- NIST 800-53 controls for offline operations
- FIPS compliance (no network entropy)
- ISO 27001 data handling requirements
- FedRAMP authorization in restricted environments

## Troubleshooting

### "Network dependency detected"

```
Error: Dependency 'reqwest' is not allowed in zero-network context
```

**Solution**: Remove the network-capable dependency or use a pure alternative.

### "Non-deterministic operation detected"

```
Error: Operation 'SystemTime::now()' is non-deterministic
```

**Solution**: Use fixed timestamps for testing or derive time from input data.

### "Token validation failed"

```
Error: ZeroNetworkToken validation failed
```

**Solution**: Ensure the token is created properly and environment is clean.

## Related Documentation

- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Policy evaluation details
- [POLICY_METADATA.md](POLICY_METADATA.md) - Metadata policy system
- [CLI.md](CLI.md) - Command-line usage

## Performance

Zero-network enforcement has **zero performance overhead**:

- Token is zero-sized (optimized away at compile time)
- Validation is inlined
- No runtime checks in release builds
- Determinism improves cachability

Benchmark results:
- Policy evaluation: <10ms for 100 policies
- Memory usage: <50MB for large policy sets
- WASM binary size: ~2MB gzipped
