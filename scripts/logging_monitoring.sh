#!/bin/bash
set -euo pipefail

# CostPilot logging and monitoring system
# Provides structured logging, metrics collection, and operational visibility

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
LOG_DIR="${HOME}/.costpilot/logs"
METRICS_DIR="${HOME}/.costpilot/metrics"
LOG_FILE="${LOG_DIR}/costpilot.log"
METRICS_FILE="${METRICS_DIR}/metrics.json"

# Colors for console output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging levels
LEVEL_DEBUG=0
LEVEL_INFO=1
LEVEL_WARN=2
LEVEL_ERROR=3

# Current log level (default: INFO)
LOG_LEVEL=${LOG_LEVEL:-$LEVEL_INFO}

# Initialize logging system
init_logging() {
    mkdir -p "$LOG_DIR"
    mkdir -p "$METRICS_DIR"

    # Rotate old logs (keep last 10)
    if [[ -f "$LOG_FILE" ]]; then
        for i in {9..1}; do
            if [[ -f "${LOG_FILE}.${i}" ]]; then
                mv "${LOG_FILE}.${i}" "${LOG_FILE}.$((i+1))"
            fi
        done
        mv "$LOG_FILE" "${LOG_FILE}.1"
    fi

    # Initialize metrics file
    if [[ ! -f "$METRICS_FILE" ]]; then
        cat > "$METRICS_FILE" << 'EOF'
{
  "version": "1.0",
  "start_time": "",
  "metrics": {
    "commands_executed": 0,
    "errors_encountered": 0,
    "cost_calculations": 0,
    "policy_checks": 0,
    "slo_evaluations": 0,
    "audit_events": 0
  },
  "performance": {
    "avg_command_time": 0.0,
    "peak_memory_usage": 0,
    "total_runtime": 0
  },
  "errors": []
}
EOF
    fi

    # Set start time
    local start_time
    start_time="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    jq --arg start_time "$start_time" '.start_time = $start_time' "$METRICS_FILE" > "${METRICS_FILE}.tmp"
    mv "${METRICS_FILE}.tmp" "$METRICS_FILE"
}

# Log a message with structured format
log_message() {
    local level="$1"
    local level_num="$2"
    local message="$3"
    local component="${4:-main}"
    local extra="${5:-}"

    # Check if we should log this level
    if [[ $level_num -lt $LOG_LEVEL ]]; then
        return
    fi

    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    local pid=$$
    local hostname
    hostname="$(hostname 2>/dev/null || echo "unknown")"

    # Create structured log entry
    local log_entry
    log_entry="$(jq -n \
        --arg timestamp "$timestamp" \
        --arg level "$level" \
        --arg message "$message" \
        --arg component "$component" \
        --arg pid "$pid" \
        --arg hostname "$hostname" \
        --arg extra "$extra" \
        '{
            timestamp: $timestamp,
            level: $level,
            message: $message,
            component: $component,
            pid: $pid,
            hostname: $hostname
        } | if $extra != "" then . + {extra: $extra} else . end')"

    # Write to log file
    echo "$log_entry" >> "$LOG_FILE"

    # Also output to console with colors
    case "$level" in
        DEBUG) echo -e "${BLUE}[DEBUG]${NC} $message" >&2 ;;
        INFO)  echo -e "${GREEN}[INFO]${NC} $message" >&2 ;;
        WARN)  echo -e "${YELLOW}[WARN]${NC} $message" >&2 ;;
        ERROR) echo -e "${RED}[ERROR]${NC} $message" >&2 ;;
    esac
}

# Convenience logging functions
log_debug() {
    log_message "DEBUG" $LEVEL_DEBUG "$1" "${2:-main}" "${3:-}"
}

log_info() {
    log_message "INFO" $LEVEL_INFO "$1" "${2:-main}" "${3:-}"
}

log_warn() {
    log_message "WARN" $LEVEL_WARN "$1" "${2:-main}" "${3:-}"
}

log_error() {
    log_message "ERROR" $LEVEL_ERROR "$1" "${2:-main}" "${3:-}"

    # Also record error in metrics
    record_error "$1" "${2:-main}"
}

# Record metrics
record_metric() {
    local metric="$1"
    local value="${2:-1}"

    if [[ ! -f "$METRICS_FILE" ]]; then
        log_warn "Metrics file not found, skipping metric recording"
        return
    fi

    # Update metric using jq
    jq --arg metric "$metric" --argjson value "$value" \
        ".metrics[$metric] += $value" "$METRICS_FILE" > "${METRICS_FILE}.tmp" 2>/dev/null || {
        log_warn "Failed to update metrics for $metric"
        return
    }
    mv "${METRICS_FILE}.tmp" "$METRICS_FILE"
}

# Record error
record_error() {
    local message="$1"
    local component="${2:-main}"

    if [[ ! -f "$METRICS_FILE" ]]; then
        return
    fi

    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    local error_entry
    error_entry="$(jq -n \
        --arg timestamp "$timestamp" \
        --arg message "$message" \
        --arg component "$component" \
        '{
            timestamp: $timestamp,
            message: $message,
            component: $component
        }')"

    # Add error to array
    jq --argjson error "$error_entry" '.errors += [$error]' "$METRICS_FILE" > "${METRICS_FILE}.tmp"
    mv "${METRICS_FILE}.tmp" "$METRICS_FILE"

    # Increment error count
    record_metric "errors_encountered"
}

