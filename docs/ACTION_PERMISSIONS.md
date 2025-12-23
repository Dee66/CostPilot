# GitHub Action Permissions Documentation

## Overview

The CostPilot GitHub Action requires specific permissions to function correctly. This document details the required permissions, why they're needed, and security best practices.

## Required Permissions

### Minimum Permissions (Read-Only Analysis)

```yaml
permissions:
  contents: read        # Read repository files and Terraform plans
  pull-requests: read   # Read PR metadata for context
```

**Use Case:** Cost analysis without PR comments
- Reads Terraform plan files from repository
- Accesses configuration files (policies, baselines, exemptions)
- No write operations

### Standard Permissions (With PR Comments)

```yaml
permissions:
  contents: read        # Read repository files and Terraform plans
  pull-requests: write  # Post analysis results as PR comments
```

**Use Case:** Default configuration with automated PR comments
- Everything from read-only mode
- Posts cost analysis as PR comment
- Updates existing comments on new commits
- Does NOT modify code or merge PRs

### Enhanced Permissions (With Drift Detection)

```yaml
permissions:
  contents: read        # Read repository files and Terraform plans
  pull-requests: write  # Post analysis results as PR comments
  actions: read         # Read previous workflow runs for drift baselines
```

**Use Case:** Drift detection comparing against previous deployments
- Everything from standard mode
- Reads previous workflow artifacts for baseline comparison
- Detects changes between runs

## Permission Scope

### Repository-Level Permissions

CostPilot operates at **repository level** only:
- ✅ Reads files from current repository
- ✅ Writes comments to current PR
- ❌ Does NOT access other repositories
- ❌ Does NOT access organization secrets
- ❌ Does NOT modify repository settings
- ❌ Does NOT merge PRs or push commits

### Token Scope

The action uses `${{ github.token }}` (automatically provided by GitHub Actions):
- **Lifetime:** Single workflow run only
- **Scope:** Current repository only
- **Expiration:** Automatic after workflow completes
- **Permissions:** Limited to specified `permissions` block

## Configuration Examples

### Example 1: Read-Only Analysis (No Comments)

```yaml
name: Cost Analysis
on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: read

    steps:
      - uses: actions/checkout@v4

      - name: Generate Terraform Plan
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json

      - name: Run CostPilot
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          comment_pr: 'false'  # Disable PR comments
```

**Security:** Minimal permissions, no write access

### Example 2: Standard Analysis with Comments

```yaml
name: Cost Analysis
on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write

    steps:
      - uses: actions/checkout@v4

      - name: Generate Terraform Plan
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json

      - name: Run CostPilot
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          policy_file: policies/prod.json
          baseline_file: baselines/main.json
```

**Security:** Read files, write comments only

### Example 3: Full Analysis with Drift Detection

```yaml
name: Cost Analysis
on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      actions: read

    steps:
      - uses: actions/checkout@v4

      - name: Generate Terraform Plan
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json

      - name: Run CostPilot
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          policy_file: policies/prod.json
          baseline_file: baselines/main.json
          mode: 'all'
          fail_on_drift: 'true'
```

**Security:** Read files and workflows, write comments

### Example 4: Restricted Environment (No External Access)

```yaml
name: Cost Analysis (Air-Gapped)
on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write

    steps:
      - uses: actions/checkout@v4

      - name: Generate Terraform Plan
        run: |
          terraform init
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json

      - name: Run CostPilot (Offline Mode)
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: plan.json
          mode: 'estimate'
        env:
          COSTPILOT_OFFLINE: 'true'  # No external network calls
```

**Security:** Completely offline, no external API calls

## Security Best Practices

### 1. Principle of Least Privilege

**DO:**
```yaml
permissions:
  contents: read
  pull-requests: write
```

**DON'T:**
```yaml
permissions: write-all  # Too broad!
```

Grant only the minimum permissions needed for your use case.

### 2. Pin Action Versions

**DO:**
```yaml
- uses: Dee66/CostPilot@v1.0.0  # Pinned to specific version
```

**DON'T:**
```yaml
- uses: Dee66/CostPilot@main    # Unpredictable!
```

Use specific version tags to prevent unexpected changes.

### 3. Review Permission Changes

When updating workflows, review permission changes:

```bash
git diff .github/workflows/terraform.yml
```

Ensure new permissions are justified and documented.

### 4. Separate Sensitive Workflows

For production deployments with secrets:

```yaml
# analysis.yml (Pull Requests)
permissions:
  contents: read
  pull-requests: write

# deploy.yml (Protected Branches)
permissions:
  contents: read
  id-token: write  # For OIDC with AWS/Azure/GCP
```

