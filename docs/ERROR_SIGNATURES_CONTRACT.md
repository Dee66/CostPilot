# Error Signatures Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

CostPilot errors are **elegant, helpful, and hashable**. Every error has a unique signature, clear context, and actionable hints. No cryptic stack traces, no vague messages, no guessing.

---

## Core Principles

### 1. Every Error Has a Signature
```rust
pub struct ErrorSignature {
    pub code: ErrorCode,           // E001, E002, etc.
    pub category: ErrorCategory,   // Parse, Validation, Runtime, etc.
    pub message: String,            // Human-readable description
    pub context: HashMap<String, String>,  // Relevant details
    pub hint: Option<String>,       // Actionable suggestion
    pub hash: String,               // Deterministic error signature hash
}
```

### 2. Error Codes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    // Parse errors (E001-E099)
    E001,  // Invalid JSON
    E002,  // Invalid Terraform plan
    E003,  // Invalid CDK structure
    E004,  // Invalid policy YAML

    // Validation errors (E100-E199)
    E100,  // Missing required field
    E101,  // Invalid resource type
    E102,  // Invalid module path
    E103,  // Invalid cost value
    E104,  // Invalid confidence value

    // Runtime errors (E200-E299)
    E200,  // Heuristic not found
    E201,  // Heuristic stale
    E202,  // Cold-start inference failed
    E203,  // Graph cycle detected
    E204,  // Policy evaluation failed

    // I/O errors (E300-E399)
    E300,  // File not found
    E301,  // File read error
    E302,  // File write error
    E303,  // Permission denied

    // Configuration errors (E400-E499)
    E400,  // Invalid configuration
    E401,  // Missing configuration
    E402,  // Configuration conflict

    // Internal errors (E500-E599)
    E500,  // Unexpected panic
    E501,  // Assertion failed
    E502,  // Internal consistency error
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "E001",
            ErrorCode::E002 => "E002",
            // ... etc
        }
    }

    pub fn category(&self) -> ErrorCategory {
        let code_num = self.as_str()[1..].parse::<u32>().unwrap();
        match code_num {
            1..=99 => ErrorCategory::Parse,
            100..=199 => ErrorCategory::Validation,
            200..=299 => ErrorCategory::Runtime,
            300..=399 => ErrorCategory::IO,
            400..=499 => ErrorCategory::Configuration,
            500..=599 => ErrorCategory::Internal,
            _ => ErrorCategory::Unknown,
        }
    }
}
```

### 3. Error Categories
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    Parse,          // Input parsing failed
    Validation,     // Input validation failed
    Runtime,        // Runtime execution failed
    IO,             // File/network I/O failed
    Configuration,  // Configuration error
    Internal,       // Internal bug (please report)
    Unknown,        // Uncategorized
}

impl ErrorCategory {
    pub fn emoji(&self) -> &'static str {
        match self {
            ErrorCategory::Parse => "📝",
            ErrorCategory::Validation => "✅",
            ErrorCategory::Runtime => "⚙️",
            ErrorCategory::IO => "💾",
            ErrorCategory::Configuration => "⚙️",
            ErrorCategory::Internal => "🐛",
            ErrorCategory::Unknown => "❓",
        }
    }
}
```

---

## Error Formatting

### Terminal Output
```rust
impl Display for ErrorSignature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{} Error {}: {}",
            self.category.emoji(),
            self.code.as_str(),
            self.message
        )?;

        if !self.context.is_empty() {
            writeln!(f, "\nContext:")?;
            let mut sorted_context: Vec<_> = self.context.iter().collect();
            sorted_context.sort_by_key(|(k, _)| *k);

            for (key, value) in sorted_context {
                writeln!(f, "  {}: {}", key, value)?;
            }
        }

        if let Some(hint) = &self.hint {
            writeln!(f, "\n💡 Hint: {}", hint)?;
        }

        writeln!(f, "\nError signature: {}", self.hash)?;

        Ok(())
    }
}
```

