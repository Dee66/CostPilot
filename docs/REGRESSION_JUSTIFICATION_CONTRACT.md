# Regression Justification Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

Every detected cost regression must have a **complete, professional justification** suitable for PR comments. No vague explanations, no missing context, no guessing.

---

## Core Principle

**Every regression has 6 mandatory elements:**

1. **Type** - What changed (cost increase/decrease, new resource, etc.)
2. **Driver** - What caused it (instance size, new NAT Gateway, etc.)
3. **Delta** - Quantified impact ($X/month, +Y%)
4. **Confidence** - How certain we are (0-100%)
5. **Dependency Context** - What else is affected
6. **Root Cause** - Why this change happened

---

## Regression Data Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionJustification {
    // MANDATORY: What changed
    pub regression_type: RegressionType,

    // MANDATORY: Primary driver of change
    pub driver: RegressionDriver,

    // MANDATORY: Quantified impact
    pub delta: CostDelta,

    // MANDATORY: Confidence in analysis
    pub confidence: f64,  // 0.0 - 1.0

    // MANDATORY: Dependency context
    pub dependencies: DependencyContext,

    // MANDATORY: Root cause explanation
    pub root_cause: RootCause,

    // OPTIONAL: Additional details
    pub details: Option<String>,

    // OPTIONAL: Recommendations
    pub recommendations: Vec<Recommendation>,
}
```

---

## Regression Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionType {
    CostIncrease,        // Monthly cost went up
    CostDecrease,        // Monthly cost went down
    NewResource,         // Resource added
    DeletedResource,     // Resource removed
    ModifiedResource,    // Resource attributes changed
    DependencyChange,    // Dependency graph changed
    ConfigurationChange, // Configuration updated
}

impl RegressionType {
    pub fn emoji(&self) -> &'static str {
        match self {
            RegressionType::CostIncrease => "📈",
            RegressionType::CostDecrease => "📉",
            RegressionType::NewResource => "🆕",
            RegressionType::DeletedResource => "🗑️",
            RegressionType::ModifiedResource => "✏️",
            RegressionType::DependencyChange => "🔗",
            RegressionType::ConfigurationChange => "⚙️",
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            RegressionType::CostIncrease => Severity::Medium,
            RegressionType::CostDecrease => Severity::Low,
            RegressionType::NewResource => Severity::Medium,
            RegressionType::DeletedResource => Severity::Low,
            RegressionType::ModifiedResource => Severity::Medium,
            RegressionType::DependencyChange => Severity::Low,
            RegressionType::ConfigurationChange => Severity::Low,
        }
    }
}
```

---

## Regression Drivers

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionDriver {
    // Compute changes
    InstanceSizeChange {
        from: String,
        to: String,
    },
    InstanceCountChange {
        from: usize,
        to: usize,
    },

    // Storage changes
    StorageSizeChange {
        from: u64,  // GB
        to: u64,
    },
    StorageTypeChange {
        from: String,
        to: String,
    },

    // Network changes
    NATGatewayAdded {
        count: usize,
    },
    LoadBalancerAdded {
        lb_type: String,
    },

    // Database changes
    DatabaseInstanceChange {
        from: String,
        to: String,
    },
    DatabaseStorageChange {
        from: u64,
        to: u64,
    },

    // New resources
    NewResourceAdded {
        resource_type: String,
        count: usize,
    },

    // Deleted resources
    ResourceDeleted {
        resource_type: String,
        count: usize,
    },

    // Configuration
    ConfigurationUpdate {
        field: String,
        from: String,
        to: String,
    },

    // Multiple drivers
    MultipleDrivers {
        drivers: Vec<Box<RegressionDriver>>,
    },
}

