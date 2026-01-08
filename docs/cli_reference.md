# CLI Reference

## Commands

### `scan`
Scan infrastructure plan for cost issues and policy violations

**Usage:**
```bash
costpilot scan --plan <PLAN_FILE> [OPTIONS]
```

**Required Flags:**
- `--plan <FILE>` - Terraform JSON plan file (from `terraform show -json`)

**Optional Flags:**
- `--policy <FILE>` - Policy file for enforcement (YAML format)
- `--baseline <FILE>` - Baseline file for cost comparison
- `--format <FORMAT>` - Output format: `text`, `json`, `yaml` (default: `text`)
- `--silent` - Suppress output unless violations found
- `--fail-on-violation` - Exit with non-zero code on policy violations

**Examples:**
```bash
# Basic scan
costpilot scan --plan plan.json

# Scan with policy enforcement
costpilot scan --plan plan.json --policy production-policy.yml

# JSON output for CI/CD
costpilot scan --plan plan.json --format json --fail-on-violation
```

---

### `diff`
Show cost differences between two infrastructure versions

**Usage:**
```bash
costpilot diff --before <OLD_PLAN> --after <NEW_PLAN>
```

**Required Flags:**
- `--before <FILE>` - Previous plan JSON
- `--after <FILE>` - Current plan JSON

**Optional Flags:**
- `--format <FORMAT>` - Output format: `text`, `json`

**Example:**
```bash
costpilot diff --before baseline-plan.json --after current-plan.json
```

---

### `autofix` _(Premium)_
Generate fixes for detected cost issues

**Usage:**
```bash
costpilot autofix --plan <PLAN_FILE> --output <OUTPUT_FILE>
```

**Required Flags:**
- `--plan <FILE>` - Terraform JSON plan
- `--output <FILE>` - Output file for suggested fixes

**Example:**
```bash
costpilot autofix --plan plan.json --output fixes.tf
```

---

### `map` _(Premium)_
Generate dependency maps for infrastructure resources

**Usage:**
```bash
costpilot map --plan <PLAN_FILE> [OPTIONS]
```

**Required Flags:**
- `--plan <FILE>` - Terraform JSON plan

**Optional Flags:**
- `--output <FILE>` - Output file for dependency graph (JSON)
- `--depth <N>` - Maximum dependency depth (default: unlimited)

**Example:**
```bash
costpilot map --plan plan.json --output dependencies.json
```

---

### `slo-check` _(Premium)_
Check SLO compliance for infrastructure changes

**Usage:**
```bash
costpilot slo-check --plan <PLAN_FILE> --slo <SLO_FILE>
```

**Required Flags:**
- `--plan <FILE>` - Terraform JSON plan
- `--slo <FILE>` - SLO configuration YAML

**Example:**
```bash
costpilot slo-check --plan plan.json --slo slo-config.yml
```

---

### `version`
Show CostPilot version and edition

**Usage:**
```bash
costpilot --version
```

---

### `license`
Display current license information (Premium only)

**Usage:**
```bash
costpilot license
```

Shows licensee, expiration date, and enabled features.

---

## Global Flags

- `--help` - Show help information
- `--quiet` - Suppress informational messages
- `--color <WHEN>` - Colorize output: `always`, `auto`, `never`

---

## Exit Codes

- `0` - Success (no issues found or analysis complete)
- `1` - Policy violations detected (with `--fail-on-violation`)
- `2` - Invalid input or configuration error
- `3` - License error (Premium features without valid license)

---

## Configuration Files

CostPilot looks for configuration in:
- `~/.costpilot/license.json` - License file (Premium)
- `~/.costpilot/policy.yml` - Default policy file
- `~/.costpilot/config.yml` - Global configuration

---

## Environment Variables

- `COSTPILOT_LICENSE_PATH` - Override default license location
- `COSTPILOT_CONFIG_PATH` - Override config directory (default: `~/.costpilot`)

---

## Examples

### CI/CD Integration

```bash
#!/bin/bash
# .github/workflows/cost-check.yml equivalent

terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

# Fail build if cost exceeds threshold
costpilot scan \
  --plan plan.json \
  --policy .costpilot/ci-policy.yml \
  --format json \
  --fail-on-violation > cost-report.json
```

### Local Development

```bash
# Quick cost check during development
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan | costpilot scan --plan -
```

---

## See Also

- [Quickstart Guide](quickstart.md) - Getting started tutorial
- [Policies Guide](policies_guide.md) - Policy configuration reference
- [SLO Guide](slo_guide.md) - SLO enforcement configuration
