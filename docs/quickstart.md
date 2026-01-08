# CostPilot Quickstart Guide

## Installation

### From Binary (Recommended)

Download the latest release from GitHub:

```bash
# Linux/macOS
curl -L https://github.com/Dee66/CostPilot/releases/latest/download/costpilot-linux-x64.tar.gz | tar xz
sudo mv costpilot /usr/local/bin/

# Or build from source
git clone https://github.com/Dee66/CostPilot.git
cd CostPilot
cargo build --release
sudo cp target/release/costpilot /usr/local/bin/
```

### Verify Installation

```bash
costpilot --version
```

## Basic Usage

### 1. Scan Infrastructure Plan

CostPilot analyzes Terraform plan JSON files:

```bash
# Generate Terraform plan
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

# Scan for cost issues
costpilot scan --plan plan.json
```

**Expected Input**: Terraform JSON plan (generated via `terraform show -json`)

### 2. Example Output

```
CostPilot Cost Analysis
=======================

Detected Changes:
  ✓ aws_instance.web_server - New EC2 t3.medium instance
  ⚠ aws_rds_instance.database - Upgraded from db.t3.small to db.t3.large

Estimated Monthly Cost Delta: +$147.50

Policy Violations:
  ❌ COST_THRESHOLD: Change exceeds $100 limit (Policy: production-budget)

Recommendation: Review RDS instance sizing before merge.
```

### 3. Check Cost Differences

```bash
# Compare two plans
costpilot diff --before old-plan.json --after new-plan.json
```

### 4. Set Up Policies

Create `~/.costpilot/policy.yml`:

```yaml
policies:
  - name: production-budget
    rules:
      - type: cost_threshold
        max_delta: 100
        severity: error
```

Then scan with policy enforcement:

```bash
costpilot scan --plan plan.json --policy ~/.costpilot/policy.yml
```

## Premium Features

CostPilot Free includes basic cost detection. Premium features:
- Advanced cost prediction algorithms
- Drift detection and auto-fix suggestions
- Dependency mapping
- SLO enforcement

To enable Premium, place your `license.json` in `~/.costpilot/`:

```bash
# Provided by your CostPilot license issuer
cp license.json ~/.costpilot/
```

## Next Steps

- See [CLI Reference](cli_reference.md) for all commands and flags
- Read [Architecture](architecture.md) for system design
- View [Policies Guide](policies_guide.md) for policy configuration
