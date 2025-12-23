#!/bin/bash
# CostPilot Metrics Monitoring Script
# Automated code quality metrics analysis

set -e

echo "📊 Running CostPilot Metrics Monitoring"
echo "======================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
RESULTS_DIR="code-review-results"
mkdir -p "$RESULTS_DIR"

# Historical data file
METRICS_HISTORY="metrics-history.json"

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

# Function to calculate cyclomatic complexity
calculate_cyclomatic_complexity() {
    print_status "INFO" "Calculating cyclomatic complexity..."

    local total_complexity=0
    local function_count=0
    local high_complexity_count=0

    # Find all Rust source files
    find src -name "*.rs" -type f | while read -r file; do
        print_status "INFO" "Analyzing $file"

        # Count functions/methods
        local func_count
        func_count=$(grep -c "^\s*fn " "$file" || true)
        function_count=$((function_count + func_count))

        # Count control flow statements (rough approximation)
        local if_count
        local match_count
        local loop_count
        local while_count
        local for_count

        if_count=$(grep -c "\s*if " "$file" || true)
        match_count=$(grep -c "\s*match " "$file" || true)
        loop_count=$(grep -c "\s*loop " "$file" || true)
        while_count=$(grep -c "\s*while " "$file" || true)
        for_count=$(grep -c "\s*for " "$file" || true)

        # Calculate complexity per function (simplified)
        local file_complexity=$((if_count + match_count + loop_count + while_count + for_count + func_count))

        # Check for high complexity functions
        if [ $file_complexity -gt 20 ]; then
            print_status "WARN" "$file has high complexity: $file_complexity"
            high_complexity_count=$((high_complexity_count + 1))
        fi

        total_complexity=$((total_complexity + file_complexity))

        # Save per-file metrics
        echo "{\"file\": \"$file\", \"complexity\": $file_complexity, \"functions\": $func_count}" >> "$RESULTS_DIR/cyclomatic-complexity.json"
    done

    # Calculate average complexity
    local avg_complexity=0
    if [ $function_count -gt 0 ]; then
        avg_complexity=$((total_complexity / function_count))
    fi

    echo "{\"total_complexity\": $total_complexity, \"function_count\": $function_count, \"average_complexity\": $avg_complexity, \"high_complexity_files\": $high_complexity_count, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/cyclomatic-summary.json"

    if [ $avg_complexity -gt 15 ]; then
        print_status "FAIL" "Average cyclomatic complexity too high: $avg_complexity"
    elif [ $avg_complexity -gt 10 ]; then
        print_status "WARN" "Average cyclomatic complexity elevated: $avg_complexity"
    else
        print_status "PASS" "Average cyclomatic complexity acceptable: $avg_complexity"
    fi
}

# Function to calculate maintainability index
calculate_maintainability_index() {
    print_status "INFO" "Calculating maintainability index..."

    # Get code metrics using tokei if available
    if command -v tokei &> /dev/null; then
        print_status "INFO" "Using tokei for code metrics..."
        tokei --output json > "$RESULTS_DIR/tokei-metrics.json" 2>/dev/null || true
    fi

    # Calculate basic metrics
    local total_lines=0
    local code_lines=0
    local comment_lines=0

    find src -name "*.rs" -type f | while read -r file; do
        local lines
        local code
        local comments

        lines=$(wc -l < "$file")
        code=$(grep -v "^\s*//" "$file" | grep -v "^\s*$" | wc -l)
        comments=$(grep "^\s*//" "$file" | wc -l)

        total_lines=$((total_lines + lines))
        code_lines=$((code_lines + code))
        comment_lines=$((comment_lines + comments))
    done

    # Calculate comment ratio
    local comment_ratio=0
    if [ $code_lines -gt 0 ]; then
        comment_ratio=$((comment_lines * 100 / code_lines))
    fi

    # Simple maintainability index calculation (simplified formula)
    # MI = 171 - 5.2 * ln(Halstead Volume) - 0.23 * CC - 16.2 * ln(LOC) + 50 * sin(sqrt(2.4 * perCM))
    # Using simplified version based on available metrics
    local mi=100

    # Reduce MI based on low comment ratio
    if [ $comment_ratio -lt 10 ]; then
        mi=$((mi - 20))
    elif [ $comment_ratio -lt 20 ]; then
        mi=$((mi - 10))
    fi

    # Reduce MI based on high cyclomatic complexity
    if [ -f "$RESULTS_DIR/cyclomatic-summary.json" ]; then
        local avg_cc
        avg_cc=$(jq '.average_complexity' "$RESULTS_DIR/cyclomatic-summary.json" 2>/dev/null || echo "0")
        if [ "$avg_cc" -gt 15 ]; then
            mi=$((mi - 15))
        elif [ "$avg_cc" -gt 10 ]; then
            mi=$((mi - 5))
        fi
    fi

    # Reduce MI based on large files
    if [ $total_lines -gt 10000 ]; then
        mi=$((mi - 10))
    fi

    echo "{\"maintainability_index\": $mi, \"total_lines\": $total_lines, \"code_lines\": $code_lines, \"comment_lines\": $comment_lines, \"comment_ratio_percent\": $comment_ratio, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/maintainability-index.json"

    if [ $mi -lt 50 ]; then
        print_status "FAIL" "Maintainability index is poor: $mi/100"
    elif [ $mi -lt 70 ]; then
        print_status "WARN" "Maintainability index needs improvement: $mi/100"
    else
        print_status "PASS" "Maintainability index acceptable: $mi/100"
    fi
}