### Example Output
```
📝 Error E002: Invalid Terraform plan

Context:
  file: terraform.plan.json
  line: 47
  resource: aws_instance.web

💡 Hint: Ensure the plan was generated with 'terraform plan -out=plan.json' and then converted with 'terraform show -json plan.json'

Error signature: a3f2c1d9e8b7a6f5
```

---

## Error Construction

### Builder Pattern
```rust
impl ErrorSignature {
    pub fn builder(code: ErrorCode, message: impl Into<String>) -> ErrorBuilder {
        ErrorBuilder {
            code,
            message: message.into(),
            context: HashMap::new(),
            hint: None,
        }
    }
}

pub struct ErrorBuilder {
    code: ErrorCode,
    message: String,
    context: HashMap<String, String>,
    hint: Option<String>,
}

impl ErrorBuilder {
    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn build(self) -> ErrorSignature {
        let hash = Self::compute_hash(&self.code, &self.context);

        ErrorSignature {
            code: self.code,
            category: self.code.category(),
            message: self.message,
            context: self.context,
            hint: self.hint,
            hash,
        }
    }

    fn compute_hash(code: &ErrorCode, context: &HashMap<String, String>) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();

        // Hash error code
        hasher.update(code.as_str().as_bytes());

        // Hash context (sorted for determinism)
        let mut sorted_context: Vec<_> = context.iter().collect();
        sorted_context.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_context {
            hasher.update(key.as_bytes());
            hasher.update(b":");
            hasher.update(value.as_bytes());
            hasher.update(b";");
        }

        // Take first 16 chars of hex
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
}
```

### Usage Examples
```rust
// Parse error
let error = ErrorSignature::builder(
    ErrorCode::E002,
    "Invalid Terraform plan"
)
.context("file", "terraform.plan.json")
.context("line", "47")
.context("resource", "aws_instance.web")
.hint("Ensure the plan was generated with 'terraform plan -out=plan.json'")
.build();

// Heuristic not found
let error = ErrorSignature::builder(
    ErrorCode::E200,
    "Heuristic not found"
)
.context("region", "us-east-1")
.context("instance_type", "t3.xlarge")
.context("provider", "aws")
.hint("Update heuristics with 'costpilot heuristics update'")
.build();

// File not found
let error = ErrorSignature::builder(
    ErrorCode::E300,
    "File not found"
)
.context("file", "/path/to/plan.json")
.hint("Check that the file exists and you have read permissions")
.build();
```

---

## Error Handling Patterns

### Result Type
```rust
pub type CostPilotResult<T> = Result<T, ErrorSignature>;

// Example function
pub fn parse_terraform_plan(path: &Path) -> CostPilotResult<TerraformPlan> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| {
            ErrorSignature::builder(
                ErrorCode::E301,
                format!("Failed to read file: {}", e)
            )
            .context("file", path.display().to_string())
            .hint("Check that the file exists and you have read permissions")
            .build()
        })?;

    serde_json::from_str(&content)
        .map_err(|e| {
            ErrorSignature::builder(
                ErrorCode::E001,
                format!("Invalid JSON: {}", e)
            )
            .context("file", path.display().to_string())
            .context("error", e.to_string())
            .hint("Ensure the file is valid JSON generated by Terraform")
            .build()
        })
}
```

### Context Propagation
```rust
pub fn predict_cost(resource: &Resource) -> CostPilotResult<Prediction> {
    let heuristic = load_heuristic(&resource.region, &resource.instance_type)
        .map_err(|e| {
            // Add more context to existing error
            let mut builder = ErrorSignature::builder(
                e.code,
                e.message
            );

            for (k, v) in e.context {
                builder = builder.context(k, v);
            }

            builder = builder
                .context("resource_id", &resource.id)
                .hint("This resource will use cold-start inference");

            builder.build()
        })?;

    Ok(compute_prediction(&resource, &heuristic))
}
```

