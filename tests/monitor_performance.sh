#!/bin/bash
# Performance KPIs Monitoring Script
# Monitors execution time and flaky test rates for CostPilot
# Targets: execution <5min, flaky rate <0.1%

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
OUTPUT_DIR="$PROJECT_ROOT/tests/performance_reports"
HISTORY_FILE="$OUTPUT_DIR/performance_history.json"
mkdir -p "$OUTPUT_DIR"

# Safety warning
echo -e "${YELLOW}⚠️  SAFETY NOTICE: This monitoring analyzes performance metrics only.${NC}"
echo -e "${YELLOW}⚠️  NO actual deployments or infrastructure changes are made.${NC}"
echo

# Function to measure execution time
measure_execution_time() {
    local command="$1"
    local start_time=$(date +%s.%3N)

    if eval "$command"; then
        local end_time=$(date +%s.%3N)
        local duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
        echo "$duration"
    else
        echo "-1"  # Error indicator
    fi
}

# Function to run tests multiple times to detect flakiness
detect_flaky_tests() {
    local test_command="$1"
    local runs=${2:-3}  # Default 3 runs
    local total_runs=0
    local failed_runs=0
    local results=()

    echo "Running $runs iterations to detect flaky tests..."

    for i in $(seq 1 "$runs"); do
        echo "Run $i/$runs..."
        total_runs=$((total_runs + 1))

        if ! eval "$test_command" >/dev/null 2>&1; then
            failed_runs=$((failed_runs + 1))
            results+=("FAIL")
        else
            results+=("PASS")
        fi
    done

    # Calculate flaky rate (intermittent failures)
    # True flakiness is when tests sometimes pass and sometimes fail
    local pass_count=0
    local fail_count=0
    for result in "${results[@]}"; do
        if [ "$result" = "PASS" ]; then
            pass_count=$((pass_count + 1))
        else
            fail_count=$((fail_count + 1))
        fi
    done

    local flaky_rate="0.00"
    if [ "$total_runs" -gt 0 ]; then
        # If all runs pass or all runs fail, it's not flaky (it's consistent)
        if [ "$pass_count" -eq "$total_runs" ] || [ "$fail_count" -eq "$total_runs" ]; then
            flaky_rate="0.00"  # Not flaky, just consistently passing or failing
        else
            # Calculate rate of failures when there are mixed results
            flaky_rate=$(echo "scale=2; ($fail_count * 100) / $total_runs" | bc -l 2>/dev/null || echo "0.00")
        fi
    fi

    echo "$flaky_rate"
}

# Function to analyze test execution time
analyze_test_performance() {
    echo "Analyzing test execution performance..."

    # Measure full test suite execution time (ignore failures for timing)
    local test_start=$(date +%s)
    cargo test --quiet >/dev/null 2>&1 || true  # Ignore failures for timing
    local test_end=$(date +%s)
    local test_duration=$((test_end - test_start))
    echo "$test_duration"
}

# Function to check build performance
analyze_build_performance() {
    echo "Analyzing build performance..."

    # Clean and measure build time
    local build_start=$(date +%s)
    if cargo clean >/dev/null 2>&1 && cargo build --release --quiet 2>/dev/null; then
        local build_end=$(date +%s)
        local build_duration=$((build_end - build_start))
        echo "$build_duration"
    else
        echo "-1"  # Error
    fi
}

# Function to load historical data
load_history() {
    if [ -f "$HISTORY_FILE" ]; then
        cat "$HISTORY_FILE"
    else
        echo "{}"
    fi
}

# Function to save historical data
save_history() {
    local data="$1"
    echo "$data" > "$HISTORY_FILE"
}

# Function to update historical trends
update_history() {
    local timestamp="$1"
    local execution_time="$2"
    local flaky_rate="$3"
    local build_time="$4"

    local history=$(load_history)

    # Add new data point (keep last 10 entries)
    local new_entry="{\"timestamp\":\"$timestamp\",\"execution_time\":$execution_time,\"flaky_rate\":$flaky_rate,\"build_time\":$build_time}"

    # Simple JSON array management (in production, use jq)
    if [ "$history" = "{}" ]; then
        history="[$new_entry]"
    else
        # Remove last character (]) and add new entry
        history="${history%?},$new_entry]"
    fi

    save_history "$history"
}

