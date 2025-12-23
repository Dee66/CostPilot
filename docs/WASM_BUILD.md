# WASM Build Configuration for CostPilot

This document describes the WebAssembly (WASM) compilation pipeline for CostPilot's core engines, enabling zero-IAM security through sandboxed execution.

## Overview

CostPilot compiles its core cost analysis engines to WebAssembly for:
- **Zero-IAM Security**: No AWS credentials or network access possible
- **Deterministic Execution**: Identical inputs always produce identical outputs
- **Sandboxed Safety**: Memory and execution time limits enforced
- **Cross-platform**: Run anywhere WASM is supported

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CostPilot CLI (Native)                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              WASM Runtime (wasmtime)                   │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │          Core Engines (WASM Module)             │  │  │
│  │  │                                                 │  │  │
│  │  │  • Prediction Engine                            │  │  │
│  │  │  • Detection Engine                             │  │  │
│  │  │  • Policy Engine                                │  │  │
│  │  │  • Mapping Engine                               │  │  │
│  │  │  • Grouping Engine                              │  │  │
│  │  │  • SLO Engine                                   │  │  │
│  │  │                                                 │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Build Targets

CostPilot supports two build targets:

### 1. Native Binary (default)
```bash
cargo build --release
```
- Full CLI functionality
- Native performance
- Development and CI/CD use

### 2. WASM Module
```bash
cargo build --target wasm32-unknown-unknown --release --lib
```
- Core engines only (no CLI)
- Sandboxed execution
- Browser or runtime embedding

## WASM Configuration

### Cargo.toml Configuration

The project includes WASM-specific configuration:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"

[target.'cfg(target_arch = "wasm32")'.dev-dependencies]
wasm-bindgen-test = "0.3"
```

### Sandbox Limits

WASM execution is constrained by hard limits:

| Limit | Value | Rationale |
|-------|-------|-----------|
| **Memory** | 256 MB | Sufficient for large Terraform plans |
| **Timeout** | 2000 ms | Total scan budget |
| **File Size** | 20 MB | Max plan JSON size |
| **Stack Size** | 1 MB | Deep recursion protection |

### Engine-Specific Budgets

| Engine | Time Budget | Memory Budget |
|--------|-------------|---------------|
| Prediction | 300 ms | 64 MB |
| Detection | 400 ms | 128 MB |
| Policy | 200 ms | 64 MB |
| Mapping | 500 ms | 128 MB |
| Grouping | 400 ms | 128 MB |
| SLO | 150 ms | 32 MB |

## Building for WASM

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (optional, for JS bindings)
cargo install wasm-bindgen-cli

# Install wasm-opt (optional, for optimization)
cargo install wasm-opt
```

### Build Commands

#### Basic WASM Build
```bash
cargo build --target wasm32-unknown-unknown --release --lib
```

Output: `target/wasm32-unknown-unknown/release/costpilot.wasm`

#### Optimized WASM Build
```bash
# Build
cargo build --target wasm32-unknown-unknown --release --lib

# Optimize (reduce size by ~50%)
wasm-opt -Oz -o costpilot_opt.wasm \
  target/wasm32-unknown-unknown/release/costpilot.wasm

# Generate JS bindings
wasm-bindgen target/wasm32-unknown-unknown/release/costpilot.wasm \
  --out-dir pkg \
  --target web
```

#### Build Script
```bash
#!/bin/bash
# scripts/build_wasm.sh

set -e

echo "Building CostPilot for WASM..."

# Clean previous builds
cargo clean

# Build WASM module
cargo build --target wasm32-unknown-unknown --release --lib

# Check output size
WASM_FILE="target/wasm32-unknown-unknown/release/costpilot.wasm"
SIZE=$(du -h "$WASM_FILE" | cut -f1)
echo "WASM module size: $SIZE"

# Validate size limit (10 MB unoptimized)
SIZE_BYTES=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE")
MAX_SIZE=$((10 * 1024 * 1024))

if [ "$SIZE_BYTES" -gt "$MAX_SIZE" ]; then
    echo "ERROR: WASM module exceeds 10 MB size limit"
    exit 1
fi

echo "WASM build successful!"
```

## WASM Runtime Integration

### Using wasmtime

```rust
use wasmtime::*;

fn load_costpilot_wasm() -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "costpilot.wasm")?;

    let mut store = Store::new(
        &engine,
        WasmState {
            memory_limit: 256 * 1024 * 1024, // 256 MB
        }
    );

    // Configure limits
    let mut config = Config::new();
    config.max_wasm_stack(1024 * 1024); // 1 MB stack
    config.epoch_interruption(true); // Enable timeout

    let instance = Instance::new(&mut store, &module, &[])?;

    // Call engine functions...
    Ok(())
}
```

### JavaScript Integration

```javascript
import init, { predict_cost, analyze_plan } from './pkg/costpilot.js';

async function runCostPilot() {
    await init();

    const plan = JSON.stringify({ /* Terraform plan */ });
    const result = predict_cost(plan);

    console.log('Predicted monthly cost:', result.monthly_cost);
}
```

## Determinism Guarantees

CostPilot ensures deterministic execution through:

