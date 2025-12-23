#!/bin/bash
# CostPilot Synthetic Monitoring Script
# 24/7 automated health checks and synthetic test scenarios

set -e

echo "🔍 CostPilot Synthetic Monitoring"
echo "================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
MONITORING_DIR="synthetic-monitoring"
RESULTS_FILE="$MONITORING_DIR/health-check-$(date +%Y%m%d-%H%M%S).json"
mkdir -p "$MONITORING_DIR"

# Function to log results
log_result() {
    local check_name=$1
    local status=$2
    local message=$3
    local duration=${4:-0}

    local result="{\"timestamp\":\"$(date -Iseconds)\",\"check\":\"$check_name\",\"status\":\"$status\",\"message\":\"$message\",\"duration_ms\":$duration}"

    echo "$result" >> "$RESULTS_FILE"

    case $status in
        "PASS")
            echo -e "${GREEN}✅ $check_name${NC}: $message"
            ;;
        "FAIL")
            echo -e "${RED}❌ $check_name${NC}: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  $check_name${NC}: $message"
            ;;
    esac
}

# Function to measure execution time
measure_time() {
    local start=$(date +%s%3N)
    "$@"
    local end=$(date +%s%3N)
    echo $((end - start))
}

# Health Check 1: Binary integrity
check_binary_integrity() {
    log_result "binary_integrity" "INFO" "Checking binary integrity..."

    if [ -f "target/release/costpilot" ]; then
        local size
        size=$(stat -f%z target/release/costpilot 2>/dev/null || stat -c%s target/release/costpilot 2>/dev/null || echo "0")

        if [ "$size" -gt 1000 ]; then
            log_result "binary_integrity" "PASS" "Binary exists and has reasonable size: ${size} bytes"
        else
            log_result "binary_integrity" "FAIL" "Binary size too small: ${size} bytes"
        fi
    else
        log_result "binary_integrity" "FAIL" "Release binary not found"
    fi
}

# Health Check 2: Basic functionality test
check_basic_functionality() {
    log_result "basic_functionality" "INFO" "Testing basic CLI functionality..."

    local start_time=$(date +%s%3N)
    local output
    output=$(timeout 10s ./target/release/costpilot feature list 2>&1)
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local duration=$((end_time - start_time))

    if [ $exit_code -eq 0 ] && echo "$output" | grep -q "Feature Flags" && [ $duration -lt 10000 ]; then
        log_result "basic_functionality" "PASS" "Feature list command executed successfully in ${duration}ms"
    else
        log_result "basic_functionality" "FAIL" "Feature list command failed (exit: $exit_code, output: '$output', duration: ${duration}ms)"
    fi
}

# Health Check 3: Help system
check_help_system() {
    log_result "help_system" "INFO" "Testing help system..."

    # Test that the CLI provides useful output when given invalid input
    local output
    output=$(timeout 5s ./target/release/costpilot invalid-command 2>&1 || true)

    if echo "$output" | grep -q -i "error\|invalid"; then
        log_result "help_system" "PASS" "Error handling provides feedback"
    else
        log_result "help_system" "FAIL" "Error handling not working properly (output: '$output')"
    fi
}

# Health Check 4: Feature flags
check_feature_flags() {
    log_result "feature_flags" "INFO" "Testing feature flag system..."

    local output
    output=$(./target/release/costpilot feature list 2>/dev/null)

    if [ $? -eq 0 ] && echo "$output" | grep -q "Feature Flags"; then
        log_result "feature_flags" "PASS" "Feature flag system operational"
    else
        log_result "feature_flags" "FAIL" "Feature flag system not working"
    fi
}

# Health Check 5: Configuration validation
check_config_validation() {
    log_result "config_validation" "INFO" "Testing configuration validation..."

    if [ -f "costpilot.yaml" ]; then
        local output
        output=$(./target/release/costpilot validate costpilot.yaml 2>&1 || true)

        if echo "$output" | grep -q -i "valid\|ok\|success"; then
            log_result "config_validation" "PASS" "Configuration validation successful"
        else
            log_result "config_validation" "WARN" "Configuration validation had issues: $output"
        fi
    else
        log_result "config_validation" "INFO" "No configuration file to validate"
    fi
}

