# CostPilot GitHub Action

This directory contains example GitHub Actions workflows for integrating CostPilot into your CI/CD pipeline.

## Quick Start

The `costpilot init --no-ci=false` command will automatically create `.github/workflows/costpilot.yml` for you.

Alternatively, copy the example below:

## Basic Example

```yaml
name: CostPilot

on:
  pull_request:
    paths:
      - '**.tf'
      - '**.tfvars'
      - 'infrastructure/**'

permissions:
  contents: read
  pull-requests: write

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Plan
        run: |
          cd infrastructure
          terraform init
          terraform plan -out=tfplan.binary
          terraform show -json tfplan.binary > tfplan.json

      - name: Install CostPilot
        run: |
          # Download and install CostPilot
          # TODO: Update with actual installation method
          curl -sSL https://costpilot.dev/install.sh | bash

      - name: Run CostPilot
        run: |
          costpilot scan \
            --plan=infrastructure/tfplan.json \
            --explain \
            --autofix=snippet \
            > cost-report.md

      - name: Comment on PR
        uses: actions/github-script@v7
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

## Advanced Example with Thresholds

```yaml
name: CostPilot - Strict

on:
  pull_request:
    paths:
      - '**.tf'
      - '**.tfvars'

permissions:
  contents: read
  pull-requests: write

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3

      - name: Terraform Plan
        run: |
          terraform init
          terraform plan -out=tfplan.binary
          terraform show -json tfplan.binary > tfplan.json

      - name: Run CostPilot with Thresholds
        run: |
          costpilot scan \
            --plan=tfplan.json \
            --explain \
            --format=json \
            > cost-results.json

          # Extract total cost
          TOTAL_COST=$(jq '.total_monthly_cost' cost-results.json)

          # Check threshold
          if (( $(echo "$TOTAL_COST > 500" | bc -l) )); then
            echo "::error::Monthly cost exceeds $500 threshold: \$$TOTAL_COST"
            exit 1
          fi

      - name: Generate Report
        if: always()
        run: |
          costpilot scan \
            --plan=tfplan.json \
            --explain \
            --autofix=snippet \
            --format=markdown \
            > cost-report.md

      - name: Post Results
        if: always()
        uses: actions/github-script@v7
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

## Multi-Environment Example

```yaml
name: CostPilot - Multi-Environment

on:
  pull_request:

jobs:
  cost-analysis:
    strategy:
      matrix:
        environment: [dev, staging, prod]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3

      - name: Plan ${{ matrix.environment }}
        run: |
          cd infrastructure/${{ matrix.environment }}
          terraform init
          terraform plan -out=tfplan.binary
          terraform show -json tfplan.binary > tfplan.json

      - name: Analyze ${{ matrix.environment }}
        run: |
          costpilot scan \
            --plan=infrastructure/${{ matrix.environment }}/tfplan.json \
            --explain \
            > cost-report-${{ matrix.environment }}.md

      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: cost-report-${{ matrix.environment }}
          path: cost-report-${{ matrix.environment }}.md

  summarize:
    needs: cost-analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4

      - name: Combine Reports
        run: |
          echo "# 💰 Cost Analysis Summary" > combined-report.md
          echo "" >> combined-report.md

          for env in dev staging prod; do
            echo "## $env Environment" >> combined-report.md
            cat cost-report-$env/cost-report-$env.md >> combined-report.md
            echo "" >> combined-report.md
          done

      - name: Comment PR
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('combined-report.md', 'utf8');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

## Configuration

CostPilot respects `.costpilot/config.yml` for settings. See [configuration documentation](../docs/configuration.md) for details.

## Troubleshooting

### Installation Issues

If the installation script fails, you can manually download the binary:

```yaml
- name: Install CostPilot (manual)
  run: |
    wget https://github.com/costpilot/costpilot/releases/latest/download/costpilot-linux-amd64
    chmod +x costpilot-linux-amd64
    sudo mv costpilot-linux-amd64 /usr/local/bin/costpilot
```

### Plan File Not Found

Ensure the Terraform plan file path matches your working directory structure.

### Permission Issues

The workflow requires `pull-requests: write` permission to comment on PRs.

## Features

- ✅ Automatic cost detection
- ✅ Detailed explanations for top anti-patterns
- ✅ Snippet-based autofix recommendations
- ✅ PR comments with cost analysis
- ✅ Configurable thresholds
- ✅ Multi-environment support
- ✅ Zero AWS credentials required (Zero-IAM)

## Next Steps

1. Customize thresholds in `.costpilot/config.yml`
2. Add budget policies in `.costpilot/policy.yml`
3. Enable autofix suggestions with `--autofix=snippet`
4. Integrate with cost tracking tools (Phase 2)