impl RegressionDriver {
    pub fn summary(&self) -> String {
        match self {
            RegressionDriver::InstanceSizeChange { from, to } => {
                format!("Instance size changed: {} → {}", from, to)
            }
            RegressionDriver::InstanceCountChange { from, to } => {
                format!("Instance count changed: {} → {}", from, to)
            }
            RegressionDriver::NATGatewayAdded { count } => {
                format!("Added {} NAT Gateway{}", count, if *count == 1 { "" } else { "s" })
            }
            RegressionDriver::NewResourceAdded { resource_type, count } => {
                format!("Added {} {}{}", count, resource_type, if *count == 1 { "" } else { "s" })
            }
            // ... etc
        }
    }
}
```

---

## Cost Delta

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDelta {
    pub old_cost: f64,
    pub new_cost: f64,
    pub delta: f64,           // new - old
    pub percentage: f64,      // (delta / old) * 100
    pub interval: Interval,   // month, hour, etc.
}

impl CostDelta {
    pub fn new(old_cost: f64, new_cost: f64, interval: Interval) -> Self {
        let delta = new_cost - old_cost;
        let percentage = if old_cost > 0.0 {
            (delta / old_cost) * 100.0
        } else {
            0.0
        };

        Self {
            old_cost,
            new_cost,
            delta,
            percentage,
            interval,
        }
    }

    pub fn format(&self) -> String {
        let sign = if self.delta >= 0.0 { "+" } else { "" };
        format!(
            "{}{:.2}/{} ({}{:.1}%)",
            sign,
            self.delta,
            self.interval.as_str(),
            sign,
            self.percentage
        )
    }

    pub fn severity(&self) -> Severity {
        let abs_percentage = self.percentage.abs();

        if abs_percentage >= 50.0 {
            Severity::High
        } else if abs_percentage >= 20.0 {
            Severity::Medium
        } else if abs_percentage >= 5.0 {
            Severity::Low
        } else {
            Severity::Info
        }
    }
}
```

---

## Dependency Context

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyContext {
    // Direct dependencies affected
    pub direct_dependencies: Vec<String>,

    // Downstream impact
    pub downstream_count: usize,

    // Modules affected
    pub modules_affected: Vec<String>,

    // Critical path impact
    pub on_critical_path: bool,
}

impl DependencyContext {
    pub fn summary(&self) -> String {
        if self.direct_dependencies.is_empty() {
            "No dependencies affected".to_string()
        } else if self.downstream_count == 0 {
            format!("{} direct dependencies affected", self.direct_dependencies.len())
        } else {
            format!(
                "{} direct dependencies, {} downstream resources",
                self.direct_dependencies.len(),
                self.downstream_count
            )
        }
    }
}
```

---

## Root Cause

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    // What triggered the change
    pub trigger: RootCauseTrigger,

    // Explanation
    pub explanation: String,

    // Provenance (heuristic/baseline used)
    pub provenance: Option<HeuristicProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RootCauseTrigger {
    CodeChange {
        file: String,
        line: Option<usize>,
    },
    ConfigChange {
        field: String,
        value: String,
    },
    DependencyUpdate {
        dependency: String,
        version: String,
    },
    ScalingDecision {
        reason: String,
    },
    ManualIntervention {
        author: Option<String>,
    },
    Unknown,
}

impl RootCause {
    pub fn summary(&self) -> String {
        match &self.trigger {
            RootCauseTrigger::CodeChange { file, line } => {
                if let Some(l) = line {
                    format!("Code change in {}:{}", file, l)
                } else {
                    format!("Code change in {}", file)
                }
            }
            RootCauseTrigger::ConfigChange { field, value } => {
                format!("Configuration change: {} = {}", field, value)
            }
            RootCauseTrigger::ScalingDecision { reason } => {
                format!("Scaling decision: {}", reason)
            }
            _ => self.explanation.clone(),
        }
    }
}
```

---

## Recommendations

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub impact: Option<CostDelta>,
    pub confidence: f64,
    pub effort: EffortLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,     // < 1 hour
    Medium,  // 1-4 hours
    High,    // > 4 hours
}

