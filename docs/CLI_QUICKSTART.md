# CostPilot CLI Quickstart Guide

Get started with CostPilot in under 5 minutes. This guide covers the most common workflows for cloud cost optimization.

## Installation

```bash
# From releases (recommended)
curl -L https://github.com/yourusername/costpilot/releases/latest/download/costpilot-linux-x64.tar.gz | tar xz
sudo mv costpilot /usr/local/bin/

# From source
git clone https://github.com/yourusername/costpilot.git
cd costpilot
cargo build --release
sudo cp target/release/costpilot /usr/local/bin/
```

### Shell Completion (Optional)

**Bash:**
```bash
sudo cp completions/costpilot.bash /etc/bash_completion.d/
source ~/.bashrc
```

**Zsh:**
```bash
mkdir -p ~/.zsh/completions
cp completions/costpilot.zsh ~/.zsh/completions/_costpilot
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc
source ~/.zshrc
```

**Fish:**
```bash
cp completions/costpilot.fish ~/.config/fish/completions/
```

## Quick Start

### 1. Initialize Your Project

Set up CostPilot in your Terraform project:

```bash
cd your-terraform-project/
costpilot init
```

This creates:
- `.costpilot/config.yaml` - configuration file
- `.github/workflows/costpilot.yml` - CI/CD template (GitHub Actions)
- `.costpilot/policies/` - policy directory

### 2. Scan a Plan for Cost Issues

Generate a Terraform plan and scan it:

```bash
# Generate plan
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

# Scan for cost issues
costpilot scan --plan plan.json

# With detailed explanations
costpilot scan --plan plan.json --explain

# With custom policy
costpilot scan --plan plan.json --policy .costpilot/policies/production.yaml
```

**Example Output:**
```
🔍 CostPilot Scan Results
═══════════════════════════════════════════════════════════════

📊 Total Monthly Cost: $4,256.80 (confidence: 87%)

🔴 Critical Issues (2)
  • EC2: m5.24xlarge instance in production
    Predicted cost: $3,456/month
    Recommendation: Consider m5.12xlarge ($1,728/month)

  • RDS: db.r5.16xlarge without reserved capacity
    Predicted cost: $8,640/month
    Recommendation: Use reserved instances (save 40%)

🟡 Warnings (5)
  • S3: Lifecycle policy missing on logs-bucket
  • Lambda: 10GB memory allocation for simple function
  • EBS: 3 unattached volumes ($180/month)

💡 Potential Savings: $2,880/month (67% reduction)
```

### 3. Compare Before/After Costs (Diff)

Compare baseline and proposed changes (useful in PRs):

```bash
# Generate baseline and proposed plans
terraform plan -out=baseline.tfplan
terraform show -json baseline.tfplan > baseline.json

# Make changes, then generate new plan
terraform plan -out=proposed.tfplan
terraform show -json proposed.tfplan > proposed.json

# Compare costs
costpilot diff --before baseline.json --after proposed.json

# JSON output for CI/CD
costpilot diff -b baseline.json -a proposed.json --format json
```

**Example Output:**
```
📊 Cost Comparison
═══════════════════════════════════════════════════════════════

Baseline:  $4,256.80/month (87% confidence)
Proposed:  $6,512.40/month (89% confidence)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Delta:     +$2,255.60/month (+53%) 🔴 HIGH SEVERITY

⚠️  Changes introduce significant cost increase!

💡 Next Steps:
  • Run: costpilot scan --explain proposed.json
  • Review: EC2 instance type upgrades
  • Consider: Reserved instances or Savings Plans
```

### 4. Generate Cost Optimizations (Autofix)

Get automated recommendations:

```bash
# Show code snippets to fix issues
costpilot autofix --plan plan.json --mode snippet

# Generate full patch file
costpilot autofix --plan plan.json --mode patch > cost-optimizations.patch

# Apply patch
git apply cost-optimizations.patch

# Drift-safe mode (preserves existing state)
costpilot autofix --plan plan.json --mode patch --drift-safe
```

**Example Output (Snippet Mode):**
```
🔧 Cost Optimization Recommendations
═══════════════════════════════════════════════════════════════

1. EC2 Instance Right-Sizing (save $1,728/month)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
File: modules/compute/main.tf

resource "aws_instance" "app" {
-  instance_type = "m5.24xlarge"
+  instance_type = "m5.12xlarge"
   ami           = var.ami_id
}

Rationale: CPU utilization averages 35%, memory at 40%
Impact: -$1,728/month (-50%)
Risk: Low (same instance family)

2. RDS Reserved Instance (save $3,456/month)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
File: modules/database/main.tf

resource "aws_db_instance" "primary" {
   instance_class = "db.r5.16xlarge"
+  
+  # Consider purchasing reserved capacity
+  # Run: aws rds purchase-reserved-db-instances-offering
}

Rationale: Running 24/7, 3-year RI saves 40%
Impact: -$3,456/month (-40%)
Risk: Low (commitment required)
```

## Common Workflows

### Workflow 1: Cost Review Before Deployment

```bash
# Generate plan
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

# Scan and get recommendations
costpilot scan --plan plan.json --explain --autofix

# Review output, apply fixes
costpilot autofix --plan plan.json --mode patch > fixes.patch
git apply fixes.patch

# Re-scan to verify
terraform plan -out=plan-fixed.tfplan
terraform show -json plan-fixed.tfplan > plan-fixed.json
costpilot scan --plan plan-fixed.json
```

### Workflow 2: PR Cost Impact Assessment

