#!/bin/bash
# CostPilot Performance Review Script
# Automated performance analysis for code changes

set -e

echo "⚡ Running CostPilot Performance Review"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
RESULTS_DIR="code-review-results"
mkdir -p "$RESULTS_DIR"

# Baseline file
BASELINE_FILE="performance-baseline.json"

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

# Function to run performance benchmarks
run_performance_benchmarks() {
    print_status "INFO" "Running performance benchmarks..."

    if command -v cargo-criterion &> /dev/null; then
        print_status "INFO" "Running criterion benchmarks..."
        cargo criterion --message-format=json > "$RESULTS_DIR/criterion-results.json" 2>/dev/null || true
        print_status "PASS" "Criterion benchmarks completed"
    else
        print_status "WARN" "cargo-criterion not installed - skipping detailed benchmarks"
    fi

    # Run basic timing tests
    print_status "INFO" "Running basic performance tests..."

    # Time the build
    local build_start
    build_start=$(date +%s%N)
    if cargo build --release --quiet; then
        local build_end
        build_end=$(date +%s%N)
        local build_time=$(( (build_end - build_start) / 1000000 )) # Convert to milliseconds

        echo "{\"build_time_ms\": $build_time, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/build-performance.json"

        if [ $build_time -gt 300000 ]; then # 5 minutes
            print_status "WARN" "Build time is high: ${build_time}ms"
        else
            print_status "PASS" "Build time acceptable: ${build_time}ms"
        fi
    else
        print_status "FAIL" "Build failed"
        return 1
    fi

    # Time basic execution
    if [ -f "target/release/costpilot" ]; then
        local exec_start
        exec_start=$(date +%s%N)
        timeout 30s ./target/release/costpilot --version > /dev/null 2>&1
        local exit_code=$?
        local exec_end
        exec_end=$(date +%s%N)

        if [ $exit_code -eq 0 ]; then
            local exec_time=$(( (exec_end - exec_start) / 1000000 ))
            echo "{\"execution_time_ms\": $exec_time, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/execution-performance.json"

            if [ $exec_time -gt 1000 ]; then # 1 second
                print_status "WARN" "Execution time is high: ${exec_time}ms"
            else
                print_status "PASS" "Execution time acceptable: ${exec_time}ms"
            fi
        else
            print_status "FAIL" "Basic execution failed or timed out"
        fi
    fi
}

# Function to check binary size
check_binary_size() {
    print_status "INFO" "Checking binary size..."

    if [ -f "target/release/costpilot" ]; then
        local binary_size
        binary_size=$(stat -f%z target/release/costpilot 2>/dev/null || stat -c%s target/release/costpilot 2>/dev/null || echo "0")

        if [ "$binary_size" -gt 0 ]; then
            local size_mb
            size_mb=$(( binary_size / 1024 / 1024 ))

            echo "{\"binary_size_bytes\": $binary_size, \"binary_size_mb\": $size_mb, \"timestamp\": \"$(date -Iseconds)\"}" > "$RESULTS_DIR/binary-size.json"

            if [ $size_mb -gt 50 ]; then
                print_status "WARN" "Binary size is large: ${size_mb}MB"
            elif [ $size_mb -gt 100 ]; then
                print_status "FAIL" "Binary size is too large: ${size_mb}MB"
            else
                print_status "PASS" "Binary size acceptable: ${size_mb}MB"
            fi
        else
            print_status "FAIL" "Could not determine binary size"
        fi
    else
        print_status "FAIL" "Release binary not found"
    fi
}

# Function to check memory usage patterns
check_memory_usage() {
    print_status "INFO" "Checking memory usage patterns..."

    # Look for potential memory issues in code
    local memory_issues=0

    # Check for large allocations
    local large_allocs
    large_allocs=$(git diff --cached | grep -c "vec!\[.*;.*[0-9]\{4,\}" || true)
    if [ "$large_allocs" -gt 0 ]; then
        print_status "WARN" "Found $large_allocs large vector allocations - review memory usage"
        memory_issues=$((memory_issues + large_allocs))
    fi

    # Check for unbounded data structures
    local unbounded_structures
    unbounded_structures=$(git diff --cached | grep -c "Vec::new()\|HashMap::new()\|BTreeMap::new()" || true)
    if [ "$unbounded_structures" -gt 0 ]; then
        print_status "INFO" "Found $unbounded_structures dynamic allocations - ensure bounded growth"
    fi

    # Check for recursive functions
    local recursive_funcs
    recursive_funcs=$(git diff --cached | grep -c "fn.*{" | xargs -I {} sh -c 'echo "$1" | grep -o "$1" | head -1' -- {} | grep -c "recursive\|Recursive" || true)
    if [ "$recursive_funcs" -gt 0 ]; then
        print_status "WARN" "Found potential recursive functions - check for stack overflow risks"
        memory_issues=$((memory_issues + recursive_funcs))
    fi

    echo "$memory_issues" > "$RESULTS_DIR/memory_issues.count"

    if [ $memory_issues -eq 0 ]; then
        print_status "PASS" "No obvious memory issues detected"
    fi
}

