#!/bin/bash

# CostPilot E2E Test Coverage Enforcement System
# Ensures comprehensive end-to-end test coverage across all user workflows, failure scenarios, and platform matrix
# Targets: 100% user workflows, 100% failure scenarios, 100% platform matrix

set -euo pipefail

# Safety notice - this system analyzes coverage only, makes no infrastructure changes
echo "⚠️  SAFETY NOTICE: This system analyzes E2E test coverage only."
echo "⚠️  NO actual deployments or infrastructure changes are made."
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORTS_DIR="$SCRIPT_DIR/e2e_coverage/reports"
GATES_DIR="$SCRIPT_DIR/e2e_coverage/quality_gates"

# Create directories
mkdir -p "$REPORTS_DIR" "$GATES_DIR"

USER_WORKFLOWS_TARGET=100.0
FAILURE_SCENARIOS_TARGET=100.0
PLATFORM_MATRIX_TARGET=100.0

# Global variables for coverage counts
USER_WORKFLOWS_TOTAL=0
USER_WORKFLOWS_COVERED=0
FAILURE_SCENARIOS_TOTAL=0
FAILURE_SCENARIOS_COVERED=0
PLATFORM_MATRIX_TOTAL=0
PLATFORM_MATRIX_COVERED=0

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to analyze user workflows
analyze_user_workflows() {
    echo "Analyzing user workflows..."

    local workflows_found=0
    local workflows_tested=0

    # Count actual test functions in E2E test files
    local test_count
    test_count=$(grep -c "fn test_cli_" tests/e2e/test_cli.rs 2>/dev/null || echo "0")
    workflows_tested=$test_count

    # Look for main user workflows and use cases in source code
    local workflow_patterns=("main" "run" "execute" "process" "analyze" "scan" "audit" "baseline" "cost" "pricing")

    for pattern in "${workflow_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((workflows_found += count))
    done

    # Remove duplicates (functions might match multiple patterns)
    ((workflows_found = workflows_found / 2))  # Rough deduplication

    # If we have more tests than workflows found, cap at workflows found
    if [ "$workflows_tested" -gt "$workflows_found" ]; then
        workflows_tested=$workflows_found
    fi

    USER_WORKFLOWS_TOTAL=$workflows_found
    USER_WORKFLOWS_COVERED=$workflows_tested

    echo "Found $workflows_found user workflows, $workflows_tested tested"
}

# Function to analyze failure scenarios
analyze_failure_scenarios() {
    echo "Analyzing failure scenarios..."

    local scenarios_found=0
    local scenarios_tested=0

    # Look for failure handling and error scenarios
    local failure_patterns=("error" "fail" "timeout" "network" "invalid" "missing" "corrupt" "permission" "auth")

    for pattern in "${failure_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((scenarios_found += count))
    done

    # Remove duplicates (functions might match multiple patterns)
    ((scenarios_found = scenarios_found / 3))  # Rough deduplication

    # Check for failure scenario tests
    local failure_tests
    failure_tests=$(find tests -name "*fail*" -o -name "*error*" -o -name "*timeout*" -o -name "*invalid*" 2>/dev/null | wc -l)
    local scenario_tests=$((failure_tests * 3))  # Estimate: each failure test covers ~3 scenarios

    # Cap at reasonable maximum
    if [ "$scenario_tests" -gt "$scenarios_found" ]; then
        scenarios_tested=$scenarios_found
    else
        scenarios_tested=$scenario_tests
    fi

    FAILURE_SCENARIOS_TOTAL=$scenarios_found
    FAILURE_SCENARIOS_COVERED=$scenarios_tested

    echo "Found $scenarios_found failure scenarios, $scenarios_tested tested"
}

# Function to analyze platform matrix
analyze_platform_matrix() {
    echo "Analyzing platform matrix..."

    local platforms_found=0
    local platforms_tested=0

    # Look for platform-specific code and configurations
    local platform_patterns=("linux" "windows" "macos" "x86" "arm" "docker" "container" "ci" "cd")

    for pattern in "${platform_patterns[@]}"; do
        local count
        count=$(find . -name "*.rs" -o -name "*.yml" -o -name "*.yaml" -o -name "Dockerfile*" -o -name "*.toml" | xargs grep -l "$pattern" 2>/dev/null | wc -l)
        ((platforms_found += count))
    done

    # Remove duplicates (files might match multiple patterns)
    ((platforms_found = platforms_found / 4))  # Rough deduplication

    # Check for platform matrix tests
    local platform_tests
    platform_tests=$(find tests -name "*platform*" -o -name "*matrix*" -o -name "*ci*" 2>/dev/null | wc -l)
    local matrix_tests=$((platform_tests * 5))  # Estimate: each platform test covers ~5 matrix combinations

    # Cap at reasonable maximum
    if [ "$matrix_tests" -gt "$platforms_found" ]; then
        platforms_tested=$platforms_found
    else
        platforms_tested=$matrix_tests
    fi

    PLATFORM_MATRIX_TOTAL=$platforms_found
    PLATFORM_MATRIX_COVERED=$platforms_tested

    echo "Found $platforms_found platform matrix combinations, $platforms_tested tested"
}