impl Recommendation {
    pub fn format(&self) -> String {
        let mut parts = vec![self.action.clone()];

        if let Some(impact) = &self.impact {
            parts.push(format!("(saves {})", impact.format()));
        }

        parts.join(" ")
    }
}
```

---

## PR Comment Format

### Template
```rust
pub fn format_pr_comment(justification: &RegressionJustification) -> String {
    format!(
        r#"## {emoji} Cost Regression Detected

**Type:** {regression_type}
**Impact:** {delta}
**Confidence:** {confidence}%

### 🔍 Analysis
{driver_summary}

### 📊 Cost Breakdown
- **Before:** {old_cost}
- **After:** {new_cost}
- **Delta:** {delta_formatted}

### 🔗 Dependencies
{dependencies}

### 🎯 Root Cause
{root_cause}

{recommendations}

---
<details>
<summary>View detailed breakdown</summary>

{details}

</details>
"#,
        emoji = justification.regression_type.emoji(),
        regression_type = format!("{:?}", justification.regression_type),
        delta = justification.delta.format(),
        confidence = (justification.confidence * 100.0) as u32,
        driver_summary = justification.driver.summary(),
        old_cost = format_currency(justification.delta.old_cost),
        new_cost = format_currency(justification.delta.new_cost),
        delta_formatted = justification.delta.format(),
        dependencies = justification.dependencies.summary(),
        root_cause = justification.root_cause.summary(),
        recommendations = format_recommendations(&justification.recommendations),
        details = justification.details.as_ref().unwrap_or(&"No additional details".to_string()),
    )
}

fn format_recommendations(recommendations: &[Recommendation]) -> String {
    if recommendations.is_empty() {
        return String::new();
    }

    let mut output = String::from("### 💡 Recommendations\n");

    for (i, rec) in recommendations.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, rec.format()));
    }

    output
}
```

### Example Output
```markdown
## 📈 Cost Regression Detected

**Type:** CostIncrease
**Impact:** +$32.88/month (+108.3%)
**Confidence:** 92%

### 🔍 Analysis
Added 2 NAT Gateways

### 📊 Cost Breakdown
- **Before:** $30.37/month
- **After:** $63.25/month
- **Delta:** +$32.88/month (+108.3%)

### 🔗 Dependencies
3 direct dependencies, 7 downstream resources

### 🎯 Root Cause
Code change in infrastructure/network.tf:45

New NAT Gateways added for multi-AZ high availability. Each NAT Gateway costs $32.88/month ($0.045/hour × 730 hours).

### 💡 Recommendations
1. Consider using single NAT Gateway with route table optimization (saves -$32.88/month)
2. Evaluate if multi-AZ NAT is required for this environment

---
<details>
<summary>View detailed breakdown</summary>

#### New Resources
| Resource | Type | Cost/Month |
|----------|------|------------|
| aws_nat_gateway.az1 | NAT Gateway | $32.88 |
| aws_nat_gateway.az2 | NAT Gateway | $32.88 |

#### Downstream Impact
- aws_route_table.private_az1
- aws_route_table.private_az2
- aws_subnet.private_az1
- aws_subnet.private_az2
- aws_instance.app_server_az1
- aws_instance.app_server_az2
- aws_instance.worker_az1