# Record command execution
record_command() {
    local command="$1"
    local duration="${2:-0}"

    record_metric "commands_executed"

    # Log command execution
    log_info "Command executed: $command" "cli" "{\"duration\": $duration}"

    # Update performance metrics
    if [[ $duration -gt 0 ]]; then
        update_performance_metrics "$duration"
    fi
}

# Update performance metrics
update_performance_metrics() {
    local duration="$1"

    if [[ ! -f "$METRICS_FILE" ]]; then
        return
    fi

    # Calculate new average (simple moving average)
    local current_avg
    current_avg="$(jq '.performance.avg_command_time' "$METRICS_FILE")"
    local command_count
    command_count="$(jq '.metrics.commands_executed' "$METRICS_FILE")"

    if [[ "$current_avg" == "0" ]]; then
        local new_avg="$duration"
    else
        local new_avg
        new_avg="$(echo "scale=2; ($current_avg * ($command_count - 1) + $duration) / $command_count" | bc 2>/dev/null || echo "$duration")"
    fi

    # Update metrics
    jq --argjson new_avg "$new_avg" '.performance.avg_command_time = $new_avg' "$METRICS_FILE" > "${METRICS_FILE}.tmp"
    mv "${METRICS_FILE}.tmp" "$METRICS_FILE"
}

# Show metrics summary
show_metrics() {
    if [[ ! -f "$METRICS_FILE" ]]; then
        echo "No metrics file found"
        return
    fi

    echo "CostPilot Metrics Summary"
    echo "========================="
    echo

    # Basic metrics
    local commands
    commands="$(jq '.metrics.commands_executed' "$METRICS_FILE")"
    local errors
    errors="$(jq '.metrics.errors_encountered' "$METRICS_FILE")"
    local cost_calcs
    cost_calcs="$(jq '.metrics.cost_calculations' "$METRICS_FILE")"

    echo "Usage Metrics:"
    echo "  Commands executed: $commands"
    echo "  Errors encountered: $errors"
    echo "  Cost calculations: $cost_calcs"
    echo

    # Performance metrics
    local avg_time
    avg_time="$(jq '.performance.avg_command_time' "$METRICS_FILE")"
    echo "Performance Metrics:"
    echo "  Average command time: ${avg_time}s"
    echo

    # Recent errors
    local error_count
    error_count="$(jq '.errors | length' "$METRICS_FILE")"
    if [[ $error_count -gt 0 ]]; then
        echo "Recent Errors:"
        jq -r '.errors[-5:][] | "  \(.timestamp): \(.message)"' "$METRICS_FILE" 2>/dev/null || echo "  (Could not parse errors)"
        echo
    fi
}

# Show log tail
show_logs() {
    local lines="${1:-50}"

    if [[ ! -f "$LOG_FILE" ]]; then
        echo "No log file found"
        return
    fi

    echo "Recent CostPilot Logs (last $lines lines)"
    echo "=========================================="
    tail -n "$lines" "$LOG_FILE" | jq -r '"\(.timestamp) [\(.level)] \(.component): \(.message)"' 2>/dev/null || tail -n "$lines" "$LOG_FILE"
}

# Clean old logs
clean_logs() {
    local days="${1:-30}"

    log_info "Cleaning logs older than $days days..."

    # Remove old log files
    find "$LOG_DIR" -name "costpilot.log.*" -mtime +"$days" -delete 2>/dev/null || true

    # Truncate metrics errors (keep last 100)
    if [[ -f "$METRICS_FILE" ]]; then
        jq 'if .errors then .errors |= .[-100:] else . end' "$METRICS_FILE" > "${METRICS_FILE}.tmp"
        mv "${METRICS_FILE}.tmp" "$METRICS_FILE"
    fi

    log_info "Log cleanup completed"
}

# Export logs for analysis
export_logs() {
    local output_file="${1:-costpilot_logs_$(date +%Y%m%d_%H%M%S).json}"

    if [[ ! -f "$LOG_FILE" ]]; then
        echo "No log file found to export"
        return
    fi

    echo "Exporting logs to $output_file..."

    # Combine all log entries into a JSON array
    echo '[' > "$output_file"
    local first=true
    while IFS= read -r line; do
        if [[ "$first" == true ]]; then
            first=false
        else
            echo ',' >> "$output_file"
        fi
        echo "$line" >> "$output_file"
    done < "$LOG_FILE"
    echo ']' >> "$output_file"

    log_info "Logs exported to $output_file"
}

# Main command handling
case "${1:-}" in
    init)
        init_logging
        log_info "Logging system initialized"
        ;;
    metrics)
        show_metrics
        ;;
    logs)
        show_logs "${2:-50}"
        ;;
    clean)
        clean_logs "${2:-30}"
        ;;
    export)
        export_logs "${2:-}"
        ;;
    *)
        echo "CostPilot Logging and Monitoring"
        echo
        echo "Usage: $0 <command> [options]"
        echo
        echo "Commands:"
        echo "  init              Initialize logging system"
        echo "  metrics           Show metrics summary"
        echo "  logs [lines]      Show recent logs (default: 50 lines)"
        echo "  clean [days]      Clean logs older than N days (default: 30)"
        echo "  export [file]     Export logs to JSON file"
        echo
        echo "Log Directory: $LOG_DIR"
        echo "Metrics Directory: $METRICS_DIR"
        ;;
esac</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/logging_monitoring.sh