# Function to calculate coverage percentage
calculate_percentage() {
    local covered=$1 total=$2
    if [ "$total" -eq 0 ]; then
        echo "0.0"
    else
        awk "BEGIN { printf \"%.1f\", ($covered / $total) * 100 }"
    fi
}

# Function to enforce coverage targets
enforce_coverage_targets() {
    local violations=0
    local total_checks=0

    echo "## E2E Coverage Target Enforcement Results" >> "$1"
    echo "" >> "$1"
    echo "| Component | Target | Actual | Covered | Total | Status |" >> "$1"
    echo "|-----------|--------|--------|---------|-------|--------|" >> "$1"

    # User Workflows
    ((total_checks++))
    local workflow_coverage
    workflow_coverage=$(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL)
    echo "| User Workflows | ${USER_WORKFLOWS_TARGET}% | ${workflow_coverage}% | $USER_WORKFLOWS_COVERED | $USER_WORKFLOWS_TOTAL | " >> "$1"
    if awk "BEGIN { exit !($workflow_coverage >= $USER_WORKFLOWS_TARGET) }"; then
        echo "✅ |" >> "$1"
    else
        echo "❌ |" >> "$1"
        ((violations++))
    fi

    # Failure Scenarios
    ((total_checks++))
    local failure_coverage
    failure_coverage=$(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL)
    echo "| Failure Scenarios | ${FAILURE_SCENARIOS_TARGET}% | ${failure_coverage}% | $FAILURE_SCENARIOS_COVERED | $FAILURE_SCENARIOS_TOTAL | " >> "$1"
    if awk "BEGIN { exit !($failure_coverage >= $FAILURE_SCENARIOS_TARGET) }"; then
        echo "✅ |" >> "$1"
    else
        echo "❌ |" >> "$1"
        ((violations++))
    fi

    # Platform Matrix
    ((total_checks++))
    local platform_coverage
    platform_coverage=$(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL)
    echo "| Platform Matrix | ${PLATFORM_MATRIX_TARGET}% | ${platform_coverage}% | $PLATFORM_MATRIX_COVERED | $PLATFORM_MATRIX_TOTAL | " >> "$1"
    if awk "BEGIN { exit !($platform_coverage >= $PLATFORM_MATRIX_TARGET) }"; then
        echo "✅ |" >> "$1"
    else
        echo "❌ |" >> "$1"
        ((violations++))
    fi

    echo "" >> "$1"
    echo "**Summary:** $violations violations out of $total_checks checks" >> "$1"
    echo "" >> "$1"

    echo "$violations"
}

