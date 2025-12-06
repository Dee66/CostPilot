# Execution Determinism Contract

**Version:** 1.0.0  
**Status:** Enforced  
**Last Updated:** 2025-12-06

---

## Overview

CostPilot guarantees **byte-for-byte identical outputs** across all supported platforms, architectures, and CI environments. This contract elevates CostPilot from "trustworthy" to "research-grade consistent."

---

## Invariants

### 1. No Entropy Sources Allowed
```rust
// ❌ FORBIDDEN
use rand::random;
let value = random::<f64>();

// ✅ REQUIRED
use sha2::{Sha256, Digest};
let mut hasher = Sha256::new();
hasher.update(deterministic_input);
let hash = hasher.finalize();
```

**Enforcement:**
- Dependency validation blocks `rand`, `getrandom`, `fastrand`
- CI checks for entropy-generating imports
- WASM sandbox prevents `crypto.getRandomValues()`

### 2. No Thread Non-Determinism
```rust
// ❌ FORBIDDEN
use rayon::prelude::*;
resources.par_iter().map(predict_cost) // Non-deterministic iteration order

// ✅ REQUIRED
use rayon::prelude::*;
resources
    .iter()
    .enumerate()
    .collect::<Vec<_>>()
    .par_iter()
    .map(|(idx, res)| (idx, predict_cost(res)))
    .collect::<BTreeMap<_, _>>()  // Stable ordering
    .into_iter()
    .map(|(_, result)| result)
    .collect()
```

**Enforcement:**
- All parallel operations must use indexed iteration
- Results collected into `BTreeMap` for stable ordering
- Tests verify worker count doesn't affect output

### 3. Stable Float Math (IEEE 754 Normalization)
```rust
// ✅ REQUIRED
fn normalize_float(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else if value.is_infinite() {
        if value.is_sign_positive() { f64::MAX } else { f64::MIN }
    } else {
        value
    }
}

// Always use for cost calculations
let cost = normalize_float(price * quantity);
```

**Enforcement:**
- All float operations wrapped in normalization
- NaN/Inf converted to deterministic values
- Rounding mode fixed to nearest-even
- Tests verify cross-platform float stability

### 4. Stable Key Sorting Across JSON Emitters
```rust
// ❌ FORBIDDEN
use std::collections::HashMap;
let map: HashMap<String, f64> = costs;
serde_json::to_string(&map)  // Non-deterministic ordering

// ✅ REQUIRED
use std::collections::BTreeMap;
let map: BTreeMap<String, f64> = costs;
serde_json::to_string_pretty(&map)  // Stable alphabetical ordering
```

**Enforcement:**
- `HashMap` banned in serializable structs
- `BTreeMap` required for all key-value data
- Custom serializers enforce key sorting
- Tests verify JSON byte-for-byte equality

### 5. Markdown Wrapping Rules Fixed
```yaml
markdown_rules:
  wrap_column: 80
  wrap_mode: word_boundary
  preserve_code_blocks: true
  preserve_lists: true
  preserve_tables: true
```

**Enforcement:**
- All markdown generators use `textwrap` crate with fixed width
- Code blocks never wrapped
- Tests verify consistent line breaks

### 6. Newline Convention Normalized to LF
```rust
// ✅ REQUIRED
const NEWLINE: &str = "\n";  // Always LF, never CRLF

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace("\r", "\n")
}
```

**Enforcement:**
- All file writes use LF
- Input normalization on read
- Git `.gitattributes` enforces LF
- Tests verify no CRLF in outputs

### 7. Error Messages Must Be Deterministic
```rust
// ❌ FORBIDDEN
format!("Error at {:?}", Instant::now())
format!("Thread {} failed", thread::current().id())

// ✅ REQUIRED
format!("Error in resource {}", resource_address)
format!("Validation failed: {}", error_code)
```

**Enforcement:**
- No timestamps in error messages
- No thread IDs in error messages
- No memory addresses in error messages
- Stable error templates only

---

## Required Tests

