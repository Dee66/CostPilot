# CostPilot Demo Repository

Public demonstration repository showcasing CostPilot in action with real Terraform changes.

## Repository: costpilot-demo

**URL:** https://github.com/Dee66/costpilot-demo

### Purpose
- Live demonstration of CostPilot features
- Example pull requests with cost analysis
- Real-world Terraform configurations
- CI/CD integration examples

## Demo Scenarios

### 1. Basic Cost Analysis
**PR:** [#1 - Add EC2 Instance](https://github.com/Dee66/costpilot-demo/pull/1)

```hcl
resource "aws_instance" "web" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t3.micro"
  
  tags = {
    Name = "demo-web-server"
  }
}
```

**CostPilot Output:**
- Estimated monthly cost: $7.50
- New resource detected
- No policy violations
- ✅ PR approved

### 2. Cost Regression Detection
**PR:** [#2 - Upsize Instance](https://github.com/Dee66/costpilot-demo/pull/2)

```diff
  resource "aws_instance" "web" {
    ami           = "ami-0c55b159cbfafe1f0"
-   instance_type = "t3.micro"
+   instance_type = "t3.2xlarge"
  }
```

**CostPilot Output:**
- Estimated monthly cost: $120.00
- Cost increase: +$112.50 (+1500%)
- 🚨 Regression detected
- ❌ Build blocked

### 3. Policy Violation
**PR:** [#3 - Invalid Instance Type](https://github.com/Dee66/costpilot-demo/pull/3)

```hcl
resource "aws_instance" "gpu" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "p3.8xlarge"  # Expensive GPU instance
}
```

**CostPilot Output:**
- Estimated monthly cost: $12,240.00
- Policy violation: Instance type not allowed
- ❌ Build blocked
- Requires approval reference

### 4. Drift Detection
**PR:** [#4 - Detect Manual Changes](https://github.com/Dee66/costpilot-demo/pull/4)

**Scenario:** Security group modified manually in AWS console

**CostPilot Output:**
- Critical drift detected
- Security group changed: sg-12345 → sg-67890
- SHA256 checksum mismatch
- 🔴 Execution blocked

### 5. Exemption Workflow
**PR:** [#5 - Temporary Exception](https://github.com/Dee66/costpilot-demo/pull/5)

```yaml
# exemptions.yaml
exemptions:
  - id: EXP-001
    policy: INSTANCE_TYPE_LIMIT
    resource: module.demo.aws_instance.special
    justification: "Performance testing for Q4"
    expires_at: "2026-01-31"
    approved_by: "tech-lead@example.com"
    ticket_ref: "JIRA-1234"
```

**CostPilot Output:**
- Exemption applied
- Valid until 2026-01-31
- ✅ Build passed with exemption

### 6. Multi-Environment Deployment
**PR:** [#6 - Deploy to Multiple Environments](https://github.com/Dee66/costpilot-demo/pull/6)

```yaml
# .github/workflows/costpilot-multi-env.yml
strategy:
  matrix:
    environment: [dev, staging, prod]
```

**CostPilot Output:**
- Dev: $50/month ✅
- Staging: $200/month ✅
- Prod: $1,500/month ⚠️ (within budget)

## Repository Structure

```
costpilot-demo/
├── .github/
│   └── workflows/
│       ├── costpilot.yml           # Main workflow
│       ├── costpilot-multi-env.yml # Multi-environment
│       └── costpilot-advanced.yml  # Advanced features
├── terraform/
│   ├── main.tf
│   ├── variables.tf
│   ├── outputs.tf
│   └── environments/
│       ├── dev/
│       ├── staging/
│       └── prod/
├── policies/
│   ├── production.json
│   ├── development.json
│   └── README.md
├── baselines/
│   ├── dev-baseline.json
│   ├── staging-baseline.json
│   └── prod-baseline.json
├── exemptions.yaml
├── .costpilot.yml
└── README.md
```

## Workflow Configuration

```yaml
name: CostPilot Demo

on:
  pull_request:
    paths:
      - 'terraform/**'

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: hashicorp/setup-terraform@v3
      
      - name: Terraform Plan
        working-directory: terraform
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json
      
      - uses: Dee66/CostPilot@v1
        with:
          terraform_plan: terraform/plan.json
          config_file: .costpilot.yml
          policy_file: policies/production.json
          baseline_file: baselines/prod-baseline.json
          exemptions_file: exemptions.yaml
          comment_pr: true
```

## Demo Pull Requests

All PRs are kept open for demonstration purposes.

| PR # | Title | Status | Demonstrates |
|------|-------|--------|--------------|
| #1 | Add EC2 instance | ✅ Passed | Basic cost analysis |
| #2 | Upsize to t3.2xlarge | ❌ Blocked | Regression detection |
| #3 | Add GPU instance | ❌ Blocked | Policy violation |
| #4 | Security group update | 🔴 Blocked | Drift detection |
| #5 | With exemption | ✅ Passed | Exemption workflow |
| #6 | Multi-environment | ✅ Passed | Matrix deployment |
| #7 | Savings plan suggestion | ℹ️ Info | Cost optimization |
| #8 | SLO breach | ⚠️ Warning | SLO monitoring |

## Interactive Demo

Visit the repository to:
1. Browse example PRs with CostPilot comments
2. Click "Files changed" to see Terraform modifications
3. Read CostPilot analysis in PR comments
4. View workflow runs in Actions tab
5. Fork and try it yourself

## Setup Instructions

### Fork and Try

```bash
# Fork the repository
gh repo fork Dee66/costpilot-demo

# Clone your fork
git clone https://github.com/YOUR_USERNAME/costpilot-demo
cd costpilot-demo

# Create a branch
git checkout -b test-costpilot

# Make Terraform changes
vim terraform/main.tf

# Commit and push
git add .
git commit -m "Test CostPilot"
git push origin test-costpilot

# Create PR
gh pr create --title "Test CostPilot" --body "Testing cost analysis"
```

### Watch CostPilot in Action

1. Navigate to your PR
2. Wait for GitHub Actions to complete (~30 seconds)
3. See CostPilot comment with cost analysis
4. Check Actions tab for detailed logs

## Key Features Demonstrated

✅ **Automated Cost Analysis**
- Real-time cost estimates
- Cost deltas from baseline
- Resource-by-resource breakdown

✅ **Policy Enforcement**
- Budget limits
- Allowed instance types
- Tagging requirements
- Approval workflows

✅ **Drift Detection**
- SHA256 checksums
- Manual change detection
- Critical drift blocking

✅ **SLO Monitoring**
- Cost SLO tracking
- Burn rate alerts
- Error budget management

✅ **Exemption Management**
- Time-bound exemptions
- Approval references
- Expiration tracking

✅ **PR Integration**
- Beautiful markdown comments
- Cost trends
- Action items

## Educational Content

Each PR includes:
- **What Changed:** Terraform diff
- **Cost Impact:** Before/after comparison
- **CostPilot Analysis:** Detailed breakdown
- **Action Items:** What to do next
- **Learning Points:** Best practices

## Community Examples

Users can submit their own examples via PR to the demo repo.

## Live Demo Link

Include in presentations, blog posts, and documentation:

```markdown
🎬 [See CostPilot in Action](https://github.com/Dee66/costpilot-demo)
```

## Maintenance

- Keep Terraform providers up to date
- Refresh baseline files quarterly
- Add new demo scenarios for new features
- Update policy examples
- Keep PRs open for demonstration

## Questions?

Open an issue in the demo repository for questions or suggestions.
