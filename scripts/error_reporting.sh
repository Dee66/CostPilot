#!/bin/bash
set -euo pipefail

# CostPilot error reporting and crash handling
# Provides graceful error handling, crash reporting, and user-friendly error messages

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
CRASH_DIR="${HOME}/.costpilot/crashes"
ERROR_LOG="${HOME}/.costpilot/logs/errors.log"
MAX_CRASH_REPORTS=10

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Initialize crash reporting
init_crash_reporting() {
    mkdir -p "$CRASH_DIR"

    # Clean old crash reports
    local crash_count
    crash_count="$(find "$CRASH_DIR" -name "crash_*.json" 2>/dev/null | wc -l || echo "0")"

    if [[ $crash_count -gt $MAX_CRASH_REPORTS ]]; then
        log_info "Cleaning old crash reports..."
        find "$CRASH_DIR" -name "crash_*.json" -type f | head -n "$((crash_count - MAX_CRASH_REPORTS))" | xargs rm -f
    fi
}

# Generate crash report
generate_crash_report() {
    local exit_code="$1"
    local command_line="${2:-unknown}"
    local error_message="${3:-unknown error}"

    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    local crash_id
    crash_id="$(date +%Y%m%d_%H%M%S)_${RANDOM}"

    local report_file="${CRASH_DIR}/crash_${crash_id}.json"

    # Collect system information
    local os_info
    os_info="$(uname -a 2>/dev/null || echo "unknown")"

    local costpilot_version
    costpilot_version="$(costpilot --version 2>/dev/null | head -1 || echo "unknown")"

    local rust_version
    rust_version="$(rustc --version 2>/dev/null || echo "unknown")"

    local memory_info
    memory_info="$(free -h 2>/dev/null || echo "unknown")"

    local disk_info
    disk_info="$(df -h "$HOME" 2>/dev/null | tail -1 || echo "unknown")"

    # Create crash report
    cat > "$report_file" << EOF
{
  "crash_id": "${crash_id}",
  "timestamp": "${timestamp}",
  "costpilot_version": "${costpilot_version}",
  "exit_code": ${exit_code},
  "command_line": "${command_line}",
  "error_message": "${error_message}",
  "system_info": {
    "os": "${os_info}",
    "rust_version": "${rust_version}",
    "memory": "${memory_info}",
    "disk": "${disk_info}"
  },
  "environment": {
    "shell": "${SHELL:-unknown}",
    "terminal": "${TERM:-unknown}",
    "user": "${USER:-unknown}",
    "home": "${HOME:-unknown}"
  },
  "stack_trace": []
}
EOF

    echo "$report_file"
}

# Report crash to user
report_crash() {
    local exit_code="$1"
    local command_line="$2"
    local error_message="$3"
    local crash_report="$4"

    echo
    echo -e "${RED}💥 Fatal Error: CostPilot encountered an unexpected error${NC}"
    echo
    echo "Exit Code: $exit_code"
    echo "Command: $command_line"
    echo "Error: $error_message"
    echo
    echo "A crash report has been saved to:"
    echo "  $crash_report"
    echo
    echo "This report contains system information and can help us improve CostPilot."
    echo "Please consider reporting this issue at:"
    echo "  https://github.com/guardsuite/costpilot/issues"
    echo
    echo "To include the crash report with your issue, attach the file above."
    echo
}

# Handle common errors with user-friendly messages
handle_common_errors() {
    local exit_code="$1"
    local command_line="$2"
    local error_output="$3"

    case $exit_code in
        2)
            # Policy violation
            echo -e "${YELLOW}⚠️  Policy Violation${NC}"
            echo
            echo "CostPilot blocked this change because it violates your cost policies."
            echo "This is normal behavior when cost limits would be exceeded."
            echo
            echo "To resolve this:"
            echo "1. Review the cost estimate above"
            echo "2. Consider optimizing your infrastructure changes"
            echo "3. Update your cost policies if needed"
            echo "4. Use exemptions for approved exceptions"
            echo
            return 0
            ;;
        3)
            # SLO burn
            echo -e "${RED}🔥 SLO Burn Alert${NC}"
            echo
            echo "CostPilot detected that this change would cause SLO budget exhaustion."
            echo "The projected costs exceed your defined service level objectives."
            echo
            echo "To resolve this:"
            echo "1. Review the burn rate analysis above"
            echo "2. Consider smaller, incremental changes"
            echo "3. Update your SLO budgets if needed"
            echo "4. Contact your platform team for approval"
            echo
            return 0
            ;;
        4)
            # Invalid input
            echo -e "${YELLOW}❌ Invalid Input${NC}"
            echo
            echo "CostPilot received invalid or malformed input."
            echo "Please check your command syntax and input files."
            echo
            echo "Common issues:"
            echo "• Invalid Terraform plan JSON"
            echo "• Malformed policy files"
            echo "• Incorrect command arguments"
            echo "• Missing required files"
            echo
            echo "Run 'costpilot --help' for usage information."
            echo
            return 0
            ;;
    esac

    return 1
}

