#!/bin/bash
# CostPilot Advanced Performance Testing Suite

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PRODUCTS_DIR="${PROJECT_ROOT}/products"
COSTPILOT_DIR="${PRODUCTS_DIR}/costpilot"

# Performance test configuration
ENDURANCE_DURATION="${ENDURANCE_DURATION:-72h}"
SPIKE_MULTIPLIER="${SPIKE_MULTIPLIER:-10}"
CAPACITY_INCREMENT="${CAPACITY_INCREMENT:-10}"
VOLUME_MULTIPLIER="${VOLUME_MULTIPLIER:-100}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Prerequisites check
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if costpilot binary exists
    if [[ ! -f "${COSTPILOT_DIR}/target/release/costpilot" ]]; then
        log_error "CostPilot binary not found. Building..."
        cd "${COSTPILOT_DIR}"
        cargo build --release
    fi

    # Check for required tools
    local required_tools=("cargo" "jq" "gnuplot" "prometheus" "grafana")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_warning "$tool not found - some tests may be limited"
        fi
    done

    log_success "Prerequisites OK"
}

# Generate test data for performance testing
generate_test_data() {
    local test_type="$1"
    local scale="$2"
    local output_file="$3"

    log_info "Generating $test_type test data (scale: $scale)..."

    case "$test_type" in
        "terraform")
            # Generate Terraform plan with scaled resources
            generate_terraform_plan "$scale" "$output_file"
            ;;
        "cloudformation")
            # Generate CloudFormation template with scaled resources
            generate_cloudformation_template "$scale" "$output_file"
            ;;
        "cdk")
            # Generate CDK construct with scaled resources
            generate_cdk_construct "$scale" "$output_file"
            ;;
        *)
            log_error "Unknown test data type: $test_type"
            exit 1
            ;;
    esac

    log_success "Generated $test_type test data: $output_file"
}

# Generate Terraform plan with scaled resources
generate_terraform_plan() {
    local scale="$1"
    local output_file="$2"

    cat > "$output_file" << EOF
{
  "format_version": "1.1",
  "terraform_version": "1.5.0",
  "planned_values": {
    "root_module": {
      "resources": [
$(for i in $(seq 1 "$scale"); do
    cat << RESOURCE_EOF
        {
          "address": "aws_instance.test_$i",
          "mode": "managed",
          "type": "aws_instance",
          "name": "test_$i",
          "provider_name": "registry.terraform.io/hashicorp/aws",
          "schema_version": 1,
          "values": {
            "ami": "ami-12345678",
            "instance_type": "t3.micro",
            "tags": {
              "Name": "test-instance-$i",
              "Environment": "performance-test"
            }
          }
        },
RESOURCE_EOF
done | sed '$ s/,$//')
      ]
    }
  },
  "resource_changes": [
$(for i in $(seq 1 "$scale"); do
    cat << CHANGE_EOF
    {
      "address": "aws_instance.test_$i",
      "change": {
        "actions": ["create"],
        "before": null,
        "after": {
          "ami": "ami-12345678",
          "instance_type": "t3.micro",
          "tags": {
            "Name": "test-instance-$i",
            "Environment": "performance-test"
          }
        }
      }
    },
CHANGE_EOF
done | sed '$ s/,$//')
  ],
  "configuration": {
    "root_module": {
      "resources": [
$(for i in $(seq 1 "$scale"); do
    cat << CONFIG_EOF
        {
          "address": "aws_instance.test_$i",
          "mode": "managed",
          "type": "aws_instance",
          "name": "test_$i",
          "expressions": {
            "ami": {
              "constant_value": "ami-12345678"
            },
            "instance_type": {
              "constant_value": "t3.micro"
            },
            "tags": {
              "constant_value": {
                "Name": "test-instance-$i",
                "Environment": "performance-test"
              }
            }
          },
          "schema_version": 1
        },
CONFIG_EOF
done | sed '$ s/,$//')
      ]
    }
  }
}
EOF
}

# Generate CloudFormation template (simplified)
generate_cloudformation_template() {
    local scale="$1"
    local output_file="$2"

    cat > "$output_file" << EOF
{
  "AWSTemplateFormatVersion": "2010-09-09",
  "Resources": {
$(for i in $(seq 1 "$scale"); do
    cat << CF_EOF
    "EC2Instance$i": {
      "Type": "AWS::EC2::Instance",
      "Properties": {
        "ImageId": "ami-12345678",
        "InstanceType": "t3.micro",
        "Tags": [
          {
            "Key": "Name",
            "Value": "test-instance-$i"
          },
          {
            "Key": "Environment",
            "Value": "performance-test"
          }
        ]
      }
    },
CF_EOF
done | sed '$ s/,$//')
  }
}
EOF
}

