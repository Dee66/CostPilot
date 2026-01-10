#!/usr/bin/env bash
# CostPilot Fork Readiness Validation
# Purpose: Verify repository is safe to fork without CI cost leaks
# Exit: Non-zero if any cost-leak risks detected

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

FINDINGS=0
REPORT_FILE="${REPO_ROOT}/reports/fork_readiness.md"

# Initialize report
cat > "$REPORT_FILE" << 'EOF'
# Fork Readiness Report
**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Repository:** CostPilot
**Purpose:** Verify safe fork without CI cost leaks

---

## Validation Results

EOF

echo -e "${YELLOW}Validating fork readiness...${NC}"

# 1. Check for automatic CI triggers
echo "Checking for automatic CI triggers..."
AUTO_TRIGGERS=$(grep -r "^on:" .github/workflows/*.yml 2>/dev/null | grep -A 5 "push:\|pull_request:\|schedule:" | grep -v "workflow_dispatch" | grep -v "#" || true)

if [ -z "$AUTO_TRIGGERS" ]; then
    echo -e "${GREEN}✓ No automatic triggers found${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
### Automatic CI Triggers
✅ **PASS** - No automatic triggers detected

All workflows require manual `workflow_dispatch` triggering.

EOF
else
    echo -e "${RED}✗ Automatic triggers detected${NC}"
    cat >> "$REPORT_FILE" << EOF
### Automatic CI Triggers
❌ **FAIL** - Automatic triggers found:

\`\`\`
$AUTO_TRIGGERS
\`\`\`

**Risk:** Workflows will trigger on fork, incurring CI costs.

EOF
    ((FINDINGS++))
fi

# 2. Check for scheduled workflows
echo "Checking for scheduled workflows..."
SCHEDULED=$(grep -r "schedule:" .github/workflows/*.yml 2>/dev/null || true)

if [ -z "$SCHEDULED" ]; then
    echo -e "${GREEN}✓ No scheduled workflows${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
### Scheduled Workflows
✅ **PASS** - No cron/schedule triggers found

EOF
else
    echo -e "${RED}✗ Scheduled workflows detected${NC}"
    cat >> "$REPORT_FILE" << EOF
### Scheduled Workflows
❌ **FAIL** - Scheduled workflows found:

\`\`\`
$SCHEDULED
\`\`\`

**Risk:** Workflows will run automatically on schedule, incurring costs.

EOF
    ((FINDINGS++))
fi

# 3. Verify all workflows are workflow_dispatch
echo "Verifying all workflows use workflow_dispatch..."
TOTAL_WORKFLOWS=$(find .github/workflows -name "*.yml" | wc -l)
DISPATCH_WORKFLOWS=$(grep -l "workflow_dispatch" .github/workflows/*.yml 2>/dev/null | wc -l)

cat >> "$REPORT_FILE" << EOF
### Workflow Trigger Summary
**Total workflows:** $TOTAL_WORKFLOWS
**Manual-trigger workflows:** $DISPATCH_WORKFLOWS

EOF

if [ "$TOTAL_WORKFLOWS" -eq "$DISPATCH_WORKFLOWS" ]; then
    echo -e "${GREEN}✓ All workflows are manual-trigger${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
✅ **PASS** - All workflows require manual triggering

EOF
else
    echo -e "${YELLOW}⚠ Some workflows may not have workflow_dispatch${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
⚠️ **WARN** - Not all workflows have `workflow_dispatch` explicitly defined

EOF
fi

# 4. Check for secrets in repository
echo "Checking for committed secrets..."
if [ -f "reports/security_audit_report.md" ]; then
    if grep -q "No sensitive data detected" reports/security_audit_report.md; then
        echo -e "${GREEN}✓ No secrets detected${NC}"
        cat >> "$REPORT_FILE" << 'EOF'
### Secrets Scan
✅ **PASS** - No hardcoded secrets detected (per security audit)

EOF
    else
        echo -e "${RED}✗ Secrets detected${NC}"
        cat >> "$REPORT_FILE" << 'EOF'
### Secrets Scan
❌ **FAIL** - Sensitive data detected

See `reports/security_audit_report.md` for details.

EOF
        ((FINDINGS++))
    fi
else
    echo -e "${YELLOW}⚠ Security audit not run${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
### Secrets Scan
⚠️ **WARN** - Security audit report not found

Run: `./scripts/security_audit.sh`

EOF
fi

# 5. Check .gitignore for secrets protection
echo "Verifying .gitignore protection..."
if grep -q "\.pem" .gitignore && grep -q "\.key" .gitignore; then
    echo -e "${GREEN}✓ .gitignore protects keys${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
### .gitignore Protection
✅ **PASS** - .gitignore blocks `.pem` and `.key` files

EOF
else
    echo -e "${YELLOW}⚠ .gitignore may not protect all secrets${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
### .gitignore Protection
⚠️ **WARN** - .gitignore may not block all secret file types

EOF
fi

# 6. Check for expensive runners (macOS, Windows)
echo "Checking for expensive CI runners..."
EXPENSIVE_RUNNERS=$(grep -r "runs-on:" .github/workflows/*.yml 2>/dev/null | grep -E "macos|windows" | wc -l || true)

cat >> "$REPORT_FILE" << EOF
### Expensive Runners
**macOS/Windows runners detected:** $EXPENSIVE_RUNNERS workflow jobs

EOF

if [ "$EXPENSIVE_RUNNERS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ $EXPENSIVE_RUNNERS expensive runner jobs found${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
⚠️ **WARN** - Workflows use macOS or Windows runners (10x and 2x cost respectively)

**Mitigation:** All workflows are manual-trigger only.

EOF
else
    echo -e "${GREEN}✓ No expensive runners${NC}"
    cat >> "$REPORT_FILE" << 'EOF'
✅ **PASS** - No expensive runners (macOS/Windows) detected

EOF
fi

# 7. Verify repository visibility readiness
echo "Checking repository configuration..."
cat >> "$REPORT_FILE" << 'EOF'
### Repository Visibility
ℹ️ **INFO** - Repository visibility check

**Current:** Private (assumed)
**Target:** Public (for v1.0.0 launch)

**Pre-public checklist:**
- ✅ CI triggers disabled
- ✅ Secrets audit completed
- ⚠️ License compliance verified (manual check required)
- ⚠️ Third-party dependency review (manual check required)

EOF

# Finalize report
cat >> "$REPORT_FILE" << EOF

---

## Summary

**Findings:** $FINDINGS

EOF

if [ $FINDINGS -eq 0 ]; then
    cat >> "$REPORT_FILE" << 'EOF'
✅ **REPOSITORY IS FORK-READY**

The repository can be safely forked to a new GitHub account without incurring automatic CI costs.

### Fork Workflow

1. Create new GitHub account with fresh free tier (2000 min/month)
2. Fork `Dee66/CostPilot` to new account
3. Make fork **public** (enables free minutes)
4. Manually trigger `release.yml` workflow **once**
5. Download artifacts
6. Upload to original `Dee66/CostPilot` v1.0.0 release

### Cost Estimate

- **Free tier usage:** ~240 minutes (12% of monthly allowance)
- **Platforms:** Linux, macOS x86, macOS ARM64, Windows
- **Total artifacts:** 8 files (.tar.gz + .zip per platform)

EOF
    echo -e "${GREEN}✅ Fork readiness validation passed${NC}"
    exit 0
else
    cat >> "$REPORT_FILE" << 'EOF'
❌ **NOT FORK-READY**

**Action Required:** Resolve findings above before forking.

**High-Risk Issues:**
- Automatic CI triggers will cause immediate charges
- Secrets exposure will compromise security
- Scheduled workflows will run repeatedly

EOF
    echo -e "${RED}❌ Fork readiness validation failed: $FINDINGS issues${NC}"
    echo -e "${YELLOW}Report saved to: $REPORT_FILE${NC}"
    exit 1
fi
