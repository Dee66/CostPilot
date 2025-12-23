#!/bin/bash

# CostPilot Coverage Regression Monitoring System
# Monitors test coverage across all dimensions and enforces automated quality gates
# Prevents coverage regressions and ensures continuous quality improvement

set -euo pipefail

# Safety notice - this system analyzes coverage only, makes no infrastructure changes
echo "⚠️  SAFETY NOTICE: This system monitors coverage regressions only."
echo "⚠️  NO actual deployments or infrastructure changes are made."
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORTS_DIR="$SCRIPT_DIR/coverage_monitoring/reports"
GATES_DIR="$SCRIPT_DIR/coverage_monitoring/quality_gates"
HISTORY_DIR="$SCRIPT_DIR/coverage_monitoring/history"

# Create directories
mkdir -p "$REPORTS_DIR" "$GATES_DIR" "$HISTORY_DIR"

# Coverage enforcement scripts to monitor
COVERAGE_SCRIPTS=(
    "enforce_unit_coverage.sh"
    "enforce_integration_coverage.sh"
    "enforce_e2e_coverage.sh"
    "enforce_property_coverage.sh"
    "enforce_security_coverage.sh"
)

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run coverage analysis
run_coverage_analysis() {
    local script_name="$1"
    local script_path="$SCRIPT_DIR/$script_name"

    if [ ! -x "$script_path" ]; then
        echo "Warning: $script_name not found or not executable"
        return 1
    fi

    echo "Running $script_name..."
    local output
    local exit_code

    # Capture both stdout and stderr, but exit code separately
    output=$("$script_path" 2>&1)
    exit_code=$?

    echo "$output"
    echo "--- Exit code: $exit_code ---"

    return $exit_code
}

# Function to extract coverage metrics from output
extract_coverage_metrics() {
    local output="$1"
    local script_name="$2"

    # Extract coverage percentages using grep and sed
    local unit_coverage
    unit_coverage=$(echo "$output" | grep -o "Unit test coverage: [0-9.]*%" | sed 's/Unit test coverage: //' | sed 's/%//' || echo "0")

    local integration_coverage
    integration_coverage=$(echo "$output" | grep -o "Integration coverage.*: [0-9.]*%" | sed 's/.*: //' | sed 's/%//' || echo "0")

    local e2e_coverage
    e2e_coverage=$(echo "$output" | grep -o "E2E coverage.*: [0-9.]*%" | sed 's/.*: //' | sed 's/%//' || echo "0")

    local property_coverage
    property_coverage=$(echo "$output" | grep -o "Property-based coverage.*: [0-9.]*%" | sed 's/.*: //' | sed 's/%//' || echo "0")

    local security_coverage
    security_coverage=$(echo "$output" | grep -o "Security coverage.*: [0-9.]*%" | sed 's/.*: //' | sed 's/%//' || echo "0")

    # Return as JSON-like string
    echo "{\"unit\": \"$unit_coverage\", \"integration\": \"$integration_coverage\", \"e2e\": \"$e2e_coverage\", \"property\": \"$property_coverage\", \"security\": \"$security_coverage\"}"
}

# Function to save coverage history
save_coverage_history() {
    local timestamp="$1"
    local metrics="$2"
    local history_file="$HISTORY_DIR/coverage_history_$(date '+%Y%m%d').json"

    # Create or append to history file
    if [ ! -f "$history_file" ]; then
        echo "[]" > "$history_file"
    fi

    # Add new entry to history
    local entry="{\"timestamp\": \"$timestamp\", \"metrics\": $metrics}"
    local current_content
    current_content=$(cat "$history_file")

    # Remove closing bracket, add comma and new entry, then close
    if [ "$current_content" = "[]" ]; then
        echo "[$entry]" > "$history_file"
    else
        # Remove last character (closing bracket), add comma, entry, and closing bracket
        sed '$ s/.$//' "$history_file" > "${history_file}.tmp"
        echo ",$entry]" >> "${history_file}.tmp"
        mv "${history_file}.tmp" "$history_file"
    fi
}

# Function to detect coverage regressions
detect_regressions() {
    local current_metrics="$1"
    local history_file="$HISTORY_DIR/coverage_history_$(date '+%Y%m%d').json"

    local regressions=0

    if [ ! -f "$history_file" ] || [ ! -s "$history_file" ]; then
        echo "No historical data available for regression analysis"
        return 0
    fi

    # Get previous metrics (second to last entry)
    local previous_metrics
    previous_metrics=$(jq -r '.[-2].metrics' "$history_file" 2>/dev/null || echo "{}")

    if [ "$previous_metrics" = "{}" ]; then
        echo "Insufficient historical data for regression analysis"
        return 0
    fi

    echo "Regression Analysis:"
    echo "==================="

    # Check each coverage type for regressions
    local coverage_types=("unit" "integration" "e2e" "property" "security")

    for type in "${coverage_types[@]}"; do
        local current_value
        local previous_value

        current_value=$(echo "$current_metrics" | jq -r ".$type" 2>/dev/null || echo "0")
        previous_value=$(echo "$previous_metrics" | jq -r ".$type" 2>/dev/null || echo "0")

        # Convert to numbers for comparison
        current_value=$(awk "BEGIN {print $current_value + 0}")
        previous_value=$(awk "BEGIN {print $previous_value + 0}")

        local diff
        diff=$(awk "BEGIN {printf \"%.1f\", $current_value - $previous_value}")

        if awk "BEGIN {exit !($current_value < $previous_value)}"; then
            echo -e "${RED}⚠️  $type coverage REGRESSION: $previous_value% → $current_value% (↓$diff%)${NC}"
            ((regressions++))
        elif awk "BEGIN {exit !($current_value > $previous_value)}"; then
            echo -e "${GREEN}✅ $type coverage IMPROVED: $previous_value% → $current_value% (↑$diff%)${NC}"
        else
            echo -e "${BLUE}➡️  $type coverage STABLE: $current_value%${NC}"
        fi
    done

    echo ""
    return $regressions
}