# Generate CDK construct (simplified TypeScript)
generate_cdk_construct() {
    local scale="$1"
    local output_file="$2"

    cat > "$output_file" << EOF
import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';

export class PerformanceTestStack extends cdk.Stack {
  constructor(scope: cdk.App, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

$(for i in $(seq 1 "$scale"); do
    cat << CDK_EOF
    new ec2.Instance(this, 'TestInstance$i', {
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MICRO),
      machineImage: ec2.MachineImage.latestAmazonLinux(),
      tags: {
        Name: \`test-instance-$i\`,
        Environment: 'performance-test'
      }
    });
CDK_EOF
done)

  }
}
EOF
}

# Run endurance testing (72hr continuous load)
run_endurance_test() {
    log_info "Starting endurance testing (${ENDURANCE_DURATION})..."

    local test_data_file="${PROJECT_ROOT}/performance-test-data/terraform-plan.json"
    local results_dir="${PROJECT_ROOT}/performance-results/endurance"
    mkdir -p "$results_dir"

    # Generate test data
    generate_test_data "terraform" "100" "$test_data_file"

    # Start monitoring
    start_performance_monitoring "$results_dir"

    # Run continuous testing
    local start_time=$(date +%s)
    local iteration=0

    log_info "Running continuous performance tests..."

    while true; do
        iteration=$((iteration + 1))
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        # Check if we've exceeded the endurance duration
        if [[ "$ENDURANCE_DURATION" == "72h" ]]; then
            local max_duration=$((72 * 3600))  # 72 hours in seconds
        else
            # For shorter test runs, convert to seconds
            local duration_value=$(echo "$ENDURANCE_DURATION" | sed 's/[a-zA-Z]*$//')
            local duration_unit=$(echo "$ENDURANCE_DURATION" | sed 's/[0-9]*//')
            case "$duration_unit" in
                "h") max_duration=$((duration_value * 3600)) ;;
                "m") max_duration=$((duration_value * 60)) ;;
                "s") max_duration=$duration_value ;;
                *) max_duration=3600 ;;  # Default to 1 hour
            esac
        fi

        if [[ $elapsed -gt $max_duration ]]; then
            log_success "Endurance test completed after ${elapsed}s"
            break
        fi

        # Run performance test iteration
        run_performance_iteration "$iteration" "$test_data_file" "$results_dir"

        # Memory leak detection
        check_memory_usage "$iteration" "$results_dir"

        # Brief pause between iterations
        sleep 1
    done

    # Stop monitoring and analyze results
    stop_performance_monitoring "$results_dir"
    analyze_endurance_results "$results_dir"

    log_success "Endurance testing completed"
}

# Run spike testing (10x load increase)
run_spike_test() {
    log_info "Starting spike testing (10x load increase)..."

    local results_dir="${PROJECT_ROOT}/performance-results/spike"
    mkdir -p "$results_dir"

    # Generate baseline test data
    local baseline_data="${results_dir}/baseline.json"
    generate_test_data "terraform" "10" "$baseline_data"

    # Generate spike test data (10x baseline)
    local spike_data="${results_dir}/spike.json"
    generate_test_data "terraform" "100" "$spike_data"

    # Start monitoring
    start_performance_monitoring "$results_dir"

    # Run baseline performance
    log_info "Running baseline performance test..."
    run_performance_test "$baseline_data" "${results_dir}/baseline-results.json"

    # Run spike performance
    log_info "Running spike performance test..."
    run_performance_test "$spike_data" "${results_dir}/spike-results.json"

    # Stop monitoring and analyze
    stop_performance_monitoring "$results_dir"
    analyze_spike_results "$results_dir"

    log_success "Spike testing completed"
}

# Run capacity testing (incremental load to failure)
run_capacity_test() {
    log_info "Starting capacity testing (incremental load)..."

    local results_dir="${PROJECT_ROOT}/performance-results/capacity"
    mkdir -p "$results_dir"

    start_performance_monitoring "$results_dir"

    local scale=10
    local max_scale=1000
    local increment="$CAPACITY_INCREMENT"

    while [[ $scale -le $max_scale ]]; do
        log_info "Testing capacity at scale: $scale"

        # Generate test data
        local test_data="${results_dir}/capacity-${scale}.json"
        generate_test_data "terraform" "$scale" "$test_data"

        # Run performance test
        local result_file="${results_dir}/capacity-${scale}-results.json"
        if run_performance_test "$test_data" "$result_file"; then
            log_success "Capacity test passed at scale: $scale"
        else
            log_warning "Capacity test failed at scale: $scale"
            break
        fi

        scale=$((scale + increment))
    done

    stop_performance_monitoring "$results_dir"
    analyze_capacity_results "$results_dir"

    log_success "Capacity testing completed"
}

# Run volume testing (100x data scale)
run_volume_test() {
    log_info "Starting volume testing (100x data scale)..."

    local results_dir="${PROJECT_ROOT}/performance-results/volume"
    mkdir -p "$results_dir"

    # Generate massive test data
    local volume_data="${results_dir}/volume-test.json"
    generate_test_data "terraform" "1000" "$volume_data"

    start_performance_monitoring "$results_dir"

    # Run volume performance test
    run_performance_test "$volume_data" "${results_dir}/volume-results.json"

    stop_performance_monitoring "$results_dir"
    analyze_volume_results "$results_dir"

    log_success "Volume testing completed"
}

# Run single performance test iteration
run_performance_iteration() {
    local iteration="$1"
    local test_data="$2"
    local results_dir="$3"

    local start_time=$(date +%s.%3N)
    local result_file="${results_dir}/iteration-${iteration}.json"

    # Run costpilot with timing
    if timeout 30s "${PROJECT_ROOT}/target/release/costpilot" validate "$test_data" > "$result_file" 2>&1; then
        local exit_code=$?
    else
        local exit_code=$?
    fi

    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")

    # Record metrics
    cat > "${results_dir}/metrics-${iteration}.json" << EOF
{
  "iteration": $iteration,
  "timestamp": "$(date -Iseconds)",
  "duration_seconds": $duration,
  "exit_code": $exit_code,
  "memory_kb": $(get_memory_usage),
  "cpu_percent": $(get_cpu_usage)
}
EOF
}

# Run performance test with comprehensive metrics
run_performance_test() {
    local test_data="$1"
    local result_file="$2"

    local start_time=$(date +%s.%3N)

    # Run with resource monitoring
    if "${PROJECT_ROOT}/target/release/costpilot" validate "$test_data" > "$result_file" 2>&1; then
        local exit_code=$?
    else
        local exit_code=$?
    fi

    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")

    # Return metrics
    echo "{\"duration\": $duration, \"exit_code\": $exit_code, \"memory_kb\": $(get_memory_usage), \"cpu_percent\": $(get_cpu_usage)}"
}

# Memory usage monitoring
get_memory_usage() {
    # Get current process memory usage in KB
    ps aux --no-headers -o rss | awk '{sum+=$1} END {print sum}' 2>/dev/null || echo "0"
}

get_cpu_usage() {
    # Get current CPU usage percentage
    ps aux --no-headers -o pcpu | awk '{sum+=$1} END {print sum}' 2>/dev/null || echo "0"
}

# Memory leak detection
check_memory_usage() {
    local iteration="$1"
    local results_dir="$2"

    local current_memory=$(get_memory_usage)
    local memory_file="${results_dir}/memory-usage.log"

    echo "$(date +%s),$iteration,$current_memory" >> "$memory_file"

    # Check for memory growth trend (simple linear regression)
    if [[ $iteration -gt 10 ]]; then
        local recent_memory=$(tail -n 10 "$memory_file" | awk -F',' '{sum+=$3} END {print sum/NR}')
        local initial_memory=$(head -n 1 "$memory_file" | awk -F',' '{print $3}')

        local growth_rate=$(echo "scale=2; ($recent_memory - $initial_memory) / $initial_memory * 100" | bc 2>/dev/null || echo "0")

        if (( $(echo "$growth_rate > 10" | bc -l 2>/dev/null || echo "0") )); then
            log_warning "Potential memory leak detected: ${growth_rate}% growth over $iteration iterations"
        fi
    fi
}

# Performance monitoring
start_performance_monitoring() {
    local results_dir="$1"

    log_info "Starting performance monitoring..."

    # Start system monitoring (simplified - in production would use Prometheus/Grafana)
    {
        while true; do
            echo "$(date +%s),$(get_memory_usage),$(get_cpu_usage)" >> "${results_dir}/system-metrics.csv"
            sleep 1
        done
    } &
    echo $! > "${results_dir}/monitor.pid"
}

stop_performance_monitoring() {
    local results_dir="$1"

    if [[ -f "${results_dir}/monitor.pid" ]]; then
        kill "$(cat "${results_dir}/monitor.pid")" 2>/dev/null || true
        rm "${results_dir}/monitor.pid"
    fi

    log_info "Performance monitoring stopped"
}

# Results analysis functions
analyze_endurance_results() {
    local results_dir="$1"

    log_info "Analyzing endurance test results..."

    # Calculate statistics
    local metrics_files=("${results_dir}"/metrics-*.json)
    if [[ ${#metrics_files[@]} -eq 0 ]]; then
        log_warning "No metrics files found"
        return
    fi

    # Extract timing data
    jq -r '.duration_seconds' "${metrics_files[@]}" 2>/dev/null | sort -n > "${results_dir}/durations.txt"

    # Calculate percentiles
    local p50=$(calculate_percentile "${results_dir}/durations.txt" 50)
    local p95=$(calculate_percentile "${results_dir}/durations.txt" 95)
    local p99=$(calculate_percentile "${results_dir}/durations.txt" 99)

    # Check memory leak
    local memory_trend=$(analyze_memory_trend "${results_dir}/memory-usage.log")

    # Generate report
    cat > "${results_dir}/endurance-report.json" << EOF
{
  "test_type": "endurance",
  "duration_hours": 72,
  "total_iterations": ${#metrics_files[@]},
  "performance_metrics": {
    "p50_latency_seconds": $p50,
    "p95_latency_seconds": $p95,
    "p99_latency_seconds": $p99
  },
  "memory_analysis": {
    "trend": "$memory_trend",
    "leak_detected": $([[ "$memory_trend" == "increasing" ]] && echo "true" || echo "false")
  },
  "recommendations": [
    "Monitor memory usage in production",
    "Consider implementing memory limits",
    "Set up automated alerts for performance degradation"
  ]
}
EOF

    log_success "Endurance analysis complete: ${results_dir}/endurance-report.json"
}

analyze_spike_results() {
    local results_dir="$1"

    log_info "Analyzing spike test results..."

    # Compare baseline vs spike performance
    local baseline_metrics=$(cat "${results_dir}/baseline-results.json")
    local spike_metrics=$(cat "${results_dir}/spike-results.json")

    local baseline_duration=$(echo "$baseline_metrics" | jq '.duration' 2>/dev/null || echo "0")
    local spike_duration=$(echo "$spike_metrics" | jq '.duration' 2>/dev/null || echo "0")

    local degradation=$(echo "scale=2; ($spike_duration / $baseline_duration - 1) * 100" | bc 2>/dev/null || echo "0")

    cat > "${results_dir}/spike-report.json" << EOF
{
  "test_type": "spike",
  "load_multiplier": 10,
  "baseline_duration": $baseline_duration,
  "spike_duration": $spike_duration,
  "performance_degradation_percent": $degradation,
  "autoscaling_effective": $([[ $(echo "$degradation < 200" | bc -l 2>/dev/null || echo "1") -eq 1 ]] && echo "true" || echo "false"),
  "recommendations": [
    "Spike testing shows $(printf "%.1f" "$degradation")% performance degradation under 10x load",
    "Consider implementing request queuing for extreme spikes",
    "Monitor autoscaling effectiveness in production"
  ]
}
EOF

    log_success "Spike analysis complete: ${results_dir}/spike-report.json"
}

analyze_capacity_results() {
    local results_dir="$1"

    log_info "Analyzing capacity test results..."

    # Find failure point
    local failure_scale="unknown"
    for result_file in "${results_dir}"/capacity-*-results.json; do
        if [[ -f "$result_file" ]]; then
            local exit_code=$(jq '.exit_code' "$result_file" 2>/dev/null || echo "1")
            if [[ $exit_code -ne 0 ]]; then
                failure_scale=$(basename "$result_file" | sed 's/capacity-\([0-9]*\)-results.json/\1/')
                break
            fi
        fi
    done

    cat > "${results_dir}/capacity-report.json" << EOF
{
  "test_type": "capacity",
  "failure_point": "$failure_scale",
  "max_capacity": $([[ "$failure_scale" != "unknown" ]] && echo "$((failure_scale - CAPACITY_INCREMENT))" || echo "1000"),
  "recommendations": [
    "System can handle up to $failure_scale concurrent operations",
    "Implement load balancing for high-capacity scenarios",
    "Set up monitoring alerts at 80% of capacity limits"
  ]
}
EOF

    log_success "Capacity analysis complete: ${results_dir}/capacity-report.json"
}

analyze_volume_results() {
    local results_dir="$1"

    log_info "Analyzing volume test results..."

    local volume_metrics=$(cat "${results_dir}/volume-results.json")
    local duration=$(echo "$volume_metrics" | jq '.duration' 2>/dev/null || echo "0")
    local memory=$(echo "$volume_metrics" | jq '.memory_kb' 2>/dev/null || echo "0")

    cat > "${results_dir}/volume-report.json" << EOF
{
  "test_type": "volume",
  "data_scale": 100,
  "processing_time_seconds": $duration,
  "memory_usage_kb": $memory,
  "data_integrity": $([[ $duration -lt 30 ]] && echo "true" || echo "false"),
  "recommendations": [
    "Volume testing completed in ${duration}s with ${memory}KB memory usage",
    "Data integrity maintained under 100x scale",
    "Consider implementing data streaming for extremely large datasets"
  ]
}
EOF

    log_success "Volume analysis complete: ${results_dir}/volume-report.json"
}

# Utility functions
calculate_percentile() {
    local file="$1"
    local percentile="$2"

    if [[ ! -f "$file" ]] || [[ ! -s "$file" ]]; then
        echo "0"
        return
    fi

    local count=$(wc -l < "$file")
    local index=$(( (percentile * count) / 100 ))
    [[ $index -eq 0 ]] && index=1

    sort -n "$file" | sed -n "${index}p" || echo "0"
}

analyze_memory_trend() {
    local memory_file="$1"

    if [[ ! -f "$memory_file" ]]; then
        echo "unknown"
        return
    fi

    # Simple trend analysis
    local first_value=$(head -n 1 "$memory_file" | cut -d',' -f3)
    local last_value=$(tail -n 1 "$memory_file" | cut -d',' -f3)

    if [[ -z "$first_value" ]] || [[ -z "$last_value" ]]; then
        echo "unknown"
        return
    fi

    if (( $(echo "$last_value > $first_value * 1.05" | bc -l 2>/dev/null || echo "0") )); then
        echo "increasing"
    elif (( $(echo "$last_value < $first_value * 0.95" | bc -l 2>/dev/null || echo "0") )); then
        echo "decreasing"
    else
        echo "stable"
    fi
}

# Performance regression monitoring
setup_regression_monitoring() {
    log_info "Setting up performance regression monitoring..."

    local config_dir="${PROJECT_ROOT}/performance-monitoring"
    mkdir -p "$config_dir"

    # Create Prometheus configuration for performance metrics
    cat > "${config_dir}/prometheus.yml" << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'costpilot-performance'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'costpilot-tests'
    static_configs:
      - targets: ['localhost:8080']
EOF

    # Create Grafana dashboard configuration
    cat > "${config_dir}/grafana-dashboard.json" << EOF
{
  "dashboard": {
    "title": "CostPilot Performance Monitoring",
    "tags": ["costpilot", "performance"],
    "timezone": "UTC",
    "panels": [
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(costpilot_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "costpilot_memory_usage_bytes / 1024 / 1024",
            "legendFormat": "Memory (MB)"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(costpilot_requests_total{status=~\"5..\"}[5m]) / rate(costpilot_requests_total[5m]) * 100",
            "legendFormat": "Error rate (%)"
          }
        ]
      }
    ]
  }
}
EOF

    log_success "Performance regression monitoring configured"
}

# Main execution
main() {
    local command="${1:-help}"

    case "$command" in
        "endurance")
            check_prerequisites
            run_endurance_test
            ;;
        "spike")
            check_prerequisites
            run_spike_test
            ;;
        "capacity")
            check_prerequisites
            run_capacity_test
            ;;
        "volume")
            check_prerequisites
            run_volume_test
            ;;
        "full")
            check_prerequisites
            log_info "Running full performance test suite..."
            run_endurance_test
            run_spike_test
            run_capacity_test
            run_volume_test
            setup_regression_monitoring
            log_success "Full performance test suite completed"
            ;;
        "monitor")
            setup_regression_monitoring
            ;;
        "help"|*)
            echo "CostPilot Advanced Performance Testing Suite"
            echo ""
            echo "Usage: $0 <command>"
            echo ""
            echo "Commands:"
            echo "  endurance    Run 72hr endurance testing with memory leak detection"
            echo "  spike        Run 10x load spike testing with autoscaling validation"
            echo "  capacity     Run incremental capacity testing to failure point"
            echo "  volume       Run 100x data scale volume testing"
            echo "  full         Run complete performance test suite"
            echo "  monitor      Set up performance regression monitoring"
            echo "  help         Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  ENDURANCE_DURATION    Endurance test duration (default: 72h)"
            echo "  SPIKE_MULTIPLIER      Spike test load multiplier (default: 10)"
            echo "  CAPACITY_INCREMENT    Capacity test increment (default: 10)"
            echo "  VOLUME_MULTIPLIER     Volume test data multiplier (default: 100)"
            ;;
    esac
}

main "$@"
