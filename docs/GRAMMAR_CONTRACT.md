# Grammar and Style Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

CostPilot's output must be **consistent, professional, and polished**. This contract enforces grammar rules, style guidelines, and formatting standards that make the product feel expensive and trustworthy.

---

## Core Principles

### 1. No Hedging Language
```
❌ FORBIDDEN:
- "This might cost around..."
- "We think this could be..."
- "Possibly $100/month"
- "Maybe you should..."
- "It seems like..."

✅ REQUIRED:
- "Estimated cost: $30.37/month"
- "Cost increase: $45.20/month"
- "Confidence: 92%"
- "Recommended: Reduce instance size"
- "Detection: NAT Gateway overuse"
```

**Why:** Hedging undermines confidence. Be direct and precise.

### 2. No Undefined Terms
```
❌ FORBIDDEN:
- "High utilization"  (what's "high"?)
- "Expensive resource"  (compared to what?)
- "Recent changes"  (how recent?)
- "Many resources"  (how many?)

✅ REQUIRED:
- "CPU utilization: 87%"
- "Cost: $450/month (3.2× baseline)"
- "Changed: 2 hours ago"
- "Resources: 47"
```

**Why:** Vague terms force users to guess. Provide numbers.

### 3. No Randomized Wording
```
❌ FORBIDDEN (non-deterministic):
let messages = vec![
    "This looks expensive",
    "This seems costly",
    "This appears pricey"
];
let msg = messages[rand::random()];

✅ REQUIRED (deterministic template):
fn format_cost_alert(cost: f64, threshold: f64) -> String {
    format!(
        "Cost ${:.2}/month exceeds threshold ${:.2}/month",
        cost, threshold
    )
}
```

**Why:** Output must be deterministic. Same input = same wording.

### 4. Stable Sentence Templates
```rust
// Severity templates (NEVER change wording)
pub const SEVERITY_CRITICAL: &str = "CRITICAL: Immediate action required";
pub const SEVERITY_HIGH: &str = "HIGH: Address within 24 hours";
pub const SEVERITY_MEDIUM: &str = "MEDIUM: Review within 1 week";
pub const SEVERITY_LOW: &str = "LOW: Monitor for trends";
pub const SEVERITY_INFO: &str = "INFO: No action required";

// Cost change templates
pub fn format_cost_increase(delta: f64, percentage: f64) -> String {
    format!(
        "Cost increase: ${:.2}/month (+{:.1}%)",
        delta, percentage
    )
}

pub fn format_cost_decrease(delta: f64, percentage: f64) -> String {
    format!(
        "Cost decrease: ${:.2}/month (-{:.1}%)",
        delta.abs(), percentage.abs()
    )
}

// Detection templates
pub fn format_detection(
    detection_type: &str,
    resource_count: usize,
    impact: f64,
) -> String {
    format!(
        "Detected: {} ({} resource{}, ${:.2}/month impact)",
        detection_type,
        resource_count,
        if resource_count == 1 { "" } else { "s" },
        impact
    )
}
```

### 5. Severity Language Consistent
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Severity::Critical => "Immediate action required",
            Severity::High => "Address within 24 hours",
            Severity::Medium => "Review within 1 week",
            Severity::Low => "Monitor for trends",
            Severity::Info => "No action required",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
            Severity::Info => "⚪",
        }
    }
}
```

### 6. Cost Numbers Always Formatted with Currency
```rust
pub fn format_currency(amount: f64) -> String {
    format!("${:.2}", amount)
}

pub fn format_monthly_cost(amount: f64) -> String {
    format!("${:.2}/month", amount)
}

pub fn format_hourly_cost(amount: f64) -> String {
    format!("${:.4}/hour", amount)
}

pub fn format_cost_delta(old: f64, new: f64) -> String {
    let delta = new - old;
    let sign = if delta >= 0.0 { "+" } else { "" };
    format!("{}{:.2}", sign, delta)
}

// ❌ FORBIDDEN
println!("Cost: {}", 30.368);  // No currency symbol!

// ✅ REQUIRED
println!("Cost: {}", format_monthly_cost(30.368));
```

---

## Style Rules

### Capitalization
```
Resource Types: PascalCase
  ✅ aws_instance, aws_db_instance, aws_nat_gateway

Severity Levels: UPPERCASE
  ✅ CRITICAL, HIGH, MEDIUM, LOW, INFO

Actions: Sentence case
  ✅ "Reduce instance size"
  ✅ "Enable S3 lifecycle policy"
  ✅ "Review NAT Gateway usage"