```bash
# Baseline (main branch)
git checkout main
terraform plan -out=baseline.tfplan
terraform show -json baseline.tfplan > baseline.json

# Proposed changes (PR branch)
git checkout feature-branch
terraform plan -out=proposed.tfplan
terraform show -json proposed.tfplan > proposed.json

# Compare
costpilot diff -b baseline.json -a proposed.json --format markdown > cost-report.md

# Post to PR (GitHub)
gh pr comment --body-file cost-report.md
```

### Workflow 3: Policy Enforcement in CI/CD

```yaml
# .github/workflows/costpilot.yml
name: Cost Guard

on: [pull_request]

jobs:
  cost-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v2
      
      - name: Install CostPilot
        run: |
          curl -L https://github.com/yourusername/costpilot/releases/latest/download/costpilot-linux-x64.tar.gz | tar xz
          sudo mv costpilot /usr/local/bin/
      
      - name: Generate Plan
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json
      
      - name: Cost Scan (Fail on Critical)
        run: |
          costpilot scan --plan plan.json \
            --policy .costpilot/policies/production.yaml \
            --fail-on-critical \
            --format markdown > cost-report.md
      
      - name: Comment PR
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

## Advanced Features

### Dependency Mapping

Visualize resource dependencies and costs:

```bash
# Generate Mermaid diagram
costpilot map --format mermaid --show-costs > dependency-map.mmd

# Generate HTML interactive graph
costpilot map --format html --show-costs -o dependency-map.html

# With cycle detection
costpilot map --format graphviz --cycle-detection > graph.dot
dot -Tpng graph.dot -o graph.png
```

### Cost SLO Tracking

Monitor cost growth over time:

```bash
# Create SLO config (.costpilot/slo.yaml)
cat > .costpilot/slo.yaml <<EOF
slos:
  monthly_cost:
    target: 5000.00
    max_burn_rate: 1.2
    window: 30d
EOF

# Check compliance
costpilot slo check --slo .costpilot/slo.yaml --snapshots .costpilot/snapshots/

# Calculate burn rate
costpilot slo burn --slo .costpilot/slo.yaml --snapshots .costpilot/snapshots/
```

### Resource Grouping & Attribution

Analyze costs by dimension:

```bash
# By Terraform module
costpilot group module --detailed

# By AWS service
costpilot group service --top-n 10

# By environment (tag-based)
costpilot group environment

# Comprehensive attribution report
costpilot group all --detect-anomalies -o cost-attribution.json
```

### Policy Management

Create and enforce custom policies:

```bash
# List available policies
costpilot policy-dsl list

# Validate policy syntax
costpilot policy-dsl validate --policy .costpilot/policies/custom.yaml

# Test policy against plan
costpilot policy-dsl test --policy .costpilot/policies/custom.yaml --plan plan.json

# View examples
costpilot policy-dsl example
```

## Output Formats

CostPilot supports multiple output formats:

```bash
# Human-readable terminal output (default)
costpilot scan --plan plan.json --format text

# JSON for CI/CD pipelines
costpilot scan --plan plan.json --format json | jq '.total_cost'

# Markdown for documentation/PRs
costpilot scan --plan plan.json --format markdown > COST_REPORT.md
```

## Troubleshooting

### Issue: "Failed to parse Terraform plan"

**Solution:** Ensure you're using JSON format:
```bash
# Wrong: binary plan
terraform plan -out=plan.tfplan
costpilot scan --plan plan.tfplan  # ❌

# Correct: JSON format
terraform show -json plan.tfplan > plan.json
costpilot scan --plan plan.json  # ✅
```

### Issue: "Resource not detected"

**Solution:** Check heuristics coverage:
```bash
# Search for resource type
costpilot heuristics search --resource-type aws_ecs_service

# Show heuristics details
costpilot heuristics show --resource-type aws_ecs_service

# Explain detection
costpilot explain detection --resource aws_ecs_service.app --plan plan.json
```

### Issue: "Cost prediction confidence too low"

**Solution:** Add more context:
```bash
# Check what's missing
costpilot explain prediction --resource aws_instance.app --plan plan.json

# Update heuristics (if outdated)
costpilot heuristics update --region us-east-1
```

### Issue: "Policy violations too strict"

**Solution:** Use exemptions:
```yaml
# .costpilot/exemptions.yaml
exemptions:
  - resource: "aws_instance.legacy_app"
    policy: "instance_size_limit"
    reason: "Legacy app requires large instance"
    expiry: "2024-12-31"
```

```bash
costpilot scan --plan plan.json \
  --policy .costpilot/policies/production.yaml \
  --exemptions .costpilot/exemptions.yaml
```

## Configuration

Default config location: `.costpilot/config.yaml`

```yaml
# .costpilot/config.yaml
default_region: us-east-1

scan:
  fail_on_critical: true
  show_autofix: true
  explain: false

policies:
  default: .costpilot/policies/production.yaml
  exemptions: .costpilot/exemptions.yaml

output:
  format: text
  verbose: false

heuristics:
  auto_update: true
  cache_ttl: 24h

slo:
  config: .costpilot/slo.yaml
  snapshots_dir: .costpilot/snapshots/
```

## Getting Help

```bash
# General help
costpilot help

# Command-specific help
costpilot scan --help
costpilot diff --help

# Version info
costpilot version

# Detailed version (with build info)
costpilot version --detailed
```

## Next Steps

1. **Set up CI/CD integration** - Automate cost checks in pull requests
2. **Define cost policies** - Enforce budgets and resource limits
3. **Track cost SLOs** - Monitor spending trends over time
4. **Explore advanced features** - Dependency mapping, attribution, heuristics tuning

For full documentation, visit: [docs/README.md](./README.md)

For examples, see: [examples/](../examples/)