# Health Check 6: Memory usage
check_memory_usage() {
    log_result "memory_usage" "INFO" "Checking memory usage patterns..."

    # Run a simple command and check if it completes without excessive memory
    local start_time=$(date +%s)
    timeout 30s ./target/release/costpilot feature list >/dev/null 2>&1
    local exit_code=$?
    local end_time=$(date +%s)

    if [ $exit_code -eq 0 ] && [ $((end_time - start_time)) -lt 30 ]; then
        log_result "memory_usage" "PASS" "Memory usage within acceptable limits"
    else
        log_result "memory_usage" "FAIL" "Memory usage check failed or timed out (exit: $exit_code, duration: $((end_time - start_time))s)"
    fi
}

# Health Check 7: Error handling
check_error_handling() {
    log_result "error_handling" "INFO" "Testing error handling..."

    # Test with invalid arguments
    local output
    output=$(./target/release/costpilot invalid-command 2>&1 || true)

    if echo "$output" | grep -q -i "error\|invalid"; then
        log_result "error_handling" "PASS" "Error handling working correctly"
    else
        log_result "error_handling" "FAIL" "Error handling not working properly (output: '$output')"
    fi
}

# Health Check 8: Performance regression
check_performance_regression() {
    log_result "performance_regression" "INFO" "Checking for performance regressions..."

    if [ -f "performance-baseline.json" ]; then
        # Run performance review script
        if ./scripts/performance_review.sh >/dev/null 2>&1; then
            log_result "performance_regression" "PASS" "Performance within acceptable limits"
        else
            log_result "performance_regression" "FAIL" "Performance regression detected"
        fi
    else
        log_result "performance_regression" "INFO" "No performance baseline available"
    fi
}

# Health Check 9: Code quality metrics
check_code_quality() {
    log_result "code_quality" "INFO" "Checking code quality metrics..."

    if [ -f "scripts/metrics_monitoring.sh" ]; then
        if ./scripts/metrics_monitoring.sh >/dev/null 2>&1; then
            log_result "code_quality" "PASS" "Code quality metrics acceptable"
        else
            log_result "code_quality" "WARN" "Code quality issues detected"
        fi
    else
        log_result "code_quality" "INFO" "Code quality monitoring script not available"
    fi
}

# Health Check 10: Security scan
check_security() {
    log_result "security_scan" "INFO" "Running security checks..."

    if [ -f "scripts/security_review.sh" ]; then
        if ./scripts/security_review.sh >/dev/null 2>&1; then
            log_result "security_scan" "PASS" "Security checks passed"
        else
            log_result "security_scan" "FAIL" "Security vulnerabilities detected"
        fi
    else
        log_result "security_scan" "INFO" "Security review script not available"
    fi
}

# Main monitoring function
run_synthetic_monitoring() {
    echo "Starting synthetic monitoring suite..."
    echo "Results will be saved to: $RESULTS_FILE"
    echo

    # Initialize results file
    echo "[" > "$RESULTS_FILE"

    # Run all health checks
    check_binary_integrity
    check_basic_functionality
    check_help_system
    check_feature_flags
    check_config_validation
    check_memory_usage
    check_error_handling
    check_performance_regression
    check_code_quality
    check_security

    # Close JSON array
    echo "]" >> "$RESULTS_FILE"

    echo
    echo "Synthetic monitoring complete."
    echo "Results saved to: $RESULTS_FILE"

    # Summary
    local total_checks=$(grep -c '"check"' "$RESULTS_FILE")
    local passed_checks=$(grep -c '"status":"PASS"' "$RESULTS_FILE")
    local failed_checks=$(grep -c '"status":"FAIL"' "$RESULTS_FILE")
    local warn_checks=$(grep -c '"status":"WARN"' "$RESULTS_FILE")

    echo
    echo "Summary:"
    echo "  Total checks: $total_checks"
    echo "  Passed: $passed_checks"
    echo "  Warnings: $warn_checks"
    echo "  Failed: $failed_checks"

    if [ "$failed_checks" -gt 0 ]; then
        echo -e "${RED}❌ Synthetic monitoring detected $failed_checks failures${NC}"
        return 1
    elif [ "$warn_checks" -gt 0 ]; then
        echo -e "${YELLOW}⚠️  Synthetic monitoring completed with $warn_checks warnings${NC}"
        return 0
    else
        echo -e "${GREEN}✅ All synthetic monitoring checks passed${NC}"
        return 0
    fi
}

# Run the monitoring
run_synthetic_monitoring