### 1. No Random Number Generation
```rust
// ❌ FORBIDDEN
use rand::random;
let value = random::<f64>();

// ✅ ALLOWED
use sha2::{Sha256, Digest};
fn deterministic_hash(input: &str) -> u64 {
    let hash = Sha256::digest(input.as_bytes());
    u64::from_le_bytes(hash[..8].try_into().unwrap())
}
```

### 2. No System Time
```rust
// ❌ FORBIDDEN
use std::time::SystemTime;
let now = SystemTime::now();

// ✅ ALLOWED - Accept time as input
fn analyze_with_timestamp(plan: &Plan, timestamp: u64) -> Result<Analysis>
```

### 3. No Filesystem Access
```rust
// ❌ FORBIDDEN
use std::fs;
let content = fs::read_to_string("file.json")?;

// ✅ ALLOWED - Accept content as input
fn analyze_plan(plan_json: &str) -> Result<Analysis>
```

### 4. No Network Access
```rust
// ❌ FORBIDDEN - Compile error in WASM
use reqwest;
let response = reqwest::get("https://api.example.com").await?;

// ✅ ALLOWED - Data passed as input
fn analyze_with_pricing(plan: &Plan, pricing_data: &PricingData) -> Result<Analysis>
```

## Testing WASM Builds

### Unit Tests
```bash
# Run tests in WASM environment
cargo test --target wasm32-unknown-unknown
```

### Browser Tests
```bash
# Using wasm-bindgen-test
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

### Determinism Tests
```rust
#[test]
fn test_deterministic_output() {
    let plan = load_test_plan();

    let result1 = predict_cost(&plan);
    let result2 = predict_cost(&plan);

    assert_eq!(result1, result2, "Outputs must be identical");
}

#[test]
fn test_deterministic_across_runs() {
    let plan = load_test_plan();
    let expected_hash = "a1b2c3d4..."; // Known good hash

    let result = predict_cost(&plan);
    let actual_hash = hash_output(&result);

    assert_eq!(actual_hash, expected_hash);
}
```

### Memory Tests
```rust
#[test]
fn test_memory_limit() {
    let large_plan = generate_large_plan(10_000); // 10k resources

    // Should complete within 256 MB
    let result = predict_cost(&large_plan);
    assert!(result.is_ok());
}
```

### Performance Tests
```rust
#[test]
fn test_performance_budget() {
    let plan = load_test_plan();

    let start = Instant::now();
    let result = predict_cost(&plan);
    let duration = start.elapsed();

    assert!(duration.as_millis() < 300, "Prediction exceeded 300ms budget");
}
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: WASM Build

on: [push, pull_request]

jobs:
  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Build WASM
        run: cargo build --target wasm32-unknown-unknown --release --lib

      - name: Check WASM Size
        run: |
          SIZE=$(stat -c%s target/wasm32-unknown-unknown/release/costpilot.wasm)
          MAX=$((10 * 1024 * 1024))
          if [ $SIZE -gt $MAX ]; then
            echo "WASM size ($SIZE bytes) exceeds limit ($MAX bytes)"
            exit 1
          fi

      - name: Run WASM Tests
        run: cargo test --target wasm32-unknown-unknown

      - name: Upload WASM Artifact
        uses: actions/upload-artifact@v3
        with:
          name: costpilot-wasm
          path: target/wasm32-unknown-unknown/release/costpilot.wasm
```

## Optimization Strategies

### 1. Size Optimization

```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit
strip = true             # Strip symbols
panic = "abort"          # Smaller panic handler
```

### 2. Feature Flags

```toml
[features]
default = ["full"]
full = ["prediction", "mapping", "policy"]
prediction = []
mapping = []
policy = []
wasm = ["wasm-bindgen"]
```

Build minimal WASM:
```bash
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm,prediction
```

### 3. Dependency Optimization

- Use `wee_alloc` for smaller memory allocator
- Minimize external dependencies
- Use `#[cfg(not(target_arch = "wasm32"))]` for CLI-only code

## Security Considerations

### Memory Safety
- No unsafe code in core engines
- Bounds checking on all array accesses
- Validated parsing of untrusted input

### Resource Limits
- Hard memory cap (256 MB)
- Execution timeout (2 seconds)
- Stack overflow protection

### Input Validation
- JSON schema validation
- Maximum file size (20 MB)
- Recursive depth limits (32 levels)

## Troubleshooting

### Common Issues

**Error: "target not found"**
```bash
rustup target add wasm32-unknown-unknown
```

**Error: "wasm-bindgen not found"**
```bash
cargo install wasm-bindgen-cli
```

**WASM module too large**
- Enable LTO and size optimization
- Use `wasm-opt` for further compression
- Remove unused dependencies

**Determinism failures**
- Check for system time usage
- Check for random number generation
- Check for HashMap iteration (use BTreeMap)

## Future Enhancements

- [ ] WASI support for filesystem abstraction
- [ ] WASM component model integration
- [ ] Browser-based execution environment
- [ ] WASM streaming compilation
- [ ] Progressive loading for large plans

## References

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [wasmtime Documentation](https://docs.wasmtime.dev/)
- [WASM Spec](https://webassembly.github.io/spec/)

---

**Last Updated:** 2025-12-06
**Version:** 1.0.0