### Cross-Platform Snapshot Test
```rust
#[test]
fn test_cross_platform_determinism() {
    let plan = load_fixture("fixtures/terraform/ec2_create.json");
    let result = full_scan(&plan);
    
    // Generate hash of complete output
    let hash = sha256_hash(&result);
    
    // Hash must match across Linux/Mac/Windows/WASM
    assert_eq!(hash, EXPECTED_HASH);
    
    // Snapshot must match byte-for-byte
    insta::assert_json_snapshot!(result);
}
```

### Float Stability Test
```rust
#[test]
fn test_float_determinism_across_platforms() {
    let test_values = vec![
        (0.1 + 0.2, 0.3),
        (1.0 / 3.0, 0.333333333333333333),
        (f64::MAX, f64::MAX),
        (f64::NAN, 0.0),  // NaN normalized to 0
        (f64::INFINITY, f64::MAX),
    ];
    
    for (input, expected) in test_values {
        let normalized = normalize_float(input);
        assert_eq!(normalized, expected);
    }
}

#[test]
fn test_float_math_consistency() {
    // Same calculation must produce identical results
    let price_per_hour = 0.0416;  // t3.medium
    let hours_per_month = 730.0;
    
    let cost1 = normalize_float(price_per_hour * hours_per_month);
    let cost2 = normalize_float(price_per_hour * hours_per_month);
    
    assert_eq!(cost1, cost2);
    assert_eq!(format!("{:.2}", cost1), "30.37");
}
```

### Deterministic Parallel Executor Test
```rust
#[test]
fn test_parallel_execution_determinism() {
    let plan = generate_large_terraform_plan(1000);
    
    // Run with different worker counts
    let result_1_worker = scan_with_workers(&plan, 1);
    let result_2_workers = scan_with_workers(&plan, 2);
    let result_8_workers = scan_with_workers(&plan, 8);
    
    // All must be identical
    assert_eq!(result_1_worker, result_2_workers);
    assert_eq!(result_2_workers, result_8_workers);
    
    // Hash verification
    let hash1 = sha256_hash(&result_1_worker);
    let hash2 = sha256_hash(&result_2_workers);
    let hash8 = sha256_hash(&result_8_workers);
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash8);
}
```

### JSON Key Ordering Test
```rust
#[test]
fn test_json_output_stable_ordering() {
    let result = scan_plan(&sample_plan());
    let json1 = serde_json::to_string(&result).unwrap();
    let json2 = serde_json::to_string(&result).unwrap();
    
    // Must be byte-for-byte identical
    assert_eq!(json1, json2);
    
    // Keys must be alphabetically ordered
    let parsed: serde_json::Value = serde_json::from_str(&json1).unwrap();
    if let Some(obj) = parsed.as_object() {
        let keys: Vec<&String> = obj.keys().collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys, "Keys must be alphabetically sorted");
    }
}
```

### Markdown Consistency Test
```rust
#[test]
fn test_markdown_output_determinism() {
    let explanation = explain_prediction(&prediction);
    let markdown1 = format_markdown(&explanation);
    let markdown2 = format_markdown(&explanation);
    
    // Byte-for-byte identical
    assert_eq!(markdown1, markdown2);
    
    // All lines ≤ 80 chars (except code blocks)
    for line in markdown1.lines() {
        if !line.trim_start().starts_with("```") {
            assert!(line.len() <= 80, "Line exceeds 80 chars: {}", line);
        }
    }
    
    // Only LF newlines
    assert!(!markdown1.contains("\r\n"), "Found CRLF in output");
    assert!(!markdown1.contains("\r"), "Found CR in output");
}
```

---

## Validation Tools

### 1. Determinism Validator
```bash
# scripts/validate_determinism.sh
#!/bin/bash

echo "Running determinism validation..."

# Test 1: Multiple runs produce identical output
cargo run --release -- scan fixtures/test.json > /tmp/run1.json
cargo run --release -- scan fixtures/test.json > /tmp/run2.json
cargo run --release -- scan fixtures/test.json > /tmp/run3.json

diff /tmp/run1.json /tmp/run2.json || exit 1
diff /tmp/run2.json /tmp/run3.json || exit 1

echo "✅ Multiple runs are identical"

