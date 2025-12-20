#!/bin/bash
# Quality KPIs Dashboard Script
# Generates comprehensive quality metrics dashboard for CostPilot
# Targets: defect density <0.1/KLOC, effectiveness >99%

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
OUTPUT_DIR="$PROJECT_ROOT/tests/kpi_reports"
mkdir -p "$OUTPUT_DIR"

# Safety warning
echo -e "${YELLOW}⚠️  SAFETY NOTICE: This dashboard analyzes code quality metrics only.${NC}"
echo -e "${YELLOW}⚠️  NO actual deployments or infrastructure changes are made.${NC}"
echo

# Function to calculate lines of code
calculate_loc() {
    local dir="$1"
    local exclude_patterns=(
        "*.min.js"
        "*.min.css"
        "target/"
        "node_modules/"
        ".git/"
        "cleanup/"
        "*.log"
        "*.tmp"
    )

    local exclude_args=""
    for pattern in "${exclude_patterns[@]}"; do
        exclude_args="$exclude_args --exclude='$pattern'"
    done

    # Use find and wc for cross-platform compatibility
    find "$dir" -type f \( -name "*.rs" -o -name "*.js" -o -name "*.ts" -o -name "*.py" -o -name "*.sh" -o -name "*.yml" -o -name "*.yaml" -o -name "*.json" -o -name "*.md" \) \
        $exclude_args -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0"
}

# Function to count defects (compilation errors, lint issues, test failures)
count_defects() {
    local defect_count=0

    # Check for Rust compilation errors
    if command -v cargo >/dev/null 2>&1; then
        echo "Checking Rust compilation..."
        if ! cargo check --quiet 2>/dev/null; then
            defect_count=$((defect_count + 1))
        fi
    fi

    # Check for lint issues (clippy)
    if command -v cargo >/dev/null 2>&1 && cargo clippy --version >/dev/null 2>&1; then
        echo "Checking Rust lint issues..."
        local clippy_output
        clippy_output=$(cargo clippy --quiet --message-format=short 2>&1 || true)
        local clippy_warnings=$(echo "$clippy_output" | grep -c "warning:" || true)
        defect_count=$((defect_count + clippy_warnings))
    fi

    # Check for test failures
    if command -v cargo >/dev/null 2>&1; then
        echo "Checking test results..."
        if ! cargo test --quiet 2>/dev/null; then
            defect_count=$((defect_count + 1))
        fi
    fi

    # Check for YAML/JSON syntax errors
    echo "Checking configuration syntax..."
    local config_files=$(find "$PROJECT_ROOT" -name "*.yml" -o -name "*.yaml" -o -name "*.json" | head -20)
    for file in $config_files; do
        if [[ $file == *.json ]]; then
            if ! python3 -m json.tool "$file" >/dev/null 2>&1; then
                defect_count=$((defect_count + 1))
            fi
        elif [[ $file == *.yml ]] || [[ $file == *.yaml ]]; then
            if command -v python3 >/dev/null 2>&1; then
                if ! python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null; then
                    defect_count=$((defect_count + 1))
                fi
            fi
        fi
    done

    echo $defect_count
}