# Function to analyze trends
analyze_trends() {
    local history="$1"

    if [ "$history" = "{}" ] || [ "$history" = "[]" ]; then
        echo "No historical data available for trend analysis"
        return
    fi

    echo "Performance Trends (last 10 runs):"

    # Simple trend analysis (in production, use proper JSON parsing)
    local execution_times=$(echo "$history" | grep -o '"execution_time":[^,}]*' | cut -d: -f2 | tr '\n' ' ')
    local flaky_rates=$(echo "$history" | grep -o '"flaky_rate":[^,}]*' | cut -d: -f2 | tr '\n' ' ')

    # Calculate averages (simplified)
    local exec_avg=$(echo "$execution_times" | awk '{sum=0; n=0; for(i=1;i<=NF;i++){sum+=$i; n++} if(n>0) print sum/n; else print 0}')
    local flaky_avg=$(echo "$flaky_rates" | awk '{sum=0; n=0; for(i=1;i<=NF;i++){sum+=$i; n++} if(n>0) print sum/n; else print 0}')

    echo "Average execution time: ${exec_avg}s"
    echo "Average flaky rate: ${flaky_avg}%"
}

# Function to generate performance report
generate_performance_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$OUTPUT_DIR/performance_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "Generating Performance KPIs Report..."
    echo "# CostPilot Performance KPIs Report" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "" >> "$report_file"

    # Measure current performance
    echo "Measuring test execution time..."
    local execution_time=$(analyze_test_performance)

    echo "Measuring build time..."
    local build_time=$(analyze_build_performance)

    echo "Detecting flaky tests..."
    local flaky_rate=$(detect_flaky_tests "cargo test --quiet" 5)  # 5 runs for better detection

    # Update history
    update_history "$timestamp" "$execution_time" "$flaky_rate" "$build_time"

    # Display results
    echo -e "${BLUE}=== Performance KPIs Report ===${NC}"
    echo -e "${BLUE}Test Execution Time:${NC} ${execution_time}s"
    echo -e "${BLUE}Build Time:${NC} ${build_time}s"
    echo -e "${BLUE}Flaky Test Rate:${NC} ${flaky_rate}%"
    echo

    # Evaluate against targets
    local execution_target=300  # 5 minutes in seconds
    local flaky_target=0.1      # 0.1%

    if [ "$execution_time" -ne -1 ] && [ "$execution_time" -lt "$execution_target" ]; then
        echo -e "${GREEN}✅ Execution time target met (< ${execution_target}s)${NC}"
    else
        echo -e "${RED}❌ Execution time target not met (target: < ${execution_target}s)${NC}"
    fi

    if (( $(echo "$flaky_rate < $flaky_target" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${GREEN}✅ Flaky rate target met (< ${flaky_target}%)${NC}"
    else
        echo -e "${RED}❌ Flaky rate target not met (target: < ${flaky_target}%)${NC}"
    fi

    # Write to report file
    echo "## Performance Metrics" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Metric | Value | Target | Status |" >> "$report_file"
    echo "|--------|-------|--------|--------|" >> "$report_file"

    local execution_status="❌ Not Met"
    if [ "$execution_time" -ne -1 ] && [ "$execution_time" -lt "$execution_target" ]; then
        execution_status="✅ Met"
    fi

    local flaky_status="❌ Not Met"
    if (( $(echo "$flaky_rate < $flaky_target" | bc -l 2>/dev/null || echo "0") )); then
        flaky_status="✅ Met"
    fi

    echo "| Test Execution Time (s) | $execution_time | < $execution_target | $execution_status |" >> "$report_file"
    echo "| Build Time (s) | $build_time | < 300 | N/A |" >> "$report_file"
    echo "| Flaky Test Rate (%) | $flaky_rate | < $flaky_target | $flaky_status |" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Trend Analysis" >> "$report_file"
    echo "" >> "$report_file"
    local history=$(load_history)
    analyze_trends "$history" >> "$report_file"
    echo "" >> "$report_file"

    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    if [ "$execution_time" -ge "$execution_target" ]; then
        echo "- Consider parallelizing test execution" >> "$report_file"
        echo "- Review slow-running tests for optimization opportunities" >> "$report_file"
        echo "- Consider test sharding for CI/CD pipelines" >> "$report_file"
    fi
    if (( $(echo "$flaky_rate >= $flaky_target" | bc -l 2>/dev/null || echo "0") )); then
        echo "- Investigate and fix flaky tests" >> "$report_file"
        echo "- Review test isolation and cleanup procedures" >> "$report_file"
        echo "- Consider retry mechanisms for known intermittent failures" >> "$report_file"
    fi
    echo "- Regular performance monitoring recommended for continuous optimization" >> "$report_file"

    echo -e "${GREEN}✅ Performance report generated: $report_file${NC}"

    # Analyze trends in console
    echo
    echo -e "${BLUE}=== Performance Trends ===${NC}"
    analyze_trends "$history"
}

# Main execution
main() {
    echo -e "${BLUE}Starting Performance KPIs monitoring...${NC}"
    generate_performance_report
    echo -e "${GREEN}🎉 Performance KPIs analysis completed!${NC}"
}

# Run main function
main "$@"
