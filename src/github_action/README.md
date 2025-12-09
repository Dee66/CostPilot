# CostPilot GitHub Action

💰 AI-powered cost analysis and prediction for Infrastructure as Code

[![GitHub Release](https://img.shields.io/github/v/release/Dee66/CostPilot)](https://github.com/Dee66/CostPilot/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![GitHub Action](https://img.shields.io/badge/GitHub-Action-blue)](https://github.com/marketplace/actions/costpilot)

## Features

- 📊 **Cost Estimation** - Accurate cost predictions for Terraform changes
- 🤖 **AI Prediction** - Machine learning models forecast future costs
- 🛡️ **Policy Enforcement** - Custom policies with DSL for budget limits
- 🔄 **Drift Detection** - SHA256 checksums detect manual changes
- 📈 **SLO Monitoring** - Track cost SLOs with burn rate alerts
- 📉 **Baseline Tracking** - Automatic regression detection
- ✅ **Exemption Management** - Time-bound policy exemptions
- 💬 **PR Comments** - Beautiful cost analysis in pull requests
- 🔒 **Zero Network** - Works without external APIs

## Quick Start

```yaml
name: Cost Analysis
on: [pull_request]

jobs:
  cost-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate Terraform Plan
        run: terraform plan -out=plan.tfplan
      
      - name: Convert Plan to JSON
        run: terraform show -json plan.tfplan > plan.json
      
      - name: Run CostPilot
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          comment_pr: true
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `terraform_plan` | Path to Terraform plan JSON file | No | `plan.json` |
| `config_file` | Path to CostPilot configuration | No | `costpilot.yml` |
| `mode` | Analysis mode: `estimate`, `predict`, `baseline`, `policy`, `drift`, `all` | No | `all` |
| `policy_file` | Path to policy configuration | No | - |
| `baseline_file` | Path to baseline file | No | - |
| `slo_file` | Path to SLO configuration | No | - |
| `exemptions_file` | Path to exemptions file | No | - |
| `fail_on_regression` | Fail build if cost regression detected | No | `true` |
| `fail_on_policy` | Fail build if policy violations found | No | `true` |
| `fail_on_drift` | Fail build if critical drift detected | No | `true` |
| `output_format` | Output format: `text`, `json`, `markdown` | No | `markdown` |
| `comment_pr` | Post results as PR comment | No | `true` |
| `debug` | Enable debug logging | No | `false` |
| `version` | CostPilot version to use | No | `latest` |

## Outputs

| Output | Description |
|--------|-------------|
| `total_cost` | Total estimated monthly cost |
| `cost_delta` | Cost change from baseline |
| `regression_detected` | Whether cost regression was detected |
| `policy_violations` | Number of policy violations |
| `drift_detected` | Whether drift was detected |
| `exit_code` | CostPilot exit code |

## Usage Examples

### Basic Cost Estimation

```yaml
- uses: Dee66/CostPilot@v1
  with:
    terraform_plan: plan.json
    mode: estimate
```

### With Policy Enforcement

```yaml
- uses: Dee66/CostPilot@v1
  with:
    terraform_plan: plan.json
    policy_file: policies/production.json
    fail_on_policy: true
```

### With Baseline Tracking

```yaml
- uses: Dee66/CostPilot@v1
  with:
    terraform_plan: plan.json
    baseline_file: baselines/main.json
    fail_on_regression: true
```

### Complete Workflow

```yaml
name: Infrastructure Cost Control

on:
  pull_request:
    paths:
      - '**.tf'
      - 'terraform/**'

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
      
      - name: Terraform Init
        run: terraform init
      
      - name: Terraform Plan
        run: |
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json
      
      - name: Run CostPilot
        id: costpilot
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          config_file: .costpilot.yml
          policy_file: policies/prod.json
          baseline_file: baselines/main.json
          exemptions_file: exemptions.yaml
          fail_on_regression: true
          fail_on_policy: true
          comment_pr: true
      
      - name: Check Results
        run: |
          echo "Total Cost: $${{ steps.costpilot.outputs.total_cost }}"
          echo "Cost Delta: $${{ steps.costpilot.outputs.cost_delta }}"
          echo "Violations: ${{ steps.costpilot.outputs.policy_violations }}"
```

### Multi-Environment Setup

```yaml
strategy:
  matrix:
    environment: [dev, staging, prod]

steps:
  - uses: Dee66/CostPilot@v1
    with:
      terraform_plan: plans/${{ matrix.environment }}.json
      policy_file: policies/${{ matrix.environment }}.json
      baseline_file: baselines/${{ matrix.environment }}.json
      fail_on_regression: ${{ matrix.environment == 'prod' }}
```

## Configuration

Create `.costpilot.yml` in your repository:

```yaml
version: "1.0"

cost_thresholds:
  max_monthly_cost: 10000
  max_cost_increase_percent: 20

policies:
  - name: "EC2 Instance Types"
    rule: "instance_type in ['t3.micro', 't3.small']"
    action: block
  
  - name: "Budget Limit"
    rule: "monthly_cost <= 5000"
    action: warn

slo:
  target_cost: 4000
  error_budget_percent: 10
  window_days: 30

exemptions:
  enabled: true
  max_duration_days: 90
```

## PR Comment Example

<img width="600" alt="CostPilot PR Comment" src="docs/images/pr-comment-example.png">

```markdown
## 💰 CostPilot Analysis

**Total Monthly Cost:** $3,245.50
**Cost Change:** 📈 +$245.50 (+8.2%)

### Summary
- 5 new resources
- 2 modified resources
- 0 policy violations
- No drift detected

### Cost Breakdown
| Resource | Type | Monthly Cost | Change |
|----------|------|--------------|--------|
| aws_instance.web | t3.large | $62.40 | +$62.40 |
| aws_rds_instance.db | db.t3.medium | $58.40 | +$30.20 |

✅ All policies passed
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - no issues |
| 1 | Policy violations detected |
| 2 | Cost regression detected |
| 3 | Critical drift detected |
| 4 | Configuration error |

## Supported Platforms

- ✅ ubuntu-latest
- ✅ ubuntu-22.04
- ✅ ubuntu-20.04
- ✅ macos-latest
- ✅ macos-13
- ✅ macos-12

## Permissions Required

```yaml
permissions:
  contents: read          # Read repository files
  pull-requests: write    # Post PR comments
```

## Advanced Features

### Approval Workflows

Require approval references for policy violations:

```yaml
policies:
  - name: "Production Changes"
    rule: "tags.environment == 'prod'"
    action: require_approval
    approvers: ["ops-team"]
```

### Drift Detection

Detect configuration drift with SHA256 checksums:

```yaml
drift:
  enabled: true
  protected_resources:
    - "prod-*"
    - "module.security.*"
  critical_attributes:
    - security_group
    - encryption_enabled
```

### SLO Monitoring

Track cost SLOs with burn rate alerts:

```yaml
slo:
  target_cost: 5000
  error_budget_percent: 10
  burn_rate_threshold: 2.0
  alert_on_breach: true
```

## Troubleshooting

### Binary Download Fails

```yaml
- name: Debug CostPilot
  uses: Dee66/CostPilot@v1
  with:
    debug: true
    version: v1.0.0  # Pin specific version
```

### Plan JSON Not Found

```bash
# Ensure Terraform plan is converted to JSON
terraform show -json plan.tfplan > plan.json
ls -la plan.json  # Verify file exists
```

### Permission Denied

```yaml
permissions:
  contents: read
  pull-requests: write  # Required for PR comments
```

## Resources

- 📚 [Documentation](https://github.com/Dee66/CostPilot/blob/main/README.md)
- 🐛 [Report Issues](https://github.com/Dee66/CostPilot/issues)
- 💬 [Discussions](https://github.com/Dee66/CostPilot/discussions)
- 📖 [Examples](https://github.com/Dee66/CostPilot/tree/main/examples)

## License

MIT License - see [LICENSE](LICENSE) for details

## Support

- Community support via [GitHub Issues](https://github.com/Dee66/CostPilot/issues)
- Enterprise support available - contact sales@guardsuite.com
