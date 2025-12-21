# Behavior Freeze Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

CostPilot behavior is **semantically versioned** and **frozen** across minor versions. Breaking changes require major version bumps. This contract defines what constitutes a breaking change and how stability is maintained.

---

## Semantic Versioning Rules

```
MAJOR.MINOR.PATCH

1.2.3
│ │ └─ Bug fixes, optimizations (no behavior change)
│ └─── New features, backward-compatible changes
└───── Breaking changes, incompatible updates
```

### Version Increment Guide

| Change Type | Version Bump | Example |
|-------------|--------------|---------|
| Bug fix (no output change) | PATCH | Fix crash, memory leak |
| Performance improvement | PATCH | 2× faster, -50% memory |
| New optional flag | MINOR | `--verbose` flag added |
| New heuristic (additive) | MINOR | Add Azure support |
| Change cost calculation | MAJOR | EC2 pricing formula change |
| Remove CLI command | MAJOR | Delete `costpilot init` |
| Change JSON schema | MAJOR | Rename field `cost` → `price` |

---

## Frozen Behaviors

### 1. Regression Classifier Output
```rust
#[frozen(since = "1.0.0")]
pub enum RegressionType {
    CostIncrease,
    CostDecrease,
    NewResource,
    DeletedResource,
    ModifiedResource,
    DependencyChange,
}
```

**Frozen:**
- Enum variant names
- Classification logic
- JSON field names
- Confidence threshold (50%)

**Allowed (MINOR):**
- Add new variant (e.g., `ResourceTypeChange`)
- Improve confidence calculation (if output distribution unchanged)

**Breaking (MAJOR):**
- Rename variant
- Remove variant
- Change classification logic (e.g., 5% → 10% threshold)
- Change confidence calculation (if affects >10% of results)

### 2. Prediction Semantics
```rust
#[frozen(since = "1.0.0")]
pub struct PredictionInterval {
    pub p10: f64,    // 10th percentile (optimistic)
    pub p50: f64,    // 50th percentile (median)
    pub p90: f64,    // 90th percentile (pessimistic)
    pub p99: f64,    // 99th percentile (very pessimistic)
}
```

**Frozen:**
- Percentile values (p10, p50, p90, p99)
- Field names
- Interval semantics (p10 ≤ p50 ≤ p90 ≤ p99)
- Confidence range (0.0 - 1.0)

**Allowed (MINOR):**
- Add new percentile (e.g., `p95`)
- Add new field (e.g., `variance`)
- Improve heuristic accuracy (within ±5% mean absolute error)

**Breaking (MAJOR):**
- Change percentile values (p10 → p5)
- Rename field (p50 → median)
- Change interval semantics (p90 becomes 95th percentile)
- Change confidence range (0-100 instead of 0.0-1.0)

### 3. Mapping Schema (Dependency Graph)
```rust
#[frozen(since = "1.0.0")]
pub struct DependencyGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[frozen(since = "1.0.0")]
pub struct Node {
    pub id: String,
    pub module_path: Vec<String>,
    pub resource_type: String,
}

#[frozen(since = "1.0.0")]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}
```

**Frozen:**
- Node/Edge structure
- Field names (`id`, `module_path`, `resource_type`)
- Graph direction (directed acyclic)
- Cycle detection behavior

**Allowed (MINOR):**
- Add new field (e.g., `weight` to Edge)
- Add new dependency_type variant
- Improve cycle detection algorithm

**Breaking (MAJOR):**
- Change field names
- Change graph direction (undirected)
- Remove cycle detection
- Change node/edge representation

### 4. Explain Schemas (Reasoning Output)
```rust
#[frozen(since = "1.0.0")]
pub struct Explanation {
    pub reasoning: Vec<ReasoningStep>,
    pub conclusion: String,
    pub confidence: f64,
}

#[frozen(since = "1.0.0")]
pub struct ReasoningStep {
    pub step_number: usize,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence_impact: f64,
}
```

**Frozen:**
- Explanation structure
- ReasoningStep fields
- Confidence calculation
- Evidence format

**Allowed (MINOR):**
- Add new field (e.g., `provenance`)
- Improve evidence quality
- Add new reasoning step types

**Breaking (MAJOR):**
- Remove fields
- Rename fields
- Change confidence range
- Change reasoning semantics

### 5. Heuristics Format
```toml
# Frozen since 1.0.0
[heuristic.aws_ec2_t3_medium]
region = "us-east-1"
instance_type = "t3.medium"
hourly_cost = 0.0416
monthly_cost = 30.37
confidence = 0.95
updated_at = "2025-12-01"
```

