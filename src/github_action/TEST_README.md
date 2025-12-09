# GitHub Action Test Repository

This directory contains test workflows for validating the CostPilot GitHub Action.

## Test Scenarios

### 1. Basic Cost Estimation
- Single Terraform plan
- Default configuration
- Text output

### 2. Policy Enforcement
- Policy violations
- Fail on violation
- PR comments

### 3. Baseline Regression
- Cost increase detection
- Baseline comparison
- Block on regression

### 4. Drift Detection
- Critical drift
- Block execution
- Safe autofix

## Running Tests

### Prerequisites
- GitHub repository with Actions enabled
- Terraform plan JSON file
- CostPilot configuration

### Setup Test Repository

```bash
# Create ephemeral test repo
gh repo create costpilot-action-test --private --confirm

# Clone
git clone https://github.com/YOUR_USERNAME/costpilot-action-test
cd costpilot-action-test

# Add test workflow
mkdir -p .github/workflows
cp test-workflows/basic-test.yml .github/workflows/

# Add sample Terraform plan
cp test-data/sample-plan.json .

# Commit and push
git add .
git commit -m "Add CostPilot action test"
git push
```

### Test Workflow Example

`.github/workflows/costpilot-test.yml`:

```yaml
name: Test CostPilot Action

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test-estimate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run CostPilot
        uses: Dee66/CostPilot@main
        with:
          terraform_plan: 'test-data/sample-plan.json'
          mode: 'estimate'
          output_format: 'markdown'
          fail_on_regression: 'false'
      
      - name: Check outputs
        run: |
          echo "Total cost: ${{ steps.analyze.outputs.total_cost }}"
          echo "Exit code: ${{ steps.analyze.outputs.exit_code }}"

  test-policy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run CostPilot with Policy
        uses: Dee66/CostPilot@main
        with:
          terraform_plan: 'test-data/sample-plan.json'
          policy_file: 'policies/production.json'
          fail_on_policy: 'true'
          comment_pr: 'true'

  test-baseline:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run CostPilot with Baseline
        uses: Dee66/CostPilot@main
        with:
          terraform_plan: 'test-data/sample-plan.json'
          baseline_file: 'baselines/production.json'
          fail_on_regression: 'true'
```

## Validation Checklist

- [ ] Action downloads correct binary for platform
- [ ] Cost estimation produces output
- [ ] Policy violations detected correctly
- [ ] Baseline regression blocks appropriately
- [ ] PR comments are formatted correctly
- [ ] Exit codes match expected behavior
- [ ] Debug mode provides detailed logs
- [ ] Multiple input files work together
- [ ] Works on ubuntu-latest runner
- [ ] Works on macos-latest runner

## Manual Test Commands

```bash
# Test binary fetch script
cd src/github_action
./fetch-binary.sh

# Verify binary
./costpilot --version

# Test action locally with act
act -j test-estimate -s GITHUB_TOKEN=$GITHUB_TOKEN

# Test on actual repository
git push origin test-branch
# Check Actions tab for results
```

## Expected Results

### Basic Estimate
- Exit code: 0
- Output contains cost estimate
- JSON output parseable

### Policy Failure
- Exit code: 1 (if fail_on_policy=true)
- Violations listed in output
- PR comment posted

### Baseline Regression
- Exit code: 2 (if regression detected and fail_on_regression=true)
- Delta shown in output
- Comparison with baseline

## Cleanup

```bash
# Delete test repository
gh repo delete costpilot-action-test --yes
```