# Function to estimate technical debt
estimate_technical_debt() {
    print_status "INFO" "Estimating technical debt..."

    local debt_hours=0
    local debt_reasons=""

    # Check for TODO/FIXME comments
    local todo_count
    todo_count=$(find src -name "*.rs" -exec grep -l "TODO\|FIXME\|XXX" {} \; | wc -l)
    if [ "$todo_count" -gt 0 ]; then
        debt_hours=$((debt_hours + todo_count * 2)) # 2 hours per TODO
        debt_reasons="$debt_reasons, $todo_count TODO/FIXME items"
    fi

    # Check for high complexity functions
    if [ -f "$RESULTS_DIR/cyclomatic-summary.json" ]; then
        local high_complexity
        high_complexity=$(jq '.high_complexity_files' "$RESULTS_DIR/cyclomatic-summary.json" 2>/dev/null || echo "0")
        if [ "$high_complexity" -gt 0 ]; then
            debt_hours=$((debt_hours + high_complexity * 4)) # 4 hours per high complexity function
            debt_reasons="$debt_reasons, $high_complexity high complexity functions"
        fi
    fi

    # Check for low maintainability
    if [ -f "$RESULTS_DIR/maintainability-index.json" ]; then
        local mi
        mi=$(jq '.maintainability_index' "$RESULTS_DIR/maintainability-index.json" 2>/dev/null || echo "100")
        if [ "$mi" -lt 70 ]; then
            local debt_from_mi=$(((70 - mi) / 2))
            debt_hours=$((debt_hours + debt_from_mi))
            debt_reasons="$debt_reasons, low maintainability index"
        fi
    fi

    # Check for missing tests
    local test_coverage=0
    if [ -f "tarpaulin-report.json" ]; then
        test_coverage=$(jq '.coverage_percentage' tarpaulin-report.json 2>/dev/null || echo "0")
    fi

    if [ "$test_coverage" -lt 80 ]; then
        local coverage_debt=$(((80 - test_coverage) / 5))
        debt_hours=$((debt_hours + coverage_debt))
        debt_reasons="$debt_reasons, low test coverage"
    fi

    # Remove leading comma and space
    debt_reasons=$(echo "$debt_reasons" | sed 's/^, //')

    echo "{\"technical_debt_hours\": $debt_hours, \"reasons\": \"$debt_reasons\", \"test_coverage_percent\": $test_coverage, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/technical-debt.json"

    if [ $debt_hours -gt 40 ]; then
        print_status "FAIL" "High technical debt: ${debt_hours} hours"
    elif [ $debt_hours -gt 20 ]; then
        print_status "WARN" "Moderate technical debt: ${debt_hours} hours"
    else
        print_status "PASS" "Technical debt acceptable: ${debt_hours} hours"
    fi
}

# Function to analyze code coverage trends
analyze_coverage_trends() {
    print_status "INFO" "Analyzing code coverage trends..."

    # Run tarpaulin for coverage if available
    if command -v cargo-tarpaulin &> /dev/null; then
        print_status "INFO" "Running tarpaulin for code coverage..."
        cargo tarpaulin --out Json --output-dir "$RESULTS_DIR" > /dev/null 2>&1 || true

        if [ -f "$RESULTS_DIR/tarpaulin-report.json" ]; then
            local coverage_percent
            coverage_percent=$(jq '.coverage_percentage' "$RESULTS_DIR/tarpaulin-report.json" 2>/dev/null || echo "0")

            echo "{\"coverage_percentage\": $coverage_percent, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/coverage-current.json"

            if [ "$coverage_percent" -lt 70 ]; then
                print_status "FAIL" "Code coverage too low: ${coverage_percent}%"
            elif [ "$coverage_percent" -lt 80 ]; then
                print_status "WARN" "Code coverage below target: ${coverage_percent}%"
            else
                print_status "PASS" "Code coverage acceptable: ${coverage_percent}%"
            fi
        else
            print_status "WARN" "Could not generate coverage report"
        fi
    else
        print_status "WARN" "cargo-tarpaulin not available - skipping coverage analysis"
    fi
}