# Main crash handler
handle_crash() {
    local exit_code="$1"
    local command_line="${2:-unknown}"
    local error_message="${3:-unknown error}"
    local error_output="${4:-}"

    # Initialize crash reporting
    init_crash_reporting

    # Try to handle common errors first
    if handle_common_errors "$exit_code" "$command_line" "$error_output"; then
        return
    fi

    # Generate crash report
    local crash_report
    crash_report="$(generate_crash_report "$exit_code" "$command_line" "$error_message")"

    # Report to user
    report_crash "$exit_code" "$command_line" "$error_message" "$crash_report"

    # Log to error log
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] CRASH: exit_code=$exit_code, command=$command_line, error=$error_message, report=$crash_report" >> "$ERROR_LOG"
}

# Show crash reports
show_crash_reports() {
    echo "CostPilot Crash Reports"
    echo "======================"
    echo

    if [[ ! -d "$CRASH_DIR" ]]; then
        echo "No crash reports found."
        return
    fi

    local reports
    mapfile -t reports < <(find "$CRASH_DIR" -name "crash_*.json" -type f | sort -r)

    if [[ ${#reports[@]} -eq 0 ]]; then
        echo "No crash reports found."
        return
    fi

    echo "Found ${#reports[@]} crash report(s):"
    echo

    for report in "${reports[@]}"; do
        local crash_id
        crash_id="$(basename "$report" .json | sed 's/crash_//')"
        local timestamp
        timestamp="$(jq -r '.timestamp' "$report" 2>/dev/null || echo "unknown")"
        local exit_code
        exit_code="$(jq -r '.exit_code' "$report" 2>/dev/null || echo "unknown")"
        local command
        command="$(jq -r '.command_line' "$report" 2>/dev/null || echo "unknown")"

        echo "📄 Crash ID: $crash_id"
        echo "   Timestamp: $timestamp"
        echo "   Exit Code: $exit_code"
        echo "   Command: $command"
        echo "   File: $report"
        echo
    done
}

# Clean crash reports
clean_crash_reports() {
    local days="${1:-30}"

    echo "Cleaning crash reports older than $days days..."

    if [[ -d "$CRASH_DIR" ]]; then
        find "$CRASH_DIR" -name "crash_*.json" -mtime +"$days" -delete 2>/dev/null || true
        echo "Cleanup completed."
    else
        echo "No crash directory found."
    fi
}

# Export crash reports
export_crash_reports() {
    local output_file="${1:-costpilot_crashes_$(date +%Y%m%d_%H%M%S).tar.gz}"

    if [[ ! -d "$CRASH_DIR" ]]; then
        echo "No crash reports to export."
        return
    fi

    local report_count
    report_count="$(find "$CRASH_DIR" -name "crash_*.json" | wc -l)"

    if [[ $report_count -eq 0 ]]; then
        echo "No crash reports to export."
        return
    fi

    echo "Exporting $report_count crash report(s) to $output_file..."
    tar -czf "$output_file" -C "$CRASH_DIR" .
    echo "Crash reports exported to $output_file"
}

# Main command handling
case "${1:-}" in
    handle)
        # Called by CostPilot on crash
        handle_crash "${2:-1}" "${3:-unknown}" "${4:-unknown error}" "${5:-}"
        ;;
    reports)
        show_crash_reports
        ;;
    clean)
        clean_crash_reports "${2:-30}"
        ;;
    export)
        export_crash_reports "${2:-}"
        ;;
    *)
        echo "CostPilot Error Reporting and Crash Handling"
        echo
        echo "Usage: $0 <command> [options]"
        echo
        echo "Commands:"
        echo "  handle <exit_code> <command> <error>  Handle a crash (called by CostPilot)"
        echo "  reports                                   Show all crash reports"
        echo "  clean [days]                             Clean reports older than N days (default: 30)"
        echo "  export [file]                            Export crash reports to tar.gz file"
        echo
        echo "Crash Directory: $CRASH_DIR"
        ;;
esac</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/error_reporting.sh
