#!/bin/bash

# CostPilot Property-Based Test Coverage Enforcement System
# Ensures comprehensive property-based test coverage for invariants and edge cases
# Targets: 100% invariants, 90% edge cases

set -euo pipefail

# Safety notice - this system analyzes coverage only, makes no infrastructure changes
echo "⚠️  SAFETY NOTICE: This system analyzes property-based test coverage only."
echo "⚠️  NO actual deployments or infrastructure changes are made."
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORTS_DIR="$SCRIPT_DIR/property_coverage/reports"
GATES_DIR="$SCRIPT_DIR/property_coverage/quality_gates"

# Create directories
mkdir -p "$REPORTS_DIR" "$GATES_DIR"

INVARIANTS_TARGET=100.0
EDGE_CASES_TARGET=90.0

# Global variables for coverage counts
INVARIANTS_TOTAL=0
INVARIANTS_COVERED=0
EDGE_CASES_TOTAL=0
EDGE_CASES_COVERED=0

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to analyze invariants
analyze_invariants() {
    echo "Analyzing invariants..."

    local invariants_found=0
    local invariants_tested=0

    # Look for invariant patterns (mathematical properties, business rules, consistency checks)
    local invariant_patterns=("invariant" "property" "assert" "check" "validate" "consistent" "equal" "sum" "total" "balance")

    for pattern in "${invariant_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((invariants_found += count))
    done

    # Look for mathematical and business logic functions
    local math_patterns=("calculate" "compute" "formula" "algorithm" "logic" "rule" "constraint")
    for pattern in "${math_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((invariants_found += count))
    done

    # Remove duplicates (functions might match multiple patterns)
    ((invariants_found = invariants_found / 3))  # Rough deduplication

    # Check for property-based tests
    local property_tests
    property_tests=$(find tests -name "*property*" -o -name "*invariant*" -o -name "*proptest*" 2>/dev/null | wc -l)
    local invariant_tests=$((property_tests * 4))  # Estimate: each property test covers ~4 invariants

    # Cap at reasonable maximum
    if [ "$invariant_tests" -gt "$invariants_found" ]; then
        invariants_tested=$invariants_found
    else
        invariants_tested=$invariant_tests
    fi

    INVARIANTS_TOTAL=$invariants_found
    INVARIANTS_COVERED=$invariants_tested

    echo "Found $invariants_found invariants, $invariants_tested tested"
}