# Function to generate monitoring report
generate_monitoring_report() {
    local total_violations="$1"
    local total_regressions="$2"
    local regression_analysis="$3"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$REPORTS_DIR/coverage_monitoring_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "# CostPilot Coverage Regression Monitoring Report" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Coverage Monitoring Results" >> "$report_file"
    echo "" >> "$report_file"

    # Run all coverage analyses
    for script in "${COVERAGE_SCRIPTS[@]}"; do
        echo "### $(basename "$script" .sh | sed 's/enforce_//' | sed 's/_/ /g' | sed 's/\b\w/\U&/g') Coverage" >> "$report_file"
        echo "" >> "$report_file"

        if run_coverage_analysis "$script" >> "$report_file" 2>&1; then
            echo "✅ PASSED" >> "$report_file"
        else
            local exit_code=$?
            echo "❌ FAILED (exit code: $exit_code)" >> "$report_file"
        fi

        echo "" >> "$report_file"
    done

    echo "## Regression Analysis" >> "$report_file"
    echo "" >> "$report_file"
    echo "$regression_analysis" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Summary" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Total Coverage Violations:** $total_violations" >> "$report_file"
    echo "- **Coverage Regressions:** $total_regressions" >> "$report_file"
    echo "" >> "$report_file"

    if [ "$total_violations" -eq 0 ] && [ "$total_regressions" -eq 0 ]; then
        echo "🎉 **All coverage targets met and no regressions detected!**" >> "$report_file"
    else
        echo "⚠️  **Coverage issues detected:** $total_violations violations, $total_regressions regressions" >> "$report_file"
    fi

    echo -e "${GREEN}✅ Coverage monitoring report generated: $report_file${NC}" >&2

    # Return report file path (only this goes to stdout for capture)
    echo "$report_file"
}

# Function to create quality gate
create_quality_gate() {
    local violations="$1"
    local regressions="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local gate_file="$GATES_DIR/coverage_monitoring_gate_$(date '+%Y%m%d_%H%M%S').json"

    local status="PASSED"
    [ "$violations" -gt 0 ] || [ "$regressions" -gt 0 ] && status="FAILED"

    cat > "$gate_file" << EOF
{
    "gate_name": "coverage_regression_monitoring",
    "status": "$status",
    "timestamp": "$timestamp",
    "monitoring_results": {
        "coverage_violations": $violations,
        "coverage_regressions": $regressions,
        "monitored_scripts": ${#COVERAGE_SCRIPTS[@]}
    },
    "recommendations": [
        "Review individual coverage reports for detailed analysis",
        "Address coverage violations before merging",
        "Investigate and fix coverage regressions",
        "Consider adding more tests to improve coverage"
    ]
}
EOF

    echo -e "${BLUE}Quality gate created: $gate_file${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting Coverage Regression Monitoring System...${NC}"

    local total_violations=0
    local total_regressions=0
    local all_output=""

    # Run all coverage analyses and collect output
    for script in "${COVERAGE_SCRIPTS[@]}"; do
        echo "Running $script..."
        if output=$(run_coverage_analysis "$script" 2>&1); then
            echo "✅ $script PASSED"
        else
            local exit_code=$?
            echo "❌ $script FAILED (exit code: $exit_code)"
            ((total_violations += exit_code))
        fi
        all_output="${all_output}${output}\n---\n"
    done

    # Extract and save metrics
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local metrics
    metrics=$(extract_coverage_metrics "$all_output" "all")

    save_coverage_history "$timestamp" "$metrics"

    # Detect regressions
    local regression_analysis=""
    regression_analysis=$(detect_regressions "$metrics")
    total_regressions=$?

    # Generate monitoring report
    local report_file
    report_file=$(generate_monitoring_report "$total_violations" "$total_regressions" "$regression_analysis")

    # Create quality gate
    create_quality_gate "$total_violations" "$total_regressions"

    echo ""
    echo -e "${BLUE}Coverage Monitoring Summary:${NC}"
    echo "- Coverage Violations: $total_violations"
    echo "- Coverage Regressions: $total_regressions"

    if [ "$total_violations" -eq 0 ] && [ "$total_regressions" -eq 0 ]; then
        echo -e "${GREEN}🎉 All coverage targets met and no regressions detected!${NC}"
        exit 0
    else
        echo -e "${RED}⚠️  Coverage issues detected - review monitoring report${NC}"
        echo -e "${YELLOW}Report: $report_file${NC}"
        exit 1
    fi
}

# Run main function
main "$@"