Headers: Title Case
  ✅ Cost Analysis Summary
  ✅ Policy Violations
  ✅ Recommended Actions
```

### Punctuation
```
Lists: End with period only if complete sentence
  ✅ "- Detected 3 NAT Gateways"
  ✅ "- Cost increase of $45.20/month"
  ❌ "- Detected 3 NAT Gateways."  (no period)

Sentences: Always end with period
  ✅ "This resource exceeds the cost threshold."
  ❌ "This resource exceeds the cost threshold"

Numbers: Use commas for thousands
  ✅ "1,247 resources"
  ✅ "$1,450.00/month"
  ❌ "1247 resources"
```

### Abbreviations
```
Allowed (Common):
  ✅ EC2, RDS, S3, VPC, NAT, ALB, NLB, EBS, SLO
  ✅ CPU, RAM, GB, TB, MB
  ✅ IAM, ARN, CIDR

Forbidden (Spell out):
  ❌ "Pred" → ✅ "Prediction"
  ❌ "Cfg" → ✅ "Configuration"
  ❌ "Res" → ✅ "Resource"
  ❌ "Dep" → ✅ "Deployment"
```

---

## Message Templates

### Cost Analysis
```rust
pub struct CostMessage {
    pub template: &'static str,
}

impl CostMessage {
    pub const INCREASE_DETECTED: &'static str =
        "Cost increase detected: {delta} ({percentage}% over baseline)";

    pub const DECREASE_DETECTED: &'static str =
        "Cost decrease: {delta} ({percentage}% under baseline)";

    pub const THRESHOLD_EXCEEDED: &'static str =
        "Cost ${cost}/month exceeds threshold ${threshold}/month";

    pub const BASELINE_MATCH: &'static str =
        "Cost ${cost}/month within {percentage}% of baseline";

    pub const NO_BASELINE: &'static str =
        "No baseline found for module {module}. Consider running 'costpilot baseline set'";
}
```

### Policy Violations
```rust
pub const POLICY_VIOLATION_TEMPLATE: &str =
    "{severity}: Policy '{policy_name}' violated\n\
     Resource: {resource}\n\
     Rule: {rule}\n\
     Action: {action}";

pub const POLICY_PASSED_TEMPLATE: &str =
    "✅ All {count} policies passed";

pub const POLICY_WARNING_TEMPLATE: &str =
    "⚠️  {count} warning{plural}: Review recommended";
```

### Recommendations
```rust
pub const RECOMMENDATION_TEMPLATE: &str =
    "Recommendation: {action}\n\
     Impact: {impact}\n\
     Confidence: {confidence}%";

pub const AUTOFIX_AVAILABLE: &str =
    "💡 Autofix available: Run 'costpilot autofix patch {resource}'";

pub const AUTOFIX_APPLIED: &str =
    "✅ Autofix applied: {action}\n\
     Estimated savings: {savings}/month";
```

---

## Formatting Rules

### Numbers
```rust
pub fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

// Examples:
// 1_247 → "1.2K"
// 1_450_000 → "1.5M"
// 347 → "347"
```

### Percentages
```rust
pub fn format_percentage(value: f64) -> String {
    if value.abs() >= 100.0 {
        format!("{:.0}%", value)
    } else if value.abs() >= 10.0 {
        format!("{:.1}%", value)
    } else {
        format!("{:.2}%", value)
    }
}