Keep cost analysis separate from deployment workflows.

### 5. Use Environment Protection Rules

For workflows that can fail builds:

```yaml
jobs:
  analyze:
    runs-on: ubuntu-latest
    environment: production  # Requires approval for production
    permissions:
      contents: read
      pull-requests: write
```

Protect critical environments with approval gates.

## Permission Matrix

| Feature | `contents` | `pull-requests` | `actions` | `id-token` |
|---------|------------|----------------|-----------|-----------|
| Read Terraform plans | read | - | - | - |
| Read config files | read | - | - | - |
| Post PR comments | read | write | - | - |
| Drift detection | read | write | read | - |
| Cloud provider auth | read | write | - | write |

## Troubleshooting

### Issue: "Resource not accessible by integration"

**Symptom:**
```
Error: Resource not accessible by integration
```

**Cause:** Missing required permissions

**Fix:**
```yaml
permissions:
  contents: read
  pull-requests: write
```

### Issue: "Cannot post comment on pull request"

**Symptom:**
```
Error: 403 Forbidden - Cannot create comment
```

**Cause:** Missing `pull-requests: write` permission

**Fix:**
```yaml
permissions:
  pull-requests: write
```

### Issue: "Cannot read workflow artifacts"

**Symptom:**
```
Error: Cannot access workflow run artifacts
```

**Cause:** Missing `actions: read` permission for drift detection

**Fix:**
```yaml
permissions:
  actions: read
```

## Audit and Compliance

### Logging

CostPilot logs all permission checks:

```
[INFO] Checking permissions...
[INFO] ✓ contents: read
[INFO] ✓ pull-requests: write
[INFO] ✗ actions: read (optional, skipping drift detection)
```

### Audit Trail

GitHub provides audit logs for action executions:

1. Repository → Settings → Actions → General
2. View workflow run logs
3. Review permission grants

### Compliance Considerations

**For SOC 2/ISO 27001:**
- Document required permissions in security policy
- Regularly review workflow configurations
- Implement branch protection rules
- Require code review for workflow changes

**For HIPAA/PCI-DSS:**
- Use read-only mode where possible
- Separate cost analysis from deployment workflows
- Implement environment protection rules
- Audit workflow executions quarterly

## Advanced: Custom Token Permissions

For fine-grained control, use custom GitHub Apps:

```yaml
steps:
  - name: Generate App Token
    uses: tibdex/github-app-token@v1
    id: generate-token
    with:
      app_id: ${{ secrets.APP_ID }}
      private_key: ${{ secrets.APP_PRIVATE_KEY }}
      permissions: >-
        {
          "contents": "read",
          "pull_requests": "write"
        }

  - name: Run CostPilot
    uses: Dee66/CostPilot@v1
    env:
      GITHUB_TOKEN: ${{ steps.generate-token.outputs.token }}
```

**Benefits:**
- More granular permission control
- Longer token lifetime (if needed)
- Cross-repository access (if required)

## FAQ

### Q: Why does CostPilot need write access to pull requests?

**A:** Only to post cost analysis comments. It does NOT:
- Merge or close PRs
- Modify PR files
- Approve or request changes
- Add labels or reviewers

### Q: Can CostPilot access secrets?

**A:** No. CostPilot does not require or access repository secrets. It uses only the automatically-provided `github.token` with limited scope.

### Q: Is it safe to use on public repositories?

**A:** Yes. CostPilot operates entirely within the repository and posts only cost analysis results (no sensitive data). However, ensure your Terraform plans don't contain secrets.

### Q: What if my organization restricts GitHub Actions?

**A:** You can:
1. Run CostPilot as CLI in existing CI/CD (Jenkins, GitLab CI, CircleCI)
2. Request approval from security team with this documentation
3. Use read-only mode if write permissions are blocked

### Q: Does CostPilot send data to external services?

**A:** No. CostPilot runs entirely within the GitHub Actions runner. It does NOT:
- Send data to external APIs
- Require API keys or authentication
- Transmit Terraform plans outside the runner
- Track usage or telemetry

### Q: How do I verify permissions are correctly set?

**A:** Check workflow run logs:

```bash
gh run view <run-id> --log
```

Look for permission checks in the output.

## References

- [GitHub Actions Permissions](https://docs.github.com/en/actions/security-guides/automatic-token-authentication#permissions-for-the-github_token)
- [Least Privilege for GitHub Actions](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions#using-least-privileged-permissions)
- [CostPilot Security Policy](https://github.com/Dee66/CostPilot/security/policy)

---

**Last Updated:** December 2025
**Version:** 1.0.0
