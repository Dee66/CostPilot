# PR Comment Quality Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

CostPilot PR comments are **marketing-quality**, **copy-paste safe**, and **under 15 lines**. They must look professional enough to screenshot for a landing page.

---

## Core Principles

### 1. Marketing Quality
Every PR comment should be **screenshot-worthy** for marketing materials.

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
```

**This is good enough to use in marketing.**

---

## Format Rules

### Template Structure
```markdown
## {emoji} CostPilot Analysis

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

### Rules
1. **Header:** Always "## 💰 CostPilot Analysis"
2. **Delta line:** Always bold, formatted with currency and percentage
3. **Sections:** 📊 Summary, 🔍 Key Findings, 💡 Recommendations, 📈 Confidence
4. **Line limit:** Maximum 15 lines before `<details>` tag
5. **Emoji:** Consistent (💰 📊 🔍 💡 📈)
6. **Currency:** Always formatted ($X.XX)
7. **Percentages:** Always include (+X.X%)
8. **Severity:** Always uppercase (CRITICAL, HIGH, MEDIUM, LOW, INFO)

---

## Implementation

```rust
pub struct PrComment {
    pub delta: CostDelta,
    pub resources_changed: usize,
    pub severity: Severity,
    pub key_findings: Vec<String>,  // Max 5
    pub recommendations: Vec<String>,  // Max 3
    pub confidence: f64,
    pub detailed_breakdown: String,
}

impl PrComment {
    pub fn format(&self) -> String {
        // Validate before formatting
        self.validate();

        let mut output = String::new();

        // Header
        output.push_str("## 💰 CostPilot Analysis\n\n");

        // Delta line
        output.push_str(&format!(
            "**Monthly Cost Delta:** {} ({}%)\n\n",
            format_currency(self.delta.delta),
            format_percentage(self.delta.percentage)
        ));

        // Summary section
        output.push_str("### 📊 Summary\n");
        output.push_str(&format!("- **Resources Changed:** {}\n", self.resources_changed));
        output.push_str(&format!(
            "- **Cost Impact:** {}\n",
            self.format_impact()
        ));
        output.push_str(&format!("- **Severity:** {}\n\n", self.severity.as_str()));

        // Key Findings section
        output.push_str("### 🔍 Key Findings\n");
        for (i, finding) in self.key_findings.iter().enumerate().take(5) {
            output.push_str(&format!("{}. {}\n", i + 1, finding));
        }
        output.push('\n');

        // Recommendations section
        if !self.recommendations.is_empty() {
            output.push_str("### 💡 Recommendations\n");
            for rec in self.recommendations.iter().take(3) {
                output.push_str(&format!("- {}\n", rec));
            }
            output.push('\n');
        }

        // Confidence section
        output.push_str("### 📈 Confidence\n");
        output.push_str(&format!(
            "{}% confidence in estimates\n\n",
            (self.confidence * 100.0) as u32
        ));

        // Detailed breakdown (collapsible)
        output.push_str("---\n");
        output.push_str("<details>\n");
        output.push_str("<summary>View detailed breakdown</summary>\n\n");
        output.push_str(&self.detailed_breakdown);
        output.push_str("\n\n</details>\n");

        output
    }

    fn format_impact(&self) -> String {
        let delta = self.delta.delta;
        if delta > 0.0 {
            format!("{}/month increase", format_currency(delta))
        } else if delta < 0.0 {
            format!("{}/month decrease", format_currency(delta.abs()))
        } else {
            "No cost change".to_string()
        }
    }

    fn validate(&self) {
        // Must have 1-5 key findings
        assert!(
            !self.key_findings.is_empty() && self.key_findings.len() <= 5,
            "Must have 1-5 key findings"
        );

        // Must have 0-3 recommendations
        assert!(
            self.recommendations.len() <= 3,
            "Must have at most 3 recommendations"
        );

        // Confidence must be valid
        assert!(
            self.confidence >= 0.0 && self.confidence <= 1.0,
            "Confidence must be 0.0-1.0"
        );

        // Each finding must be under 80 chars
        for finding in &self.key_findings {
            assert!(
                finding.len() <= 80,
                "Finding too long: {} chars (max 80)",
                finding.len()
            );
        }

        // Each recommendation must be under 80 chars
        for rec in &self.recommendations {
            assert!(
                rec.len() <= 80,
                "Recommendation too long: {} chars (max 80)",
                rec.len()
            );
        }
    }
}
```