**Frozen:**
- TOML structure
- Required fields (`region`, `instance_type`, `hourly_cost`, `monthly_cost`)
- Cost calculation formula (hourly × 730)
- Confidence interpretation

**Allowed (MINOR):**
- Add optional fields (e.g., `spot_price`)
- Add new heuristic entries
- Update cost values
- Improve confidence estimation

**Breaking (MAJOR):**
- Change to JSON/YAML format
- Remove required fields
- Change cost calculation formula
- Rename fields

### 6. Policy Evaluation Logic
```rust
#[frozen(since = "1.0.0")]
pub struct PolicyResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<Warning>,
}

#[frozen(since = "1.0.0")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
```

**Frozen:**
- PolicyResult structure
- Severity levels
- Violation/Warning format
- Evaluation semantics (fail if any critical)

**Allowed (MINOR):**
- Add new severity level (if ordered correctly)
- Add new policy types
- Improve violation messages

**Breaking (MAJOR):**
- Change severity ordering
- Remove severity level
- Change evaluation semantics
- Rename fields

---

## CLI Interface Stability

### Command Structure
```bash
# Frozen since 1.0.0
costpilot scan <file>
costpilot explain --resource <id>
costpilot autofix patch <file>
costpilot map --output graph.mermaid
costpilot group --by module
costpilot policy check --config policy.yaml
```

**Frozen:**
- Command names (`scan`, `explain`, `autofix`, `map`, `group`, `policy`)
- Required arguments
- Flag names (`--resource`, `--output`, `--by`, `--config`)
- Exit codes (0 = success, 1 = error, 2 = policy violation)

**Allowed (MINOR):**
- Add new commands (e.g., `costpilot baseline`)
- Add optional flags (e.g., `--verbose`)
- Add subcommands (e.g., `costpilot autofix apply`)

**Breaking (MAJOR):**
- Rename commands
- Remove commands
- Change required arguments
- Rename flags
- Change exit codes

---

## Output Format Stability

### JSON Schema Versioning
```json
{
  "schema_version": "1.0.0",
  "data": {
    "cost": 30.37,
    "resource": "aws_instance.web"
  }
}
```

**Rules:**
- Every JSON output includes `schema_version`
- Schema version follows SemVer
- Schema changes increment version
- Old schemas supported for 2 major versions

**Example:**
```
v1.0.0: Original schema
v1.1.0: Add optional field (backward-compatible)
v2.0.0: Remove field (breaking change)
v3.0.0: v1.x.x support dropped
```

### Markdown Format
```markdown
# Cost Analysis Report

**Schema Version:** 1.0.0

## Summary
...
```

**Frozen:**
- Header structure (`#`, `##`, `###`)
- Section names (`Summary`, `Details`, `Recommendations`)
- Table column order
- Severity emoji (🔴 🟠 🟡 🔵 ⚪)

**Allowed (MINOR):**
- Add new sections
- Add new table columns
- Improve formatting

**Breaking (MAJOR):**
- Remove sections
- Rename sections
- Change table schema
- Change emoji

---

## Testing Behavior Freeze

### Snapshot Tests
```rust
#[test]
fn test_regression_classifier_frozen() {
    let plan = load_fixture("ec2_cost_increase.json");

    let result = classify_regression(&plan).unwrap();

    // Snapshot test - output must not change
    insta::assert_json_snapshot!(result);
}

#[test]
fn test_prediction_interval_frozen() {
    let resource = sample_ec2_resource("t3.medium");

    let prediction = predict_cost(&resource).unwrap();

    // Frozen semantics
    assert!(prediction.p10 <= prediction.p50);
    assert!(prediction.p50 <= prediction.p90);
    assert!(prediction.p90 <= prediction.p99);

    // Snapshot test
    insta::assert_json_snapshot!(prediction);
}
```

### Golden Files
```rust
#[test]
fn test_explain_output_frozen() {
    let explanation = explain_cost_increase(&sample_plan());

    let expected = include_str!("golden/explain_output_v1.0.0.json");
    let actual = serde_json::to_string_pretty(&explanation).unwrap();

    assert_eq!(actual, expected, "Explain output must match golden file");
}
```

---

## Breaking Change Process

### Before Breaking Change
1. **Deprecation Warning** (1 minor version before)
   ```rust
   #[deprecated(since = "1.5.0", note = "Use `new_function()` instead")]
   pub fn old_function() { ... }
   ```

2. **Documentation Update**
   ```markdown
   ## Deprecations in v1.5.0
   - `old_function()` deprecated, use `new_function()`
   - Will be removed in v2.0.0
   ```

