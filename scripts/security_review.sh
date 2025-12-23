#!/bin/bash
# CostPilot Security Review Script
# Automated security analysis for code changes

set -e

echo "🔒 Running CostPilot Security Review"
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
RESULTS_DIR="code-review-results"
mkdir -p "$RESULTS_DIR"

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✅ PASS${NC}: $message"
            ;;
        "FAIL")
            echo -e "${RED}❌ FAIL${NC}: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  WARN${NC}: $message"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  INFO${NC}: $message"
            ;;
    esac
}

# Function to check for security vulnerabilities
check_security_vulnerabilities() {
    print_status "INFO" "Checking for security vulnerabilities..."

    local vulnerabilities_found=0

    # Check for cargo audit if available
    if command -v cargo-audit &> /dev/null; then
        print_status "INFO" "Running cargo audit..."
        if cargo audit --json > "$RESULTS_DIR/cargo-audit.json" 2>/dev/null; then
            local vuln_count
            vuln_count=$(jq '.vulnerabilities.count' "$RESULTS_DIR/cargo-audit.json" 2>/dev/null || echo "0")
            if [ "$vuln_count" -gt 0 ]; then
                print_status "FAIL" "Found $vuln_count security vulnerabilities in dependencies"
                vulnerabilities_found=$((vulnerabilities_found + vuln_count))
            else
                print_status "PASS" "No security vulnerabilities found in dependencies"
            fi
        else
            print_status "WARN" "Cargo audit failed to run"
        fi
    else
        print_status "WARN" "cargo-audit not installed - skipping dependency vulnerability check"
    fi

    # Check for hardcoded secrets
    print_status "INFO" "Checking for hardcoded secrets..."
    local secret_patterns=(
        "password.*="
        "secret.*="
        "key.*="
        "token.*="
        "api_key.*="
        "aws_access_key_id.*="
        "aws_secret_access_key.*="
    )

    local secrets_found=0
    for pattern in "${secret_patterns[@]}"; do
        local matches
        matches=$(git diff --cached | grep -i "$pattern" | wc -l)
        if [ "$matches" -gt 0 ]; then
            print_status "FAIL" "Found potential hardcoded secrets matching pattern: $pattern"
            secrets_found=$((secrets_found + matches))
        fi
    done

    if [ $secrets_found -eq 0 ]; then
        print_status "PASS" "No hardcoded secrets detected"
    fi

    vulnerabilities_found=$((vulnerabilities_found + secrets_found))

    echo "$vulnerabilities_found" > "$RESULTS_DIR/security_vulnerabilities.count"
}

# Function to check for insecure coding patterns
check_insecure_patterns() {
    print_status "INFO" "Checking for insecure coding patterns..."

    local issues_found=0

    # Check for unsafe code usage
    local unsafe_count
    unsafe_count=$(git diff --cached | grep -c "unsafe {" || true)
    if [ "$unsafe_count" -gt 0 ]; then
        print_status "WARN" "Found $unsafe_count uses of 'unsafe' blocks - requires security review"
        issues_found=$((issues_found + unsafe_count))
    fi

    # Check for unwrap/expect usage that could panic
    local panic_count
    panic_count=$(git diff --cached | grep -c "\.unwrap()" || true)
    panic_count=$((panic_count + $(git diff --cached | grep -c "\.expect(" || true)))
    if [ "$panic_count" -gt 0 ]; then
        print_status "WARN" "Found $panic_count uses of unwrap/expect that could cause panics"
        issues_found=$((issues_found + panic_count))
    fi

    # Check for SQL injection patterns
    local sql_injection
    sql_injection=$(git diff --cached | grep -c "format!.*SELECT\|format!.*INSERT\|format!.*UPDATE\|format!.*DELETE" || true)
    if [ "$sql_injection" -gt 0 ]; then
        print_status "FAIL" "Potential SQL injection vulnerability detected"
        issues_found=$((issues_found + sql_injection))
    fi

    # Check for command injection
    local cmd_injection
    cmd_injection=$(git diff --cached | grep -c "Command::new(\|std::process::Command" || true)
    if [ "$cmd_injection" -gt 0 ]; then
        print_status "WARN" "Command execution detected - verify input sanitization"
        issues_found=$((issues_found + cmd_injection))
    fi

    # Check for weak cryptography
    local weak_crypto
    weak_crypto=$(git diff --cached | grep -c "Md5\|Sha1\|Des\|Rc4" || true)
    if [ "$weak_crypto" -gt 0 ]; then
        print_status "FAIL" "Weak cryptographic algorithms detected"
        issues_found=$((issues_found + weak_crypto))
    fi

    echo "$issues_found" > "$RESULTS_DIR/insecure_patterns.count"
}

