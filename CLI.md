# CostPilot CLI

Zero-IAM command-line tool for cost analysis and governance across Terraform, CloudFormation, and AWS CDK.

## Overview

CostPilot CLI provides deterministic cost analysis without requiring AWS credentials. It parses Infrastructure as Code (IaC) artifacts locally, detects cost-impacting changes, predicts monthly costs, validates policies, and generates actionable fixes—all with zero network calls.

## Installation

```bash
cargo install costpilot
```

## Commands

### `costpilot scan`
Analyze IaC files for cost issues and policy violations.

```bash
# Scan Terraform plan
costpilot scan terraform.tfplan.json

# Scan CloudFormation template
costpilot scan template.yaml

# Scan CDK synthesized output
costpilot scan cdk.out/MyStack.template.json

# With policy enforcement
costpilot scan plan.json --policies .costpilot/policy.yml

# JSON output for CI/CD
costpilot scan plan.json --format json
```

**Features:**
- Auto-detects format (Terraform/CloudFormation/CDK)
- Extracts resource changes
- Predicts monthly and annual costs
- Validates against cost policies
- Checks SLOs and baselines
- Generates detailed explanations

### `costpilot diff`
Show cost differences between current and planned infrastructure.

```bash
costpilot diff --plan terraform.tfplan.json
```

**Output:**
- Resource-by-resource cost breakdown
- Before/after comparisons
- Total delta calculation
- Service-level grouping

### `costpilot autofix`
Generate automated cost optimization recommendations.

```bash
# Snippet mode (safe, deterministic)
costpilot autofix --mode snippet --plan plan.json

# Patch mode (Pro feature, full diffs)
costpilot autofix --mode patch --plan plan.json

# Drift-safe mode with rollback (Beta)
costpilot autofix --drift-safe --plan plan.json
```

**Generates:**
- Terraform/IaC code snippets
- Human-readable rationale
- Cost impact estimates
- Idempotent patches (Pro)
- Automatic rollback on failure (drift-safe)

### `costpilot init`
Initialize CostPilot configuration in your project.

```bash
# Full setup with CI templates
costpilot init

# Skip CI template generation
costpilot init --no-ci
```

**Creates:**
- `.costpilot/` configuration directory
- `policy.yml` with default rules
- `baselines.json` for cost tracking
- `.github/workflows/costpilot.yml` (optional)
- `slo.yml` for cost SLOs

### `costpilot map`
Generate dependency and cost impact visualizations.

```bash
# Mermaid diagram (default)
costpilot map --format mermaid --output diagram.mmd

# GraphViz DOT format
costpilot map --format graphviz --output graph.dot

# Auto-open in HTML viewer
costpilot map
```

**Generates:**
- Resource dependency graphs
- Cross-service impact analysis
- Cost flow visualization
- Cycle detection warnings
- HTML/SVG output with styling

### `costpilot slo`
Validate cost Service Level Objectives and analyze burn rates.

#### Check SLO Compliance

```bash
# Check all SLOs
costpilot slo check

# With custom config path
costpilot slo check --config .costpilot/slo.yml
```

#### Analyze Burn Rate (Enterprise)

Predict when SLO budgets will be breached using linear regression on historical cost data.

```bash
# Analyze burn rate for all SLOs
costpilot slo burn

# With custom paths
costpilot slo burn --slo .costpilot/slo.json --snapshots .costpilot/snapshots/

# JSON output for CI/CD
costpilot slo burn --format json

# Markdown output for PR comments
costpilot slo burn --format markdown

# Custom thresholds
costpilot slo burn --min-snapshots 5 --min-r-squared 0.85
```

**Burn Rate Analysis Features:**
- Linear regression on historical snapshots
- Time-to-breach prediction (days)
- Risk classification (Low/Medium/High/Critical)
- Confidence scoring (R² based)
- Multi-SLO aggregation
- Actionable recommendations

**Example Output:**

```
📊 SLO Burn Rate Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔶 Production Budget ($10,000/month)
  Burn Rate:      $142.86/day
  Projected Cost: $4,428.60/month
  Time to Breach: 8.5 days
  Risk Level:     High
  Confidence:     95% (R² = 0.950)

✅ VPC Module ($3,000/month)
  Burn Rate:      $35.71/day
  Projected Cost: $1,107.10/month
  Time to Breach: No breach predicted
  Risk Level:     Low
  Confidence:     92% (R² = 0.920)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary
  Total SLOs:     2
  At Risk:        1
  Overall Risk:   High

⚠️  Action Required
  1. Review cost drivers in critical SLOs
  2. Consider scaling down or optimizing resources
```