---

## Line Count Enforcement

```rust
impl PrComment {
    pub fn line_count_before_details(&self) -> usize {
        let formatted = self.format();
        let before_details = formatted.split("<details>").next().unwrap();
        before_details.lines().count()
    }

    pub fn is_under_line_limit(&self) -> bool {
        self.line_count_before_details() <= 15
    }
}

#[test]
fn test_pr_comment_under_15_lines() {
    let comment = sample_pr_comment();

    assert!(
        comment.is_under_line_limit(),
        "PR comment exceeds 15 lines: {} lines",
        comment.line_count_before_details()
    );
}
```

---

## Copy-Paste Safety

### No Special Characters
```rust
// ❌ FORBIDDEN (breaks copy-paste)
"Cost: $30.37\u{200B}month"  // Zero-width space

// ✅ REQUIRED
"Cost: $30.37/month"

// ❌ FORBIDDEN (non-ASCII minus)
"Cost: −$30.37"  // Unicode minus (U+2212)

// ✅ REQUIRED
"Cost: -$30.37"  // ASCII hyphen-minus (U+002D)
```

### No Trailing Whitespace
```rust
fn trim_lines(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_no_trailing_whitespace() {
    let comment = sample_pr_comment().format();

    for (i, line) in comment.lines().enumerate() {
        assert!(
            !line.ends_with(' ') && !line.ends_with('\t'),
            "Line {} has trailing whitespace",
            i + 1
        );
    }
}
```

### No Tab Characters
```rust
#[test]
fn test_no_tab_characters() {
    let comment = sample_pr_comment().format();

    assert!(
        !comment.contains('\t'),
        "PR comment contains tab characters"
    );
}
```

---

## Emoji Consistency

```rust
pub struct PrCommentEmoji;

impl PrCommentEmoji {
    pub const HEADER: &'static str = "💰";
    pub const SUMMARY: &'static str = "📊";
    pub const FINDINGS: &'static str = "🔍";
    pub const RECOMMENDATIONS: &'static str = "💡";
    pub const CONFIDENCE: &'static str = "📈";

    // Severity emoji
    pub const CRITICAL: &'static str = "🔴";
    pub const HIGH: &'static str = "🟠";
    pub const MEDIUM: &'static str = "🟡";
    pub const LOW: &'static str = "🔵";
    pub const INFO: &'static str = "⚪";

    // Change type emoji
    pub const INCREASE: &'static str = "📈";
    pub const DECREASE: &'static str = "📉";
    pub const NEW: &'static str = "🆕";
    pub const DELETED: &'static str = "🗑️";
    pub const MODIFIED: &'static str = "✏️";
}

#[test]
fn test_emoji_consistency() {
    let comment = sample_pr_comment().format();

    // Must use standard emoji
    assert!(comment.contains(PrCommentEmoji::HEADER));
    assert!(comment.contains(PrCommentEmoji::SUMMARY));
    assert!(comment.contains(PrCommentEmoji::FINDINGS));
    assert!(comment.contains(PrCommentEmoji::CONFIDENCE));
}
```

---

## Detailed Breakdown Format