# Function to check for proper error handling
check_error_handling() {
    print_status "INFO" "Checking error handling patterns..."

    # Check for proper Result/Option usage
    local result_usage
    result_usage=$(git diff --cached | grep -c "Result<\|Option<" || true)

    local error_handling
    error_handling=$(git diff --cached | grep -c "map_err\|and_then\|or_else\|unwrap_or" || true)

    if [ "$result_usage" -gt 0 ] && [ "$error_handling" -eq 0 ]; then
        print_status "WARN" "Result/Option types used but limited error handling patterns found"
    else
        print_status "PASS" "Error handling patterns look adequate"
    fi
}

# Function to check for logging of sensitive data
check_sensitive_logging() {
    print_status "INFO" "Checking for sensitive data logging..."

    local sensitive_logs=0

    # Check for logging of passwords, keys, tokens
    local log_patterns=(
        "log.*password"
        "log.*secret"
        "log.*key"
        "log.*token"
        "println.*password"
        "println.*secret"
        "println.*key"
        "println.*token"
    )

    for pattern in "${log_patterns[@]}"; do
        local matches
        matches=$(git diff --cached | grep -i "$pattern" | wc -l)
        if [ "$matches" -gt 0 ]; then
            print_status "FAIL" "Potential sensitive data logging detected: $pattern"
            sensitive_logs=$((sensitive_logs + matches))
        fi
    done

    if [ $sensitive_logs -eq 0 ]; then
        print_status "PASS" "No sensitive data logging detected"
    fi

    echo "$sensitive_logs" > "$RESULTS_DIR/sensitive_logging.count"
}

# Function to check for proper input validation
check_input_validation() {
    print_status "INFO" "Checking input validation..."

    # Look for functions that take user input
    local input_functions
    input_functions=$(git diff --cached | grep -c "fn.*&str\|fn.*String\|fn.*Path" || true)

    if [ "$input_functions" -gt 0 ]; then
        print_status "INFO" "Found $input_functions functions taking string/path input - ensure validation"
    fi

    # Check for validation patterns
    local validation_patterns
    validation_patterns=$(git diff --cached | grep -c "validate\|sanitize\|check_\|is_valid" || true)

    if [ "$input_functions" -gt 0 ] && [ "$validation_patterns" -eq 0 ]; then
        print_status "WARN" "Input-taking functions found but no validation patterns detected"
    else
        print_status "PASS" "Input validation patterns present"
    fi
}

# Function to generate security review report
generate_security_report() {
    print_status "INFO" "Generating security review report..."

    local total_issues=0

    # Count all issues
    if [ -f "$RESULTS_DIR/security_vulnerabilities.count" ]; then
        total_issues=$((total_issues + $(cat "$RESULTS_DIR/security_vulnerabilities.count")))
    fi
    if [ -f "$RESULTS_DIR/insecure_patterns.count" ]; then
        total_issues=$((total_issues + $(cat "$RESULTS_DIR/insecure_patterns.count")))
    fi
    if [ -f "$RESULTS_DIR/sensitive_logging.count" ]; then
        total_issues=$((total_issues + $(cat "$RESULTS_DIR/sensitive_logging.count")))
    fi

    # Create JSON report
    cat > "$RESULTS_DIR/security-review-report.json" << EOF
{
  "security_review": {
    "timestamp": "$(date -Iseconds)",
    "commit": "$(git rev-parse HEAD)",
    "total_issues": $total_issues,
    "checks_performed": [
      "dependency_vulnerabilities",
      "insecure_patterns",
      "sensitive_logging",
      "error_handling",
      "input_validation"
    ],
    "recommendations": [
      "Review all flagged security issues",
      "Ensure proper input validation",
      "Avoid logging sensitive data",
      "Use secure coding practices",
      "Run security scans regularly"
    ]
  }
}
EOF

    if [ $total_issues -eq 0 ]; then
        print_status "PASS" "Security review completed with no issues found"
    else
        print_status "WARN" "Security review completed with $total_issues issues found"
    fi
}

# Main execution
main() {
    # Change to the correct directory if running in CI
    if [ -d "products/costpilot" ]; then
        cd products/costpilot
        print_status "INFO" "Running in CI environment, changed to products/costpilot"
    fi

    check_security_vulnerabilities
    check_insecure_patterns
    check_error_handling
    check_sensitive_logging
    check_input_validation
    generate_security_report

    print_status "INFO" "Security review completed. Results saved to $RESULTS_DIR/"
}

main "$@"