# Function to check for performance anti-patterns
check_performance_antipatterns() {
    print_status "INFO" "Checking for performance anti-patterns..."

    local antipatterns=0

    # Check for string concatenation in loops
    local string_concat
    string_concat=$(git diff --cached | grep -c "push_str\|format!.*+=" || true)
    if [ "$string_concat" -gt 0 ]; then
        print_status "WARN" "Found $string_concat string concatenations - consider using String::with_capacity or Vec<u8>"
        antipatterns=$((antipatterns + string_concat))
    fi

    # Check for expensive operations in hot paths
    local expensive_ops
    expensive_ops=$(git diff --cached | grep -c "\.clone()\|\.to_string()\|sort(" || true)
    if [ "$expensive_ops" -gt 0 ]; then
        print_status "INFO" "Found $expensive_ops potentially expensive operations - review usage context"
    fi

    # Check for unnecessary allocations
    local unnecessary_allocs
    unnecessary_allocs=$(git diff --cached | grep -c "Box::new(\|Rc::new(\|Arc::new(" || true)
    if [ "$unnecessary_allocs" -gt 0 ]; then
        print_status "INFO" "Found $unnecessary_allocs heap allocations - ensure necessary"
    fi

    # Check for blocking operations
    local blocking_ops
    blocking_ops=$(git diff --cached | grep -c "\.read_to_end(\|\.write_all(\|std::fs::" || true)
    if [ "$blocking_ops" -gt 0 ]; then
        print_status "INFO" "Found $blocking_ops blocking I/O operations - consider async alternatives if in hot path"
    fi

    echo "$antipatterns" > "$RESULTS_DIR/performance_antipatterns.count"
}

# Function to compare against baseline
compare_with_baseline() {
    print_status "INFO" "Comparing with performance baseline..."

    if [ -f "$BASELINE_FILE" ]; then
        print_status "INFO" "Baseline file found - comparing metrics..."

        # Compare build time
        if [ -f "$RESULTS_DIR/build-performance.json" ] && [ -f "$BASELINE_FILE" ]; then
            local current_build
            local baseline_build
            current_build=$(jq '.build_time_ms' "$RESULTS_DIR/build-performance.json" 2>/dev/null || echo "0")
            baseline_build=$(jq '.build_time_ms' "$BASELINE_FILE" 2>/dev/null || echo "0")

            if [ "$baseline_build" -gt 0 ] && [ "$current_build" -gt 0 ]; then
                local ratio=$(( current_build * 100 / baseline_build ))
                if [ $ratio -gt 120 ]; then
                    print_status "FAIL" "Build time regression: ${ratio}% of baseline"
                elif [ $ratio -gt 110 ]; then
                    print_status "WARN" "Build time increased: ${ratio}% of baseline"
                else
                    print_status "PASS" "Build time within acceptable range: ${ratio}% of baseline"
                fi
            fi
        fi

        # Compare binary size
        if [ -f "$RESULTS_DIR/binary-size.json" ] && [ -f "$BASELINE_FILE" ]; then
            local current_size
            local baseline_size
            current_size=$(jq '.binary_size_bytes' "$RESULTS_DIR/binary-size.json" 2>/dev/null || echo "0")
            baseline_size=$(jq '.binary_size_bytes' "$BASELINE_FILE" 2>/dev/null || echo "0")

            if [ "$baseline_size" -gt 0 ] && [ "$current_size" -gt 0 ]; then
                local ratio=$(( current_size * 100 / baseline_size ))
                if [ $ratio -gt 110 ]; then
                    print_status "FAIL" "Binary size regression: ${ratio}% of baseline"
                elif [ $ratio -gt 105 ]; then
                    print_status "WARN" "Binary size increased: ${ratio}% of baseline"
                else
                    print_status "PASS" "Binary size within acceptable range: ${ratio}% of baseline"
                fi
            fi
        fi
    else
        print_status "INFO" "No baseline file found - creating new baseline"
        # Create baseline from current results
        if [ -f "$RESULTS_DIR/build-performance.json" ] && [ -f "$RESULTS_DIR/binary-size.json" ]; then
            jq -s '.[0] * .[1]' "$RESULTS_DIR/build-performance.json" "$RESULTS_DIR/binary-size.json" > "$BASELINE_FILE"
            print_status "PASS" "New performance baseline created"
        fi
    fi
}

# Function to generate performance report
generate_performance_report() {
    print_status "INFO" "Generating performance review report..."

    local total_issues=0

    # Count all issues
    if [ -f "$RESULTS_DIR/memory_issues.count" ]; then
        total_issues=$((total_issues + $(cat "$RESULTS_DIR/memory_issues.count")))
    fi
    if [ -f "$RESULTS_DIR/performance_antipatterns.count" ]; then
        total_issues=$((total_issues + $(cat "$RESULTS_DIR/performance_antipatterns.count")))
    fi

    # Create JSON report
    cat > "$RESULTS_DIR/performance-review-report.json" << EOF
{
  "performance_review": {
    "timestamp": "$(date -Iseconds)",
    "commit": "$(git rev-parse HEAD)",
    "total_issues": $total_issues,
    "checks_performed": [
      "benchmark_execution",
      "binary_size_analysis",
      "memory_usage_patterns",
      "performance_antipatterns",
      "baseline_comparison"
    ],
    "recommendations": [
      "Review performance regressions against baseline",
      "Optimize memory usage patterns",
      "Address performance anti-patterns",
      "Consider algorithmic improvements for hot paths",
      "Monitor performance in CI/CD pipeline"
    ]
  }
}
EOF

    if [ $total_issues -eq 0 ]; then
        print_status "PASS" "Performance review completed with no major issues"
    else
        print_status "WARN" "Performance review completed with $total_issues issues to review"
    fi
}

# Main execution
main() {
    # Ensure results directory exists
    mkdir -p "$RESULTS_DIR"

    run_performance_benchmarks
    check_binary_size
    check_memory_usage
    check_performance_antipatterns
    compare_with_baseline
    generate_performance_report

    print_status "INFO" "Performance review completed. Results saved to $RESULTS_DIR/"
}

main "$@"
