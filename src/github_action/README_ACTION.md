# GitHub Action Usage Guide

## Using CostPilot in GitHub Actions

Example workflow:
```yaml
steps:
  - uses: actions/checkout@v3
  - uses: ./src/github_action
    with:
      terraform_plan: plan.json
      config_file: .costpilot.yml
```