</details>
```

---

## Validation Rules

### Mandatory Field Validation
```rust
impl RegressionJustification {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Confidence must be in range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push(format!(
                "Confidence must be 0.0-1.0, got {}",
                self.confidence
            ));
        }

        // Delta must be consistent
        let expected_delta = self.delta.new_cost - self.delta.old_cost;
        if (self.delta.delta - expected_delta).abs() > 0.01 {
            errors.push(format!(
                "Delta inconsistent: {} vs {}",
                self.delta.delta,
                expected_delta
            ));
        }

        // Root cause must have explanation
        if self.root_cause.explanation.trim().is_empty() {
            errors.push("Root cause explanation is empty".to_string());
        }

        // Must have at least one driver
        if matches!(self.driver, RegressionDriver::MultipleDrivers { drivers } if drivers.is_empty()) {
            errors.push("MultipleDrivers must have at least one driver".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### Completeness Check
```rust
pub fn is_pr_ready(justification: &RegressionJustification) -> bool {
    // All 6 mandatory elements must be present
    justification.validate().is_ok()
        && !justification.root_cause.explanation.is_empty()
        && justification.confidence > 0.0
        && justification.delta.delta != 0.0
}
```

---

## Tests

### Justification Validation
```rust
#[test]
fn test_regression_justification_complete() {
    let justification = RegressionJustification {
        regression_type: RegressionType::CostIncrease,
        driver: RegressionDriver::InstanceSizeChange {
            from: "t3.medium".to_string(),
            to: "t3.large".to_string(),
        },
        delta: CostDelta::new(30.37, 60.74, Interval::Month),
        confidence: 0.92,
        dependencies: DependencyContext {
            direct_dependencies: vec!["aws_security_group.web".to_string()],
            downstream_count: 3,
            modules_affected: vec!["compute".to_string()],
            on_critical_path: true,
        },
        root_cause: RootCause {
            trigger: RootCauseTrigger::CodeChange {
                file: "main.tf".to_string(),
                line: Some(42),
            },
            explanation: "Increased instance size for better performance".to_string(),
            provenance: None,
        },
        details: Some("Full analysis details here".to_string()),
        recommendations: vec![
            Recommendation {
                action: "Consider using auto-scaling instead".to_string(),
                impact: None,
                confidence: 0.8,
                effort: EffortLevel::Medium,
            },
        ],
    };

    // Must pass validation
    assert!(justification.validate().is_ok());

    // Must be PR-ready
    assert!(is_pr_ready(&justification));
}

#[test]
fn test_pr_comment_format() {
    let justification = sample_justification();

    let pr_comment = format_pr_comment(&justification);

    // Must include all mandatory sections
    assert!(pr_comment.contains("## 📈 Cost Regression Detected"));
    assert!(pr_comment.contains("**Type:**"));
    assert!(pr_comment.contains("**Impact:**"));
    assert!(pr_comment.contains("**Confidence:**"));
    assert!(pr_comment.contains("### 🔍 Analysis"));
    assert!(pr_comment.contains("### 📊 Cost Breakdown"));
    assert!(pr_comment.contains("### 🔗 Dependencies"));
    assert!(pr_comment.contains("### 🎯 Root Cause"));

    // Must be under 15 lines (before <details>)
    let lines_before_details = pr_comment
        .split("<details>")
        .next()
        .unwrap()
        .lines()
        .count();

    assert!(
        lines_before_details <= 15,
        "PR comment too long: {} lines",
        lines_before_details
    );
}
```

---

## CLI Integration

```rust
// Generate PR comment
pub fn generate_pr_comment(plan_path: &Path) -> CostPilotResult<String> {
    let plan = parse_terraform_plan(plan_path)?;
    let regressions = detect_regressions(&plan)?;

    let mut comments = Vec::new();

    for regression in regressions {
        let justification = analyze_regression(&regression)?;
        let comment = format_pr_comment(&justification);
        comments.push(comment);
    }

    Ok(comments.join("\n\n---\n\n"))
}

// CLI command
// costpilot pr-comment terraform.plan.json
```

---

## Breaking This Contract

**Severity: HIGH (impacts PR comment quality)**

**Forbidden:**
- ❌ Missing mandatory fields (type, driver, delta, confidence, dependencies, root cause)
- ❌ Vague root cause ("something changed")
- ❌ No dependency context
- ❌ Confidence = 0
- ❌ Empty explanation

**Required:**
- ✅ All 6 mandatory elements present
- ✅ Specific root cause explanation
- ✅ Quantified delta ($X/month, +Y%)
- ✅ Dependency context included
- ✅ PR-ready format (under 15 lines)

---

## Benefits

### PR Review Quality
- **Complete context** - Reviewers see full picture
- **Actionable** - Recommendations included
- **Professional** - Clean, structured format
- **Fast review** - All info in one place

### Developer Experience
- **No guessing** - Clear root cause
- **Confidence** - Know how certain we are
- **Dependencies** - See downstream impact
- **Recommendations** - Know what to do

### Management Visibility
- **Cost tracking** - See cost trends
- **Risk assessment** - Understand severity
- **Audit trail** - Complete provenance
- **Decision support** - Informed tradeoffs

---

## Version History

- **1.0.0** (2025-12-06) - Initial regression justification contract

---

**This contract ensures every regression has complete, professional justification.**