# Function to update metrics history
update_metrics_history() {
    print_status "INFO" "Updating metrics history..."

    # Create current metrics summary
    local current_metrics="{}"

    # Read all metric files and merge them
    local all_metrics="{}"

    if [ -f "$RESULTS_DIR/cyclomatic-summary.json" ]; then
        local cc_data
        cc_data=$(cat "$RESULTS_DIR/cyclomatic-summary.json")
        all_metrics=$(jq ".cyclomatic = $cc_data" <<< "$all_metrics")
    fi

    if [ -f "$RESULTS_DIR/maintainability-index.json" ]; then
        local mi_data
        mi_data=$(cat "$RESULTS_DIR/maintainability-index.json")
        all_metrics=$(jq ".maintainability = $mi_data" <<< "$all_metrics")
    fi

    if [ -f "$RESULTS_DIR/technical-debt.json" ]; then
        local td_data
        td_data=$(cat "$RESULTS_DIR/technical-debt.json")
        all_metrics=$(jq ".technical_debt = $td_data" <<< "$all_metrics")
    fi

    if [ -f "$RESULTS_DIR/coverage-current.json" ]; then
        local cov_data
        cov_data=$(cat "$RESULTS_DIR/coverage-current.json")
        all_metrics=$(jq ".coverage = $cov_data" <<< "$all_metrics")
    fi

    # Add commit info
    all_metrics=$(jq ".commit = \"$(git rev-parse HEAD 2>/dev/null || echo 'unknown')\" | .branch = \"$(git branch --show-current 2>/dev/null || echo 'unknown')\" | .timestamp = \"$(date -Iseconds)\"" <<< "$all_metrics")

    # Update history file
    if [ -f "$METRICS_HISTORY" ]; then
        # Append to existing history
        jq ". + [$all_metrics]" "$METRICS_HISTORY" > "${METRICS_HISTORY}.tmp" 2>/dev/null || echo "[$all_metrics]" > "${METRICS_HISTORY}.tmp"
        mv "${METRICS_HISTORY}.tmp" "$METRICS_HISTORY" 2>/dev/null || true
    else
        # Create new history
        echo "[$all_metrics]" > "$METRICS_HISTORY"
    fi

    print_status "PASS" "Metrics history updated"
}

# Function to generate metrics report
generate_metrics_report() {
    print_status "INFO" "Generating metrics monitoring report..."

    local total_issues=0

    # Count issues from various checks
    if [ -f "$RESULTS_DIR/cyclomatic-summary.json" ]; then
        local high_complexity
        high_complexity=$(jq '.high_complexity_files' "$RESULTS_DIR/cyclomatic-summary.json" 2>/dev/null || echo "0")
        total_issues=$((total_issues + high_complexity))
    fi

    if [ -f "$RESULTS_DIR/technical-debt.json" ]; then
        local debt_hours
        debt_hours=$(jq '.technical_debt_hours' "$RESULTS_DIR/technical-debt.json" 2>/dev/null || echo "0")
        if [ $debt_hours -gt 20 ]; then
            total_issues=$((total_issues + 1))
        fi
    fi

    # Create JSON report
    cat > "$RESULTS_DIR/metrics-monitoring-report.json" << EOF
{
  "metrics_monitoring": {
    "timestamp": "$(date -Iseconds)",
    "commit": "$(git rev-parse HEAD)",
    "total_issues": $total_issues,
    "metrics_tracked": [
      "cyclomatic_complexity",
      "maintainability_index",
      "technical_debt",
      "code_coverage"
    ],
    "recommendations": [
      "Refactor high complexity functions",
      "Improve code documentation",
      "Address technical debt items",
      "Increase test coverage",
      "Monitor metrics trends over time"
    ]
  }
}
EOF

    if [ $total_issues -eq 0 ]; then
        print_status "PASS" "Metrics monitoring completed with no major issues"
    else
        print_status "WARN" "Metrics monitoring completed with $total_issues issues to address"
    fi
}

# Main execution
main() {
    # Ensure results directory exists
    mkdir -p "$RESULTS_DIR"

    calculate_cyclomatic_complexity
    calculate_maintainability_index
    estimate_technical_debt
    analyze_coverage_trends
    update_metrics_history
    generate_metrics_report

    print_status "INFO" "Metrics monitoring completed. Results saved to $RESULTS_DIR/"
}

main "$@"
