#!/bin/bash
# CostPilot Local Synthetic Monitoring Runner
# For development and testing of synthetic monitoring

set -e

echo "🏥 CostPilot Local Synthetic Monitoring"
echo "======================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MONITORING_SCRIPT="$PROJECT_ROOT/scripts/synthetic_monitoring.sh"
SCHEDULER_SCRIPT="$PROJECT_ROOT/scripts/synthetic_monitoring_scheduler.sh"
ALERTS_SCRIPT="$PROJECT_ROOT/scripts/synthetic_monitoring_alerts.sh"

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}Checking prerequisites...${NC}"

    local missing_deps=()

    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi

    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo -e "${RED}Missing dependencies: ${missing_deps[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi

    echo -e "${GREEN}Prerequisites OK${NC}"
}

# Function to build the project
build_project() {
    echo -e "${BLUE}Building CostPilot...${NC}"

    cd "$PROJECT_ROOT"

    if ! cargo build --release; then
        echo -e "${RED}Build failed${NC}"
        exit 1
    fi

    echo -e "${GREEN}Build successful${NC}"
}

# Function to run single monitoring cycle
run_once() {
    echo -e "${BLUE}Running single monitoring cycle...${NC}"

    cd "$PROJECT_ROOT"

    if [ ! -x "$MONITORING_SCRIPT" ]; then
        chmod +x "$MONITORING_SCRIPT"
    fi

    if "$MONITORING_SCRIPT"; then
        echo -e "${GREEN}Monitoring cycle completed${NC}"
    else
        echo -e "${RED}Monitoring cycle failed${NC}"
        exit 1
    fi
}

# Function to run with scheduler
run_scheduled() {
    local interval=${1:-5}  # Default 5 minutes for testing

    echo -e "${BLUE}Starting scheduled monitoring (interval: ${interval} minutes)...${NC}"
    echo "Press Ctrl+C to stop"

    cd "$PROJECT_ROOT"

    if [ ! -x "$SCHEDULER_SCRIPT" ]; then
        chmod +x "$SCHEDULER_SCRIPT"
    fi

    # Set environment variable for testing
    SYNTHETIC_INTERVAL="$interval" "$SCHEDULER_SCRIPT" start
}

# Function to check alerts
check_alerts() {
    echo -e "${BLUE}Checking alerts...${NC}"

    cd "$PROJECT_ROOT"

    if [ ! -x "$ALERTS_SCRIPT" ]; then
        chmod +x "$ALERTS_SCRIPT"
    fi

    "$ALERTS_SCRIPT" status
}

# Function to test alerts
test_alerts() {
    echo -e "${BLUE}Testing alert system...${NC}"

    cd "$PROJECT_ROOT"

    if [ ! -x "$ALERTS_SCRIPT" ]; then
        chmod +x "$ALERTS_SCRIPT"
    fi

    "$ALERTS_SCRIPT" test
}

# Function to show monitoring results
show_results() {
    echo -e "${BLUE}Recent monitoring results:${NC}"

    cd "$PROJECT_ROOT"

    if [ -d "synthetic-monitoring" ]; then
        echo "Results directory: synthetic-monitoring/"
        echo

        # Show latest results
        local latest_results
        latest_results=$(find synthetic-monitoring -name "health-check-*.json" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2- || echo "")

        if [ -n "$latest_results" ]; then
            echo "Latest results file: $latest_results"
            echo

            # Parse and display results
            if command -v jq >/dev/null 2>&1; then
                echo "Results summary:"
                jq -r '.[] | select(.status != "INFO") | "\(.status): \(.check) - \(.message)"' "$latest_results" 2>/dev/null || echo "Could not parse results"
            else
                echo "Install jq for better result formatting"
                grep '"status"' "$latest_results" | head -10
            fi
        else
            echo "No results files found"
        fi

        echo
        echo "All result files:"
        ls -la synthetic-monitoring/health-check-*.json 2>/dev/null || echo "None found"
    else
        echo "No synthetic-monitoring directory found"
    fi
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}Cleaning up monitoring data...${NC}"

    cd "$PROJECT_ROOT"

    # Stop any running scheduler
    if [ -f "synthetic-monitoring/scheduler.pid" ]; then
        "$SCHEDULER_SCRIPT" stop 2>/dev/null || true
    fi

    # Remove monitoring data
    rm -rf synthetic-monitoring/

    echo -e "${GREEN}Cleanup completed${NC}"
}

# Function to show help
show_help() {
    echo "CostPilot Local Synthetic Monitoring Runner"
    echo
    echo "Usage: $0 [COMMAND]"
    echo
    echo "Commands:"
    echo "  build     - Build the CostPilot binary"
    echo "  once      - Run a single monitoring cycle"
    echo "  scheduled - Run scheduled monitoring (default 5min intervals)"
    echo "  alerts    - Check alert status"
    echo "  test      - Test the alert system"
    echo "  results   - Show recent monitoring results"
    echo "  cleanup   - Clean up monitoring data and stop scheduler"
    echo "  help      - Show this help message"
    echo
    echo "Examples:"
    echo "  $0 build && $0 once          # Build and run once"
    echo "  $0 scheduled 10              # Run every 10 minutes"
    echo "  $0 results                   # View latest results"
    echo
    echo "Environment Variables:"
    echo "  SYNTHETIC_INTERVAL - Monitoring interval in minutes (default: 15)"
    echo "  ALERT_EMAIL        - Email address for alerts"
    echo "  ALERT_SLACK_WEBHOOK - Slack webhook URL"
}

# Main function
main() {
    local command=${1:-"help"}

    case $command in
        "build")
            check_prerequisites
            build_project
            ;;
        "once")
            check_prerequisites
            build_project
            run_once
            ;;
        "scheduled")
            local interval=${2:-5}
            check_prerequisites
            build_project
            run_scheduled "$interval"
            ;;
        "alerts")
            check_alerts
            ;;
        "test")
            test_alerts
            ;;
        "results")
            show_results
            ;;
        "cleanup")
            cleanup
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

main "$@"
