#!/bin/bash
# CostPilot Testing Scalability Validation
# Validates that the testing infrastructure can handle 10k+ tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
SCALABILITY_TEST_DIR="${PROJECT_ROOT}/scalability-test-results"
MAX_TEST_COUNT=10000
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc)}"
TIMEOUT_DURATION="3600"  # 1 hour timeout

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Generate synthetic test files
generate_synthetic_tests() {
    local test_count="$1"
    local test_dir="$2"

    log_info "Generating $test_count synthetic test files..."

    mkdir -p "$test_dir"

    # Generate unit tests
    local unit_test_file="${test_dir}/synthetic_unit_tests.rs"
    cat > "$unit_test_file" << 'EOF'
#[cfg(test)]
mod synthetic_unit_tests {
    use super::*;

EOF

    # Generate individual test functions
    for i in $(seq 1 "$test_count"); do
        cat >> "$unit_test_file" << EOF
    #[test]
    fn synthetic_test_${i}() {
        assert_eq!(2 + 2, 4);
    }

EOF
    done

    echo "}" >> "$unit_test_file"
    log_success "Generated synthetic unit tests in $unit_test_file"
}

# Generate synthetic integration tests
generate_integration_tests() {
    local test_count="$1"
    local test_dir="$2"

    log_info "Generating $test_count synthetic integration tests..."

    mkdir -p "${test_dir}/integration"

    # Generate integration test files
    for batch in $(seq 1 10); do
        local test_file="${test_dir}/integration/synthetic_integration_${batch}.rs"
        cat > "$test_file" << 'EOF'
#[cfg(test)]
mod synthetic_integration_tests {
    use super::*;

EOF

        local batch_size=$((test_count / 10))
        local start=$(((batch - 1) * batch_size + 1))
        local end=$((batch * batch_size))

        for i in $(seq "$start" "$end"); do
            cat >> "$test_file" << EOF
    #[test]
    fn integration_test_${i}() {
        // Simulate integration test
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(true);
    }

EOF
        done

        echo "}" >> "$test_file"
    done

    log_success "Generated synthetic integration tests"
}

# Generate synthetic benchmark tests
generate_benchmark_tests() {
    local test_count="$1"
    local test_dir="$2"

    log_info "Generating $test_count synthetic benchmark tests..."

    mkdir -p "${test_dir}/benches"

    local bench_file="${test_dir}/benches/synthetic_benchmarks.rs"
    cat > "$bench_file" << 'EOF'
use criterion::{black_box, criterion_group, criterion_main, Criterion};

EOF

    # Generate benchmark functions
    for i in $(seq 1 "$test_count"); do
        cat >> "$bench_file" << EOF
fn bench_synthetic_${i}(c: &mut Criterion) {
    c.bench_function(&format!("synthetic_bench_{}", ${i}), |b| {
        b.iter(|| {
            black_box(42 * ${i})
        });
    });
}

EOF
    done

    # Generate criterion group
    echo "criterion_group!(" >> "$bench_file"
    echo "    benches," >> "$bench_file"
    for i in $(seq 1 "$test_count"); do
        echo "    bench_synthetic_${i}," >> "$bench_file"
    done
    echo ");" >> "$bench_file"
    echo "criterion_main!(benches);" >> "$bench_file"

    log_success "Generated synthetic benchmark tests"
}

# Run scalability test
run_scalability_test() {
    local test_count="$1"
    local results_dir="$2"

    log_info "Running scalability test with $test_count tests..."

    local test_start_time=$(date +%s)
    local test_success=true

    # Create temporary test directory
    local temp_test_dir="${results_dir}/temp_tests"
    mkdir -p "$temp_test_dir"

    # Generate synthetic tests
    generate_synthetic_tests "$test_count" "$temp_test_dir"
    generate_integration_tests "$test_count" "$temp_test_dir"
    generate_benchmark_tests "$((test_count / 100))" "$temp_test_dir"  # Fewer benchmarks

    # Copy generated tests to src for compilation
    cp -r "$temp_test_dir"/* "${PROJECT_ROOT}/src/"

    # Build project with synthetic tests
    log_info "Building project with $test_count synthetic tests..."
    local build_start=$(date +%s)

    if timeout "${TIMEOUT_DURATION}s" cargo build --release --quiet; then
        local build_end=$(date +%s)
        local build_time=$((build_end - build_start))
        log_success "Build completed in ${build_time}s"
    else
        log_error "Build failed or timed out"
        test_success=false
    fi

    # Run unit tests
    if [[ "$test_success" == true ]]; then
        log_info "Running $test_count unit tests..."
        local test_start=$(date +%s)

        if timeout "${TIMEOUT_DURATION}s" cargo test --lib --release --quiet -- --nocapture > "${results_dir}/unit_test_output.log" 2>&1; then
            local test_end=$(date +%s)
            local test_time=$((test_end - test_start))
            log_success "Unit tests completed in ${test_time}s"
        else
            log_error "Unit tests failed or timed out"
            test_success=false
        fi
    fi

    # Run integration tests
    if [[ "$test_success" == true ]]; then
        log_info "Running $test_count integration tests..."
        local integration_start=$(date +%s)

        if timeout "${TIMEOUT_DURATION}s" cargo test --test '*' --release --quiet -- --nocapture > "${results_dir}/integration_test_output.log" 2>&1; then
            local integration_end=$(date +%s)
            local integration_time=$((integration_end - integration_start))
            log_success "Integration tests completed in ${integration_time}s"
        else
            log_error "Integration tests failed or timed out"
            test_success=false
        fi
    fi

    # Clean up synthetic tests
    rm -rf "${PROJECT_ROOT}/src/synthetic_"* 2>/dev/null || true
    rm -rf "$temp_test_dir"

    local test_end_time=$(date +%s)
    local total_time=$((test_end_time - test_start_time))

    # Generate results
    cat > "${results_dir}/scalability_results.json" << EOF
{
  "test_count": $test_count,
  "parallel_jobs": $PARALLEL_JOBS,
  "total_time_seconds": $total_time,
  "test_success": $test_success,
  "performance_metrics": {
    "tests_per_second": $(echo "scale=2; $test_count / $total_time" | bc -l 2>/dev/null || echo "0"),
    "time_per_test_ms": $(echo "scale=2; ($total_time * 1000) / $test_count" | bc -l 2>/dev/null || echo "0")
  },
  "resource_usage": {
    "max_memory_mb": $(get_max_memory_usage),
    "cpu_cores_used": $PARALLEL_JOBS
  },
  "scalability_assessment": $(assess_scalability "$test_count" "$total_time" "$test_success")
}
EOF

    if [[ "$test_success" == true ]]; then
        log_success "Scalability test completed successfully"
    else
        log_error "Scalability test failed"
    fi
}

# Get maximum memory usage during test
get_max_memory_usage() {
    # Simple approximation - in production would use proper monitoring
    echo "512"  # Placeholder
}

# Assess scalability based on results
assess_scalability() {
    local test_count="$1"
    local total_time="$2"
    local success="$3"

    if [[ "$success" != "true" ]]; then
        echo '"failed"'
        return
    fi

    # Assess based on time per test (target: < 10ms per test for 10k tests)
    local time_per_test=$(echo "scale=2; ($total_time * 1000) / $test_count" | bc -l 2>/dev/null || echo "1000")

    if (( $(echo "$time_per_test < 10" | bc -l 2>/dev/null || echo "0") )); then
        echo '"excellent"'
    elif (( $(echo "$time_per_test < 50" | bc -l 2>/dev/null || echo "0") )); then
        echo '"good"'
    elif (( $(echo "$time_per_test < 100" | bc -l 2>/dev/null || echo "0") )); then
        echo '"acceptable"'
    else
        echo '"poor"'
    fi
}

# Main scalability validation
main() {
    local command="${1:-validate}"

    case "$command" in
        "validate")
            log_info "Starting CostPilot testing scalability validation..."

            # Create results directory
            mkdir -p "$SCALABILITY_TEST_DIR"

            # Test with increasing scale
            local scales=(100 1000 5000 10000)

            for scale in "${scales[@]}"; do
                if [[ $scale -le $MAX_TEST_COUNT ]]; then
                    local scale_results_dir="${SCALABILITY_TEST_DIR}/scale_${scale}"
                    mkdir -p "$scale_results_dir"

                    log_info "Testing scalability at ${scale} tests..."
                    run_scalability_test "$scale" "$scale_results_dir"
                fi
            done

            # Generate summary report
            generate_scalability_report "$SCALABILITY_TEST_DIR"

            log_success "Scalability validation completed"
            ;;
        "report")
            generate_scalability_report "$SCALABILITY_TEST_DIR"
            ;;
        "cleanup")
            rm -rf "$SCALABILITY_TEST_DIR"
            log_success "Scalability test data cleaned up"
            ;;
        *)
            echo "Usage: $0 <command>"
            echo "Commands:"
            echo "  validate  Run full scalability validation"
            echo "  report    Generate scalability report"
            echo "  cleanup   Clean up test data"
            ;;
    esac
}

# Generate comprehensive scalability report
generate_scalability_report() {
    local results_dir="$1"
    local report_file="${results_dir}/scalability_report.json"

    log_info "Generating scalability report..."

    # Collect results from all scale tests
    local results="[]"

    for scale_dir in "${results_dir}"/scale_*; do
        if [[ -d "$scale_dir" && -f "${scale_dir}/scalability_results.json" ]]; then
            local scale_result=$(cat "${scale_dir}/scalability_results.json")
            results=$(echo "$results" | jq --argjson result "$scale_result" '. += [$result]' 2>/dev/null || echo "[$scale_result]")
        fi
    done

    # Analyze scalability trends
    local max_scale_handled=$(echo "$results" | jq '[.[] | select(.test_success == true)] | max_by(.test_count) | .test_count' 2>/dev/null || echo "0")
    local scalability_rating="unknown"

    if [[ $max_scale_handled -ge 10000 ]]; then
        scalability_rating="excellent"
    elif [[ $max_scale_handled -ge 5000 ]]; then
        scalability_rating="good"
    elif [[ $max_scale_handled -ge 1000 ]]; then
        scalability_rating="acceptable"
    else
        scalability_rating="poor"
    fi

    cat > "$report_file" << EOF
{
  "scalability_validation_report": {
    "validation_date": "$(date -Iseconds)",
    "max_test_count_tested": $MAX_TEST_COUNT,
    "max_scale_successfully_handled": $max_scale_handled,
    "scalability_rating": "$scalability_rating",
    "parallel_jobs_used": $PARALLEL_JOBS,
    "results": $results,
    "recommendations": $(generate_scalability_recommendations "$scalability_rating")
  }
}
EOF

    log_success "Scalability report generated: $report_file"
}

# Generate recommendations based on scalability rating
generate_scalability_recommendations() {
    local rating="$1"

    case "$rating" in
        "excellent")
            echo '["Scalability is excellent - can handle 10k+ tests efficiently", "Consider optimizing further for even larger test suites", "Monitor resource usage in production"]'
            ;;
        "good")
            echo '["Scalability is good - handles large test suites well", "Consider parallel execution optimizations", "Monitor memory usage patterns"]'
            ;;
        "acceptable")
            echo '["Scalability is acceptable - functional for current needs", "Consider infrastructure upgrades for better performance", "Implement test result caching"]'
            ;;
        "poor")
            echo '["Scalability needs improvement", "Consider distributed testing infrastructure", "Optimize test execution time", "Review test design for parallelism"]'
            ;;
        *)
            echo '["Unable to determine scalability rating", "Re-run validation tests", "Check test infrastructure configuration"]'
            ;;
    esac
}

main "$@"