**SLO Validation:**
- Monthly cost budgets
- Per-module spending limits
- Service-specific caps
- Resource count thresholds
- Cost growth rates
- Burn rate thresholds

### `costpilot trends`
Generate cost trend visualizations.

```bash
# SVG trend charts
costpilot trends --format svg --output trends/

# HTML with interactive charts
costpilot trends --format html --output report.html

# Compare against baselines
costpilot trends --with-baselines
```

**Produces:**
- Time-series cost charts (SVG/PNG)
- Regression detection
- Baseline comparison
- Anomaly highlighting
- Projection curves

## Global Flags

```bash
-v, --verbose          Enable detailed logging
-f, --format <type>    Output format: text, json, markdown
--version              Show version
--help                 Show help
```

## Configuration

### `.costpilot/policy.yml`
Define cost governance rules:

```yaml
policies:
  - id: nat_gateway_limit
    name: "NAT Gateway Limit"
    severity: error
    rule: "count('aws_nat_gateway') <= 2"
    
  - id: monthly_budget
    name: "Monthly Budget Cap"
    severity: critical
    rule: "monthly_cost <= 5000"
```

### `.costpilot/baselines.json`
Track expected costs for regression detection:

```json
{
  "module:networking": {
    "expected_monthly": 450.00,
    "last_updated": "2024-12-01",
    "justification": "2 NAT gateways + VPC endpoints"
  }
}
```

### `.costpilot/slo.yml`
Define cost SLOs:

```yaml
slos:
  - name: production_monthly
    type: monthly_cost
    target: 10000
    enforcement: warn
    
  - name: dev_resource_count
    type: resource_count
    target: 50
    enforcement: block
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Cost Analysis
on: [pull_request]

jobs:
  costpilot:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Terraform Plan
        run: terraform plan -out=plan.tfplan
      
      - name: Convert to JSON
        run: terraform show -json plan.tfplan > plan.json
      
      - name: Run CostPilot
        run: |
          costpilot scan plan.json --format markdown > cost-report.md
          
      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('cost-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

## Exit Codes

- **0** - Success, no violations
- **1** - Policy violations (severity: error or critical)
- **2** - SLO violations with block enforcement
- **3** - Parse errors or invalid input
- **130** - User interrupted (Ctrl+C)

## Architecture

**Zero-IAM Design:**
- No AWS credentials required
- No network calls during analysis
- Deterministic output (same input → same result)
- WASM-safe execution
- Offline-first operation

**Supported Formats:**
- Terraform JSON plans (`terraform show -json`)
- CloudFormation templates (JSON/YAML)
- AWS CDK synthesized output (`cdk synth`)

**Core Engines:**
- Detection: Identify cost-impacting changes
- Prediction: Estimate costs from AWS pricing data
- Explain: Generate human-readable insights
- Policy: Validate governance rules
- Autofix: Generate optimization recommendations
- Mapping: Visualize dependencies and impact
- Trends: Track cost over time
- SLO: Enforce cost objectives

## Examples

### Basic Workflow

```bash
# 1. Generate plan
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

# 2. Analyze costs
costpilot scan plan.json

# 3. Check for violations
costpilot scan plan.json --policies .costpilot/policy.yml

# 4. Generate fixes
costpilot autofix --plan plan.json

# 5. Visualize dependencies
costpilot map --output dependency-map.html
```

### CDK Workflow

```bash
# 1. Synthesize CDK app
cdk synth

# 2. Analyze all stacks
costpilot scan cdk.out/

# 3. Check specific stack
costpilot scan cdk.out/MyStack.template.json
```

### CloudFormation Workflow

```bash
# Analyze template
costpilot scan template.yaml

# Validate policies
costpilot scan template.json --policies policy.yml

# Generate cost report
costpilot scan template.yaml --format json > report.json
```

## Performance

- **Speed:** <2s for typical Terraform plans (~100 resources)
- **Memory:** <100MB resident for analysis
- **Determinism:** 100% reproducible results
- **Offline:** Zero network dependency after installation

## Security

- **Zero-IAM:** Never requires AWS credentials
- **Local-only:** No data leaves your machine
- **Deterministic:** Predictable, auditable behavior
- **WASM-safe:** Can run in sandboxed environments
- **No telemetry:** No usage tracking or phone-home

## Support

- **Documentation:** [ARTIFACT_SUPPORT.md](ARTIFACT_SUPPORT.md), [POLICY_ENGINE.md](POLICY_ENGINE.md)
- **Examples:** `examples/` directory
- **Issues:** GitHub repository
- **Roadmap:** `checklist.md`