// Examples:
// 342.7 → "343%"
// 45.3 → "45.3%"
// 2.47 → "2.47%"
```

### Time Durations
```rust
pub fn format_duration_ms(ms: u128) -> String {
    if ms >= 1000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

// Examples:
// 1_450 → "1.45s"
// 287 → "287ms"
```

---

## Markdown Output Rules

### Headers
```markdown
# Primary Header (Title Case)
## Secondary Header (Title Case)
### Tertiary Header (Title Case)

Never use:
#### Fourth level (too deep)
```

### Lists
```markdown
✅ Unordered lists use `-`
- Item one
- Item two
- Item three

✅ Ordered lists use `1.`
1. First step
2. Second step
3. Third step

❌ Never mix:
- Don't do this
1. And then this
```

### Code Blocks
```markdown
✅ Always specify language:
```rust
let cost = 30.37;
```

```json
{"cost": 30.37}
```

❌ Never use generic:
```
let cost = 30.37;
```
```

### Tables
```markdown
✅ Always align with pipes:
| Resource | Cost/Month | Severity |
|----------|------------|----------|
| Instance | $30.37     | LOW      |
| Database | $146.40    | MEDIUM   |

❌ Never misalign:
| Resource | Cost/Month |
|---|---|
| Instance |$30.37|
```

---

## PR Comment Format

### Template
```markdown
## 💰 CostPilot Analysis

**Monthly Cost Delta:** {delta} ({percentage}%)

### 📊 Summary
- **Resources Changed:** {count}
- **Cost Impact:** {impact}
- **Severity:** {severity}

### 🔍 Key Findings
1. {finding_1}
2. {finding_2}
3. {finding_3}

### 💡 Recommendations
- {recommendation_1}
- {recommendation_2}

### 📈 Confidence
{confidence}% confidence in estimates

---
<details>
<summary>View detailed breakdown</summary>

{detailed_analysis}

</details>
```

### Example
```markdown
## 💰 CostPilot Analysis

**Monthly Cost Delta:** +$45.20 (+31.5%)

### 📊 Summary
- **Resources Changed:** 3
- **Cost Impact:** $45.20/month increase
- **Severity:** MEDIUM

### 🔍 Key Findings
1. Added 2 NAT Gateways: +$32.88/month
2. Increased EC2 instance size (t3.medium → t3.large): +$12.32/month

### 💡 Recommendations
- Consider using single NAT Gateway with route table optimization
- Verify EC2 instance size requirements

### 📈 Confidence
92% confidence in estimates

---
<details>
<summary>View detailed breakdown</summary>

#### New Resources
| Resource | Type | Cost/Month |
|----------|------|------------|
| aws_nat_gateway.az1 | NAT Gateway | $32.88 |
| aws_nat_gateway.az2 | NAT Gateway | $32.88 |

#### Modified Resources
| Resource | Before | After | Delta |
|----------|--------|-------|-------|
| aws_instance.web | t3.medium<br>$30.37 | t3.large<br>$60.74 | +$30.37 |

</details>
```

---

## Validation Tests

### Grammar Validation
```rust
#[test]
fn test_no_hedging_language() {
    let output = generate_explanation(&sample_resource());

    let forbidden_words = [
        "might", "maybe", "possibly", "could be",
        "seems like", "appears to", "probably",
    ];

    for word in &forbidden_words {
        assert!(
            !output.to_lowercase().contains(word),
            "Found hedging language: '{}'",
            word
        );
    }
}

#[test]
fn test_all_costs_have_currency_symbol() {
    let output = generate_cost_report(&sample_plan());

    // Find all numbers that look like costs
    let cost_pattern = regex::Regex::new(r"\b\d+\.\d{2}\b").unwrap();

    for m in cost_pattern.find_iter(&output) {
        let cost_str = m.as_str();
        let start = m.start();

        // Check if preceded by $
        if start > 0 {
            let prev_char = output.chars().nth(start - 1).unwrap();
            assert_eq!(
                prev_char, '$',
                "Cost {} missing currency symbol",
                cost_str
            );
        }
    }
}

#[test]
fn test_severity_consistency() {
    let severities = vec![
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ];

    for severity in severities {
        let str = severity.as_str();

        // Must be uppercase
        assert_eq!(str, str.to_uppercase());

        // Must match enum name
        assert!(matches!(
            severity,
            Severity::Critical if str == "CRITICAL" |
            Severity::High if str == "HIGH" |
            Severity::Medium if str == "MEDIUM" |
            Severity::Low if str == "LOW" |
            Severity::Info if str == "INFO"
        ));
    }
}
```

---

## Breaking This Contract

**Severity: HIGH (impacts brand perception)**

**Forbidden:**
- ❌ Using hedging language
- ❌ Undefined or vague terms
- ❌ Randomized wording
- ❌ Inconsistent severity labels
- ❌ Costs without currency symbols
- ❌ Non-deterministic templates

**Required:**
- ✅ Direct, precise language
- ✅ Numerical specificity
- ✅ Stable templates
- ✅ Consistent formatting
- ✅ Professional tone
- ✅ Currency-formatted costs

---

## Benefits

### User Experience
- **Clear communication** - No ambiguity
- **Professional feel** - Polished output
- **Easy parsing** - Consistent format
- **Trust building** - Confident language

### Developer Experience
- **Predictable output** - Same input = same wording
- **Easy testing** - Stable assertions
- **Simple maintenance** - Templates in one place
- **Quick debugging** - Consistent error messages

### Brand Perception
- **Premium product** - Feels expensive
- **Attention to detail** - Shows craftsmanship
- **Trustworthy** - Professional communication
- **Enterprise-ready** - Serious tool

---

## Version History

- **1.0.0** (2025-12-06) - Initial grammar contract

---

**This contract ensures CostPilot always communicates professionally and consistently.**