# Function to calculate test effectiveness
calculate_test_effectiveness() {
    local total_tests=0
    local passing_tests=0

    # Count Rust tests
    if command -v cargo >/dev/null 2>&1; then
        echo "Analyzing Rust test coverage..."
        # Try to get test counts from cargo test output
        local test_output
        test_output=$(cargo test --quiet -- --nocapture 2>&1 || true)
        local test_count=$(echo "$test_output" | grep -c "test result:" || echo "0")
        if [ "$test_count" -gt 0 ]; then
            total_tests=$((total_tests + test_count))
            if echo "$test_output" | grep -q "test result: ok"; then
                passing_tests=$((passing_tests + test_count))
            fi
        fi
    fi

    # Count IaC validation tests
    if [ -f "$PROJECT_ROOT/tests/iac_test.sh" ]; then
        echo "Analyzing IaC validation tests..."
        total_tests=$((total_tests + 1))
        if bash "$PROJECT_ROOT/tests/iac_test.sh" all >/dev/null 2>&1; then
            passing_tests=$((passing_tests + 1))
        fi
    fi

    # Calculate effectiveness percentage
    if [ "$total_tests" -gt 0 ]; then
        echo "scale=2; ($passing_tests * 100) / $total_tests" | bc -l 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to generate KPI report
generate_kpi_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$OUTPUT_DIR/kpi_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "Generating Quality KPIs Dashboard..."
    echo "# CostPilot Quality KPIs Dashboard" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "" >> "$report_file"

    # Calculate metrics
    echo "Calculating lines of code..."
    local loc=$(calculate_loc "$PROJECT_ROOT")
    echo "Counting defects..."
    local defects=$(count_defects)
    echo "Calculating test effectiveness..."
    local effectiveness=$(calculate_test_effectiveness)

    # Calculate defect density
    local defect_density="0.00"
    if [ "$loc" -gt 0 ]; then
        defect_density=$(echo "scale=2; ($defects * 1000) / $loc" | bc -l 2>/dev/null || echo "0.00")
    fi

    # Display results
    echo -e "${BLUE}=== Quality KPIs Dashboard ===${NC}"
    echo -e "${BLUE}Lines of Code (KLOC):${NC} $(echo "scale=2; $loc / 1000" | bc -l 2>/dev/null || echo "0.00")"
    echo -e "${BLUE}Defects Found:${NC} $defects"
    echo -e "${BLUE}Defect Density (defects/KLOC):${NC} $defect_density"
    echo -e "${BLUE}Test Effectiveness (%):${NC} $effectiveness%"
    echo

    # Evaluate against targets
    local density_target=0.1
    local effectiveness_target=99.0

    if (( $(echo "$defect_density < $density_target" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${GREEN}✅ Defect density target met (< $density_target defects/KLOC)${NC}"
    else
        echo -e "${RED}❌ Defect density target not met (target: < $density_target defects/KLOC)${NC}"
    fi

    if (( $(echo "$effectiveness > $effectiveness_target" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${GREEN}✅ Test effectiveness target met (> $effectiveness_target%)${NC}"
    else
        echo -e "${RED}❌ Test effectiveness target not met (target: > $effectiveness_target%)${NC}"
    fi

    # Write to report file
    echo "## Quality Metrics" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Metric | Value | Target | Status |" >> "$report_file"
    echo "|--------|-------|--------|--------|" >> "$report_file"

    local density_status="❌ Not Met"
    if (( $(echo "$defect_density < $density_target" | bc -l 2>/dev/null || echo "0") )); then
        density_status="✅ Met"
    fi

    local effectiveness_status="❌ Not Met"
    if (( $(echo "$effectiveness > $effectiveness_target" | bc -l 2>/dev/null || echo "0") )); then
        effectiveness_status="✅ Met"
    fi

    echo "| Defect Density (defects/KLOC) | $defect_density | < $density_target | $density_status |" >> "$report_file"
    echo "| Test Effectiveness (%) | $effectiveness% | > $effectiveness_target% | $effectiveness_status |" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Detailed Analysis" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Lines of Code:** $loc" >> "$report_file"
    echo "- **Defects Found:** $defects" >> "$report_file"
    echo "- **Test Coverage Areas:** Rust unit tests, IaC validation, configuration syntax" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    if (( $(echo "$defect_density >= $density_target" | bc -l 2>/dev/null || echo "0") )); then
        echo "- Review and fix identified defects to reduce defect density" >> "$report_file"
        echo "- Consider additional code review processes" >> "$report_file"
    fi
    if (( $(echo "$effectiveness <= $effectiveness_target" | bc -l 2>/dev/null || echo "0") )); then
        echo "- Increase test coverage in under-tested areas" >> "$report_file"
        echo "- Add more integration and end-to-end tests" >> "$report_file"
    fi
    echo "- Regular KPI monitoring recommended for continuous improvement" >> "$report_file"

    echo -e "${GREEN}✅ KPI report generated: $report_file${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting Quality KPIs Dashboard generation...${NC}"
    generate_kpi_report
    echo -e "${GREEN}🎉 Quality KPIs analysis completed!${NC}"
}

# Run main function
main "$@"