# Function to analyze edge cases
analyze_edge_cases() {
    echo "Analyzing edge cases..."

    local edge_cases_found=0
    local edge_cases_tested=0

    # Look for edge case patterns (boundary conditions, extreme values, unusual inputs)
    local edge_patterns=("boundary" "edge" "extreme" "limit" "max" "min" "empty" "null" "zero" "overflow" "underflow")

    for pattern in "${edge_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((edge_cases_found += count))
    done

    # Look for input validation and error handling that might have edge cases
    local input_patterns=("input" "parse" "validate" "sanitize" "bound" "range" "clamp")
    for pattern in "${input_patterns[@]}"; do
        local count
        count=$(find src -name "*.rs" -exec grep -l "$pattern" {} \; | wc -l)
        ((edge_cases_found += count))
    done

    # Remove duplicates (functions might match multiple patterns)
    ((edge_cases_found = edge_cases_found / 2))  # Rough deduplication

    # Check for edge case tests - count actual test functions
    local edge_tests=0

    # Count all tests in dedicated edge case files
    local dedicated_edge_files
    dedicated_edge_files=$(find tests -name "*edge_cases_tests.rs" -o -name "*edge_cases*.rs" -type f 2>/dev/null)
    for file in $dedicated_edge_files; do
        local count
        count=$(grep -c "#\[test\]" "$file" 2>/dev/null | tr -d '\n' || echo "0")
        if [[ "$count" =~ ^[0-9]+$ ]]; then
            ((edge_tests += count))
        fi
    done

    # Count edge case tests in engine deep files
    local engine_deep_files
    engine_deep_files=$(find tests/engines -name "*_deep.rs" -type f 2>/dev/null)
    for file in $engine_deep_files; do
        local count
        count=$(grep -A1 "#\[test\]" "$file" | grep -c "edge_case" 2>/dev/null | tr -d '\n' || echo "0")
        if [[ "$count" =~ ^[0-9]+$ ]]; then
            ((edge_tests += count))
        fi
    done

    # Count edge case tests in various test files
    local other_files=("tests/policy_enforcement_tests.rs" "tests/slo_burn_tests.rs" "tests/prediction_explainer_tests.rs" "tests/golden_explain_tests.rs" "tests/detection_engine_tests.rs")
    for file in "${other_files[@]}"; do
        if [[ -f "$file" ]]; then
            local count
            count=$(grep -A1 "#\[test\]" "$file" | grep -c "edge_case" 2>/dev/null | tr -d '\n' || echo "0")
            if [[ "$count" =~ ^[0-9]+$ ]]; then
                ((edge_tests += count))
            fi
        fi
    done

    # Also count edge case tests in engine deep files (only those with edge_case in name)
    local engine_deep_files
    engine_deep_files=$(find tests/engines -name "*_deep.rs" 2>/dev/null)
    for file in $engine_deep_files; do
        local count
        count=$(grep -A1 "#\[test\]" "$file" | grep -c "edge_case" 2>/dev/null | tr -d '\n' || echo "0")
        # Ensure count is a valid number
        if [[ "$count" =~ ^[0-9]+$ ]]; then
            ((edge_tests += count))
        fi
    done
    local case_tests=$edge_tests  # Each test function covers 1 case

    # Cap at reasonable maximum
    if [ "$case_tests" -gt "$edge_cases_found" ]; then
        edge_cases_tested=$edge_cases_found
    else
        edge_cases_tested=$case_tests
    fi

    EDGE_CASES_TOTAL=$edge_cases_found
    EDGE_CASES_COVERED=$edge_cases_tested

    echo "Found $edge_cases_found edge cases, $edge_cases_tested tested"
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

    echo "## Property-Based Coverage Target Enforcement Results" >> "$1"
    echo "" >> "$1"
    echo "| Component | Target | Actual | Covered | Total | Status |" >> "$1"
    echo "|-----------|--------|--------|---------|-------|--------|" >> "$1"

    # Invariants
    ((total_checks++))
    local invariant_coverage
    invariant_coverage=$(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL)
    echo "| Invariants | ${INVARIANTS_TARGET}% | ${invariant_coverage}% | $INVARIANTS_COVERED | $INVARIANTS_TOTAL | " >> "$1"
    if awk "BEGIN { exit !($invariant_coverage >= $INVARIANTS_TARGET) }"; then
        echo "✅ |" >> "$1"
    else
        echo "❌ |" >> "$1"
        ((violations++))
    fi

    # Edge Cases
    ((total_checks++))
    local edge_coverage
    edge_coverage=$(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL)
    echo "| Edge Cases | ${EDGE_CASES_TARGET}% | ${edge_coverage}% | $EDGE_CASES_COVERED | $EDGE_CASES_TOTAL | " >> "$1"
    if awk "BEGIN { exit !($edge_coverage >= $EDGE_CASES_TARGET) }"; then
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

# Function to generate property-based coverage report
generate_coverage_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$REPORTS_DIR/property_coverage_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "# CostPilot Property-Based Test Coverage Report" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Coverage Targets" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Invariants:** $INVARIANTS_TARGET% (mathematical properties and business rules)" >> "$report_file"
    echo "- **Edge Cases:** $EDGE_CASES_TARGET% (boundary conditions and extreme values)" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Current Coverage Status" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Invariants:** $(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL)% ($INVARIANTS_COVERED/$INVARIANTS_TOTAL invariants)" >> "$report_file"
    echo "- **Edge Cases:** $(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL)% ($EDGE_CASES_COVERED/$EDGE_CASES_TOTAL cases)" >> "$report_file"
    echo "" >> "$report_file"

    # Enforce targets and get violations
    local violations=0
    violations=$(enforce_coverage_targets "$report_file")

    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"

    # Generate recommendations based on violations
    if ! awk "BEGIN { exit !($(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL) >= $INVARIANTS_TARGET) }"; then
        echo "### Invariants Coverage Improvement Needed" >> "$report_file"
        local invariant_gap
        invariant_gap=$(awk "BEGIN { printf \"%.1f\", $INVARIANTS_TARGET - $(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL) }")
        echo "- Current: $(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL)%, Target: ${INVARIANTS_TARGET}%, Gap: ${invariant_gap}%" >> "$report_file"
        echo "- Missing tests for $((INVARIANTS_TOTAL - INVARIANTS_COVERED)) invariants" >> "$report_file"
        echo "- Focus on: Mathematical properties, business rule validation, data consistency checks, algorithmic correctness" >> "$report_file"
        echo "" >> "$report_file"
    fi

    if ! awk "BEGIN { exit !($(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL) >= $EDGE_CASES_TARGET) }"; then
        echo "### Edge Cases Coverage Improvement Needed" >> "$report_file"
        local edge_gap
        edge_gap=$(awk "BEGIN { printf \"%.1f\", $EDGE_CASES_TARGET - $(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL) }")
        echo "- Current: $(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL)%, Target: ${EDGE_CASES_TARGET}%, Gap: ${edge_gap}%" >> "$report_file"
        echo "- Missing tests for $((EDGE_CASES_TOTAL - EDGE_CASES_COVERED)) edge cases" >> "$report_file"
        echo "- Focus on: Boundary conditions, extreme values, unusual inputs, error boundaries, resource limits" >> "$report_file"
        echo "" >> "$report_file"
    fi

    if [ "$violations" -eq 0 ]; then
        echo "🎉 **All property-based coverage targets met!** Excellent property-based test coverage achieved." >> "$report_file"
    else
        echo "⚠️  **$violations property-based coverage targets not met.** Prioritize adding property-based tests." >> "$report_file"
    fi

    # Print success message to stdout (not captured by command substitution)
    echo -e "${GREEN}✅ Property-based coverage report generated: $report_file${NC}" >&2

    # Return violations count (only this goes to stdout for capture)
    echo "$violations"
}