---

## JSON Error Format

### Schema
```json
{
  "error": {
    "code": "E002",
    "category": "Parse",
    "message": "Invalid Terraform plan",
    "context": {
      "file": "terraform.plan.json",
      "line": "47",
      "resource": "aws_instance.web"
    },
    "hint": "Ensure the plan was generated with 'terraform plan -out=plan.json'",
    "hash": "a3f2c1d9e8b7a6f5"
  }
}
```

### Serialization
```rust
impl Serialize for ErrorSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ErrorSignature", 6)?;
        state.serialize_field("code", self.code.as_str())?;
        state.serialize_field("category", &self.category)?;
        state.serialize_field("message", &self.message)?;

        // Sort context for determinism
        let mut sorted_context: BTreeMap<_, _> = self.context.iter().collect();
        state.serialize_field("context", &sorted_context)?;

        state.serialize_field("hint", &self.hint)?;
        state.serialize_field("hash", &self.hash)?;
        state.end()
    }
}
```

---

## CLI Exit Codes

```rust
impl ErrorSignature {
    pub fn exit_code(&self) -> i32 {
        match self.category {
            ErrorCategory::Parse => 10,
            ErrorCategory::Validation => 11,
            ErrorCategory::Runtime => 12,
            ErrorCategory::IO => 13,
            ErrorCategory::Configuration => 14,
            ErrorCategory::Internal => 15,
            ErrorCategory::Unknown => 1,
        }
    }
}

// CLI main function
pub fn main() {
    let result = run_cli();

    match result {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(error.exit_code());
        }
    }
}
```

**Exit Code Reference:**
| Exit Code | Category | Meaning |
|-----------|----------|---------|
| 0 | Success | Command completed successfully |
| 1 | Unknown | Uncategorized error |
| 2 | Policy | Policy violation detected |
| 10 | Parse | Failed to parse input |
| 11 | Validation | Input validation failed |
| 12 | Runtime | Runtime execution failed |
| 13 | IO | File/network I/O failed |
| 14 | Configuration | Configuration error |
| 15 | Internal | Internal bug (please report) |

---

## Error Hints (Actionable)

### Good Hints
```rust
// ✅ GOOD: Specific, actionable
.hint("Run 'terraform plan -out=plan.json && terraform show -json plan.json > plan.json'")

// ✅ GOOD: Points to documentation
.hint("See https://docs.costpilot.dev/heuristics for updating heuristics")

// ✅ GOOD: Suggests command
.hint("Update heuristics with 'costpilot heuristics update'")

// ✅ GOOD: Explains root cause
.hint("This resource type is not yet supported. Consider opening an issue")
```

### Bad Hints
```rust
// ❌ BAD: Vague
.hint("Try fixing the input")

// ❌ BAD: Not actionable
.hint("Something went wrong")

// ❌ BAD: Too technical
.hint("Ensure the AST node contains a valid ResourceConfig")

// ❌ BAD: Patronizing
.hint("You should know how to use Terraform")
```

---

## Error Logging

### Structured Logging
```rust
use tracing::{error, warn, info};

// Log error with context
error!(
    code = %error.code.as_str(),
    category = ?error.category,
    hash = %error.hash,
    "Error occurred: {}",
    error.message
);

// Log with span
let span = tracing::span!(
    tracing::Level::ERROR,
    "prediction_error",
    resource_id = %resource.id,
    region = %resource.region,
);

let _enter = span.enter();
error!("Prediction failed: {}", error.message);
```

### Error Metrics
```rust
use prometheus::{IntCounterVec, register_int_counter_vec};

lazy_static! {
    static ref ERROR_COUNTER: IntCounterVec = register_int_counter_vec!(
        "costpilot_errors_total",
        "Total number of errors by category and code",
        &["category", "code"]
    ).unwrap();
}

impl ErrorSignature {
    pub fn record_metric(&self) {
        ERROR_COUNTER
            .with_label_values(&[
                &format!("{:?}", self.category),
                self.code.as_str(),
            ])
            .inc();
    }
}
```