# Test 2: Hash verification
HASH1=$(sha256sum /tmp/run1.json | cut -d' ' -f1)
HASH2=$(sha256sum /tmp/run2.json | cut -d' ' -f1)
HASH3=$(sha256sum /tmp/run3.json | cut -d' ' -f1)

if [ "$HASH1" = "$HASH2" ] && [ "$HASH2" = "$HASH3" ]; then
    echo "✅ Output hashes match: $HASH1"
else
    echo "❌ Hash mismatch!"
    exit 1
fi

# Test 3: Cross-platform verification (if in CI)
if [ -n "$CI" ]; then
    echo "📦 Uploading artifact for cross-platform verification"
    # Upload hash for comparison with other platforms
fi
```

### 2. Float Validator
```rust
pub fn validate_float_determinism(value: f64) -> Result<f64, String> {
    if value.is_nan() {
        return Err("NaN detected - should be normalized to 0.0".into());
    }
    if value.is_infinite() {
        return Err(format!("Infinity detected: {} - should be normalized", value));
    }
    Ok(value)
}
```

### 3. JSON Validator
```rust
pub fn validate_json_key_ordering(json: &str) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    validate_object_keys(&value)?;
    Ok(())
}

fn validate_object_keys(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            let mut sorted_keys = keys.clone();
            sorted_keys.sort();
            
            if keys != sorted_keys {
                return Err(format!(
                    "Keys not sorted: {:?} should be {:?}",
                    keys, sorted_keys
                ));
            }
            
            // Recursively validate nested objects
            for v in map.values() {
                validate_object_keys(v)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                validate_object_keys(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}
```

---

## CI Enforcement

### GitHub Actions Check
```yaml
determinism-check:
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
  steps:
    - uses: actions/checkout@v4
    
    - name: Run determinism tests
      run: |
        cargo test test_cross_platform_determinism
        cargo test test_float_determinism
        cargo test test_parallel_execution_determinism
    
    - name: Generate output hash
      run: |
        cargo run --release -- scan fixtures/test.json > output.json
        sha256sum output.json > hash-${{ matrix.os }}.txt
    
    - name: Upload hash artifact
      uses: actions/upload-artifact@v3
      with:
        name: hashes
        path: hash-${{ matrix.os }}.txt
  
  compare-hashes:
    needs: determinism-check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v3
        with:
          name: hashes
      
      - name: Compare platform hashes
        run: |
          HASH_LINUX=$(cat hash-ubuntu-latest.txt | cut -d' ' -f1)
          HASH_MACOS=$(cat hash-macos-latest.txt | cut -d' ' -f1)
          HASH_WINDOWS=$(cat hash-windows-latest.txt | cut -d' ' -f1)
          
          if [ "$HASH_LINUX" = "$HASH_MACOS" ] && [ "$HASH_MACOS" = "$HASH_WINDOWS" ]; then
            echo "✅ All platforms produce identical output"
          else
            echo "❌ Platform output mismatch!"
            echo "Linux:   $HASH_LINUX"
            echo "macOS:   $HASH_MACOS"
            echo "Windows: $HASH_WINDOWS"
            exit 1
          fi
```

---

## Breaking This Contract

**If you violate determinism, you break the product's core promise.**

### Allowed Exceptions
- **None** - Determinism is absolute

### Disallowed
- ❌ Using `rand` or `getrandom`
- ❌ Reading system time
- ❌ Reading environment variables (except config)
- ❌ Using `HashMap` in serialized outputs
- ❌ Non-deterministic parallel iteration
- ❌ Platform-specific float behavior
- ❌ Timestamps in outputs
- ❌ Thread IDs in outputs

---

## Success Criteria

✅ **Pass:** All tests produce identical SHA-256 hashes across platforms  
✅ **Pass:** JSON output keys alphabetically sorted  
✅ **Pass:** Float math consistent across x86_64/ARM/WASM  
✅ **Pass:** Parallel execution with 1/2/8 workers produces identical results  
✅ **Pass:** Multiple runs produce byte-for-byte identical files  
✅ **Pass:** No entropy sources in dependency tree  

---

## Version History

- **1.0.0** (2025-12-06) - Initial contract specification

---

**This contract is binding and enforced by CI.**