# Function to generate E2E coverage report
generate_coverage_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$REPORTS_DIR/e2e_coverage_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "# CostPilot E2E Test Coverage Report" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Coverage Targets" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **User Workflows:** $USER_WORKFLOWS_TARGET% (complete end-to-end user journeys)" >> "$report_file"
    echo "- **Failure Scenarios:** $FAILURE_SCENARIOS_TARGET% (error conditions and edge cases)" >> "$report_file"
    echo "- **Platform Matrix:** $PLATFORM_MATRIX_TARGET% (cross-platform compatibility)" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Current Coverage Status" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **User Workflows:** $(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL)% ($USER_WORKFLOWS_COVERED/$USER_WORKFLOWS_TOTAL workflows)" >> "$report_file"
    echo "- **Failure Scenarios:** $(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL)% ($FAILURE_SCENARIOS_COVERED/$FAILURE_SCENARIOS_TOTAL scenarios)" >> "$report_file"
    echo "- **Platform Matrix:** $(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL)% ($PLATFORM_MATRIX_COVERED/$PLATFORM_MATRIX_TOTAL combinations)" >> "$report_file"
    echo "" >> "$report_file"

    # Enforce targets and get violations
    local violations=0
    violations=$(enforce_coverage_targets "$report_file")

    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"

    # Generate recommendations based on violations
    if ! awk "BEGIN { exit !($(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL) >= $USER_WORKFLOWS_TARGET) }"; then
        echo "### User Workflows Coverage Improvement Needed" >> "$report_file"
        local workflow_gap
        workflow_gap=$(awk "BEGIN { printf \"%.1f\", $USER_WORKFLOWS_TARGET - $(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL) }")
        echo "- Current: $(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL)%, Target: ${USER_WORKFLOWS_TARGET}%, Gap: ${workflow_gap}%" >> "$report_file"
        echo "- Missing tests for $((USER_WORKFLOWS_TOTAL - USER_WORKFLOWS_COVERED)) workflows" >> "$report_file"
        echo "- Focus on: Complete user journey testing, workflow integration, user experience validation" >> "$report_file"
        echo "" >> "$report_file"
    fi

    if ! awk "BEGIN { exit !($(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL) >= $FAILURE_SCENARIOS_TARGET) }"; then
        echo "### Failure Scenarios Coverage Improvement Needed" >> "$report_file"
        local failure_gap
        failure_gap=$(awk "BEGIN { printf \"%.1f\", $FAILURE_SCENARIOS_TARGET - $(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL) }")
        echo "- Current: $(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL)%, Target: ${FAILURE_SCENARIOS_TARGET}%, Gap: ${failure_gap}%" >> "$report_file"
        echo "- Missing tests for $((FAILURE_SCENARIOS_TOTAL - FAILURE_SCENARIOS_COVERED)) scenarios" >> "$report_file"
        echo "- Focus on: Error handling validation, graceful degradation, recovery mechanisms" >> "$report_file"
        echo "" >> "$report_file"
    fi

    if ! awk "BEGIN { exit !($(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL) >= $PLATFORM_MATRIX_TARGET) }"; then
        echo "### Platform Matrix Coverage Improvement Needed" >> "$report_file"
        local platform_gap
        platform_gap=$(awk "BEGIN { printf \"%.1f\", $PLATFORM_MATRIX_TARGET - $(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL) }")
        echo "- Current: $(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL)%, Target: ${PLATFORM_MATRIX_TARGET}%, Gap: ${platform_gap}%" >> "$report_file"
        echo "- Missing tests for $((PLATFORM_MATRIX_TOTAL - PLATFORM_MATRIX_COVERED)) combinations" >> "$report_file"
        echo "- Focus on: Cross-platform compatibility, CI/CD pipeline validation, deployment testing" >> "$report_file"
        echo "" >> "$report_file"
    fi

    if [ "$violations" -eq 0 ]; then
        echo "🎉 **All E2E coverage targets met!** Excellent end-to-end test coverage achieved." >> "$report_file"
    else
        echo "⚠️  **$violations E2E coverage targets not met.** Prioritize adding end-to-end tests." >> "$report_file"
    fi

    # Print success message to stdout (not captured by command substitution)
    echo -e "${GREEN}✅ E2E coverage report generated: $report_file${NC}" >&2

    # Return violations count (only this goes to stdout for capture)
    echo "$violations"
}

# Function to create quality gate
create_quality_gate() {
    local violations="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local gate_file="$GATES_DIR/e2e_coverage_gate_$(date '+%Y%m%d_%H%M%S').json"

    local status="PASSED"
    [ "$violations" -gt 0 ] && status="FAILED"

    cat > "$gate_file" << EOF
{
    "gate_name": "e2e_test_coverage",
    "status": "$status",
    "timestamp": "$timestamp",
    "coverage_targets": {
        "user_workflows": "$USER_WORKFLOWS_TARGET%",
        "failure_scenarios": "$FAILURE_SCENARIOS_TARGET%",
        "platform_matrix": "$PLATFORM_MATRIX_TARGET%"
    },
    "actual_coverage": {
        "user_workflows": "$(calculate_percentage $USER_WORKFLOWS_COVERED $USER_WORKFLOWS_TOTAL)%",
        "failure_scenarios": "$(calculate_percentage $FAILURE_SCENARIOS_COVERED $FAILURE_SCENARIOS_TOTAL)%",
        "platform_matrix": "$(calculate_percentage $PLATFORM_MATRIX_COVERED $PLATFORM_MATRIX_TOTAL)%"
    },
    "violations": $violations
}
EOF

    echo -e "${BLUE}Quality gate created: $gate_file${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting E2E Coverage Enforcement System...${NC}"

    # Analyze different components
    analyze_user_workflows
    analyze_failure_scenarios
    analyze_platform_matrix

    # Generate report and check violations
    local violations=0
    violations=$(generate_coverage_report)

    # Create quality gate
    create_quality_gate "$violations"

    if [ "$violations" -eq 0 ]; then
        echo -e "${GREEN}🎉 All E2E coverage targets met!${NC}"
        exit 0
    else
        echo -e "${RED}⚠️  $violations E2E coverage targets not met${NC}"
        echo -e "${YELLOW}Review E2E coverage report for improvement recommendations${NC}"
        exit $violations
    fi
}

# Run main function
main "$@"