3. **Migration Guide**
   ```markdown
   # Migrating from v1.x to v2.0

   ## Breaking Changes
   - `old_function()` removed → use `new_function()`
   - JSON field `cost` renamed to `price`

   ## Migration Steps
   1. Update all calls to `old_function()`
   2. Update JSON parsers to use `price` field
   3. Test with `costpilot --version 2.0.0`
   ```

### Version Bump Checklist
```markdown
## Major Version Bump (v2.0.0)
- [ ] List all breaking changes
- [ ] Write migration guide
- [ ] Update CHANGELOG.md
- [ ] Update golden files
- [ ] Update snapshot tests
- [ ] Announce deprecations 1 version early
- [ ] Test backward compatibility (if applicable)

## Minor Version Bump (v1.5.0)
- [ ] List new features
- [ ] Ensure backward compatibility
- [ ] Add new tests (not breaking old ones)
- [ ] Update documentation
- [ ] Update CHANGELOG.md

## Patch Version Bump (v1.4.1)
- [ ] List bug fixes
- [ ] Ensure no behavior change
- [ ] Update tests (if fixing incorrect behavior)
- [ ] Update CHANGELOG.md
```

---

## CHANGELOG.md Format

```markdown
# Changelog

All notable changes to CostPilot will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2025-12-15

### Breaking Changes
- **Regression Classifier**: Changed threshold from 5% to 10% (#234)
- **JSON Schema**: Renamed `cost` field to `price` in all outputs (#235)
- **CLI**: Removed deprecated `costpilot init` command (#236)

### Migration Guide
See [MIGRATION.md](MIGRATION.md) for detailed instructions.

## [1.5.0] - 2025-12-01

### Added
- Azure support (#200)
- `--show-provenance` flag (#201)

### Deprecated
- `old_function()` - use `new_function()` instead (#202)

### Fixed
- Fixed crash on empty Terraform plan (#203)

## [1.4.1] - 2025-11-20

### Fixed
- Fixed memory leak in prediction engine (#190)
- Fixed incorrect cost for t3.large in us-west-2 (#191)

## [1.4.0] - 2025-11-10

### Added
- New `costpilot baseline` command (#180)

## [1.0.0] - 2025-10-01

### Initial Release
- Terraform cost prediction
- Policy enforcement
- Dependency mapping
- Autofix suggestions
```

---

## Validation Tests

### Behavior Freeze Tests
```rust
#[test]
fn test_regression_classifier_stability() {
    // Load golden file from v1.0.0
    let expected = include_str!("golden/regression_v1.0.0.json");

    // Current output
    let plan = load_fixture("ec2_cost_increase.json");
    let result = classify_regression(&plan).unwrap();
    let actual = serde_json::to_string_pretty(&result).unwrap();

    assert_eq!(
        actual,
        expected,
        "Regression classifier output changed (MAJOR version bump required)"
    );
}

#[test]
fn test_json_schema_version() {
    let output = generate_cost_report(&sample_plan());
    let json: Value = serde_json::from_str(&output).unwrap();

    assert!(
        json.get("schema_version").is_some(),
        "All JSON output must include schema_version"
    );
}

#[test]
fn test_cli_exit_codes() {
    // Success
    let status = run_command(&["scan", "plan.json"]);
    assert_eq!(status.code(), Some(0));

    // Error
    let status = run_command(&["scan", "nonexistent.json"]);
    assert_eq!(status.code(), Some(1));

    // Policy violation
    let status = run_command(&["policy", "check", "--config", "strict.yaml"]);
    assert_eq!(status.code(), Some(2));
}
```

---

## Breaking This Contract

**Severity: CRITICAL (breaks user workflows)**

**Forbidden:**
- ❌ Change output format without major version bump
- ❌ Remove CLI commands without deprecation
- ❌ Change classification logic without notice
- ❌ Rename JSON fields in minor version
- ❌ Change exit codes

**Required:**
- ✅ Deprecate before removing (1 minor version)
- ✅ Write migration guides for major versions
- ✅ Maintain CHANGELOG.md
- ✅ Snapshot test all frozen behaviors
- ✅ Include schema_version in JSON output

---

## Benefits

### User Trust
- **Predictable behavior** - No surprises in updates
- **Safe upgrades** - Minor versions never break workflows
- **Clear deprecations** - Time to migrate before removal

### Developer Experience
- **Confidence in refactoring** - Frozen behaviors can't change
- **Easy testing** - Snapshot tests catch regressions
- **Clear contracts** - Know what can/can't change

### Enterprise Adoption
- **Stability guarantees** - Production systems safe
- **Long-term support** - Old schemas supported for 2 major versions
- **Compliance** - Audit trails preserved across versions

---

## Version History

- **1.0.0** (2025-12-06) - Initial behavior freeze contract

---

**This contract ensures CostPilot behavior is stable and predictable across versions.**