### Table Format
```rust
pub fn format_detailed_breakdown(regressions: &[Regression]) -> String {
    let mut output = String::new();

    // New resources
    let new_resources: Vec<_> = regressions
        .iter()
        .filter(|r| matches!(r.regression_type, RegressionType::NewResource))
        .collect();

    if !new_resources.is_empty() {
        output.push_str("#### New Resources\n");
        output.push_str("| Resource | Type | Cost/Month |\n");
        output.push_str("|----------|------|------------|\n");

        for resource in new_resources {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                resource.resource_id,
                resource.resource_type,
                format_monthly_cost(resource.cost)
            ));
        }
        output.push('\n');
    }

    // Modified resources
    let modified_resources: Vec<_> = regressions
        .iter()
        .filter(|r| matches!(r.regression_type, RegressionType::ModifiedResource))
        .collect();

    if !modified_resources.is_empty() {
        output.push_str("#### Modified Resources\n");
        output.push_str("| Resource | Before | After | Delta |\n");
        output.push_str("|----------|--------|-------|-------|\n");

        for resource in modified_resources {
            output.push_str(&format!(
                "| {} | {}<br>{} | {}<br>{} | {} |\n",
                resource.resource_id,
                resource.before_type,
                format_monthly_cost(resource.before_cost),
                resource.after_type,
                format_monthly_cost(resource.after_cost),
                format_cost_delta(resource.delta)
            ));
        }
        output.push('\n');
    }

    // Deleted resources
    let deleted_resources: Vec<_> = regressions
        .iter()
        .filter(|r| matches!(r.regression_type, RegressionType::DeletedResource))
        .collect();

    if !deleted_resources.is_empty() {
        output.push_str("#### Deleted Resources\n");
        output.push_str("| Resource | Type | Previous Cost |\n");
        output.push_str("|----------|------|---------------|\n");

        for resource in deleted_resources {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                resource.resource_id,
                resource.resource_type,
                format_monthly_cost(resource.cost)
            ));
        }
        output.push('\n');
    }

    output
}
```

---

## Real-World Examples

### Example 1: Cost Increase
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

### Example 2: Cost Decrease
```markdown
## 💰 CostPilot Analysis

**Monthly Cost Delta:** -$120.00 (-45.2%)

### 📊 Summary
- **Resources Changed:** 1
- **Cost Impact:** $120.00/month decrease
- **Severity:** LOW

### 🔍 Key Findings
1. Deleted unused RDS instance: -$146.40/month
2. Added smaller RDS instance (db.t3.small): +$26.40/month
3. Net savings: -$120.00/month

### 💡 Recommendations
- Review performance metrics after deployment
- Consider Reserved Instance for additional savings

### 📈 Confidence
88% confidence in estimates

---
<details>
<summary>View detailed breakdown</summary>

#### Deleted Resources
| Resource | Type | Previous Cost |
|----------|------|---------------|
| aws_db_instance.old | db.t3.medium | $146.40 |

#### New Resources
| Resource | Type | Cost/Month |
|----------|------|------------|
| aws_db_instance.new | db.t3.small | $26.40 |

</details>
```

### Example 3: Complex Change
```markdown
## 💰 CostPilot Analysis

**Monthly Cost Delta:** +$285.60 (+62.8%)

### 📊 Summary
- **Resources Changed:** 8
- **Cost Impact:** $285.60/month increase
- **Severity:** HIGH

### 🔍 Key Findings
1. Added Application Load Balancer: +$16.20/month
2. Added 3 EC2 instances for auto-scaling: +$91.11/month
3. Increased RDS storage (100GB → 500GB): +$80.00/month
4. Added ElastiCache cluster: +$98.29/month

### 💡 Recommendations
- Evaluate if 3 instances are needed immediately
- Consider S3 archival for old RDS data

### 📈 Confidence
95% confidence in estimates

---
<details>
<summary>View detailed breakdown</summary>

[Detailed table here...]

</details>
```

---

## Validation Tests

