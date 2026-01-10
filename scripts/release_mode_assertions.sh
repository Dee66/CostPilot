#!/usr/bin/env bash
# CostPilot Release Mode Assertions
# Purpose: Verify production-safe logging and error handling
# Exit: Non-zero if any production safety issues detected

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FINDINGS=0
REPORT_FILE="${REPO_ROOT}/reports/release_mode_report.md"

# Initialize report
cat > "$REPORT_FILE" << 'EOF'
# Release Mode Safety Report
**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Repository:** CostPilot
**Purpose:** Verify production-safe logging and error handling

---

## Scan Results

EOF

echo -e "${YELLOW}Checking release mode safety...${NC}"

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
**Content:** \`$(echo "$content" | sed 's/[`]/\\`/g')\`

EOF

    ((FINDINGS++))
}

# 1. Check for println! in non-test code
echo "Checking for println! in production code..."
if grep -rn "println!" --include="*.rs" src/ 2>/dev/null | grep -v "^src/.*test" | grep -v "/tests/" | grep -v "#\[cfg(test)\]"; then
    while IFS=: read -r file line content; do
        # Skip test modules and cfg(test) blocks
        if ! grep -q "#\[cfg(test)\]" "$file" && ! echo "$file" | grep -q "test"; then
            report_finding "MEDIUM" "println! in production" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//')"
        fi
    done < <(grep -rn "println!" --include="*.rs" src/ 2>/dev/null | grep -v "test")
fi

# 2. Check for dbg! macro
echo "Checking for dbg! macro..."
if grep -rn "dbg!" --include="*.rs" src/ 2>/dev/null | grep -v "test"; then
    while IFS=: read -r file line content; do
        report_finding "HIGH" "dbg! macro (debug-only)" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//')"
    done < <(grep -rn "dbg!" --include="*.rs" src/ 2>/dev/null | grep -v "test")
fi

# 3. Check for trace! level logging in hot paths
echo "Checking for trace logging..."
if grep -rn "trace!" --include="*.rs" src/ 2>/dev/null; then
    TRACE_COUNT=$(grep -r "trace!" --include="*.rs" src/ 2>/dev/null | wc -l)
    if [ "$TRACE_COUNT" -gt 10 ]; then
        cat >> "$REPORT_FILE" << EOF
### [INFO] Trace Logging Usage
**Count:** $TRACE_COUNT occurrences
**Note:** High trace! usage may impact performance. Verify trace level is disabled in release builds.

EOF
    fi
fi

# 4. Check for unwrap() in production code
echo "Checking for unwrap() usage..."
UNWRAP_COUNT=$(grep -rn "\.unwrap()" --include="*.rs" src/ 2>/dev/null | grep -v test | grep -v "test.rs" | wc -l || true)
if [ "$UNWRAP_COUNT" -gt 50 ]; then
    cat >> "$REPORT_FILE" << EOF
### [MEDIUM] High unwrap() Usage
**Count:** $UNWRAP_COUNT occurrences in production code
**Risk:** May cause panics with poor error messages
**Recommendation:** Review critical paths for proper error handling

EOF
    echo -e "${YELLOW}[MEDIUM] High unwrap() usage: $UNWRAP_COUNT occurrences${NC}"
    ((FINDINGS++))
fi

# 5. Check for panic! or todo! in production
echo "Checking for panic! and todo!..."
if grep -rn "panic!\|todo!\|unimplemented!" --include="*.rs" src/ 2>/dev/null | grep -v test | grep -v "#\[cfg(test)\]"; then
    while IFS=: read -r file line content; do
        if echo "$content" | grep -qE "panic!|todo!|unimplemented!"; then
            report_finding "HIGH" "Panic/TODO in production" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//')"
        fi
    done < <(grep -rn "panic!\|todo!\|unimplemented!" --include="*.rs" src/ 2>/dev/null | grep -v test)
fi

# 6. Check logging configuration
echo "Checking logging configuration..."
if [ -f "src/main.rs" ]; then
    if ! grep -q "env_logger\|tracing" src/main.rs; then
        cat >> "$REPORT_FILE" << 'EOF'
### [INFO] Logging Framework
**Status:** No obvious logging framework initialization detected in main.rs
**Note:** Verify logging is configured appropriately

EOF
    fi
fi

# 7. Check for exposed stack traces or internal paths
echo "Checking for stack trace exposure..."
if grep -rn "std::backtrace\|Backtrace::capture" --include="*.rs" src/ 2>/dev/null | grep -v test; then
    while IFS=: read -r file line content; do
        cat >> "$REPORT_FILE" << EOF
### [LOW] Backtrace Usage
**File:** \`$file:$line\`
**Note:** Ensure backtraces are not exposed to end users in production

EOF
    done < <(grep -rn "std::backtrace\|Backtrace::capture" --include="*.rs" src/ 2>/dev/null | grep -v test)
fi

# 8. Check error messages for internal details
echo "Checking error messages..."
if grep -rn "Error.*:\|panic!.*\"" --include="*.rs" src/ 2>/dev/null | grep -E "file!|line!|column!" | grep -v test; then
    while IFS=: read -r file line content; do
        report_finding "MEDIUM" "Error with file/line info" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//' | cut -c1-80)"
    done < <(grep -rn "Error.*:\|panic!.*\"" --include="*.rs" src/ 2>/dev/null | grep -E "file!|line!|column!" | grep -v test)
fi

# 9. Check default log level in Cargo.toml
echo "Checking Cargo.toml for release settings..."
if [ -f "Cargo.toml" ]; then
    if ! grep -q "\[profile.release\]" Cargo.toml; then
        cat >> "$REPORT_FILE" << 'EOF'
### [INFO] Release Profile
**Status:** No explicit [profile.release] configuration
**Default:** Cargo uses optimized release settings by default

EOF
    fi

    if grep -A 10 "\[profile.release\]" Cargo.toml | grep -q "debug = true"; then
        report_finding "HIGH" "Debug symbols enabled" "Cargo.toml" "0" "debug = true in release profile"
    fi
fi

# 10. Check for development-only dependencies in main code
echo "Checking for dev dependencies in production..."
if grep -rn "#\[cfg(not(test))\]" --include="*.rs" src/ 2>/dev/null | grep -A 5 "mock\|fake\|dummy"; then
    while IFS=: read -r file line content; do
        report_finding "LOW" "Dev/Mock code in production" "$file" "$line" "$(echo "$content" | sed 's/^[[:space:]]*//' | cut -c1-80)"
    done < <(grep -rn "#\[cfg(not(test))\]" --include="*.rs" src/ 2>/dev/null | grep -A 5 "mock\|fake\|dummy")
fi

# Finalize report
cat >> "$REPORT_FILE" << EOF

---

## Summary

**Total Findings:** $FINDINGS

EOF

if [ $FINDINGS -eq 0 ]; then
    cat >> "$REPORT_FILE" << 'EOF'
✅ **No production safety issues detected**

The codebase appears ready for release from a logging and error handling perspective.

EOF
    echo -e "${GREEN}✅ Release mode assertions passed${NC}"
    exit 0
else
    cat >> "$REPORT_FILE" << 'EOF'
⚠️ **Production safety issues detected**

**Action Required:** Review findings and ensure proper logging levels and error handling for production.

EOF
    echo -e "${YELLOW}⚠️ Release mode check found $FINDINGS potential issues${NC}"
    echo -e "${YELLOW}Report saved to: $REPORT_FILE${NC}"
    exit 1
fi
