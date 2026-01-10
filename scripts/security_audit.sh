#!/usr/bin/env bash
# CostPilot Security Audit Scanner
# Purpose: Detect sensitive data before public release
# Exit: Non-zero if ANY sensitive material detected

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FINDINGS=0
REPORT_FILE="${REPO_ROOT}/reports/security_audit_report.md"

# Initialize report
cat > "$REPORT_FILE" << 'EOF'
# Security Audit Report
**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Repository:** CostPilot
**Purpose:** Pre-public-release security scan

---

## Scan Results

EOF

echo -e "${YELLOW}Starting security audit...${NC}"

# Function to report finding
report_finding() {
    local severity="$1"
    local category="$2"
    local file="$3"
    local line="$4"
    local content="$5"

    echo -e "${RED}[${severity}] ${category}: ${file}:${line}${NC}"
    echo "  ${content}"

    cat >> "$REPORT_FILE" << EOF
### [$severity] $category
**File:** \`$file:$line\`
**Content:** \`$content\`

EOF

    ((FINDINGS++))
}

# 1. Scan for private keys and certificates
echo "Scanning for private keys and certificates..."
while IFS= read -r file; do
    if [ -f "$file" ]; then
        if grep -q "BEGIN.*PRIVATE KEY" "$file" 2>/dev/null; then
            line=$(grep -n "BEGIN.*PRIVATE KEY" "$file" | cut -d: -f1 | head -1)
            report_finding "CRITICAL" "Private Key" "$file" "$line" "Contains private key material"
        fi
        if grep -q "BEGIN CERTIFICATE" "$file" 2>/dev/null; then
            line=$(grep -n "BEGIN CERTIFICATE" "$file" | cut -d: -f1 | head -1)
            report_finding "HIGH" "Certificate" "$file" "$line" "Contains certificate"
        fi
    fi
done < <(find . -type f ! -path "./target/*" ! -path "./.git/*" ! -path "./scripts/test-data/*" ! -path "./scripts/security_audit.sh" ! -path "./scripts/test_data/*" ! -path "*/target/*" ! -path "./costpilot-license-issuer/*" ! -path "./tests/*" ! -path "./test/*" 2>/dev/null)

# 2. Scan for AWS credentials and identifiers
echo "Scanning for AWS credentials..."
if grep -r "AKIA[0-9A-Z]{16}" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.yaml" --include="*.sh" . 2>/dev/null | grep -v test | grep -v example; then
    while IFS=: read -r file line content; do
        report_finding "CRITICAL" "AWS Access Key" "$file" "$line" "$content"
    done < <(grep -rn "AKIA[0-9A-Z]{16}" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.yaml" --include="*.sh" . 2>/dev/null | grep -v test | grep -v example)
fi

if grep -r "aws_secret_access_key\s*=\s*['\"][^'\"]*['\"]" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.sh" . 2>/dev/null | grep -v test | grep -v example | grep -v "secrets\."; then
    while IFS=: read -r file line content; do
        report_finding "CRITICAL" "AWS Secret Key" "$file" "$line" "$content"
    done < <(grep -rn "aws_secret_access_key\s*=\s*['\"][^'\"]*['\"]" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.sh" . 2>/dev/null | grep -v test | grep -v example | grep -v "secrets\.")
fi

# 3. Scan for hardcoded tokens and secrets
echo "Scanning for hardcoded tokens..."
if grep -rE "token\s*[:=]\s*['\"][A-Za-z0-9+/=]{20,}['\"]" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example | grep -v "secrets\." | grep -v "github\."; then
    while IFS=: read -r file line content; do
        if ! echo "$content" | grep -q "secrets\.\|github\.event\.\|env\."; then
            report_finding "HIGH" "Hardcoded Token" "$file" "$line" "$(echo "$content" | cut -c1-80)"
        fi
    done < <(grep -rnE "token\s*[:=]\s*['\"][A-Za-z0-9+/=]{20,}['\"]" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example)
fi

# 4. Scan for .env files
echo "Scanning for .env files..."
while IFS= read -r file; do
    if [ -f "$file" ] && [ "$(basename "$file")" = ".env" ]; then
        report_finding "HIGH" ".env File Present" "$file" "0" "Environment file should not be committed"
    fi
done < <(find . -name ".env" -type f ! -path "./target/*" ! -path "./.git/*" 2>/dev/null)

# 5. Scan for .pem/.key/.crt files (excluding test data)
echo "Scanning for key files..."
while IFS= read -r file; do
    if ! echo "$file" | grep -q "test-data\|test_data"; then
        report_finding "HIGH" "Key File Present" "$file" "0" "$(basename "$file")"
    fi
done < <(find . -type f \( -name "*.pem" -o -name "*.key" -o -name "*.crt" \) ! -path "./target/*" ! -path "./.git/*" ! -path "*/test-data/*" ! -path "*/test_data/*" 2>/dev/null)

# 6. Scan for database connection strings
echo "Scanning for database credentials..."
if grep -rE "(postgres|mysql|mongodb)://[^/]+:[^@]+@" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v example | grep -v test; then
    while IFS=: read -r file line content; do
        report_finding "CRITICAL" "Database Connection String" "$file" "$line" "$(echo "$content" | cut -c1-60)..."
    done < <(grep -rnE "(postgres|mysql|mongodb)://[^/]+:[^@]+@" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v example | grep -v test)
fi

# 7. Scan for API keys in common patterns
echo "Scanning for API keys..."
if grep -rE "api[_-]?key\s*[:=]\s*['\"][A-Za-z0-9_-]{20,}['\"]" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example | grep -v "secrets\."; then
    while IFS=: read -r file line content; do
        if ! echo "$content" | grep -q "secrets\.\|env\."; then
            report_finding "HIGH" "API Key" "$file" "$line" "$(echo "$content" | cut -c1-80)"
        fi
    done < <(grep -rnE "api[_-]?key\s*[:=]\s*['\"][A-Za-z0-9_-]{20,}['\"]" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example)
fi

# 8. Scan for private license keys
echo "Scanning for private license keys..."
if [ -d ".costpilot" ] || [ -d "$HOME/.costpilot" ]; then
    report_finding "MEDIUM" "License Directory" ".costpilot" "0" "Local license directory exists (not committed, but verify .gitignore)"
fi

# 9. Scan for TODO/FIXME with security implications
echo "Scanning for security TODOs..."
if grep -rE "TODO.*security|FIXME.*security|XXX.*security|HACK.*auth" --include="*.rs" . 2>/dev/null; then
    while IFS=: read -r file line content; do
        report_finding "MEDIUM" "Security TODO" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//' | cut -c1-80)"
    done < <(grep -rnE "TODO.*security|FIXME.*security|XXX.*security|HACK.*auth" --include="*.rs" . 2>/dev/null)
fi

# 10. Scan for exposed internal URLs
echo "Scanning for internal endpoints..."
if grep -rE "https?://(localhost|127\.0\.0\.1|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.|192\.168\.)" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example | grep -v "//"; then
    while IFS=: read -r file line content; do
        if ! echo "$content" | grep -q "test\|example\|//"; then
            report_finding "LOW" "Internal URL" "$file" "$line" "$(echo "$content" | cut -c1-80)"
        fi
    done < <(grep -rnE "https?://(localhost|127\.0\.0\.1|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.|192\.168\.)" --include="*.rs" --include="*.toml" --include="*.yml" . 2>/dev/null | grep -v test | grep -v example | grep -v "//")
fi

# Finalize report
cat >> "$REPORT_FILE" << EOF

---

## Summary

**Total Findings:** $FINDINGS

EOF

if [ $FINDINGS -eq 0 ]; then
    cat >> "$REPORT_FILE" << 'EOF'
✅ **No sensitive data detected**

The repository is safe for public release from a secrets perspective.

EOF
    echo -e "${GREEN}✅ Security audit passed: No sensitive data detected${NC}"
    exit 0
else
    cat >> "$REPORT_FILE" << 'EOF'
❌ **Sensitive data detected**

**Action Required:** Review all findings above and remediate before making repository public.

EOF
    echo -e "${RED}❌ Security audit failed: $FINDINGS findings detected${NC}"
    echo -e "${YELLOW}Report saved to: $REPORT_FILE${NC}"
    exit 1
fi
