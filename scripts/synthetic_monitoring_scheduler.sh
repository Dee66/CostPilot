#!/bin/bash
# CostPilot Synthetic Monitoring Scheduler
# Runs synthetic monitoring checks at regular intervals

set -e

# Configuration
MONITORING_SCRIPT="./scripts/synthetic_monitoring.sh"
INTERVAL_MINUTES=${SYNTHETIC_INTERVAL:-15}  # Default 15 minutes
LOG_FILE="synthetic-monitoring/scheduler.log"
PID_FILE="synthetic-monitoring/scheduler.pid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to log messages
log() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"

    case $level in
        "ERROR")
            echo -e "${RED}[$timestamp] ERROR: $message${NC}"
            ;;
        "WARN")
            echo -e "${YELLOW}[$timestamp] WARN: $message${NC}"
            ;;
        "INFO")
            echo -e "${GREEN}[$timestamp] INFO: $message${NC}"
            ;;
        *)
            echo "[$timestamp] $level: $message"
            ;;
    esac
}

# Function to check if another instance is running
check_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            log "WARN" "Another scheduler instance is running (PID: $pid)"
            return 1
        else
            log "INFO" "Removing stale PID file"
            rm -f "$PID_FILE"
        fi
    fi
    return 0
}

# Function to cleanup on exit
cleanup() {
    log "INFO" "Shutting down synthetic monitoring scheduler"
    rm -f "$PID_FILE"
    exit 0
}

# Function to run monitoring cycle
run_monitoring_cycle() {
    log "INFO" "Starting synthetic monitoring cycle"

    local start_time=$(date +%s)

    if [ -x "$MONITORING_SCRIPT" ]; then
        if "$MONITORING_SCRIPT" >> "$LOG_FILE" 2>&1; then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log "INFO" "Monitoring cycle completed successfully in ${duration}s"
        else
            local exit_code=$?
            log "ERROR" "Monitoring cycle failed with exit code $exit_code"
        fi
    else
        log "ERROR" "Monitoring script not found or not executable: $MONITORING_SCRIPT"
    fi
}

# Function to display status
show_status() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "Synthetic monitoring scheduler is running (PID: $pid)"
            echo "Log file: $LOG_FILE"
            echo "Check interval: $INTERVAL_MINUTES minutes"
            return 0
        fi
    fi
    echo "Synthetic monitoring scheduler is not running"
    return 1
}

# Function to stop scheduler
stop_scheduler() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            log "INFO" "Stopping scheduler (PID: $pid)"
            kill "$pid"
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                log "WARN" "Force killing scheduler"
                kill -9 "$pid"
            fi
        fi
        rm -f "$PID_FILE"
        log "INFO" "Scheduler stopped"
    else
        echo "No scheduler PID file found"
    fi
}

# Main scheduler loop
run_scheduler() {
    # Check if already running
    if ! check_running; then
        exit 1
    fi

    # Create directories
    mkdir -p synthetic-monitoring

    # Write PID file
    echo $$ > "$PID_FILE"

    # Set up signal handlers
    trap cleanup SIGINT SIGTERM

    log "INFO" "Synthetic monitoring scheduler started"
    log "INFO" "Monitoring interval: $INTERVAL_MINUTES minutes"
    log "INFO" "PID: $$"

    # Initial run
    run_monitoring_cycle

    # Main loop
    while true; do
        local sleep_seconds=$((INTERVAL_MINUTES * 60))

        log "INFO" "Next monitoring cycle in $INTERVAL_MINUTES minutes"

        # Sleep in smaller chunks to allow for graceful shutdown
        local slept=0
        while [ $slept -lt $sleep_seconds ]; do
            sleep 60
            slept=$((slept + 60))

            # Check if we should stop (PID file removed)
            if [ ! -f "$PID_FILE" ]; then
                log "INFO" "PID file removed, shutting down"
                exit 0
            fi
        done

        run_monitoring_cycle
    done
}

# Command line interface
case "${1:-start}" in
    "start")
        if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
            echo "Scheduler is already running"
            exit 1
        fi
        run_scheduler
        ;;
    "stop")
        stop_scheduler
        ;;
    "status")
        show_status
        ;;
    "restart")
        stop_scheduler
        sleep 2
        run_scheduler
        ;;
    "once")
        run_monitoring_cycle
        ;;
    *)
        echo "Usage: $0 {start|stop|status|restart|once}"
        echo
        echo "Commands:"
        echo "  start   - Start the synthetic monitoring scheduler"
        echo "  stop    - Stop the synthetic monitoring scheduler"
        echo "  status  - Show scheduler status"
        echo "  restart - Restart the synthetic monitoring scheduler"
        echo "  once    - Run a single monitoring cycle"
        echo
        echo "Environment variables:"
        echo "  SYNTHETIC_INTERVAL - Monitoring interval in minutes (default: 15)"
        exit 1
        ;;
esac