---

## Stack Traces

### Never Show by Default
```rust
// ❌ FORBIDDEN (default behavior)
eprintln!("{:?}", error);  // Shows stack trace

// ✅ REQUIRED (default behavior)
eprintln!("{}", error);    // Shows only signature

// ✅ ALLOWED (with --debug flag)
if args.debug {
    eprintln!("{:?}", error);  // Debug output
}
```

### Capture for Debugging
```rust
use std::backtrace::Backtrace;

pub struct ErrorSignature {
    // ... existing fields
    backtrace: Option<Backtrace>,
}

impl ErrorSignature {
    pub fn with_backtrace(mut self) -> Self {
        if cfg!(debug_assertions) {
            self.backtrace = Some(Backtrace::capture());
        }
        self
    }
}

// Only shown with --debug flag
if args.debug {
    if let Some(backtrace) = &error.backtrace {
        eprintln!("\nBacktrace:\n{}", backtrace);
    }
}
```

---

## Validation Tests

### Error Signature Tests
```rust
#[test]
fn test_error_hash_deterministic() {
    let error1 = ErrorSignature::builder(
        ErrorCode::E002,
        "Invalid Terraform plan"
    )
    .context("file", "plan.json")
    .context("line", "47")
    .build();

    let error2 = ErrorSignature::builder(
        ErrorCode::E002,
        "Invalid Terraform plan"
    )
    .context("line", "47")  // Different order
    .context("file", "plan.json")
    .build();

    // Hash must be identical
    assert_eq!(error1.hash, error2.hash);
}

#[test]
fn test_all_error_codes_have_category() {
    use strum::IntoEnumIterator;  // Requires strum derive

    for code in ErrorCode::iter() {
        let category = code.category();
        assert_ne!(category, ErrorCategory::Unknown);
    }
}

#[test]
fn test_error_json_serialization() {
    let error = ErrorSignature::builder(
        ErrorCode::E200,
        "Heuristic not found"
    )
    .context("region", "us-east-1")
    .context("instance_type", "t3.large")
    .hint("Update heuristics")
    .build();

    let json = serde_json::to_string_pretty(&error).unwrap();

    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Must have all required fields
    assert!(parsed["code"].is_string());
    assert!(parsed["category"].is_string());
    assert!(parsed["message"].is_string());
    assert!(parsed["context"].is_object());
    assert!(parsed["hash"].is_string());
}
```

---

## Breaking This Contract

**Severity: MEDIUM (impacts debugging)**

**Forbidden:**
- ❌ Showing stack traces by default
- ❌ Vague error messages
- ❌ Non-actionable hints
- ❌ Missing error codes
- ❌ Non-deterministic error hashes

**Required:**
- ✅ Every error has a code and signature
- ✅ Context includes relevant details
- ✅ Hints are specific and actionable
- ✅ Errors are hashable and sortable
- ✅ Stack traces only with --debug

---

## Benefits

### User Experience
- **Clear errors** - Know exactly what went wrong
- **Actionable hints** - Know how to fix it
- **Professional** - No cryptic stack traces
- **Searchable** - Error signatures easy to search

### Developer Experience
- **Easy debugging** - Context included
- **Error tracking** - Hash enables grouping
- **Structured logs** - Machine-readable errors
- **Metrics** - Track error frequencies

### Support Experience
- **Fast resolution** - Error signature identifies issue
- **Known issues** - Hash matches previous reports
- **Context included** - No need to ask for details
- **Actionable** - User often self-resolves

---

## Version History

- **1.0.0** (2025-12-06) - Initial error signatures contract

---

**This contract ensures CostPilot errors are elegant, helpful, and debuggable.**