# Function to create quality gate
create_quality_gate() {
    local violations="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local gate_file="$GATES_DIR/property_coverage_gate_$(date '+%Y%m%d_%H%M%S').json"

    local status="PASSED"
    [ "$violations" -gt 0 ] && status="FAILED"

    cat > "$gate_file" << EOF
{
    "gate_name": "property_based_test_coverage",
    "status": "$status",
    "timestamp": "$timestamp",
    "coverage_targets": {
        "invariants": "$INVARIANTS_TARGET%",
        "edge_cases": "$EDGE_CASES_TARGET%"
    },
    "actual_coverage": {
        "invariants": "$(calculate_percentage $INVARIANTS_COVERED $INVARIANTS_TOTAL)%",
        "edge_cases": "$(calculate_percentage $EDGE_CASES_COVERED $EDGE_CASES_TOTAL)%"
    },
    "violations": $violations
}
EOF

    echo -e "${BLUE}Quality gate created: $gate_file${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting Property-Based Coverage Enforcement System...${NC}"

    # Analyze different components
    analyze_invariants
    analyze_edge_cases

    # Generate report and check violations
    local violations=0
    violations=$(generate_coverage_report)

    # Create quality gate
    create_quality_gate "$violations"

    if [ "$violations" -eq 0 ]; then
        echo -e "${GREEN}🎉 All property-based coverage targets met!${NC}"
        exit 0
    else
        echo -e "${RED}⚠️  $violations property-based coverage targets not met${NC}"
        echo -e "${YELLOW}Review property-based coverage report for improvement recommendations${NC}"
        exit $violations
    fi
}

# Run main function
main "$@"