### Format Tests
```rust
#[test]
fn test_pr_comment_format_valid() {
    let comment = sample_pr_comment().format();

    // Must have header
    assert!(comment.starts_with("## 💰 CostPilot Analysis"));

    // Must have all required sections
    assert!(comment.contains("### 📊 Summary"));
    assert!(comment.contains("### 🔍 Key Findings"));
    assert!(comment.contains("### 📈 Confidence"));

    // Must have details section
    assert!(comment.contains("<details>"));
    assert!(comment.contains("</details>"));

    // Must be under 15 lines before details
    let before_details = comment.split("<details>").next().unwrap();
    assert!(before_details.lines().count() <= 15);
}

#[test]
fn test_pr_comment_copy_paste_safe() {
    let comment = sample_pr_comment().format();

    // No tabs
    assert!(!comment.contains('\t'));

    // No trailing whitespace
    for line in comment.lines() {
        assert!(!line.ends_with(' '));
    }

    // Only ASCII minus (not Unicode minus)
    assert!(!comment.contains('−'));  // U+2212

    // No zero-width spaces
    assert!(!comment.contains('\u{200B}'));
}

#[test]
fn test_pr_comment_currency_formatting() {
    let comment = sample_pr_comment().format();

    // All costs must have currency symbol
    let cost_pattern = regex::Regex::new(r"\$\d+\.\d{2}").unwrap();
    let costs = cost_pattern.find_iter(&comment).count();

    assert!(costs > 0, "PR comment must include formatted costs");
}
```

---

## CLI Integration

```rust
// Generate PR comment
// costpilot pr-comment terraform.plan.json

pub fn generate_pr_comment(plan_path: &Path) -> CostPilotResult<String> {
    let plan = parse_terraform_plan(plan_path)?;
    let regressions = detect_regressions(&plan)?;

    // Aggregate data
    let total_delta = regressions.iter()
        .map(|r| r.delta.delta)
        .sum();

    let resources_changed = regressions.len();

    let severity = compute_overall_severity(&regressions);

    let key_findings = extract_key_findings(&regressions, 5);

    let recommendations = generate_recommendations(&regressions, 3);

    let confidence = compute_average_confidence(&regressions);

    let detailed_breakdown = format_detailed_breakdown(&regressions);

    // Build comment
    let comment = PrComment {
        delta: CostDelta::new(
            regressions[0].delta.old_cost,
            regressions[0].delta.new_cost,
            Interval::Month,
        ),
        resources_changed,
        severity,
        key_findings,
        recommendations,
        confidence,
        detailed_breakdown,
    };

    Ok(comment.format())
}
```

---

## Breaking This Contract

**Severity: HIGH (impacts brand perception)**

**Forbidden:**
- ❌ More than 15 lines before `<details>`
- ❌ Inconsistent emoji
- ❌ Missing currency symbols
- ❌ Trailing whitespace
- ❌ Tab characters
- ❌ More than 5 key findings
- ❌ More than 3 recommendations

**Required:**
- ✅ Marketing-quality format
- ✅ Under 15 lines (before details)
- ✅ Copy-paste safe (no special chars)
- ✅ Consistent emoji (💰 📊 🔍 💡 📈)
- ✅ Currency formatted ($X.XX)
- ✅ All sections present

---

## Benefits

### Marketing Value
- **Screenshot-worthy** - Good enough for landing page
- **Professional** - Looks like enterprise software
- **Clear** - Easy to understand at a glance
- **Trustworthy** - Confident and precise

### User Experience
- **Concise** - Under 15 lines
- **Expandable** - Details hidden by default
- **Copy-paste safe** - No weird characters
- **Actionable** - Clear recommendations

### Brand Perception
- **Premium** - Feels expensive
- **Polished** - Attention to detail
- **Helpful** - Provides value
- **Trustworthy** - Professional communication

---

## Version History

- **1.0.0** (2025-12-06) - Initial PR comment quality contract

---

**This contract ensures every PR comment is marketing-quality and screenshot-worthy.**
