#!/bin/bash
# CostPilot Synthetic Monitoring Alert System
# Handles notifications and alerts for monitoring failures

set -e

# Configuration
MONITORING_DIR="synthetic-monitoring"
ALERT_CONFIG_FILE="configs/alerting.yml"
LOG_FILE="$MONITORING_DIR/alerts.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to log alerts
log_alert() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# Function to send email alert
send_email_alert() {
    local subject=$1
    local body=$2
    local recipient=${ALERT_EMAIL:-"alerts@costpilot.dev"}

    if command -v mail >/dev/null 2>&1; then
        echo "$body" | mail -s "$subject" "$recipient"
        log_alert "INFO" "Email alert sent to $recipient"
    elif command -v sendmail >/dev/null 2>&1; then
        {
            echo "To: $recipient"
            echo "Subject: $subject"
            echo ""
            echo "$body"
        } | sendmail -t
        log_alert "INFO" "Email alert sent via sendmail to $recipient"
    else
        log_alert "WARN" "Email not configured or mail tools not available"
    fi
}

# Function to send webhook alert
send_webhook_alert() {
    local payload=$1
    local webhook_url=$ALERT_WEBHOOK_URL

    if [ -n "$webhook_url" ]; then
        if command -v curl >/dev/null 2>&1; then
            echo "$payload" | curl -X POST -H 'Content-type: application/json' --data @- "$webhook_url" >/dev/null 2>&1
            log_alert "INFO" "Webhook alert sent to $webhook_url"
        else
            log_alert "WARN" "curl not available for webhook notifications"
        fi
    fi
}

# Function to check for recent alerts (avoid spam)
check_recent_alerts() {
    local alert_type=$1
    local time_window_minutes=${ALERT_THROTTLE_MINUTES:-60}

    if [ -f "$LOG_FILE" ]; then
        local recent_count
        recent_count=$(grep -c "$alert_type" "$LOG_FILE" | tail -n "$time_window_minutes" | wc -l)

        if [ "$recent_count" -gt 0 ]; then
            log_alert "INFO" "Alert throttled: $alert_type (recent alerts: $recent_count)"
            return 1
        fi
    fi

    return 0
}

# Function to analyze monitoring results and send alerts
analyze_and_alert() {
    local results_file=$1

    if [ ! -f "$results_file" ]; then
        log_alert "ERROR" "Results file not found: $results_file"
        return 1
    fi

    # Extract failures and warnings
    local failures
    local warnings
    local total_checks

    failures=$(grep -c '"status":"FAIL"' "$results_file" || echo "0")
    warnings=$(grep -c '"status":"WARN"' "$results_file" || echo "0")
    total_checks=$(grep -c '"check"' "$results_file" || echo "0")

    log_alert "INFO" "Analyzing results: $total_checks checks, $failures failures, $warnings warnings"

    # Alert on failures
    if [ "$failures" -gt 0 ]; then
        if check_recent_alerts "FAILURE"; then
            local subject="🚨 CostPilot Synthetic Monitoring: $failures Failures Detected"
            local body="Synthetic monitoring detected $failures failures out of $total_checks checks.

Results file: $results_file
Timestamp: $(date)

Failed checks:
$(grep '"status":"FAIL"' "$results_file" | sed 's/.*"check":"\([^"]*\)".*"message":"\([^"]*\)".*/- \1: \2/' | head -10)

Please investigate immediately."

            echo -e "${RED}$subject${NC}"
            echo "$body"
            echo

            send_email_alert "$subject" "$body"
            send_webhook_alert "{\"alert_type\":\"failure\",\"failures\":$failures,\"total_checks\":$total_checks,\"results_file\":\"$results_file\"}"
        fi
    fi

    # Alert on warnings (less urgent)
    if [ "$warnings" -gt 0 ] && [ "$failures" -eq 0 ]; then
        if check_recent_alerts "WARNING"; then
            local subject="⚠️ CostPilot Synthetic Monitoring: $warnings Warnings Detected"
            local body="Synthetic monitoring detected $warnings warnings out of $total_checks checks.

Results file: $results_file
Timestamp: $(date)

Warning checks:
$(grep '"status":"WARN"' "$results_file" | sed 's/.*"check":"\([^"]*\)".*"message":"\([^"]*\)".*/- \1: \2/' | head -10)

Please review when convenient."

            echo -e "${YELLOW}$subject${NC}"
            echo "$body"
            echo

            send_email_alert "$subject" "$body"
        fi
    fi

    # Success notification (optional, less frequent)
    if [ "$failures" -eq 0 ] && [ "$warnings" -eq 0 ]; then
        # Only send success alerts occasionally to avoid spam
        local success_count
        success_count=$(grep -c "SUCCESS" "$LOG_FILE" | tail -n 100 | wc -l || echo "0")

        if [ "$success_count" -lt 5 ]; then
            log_alert "SUCCESS" "All $total_checks checks passed"
        fi
    fi
}

# Function to cleanup old alert logs
cleanup_old_logs() {
    local retention_days=${ALERT_LOG_RETENTION_DAYS:-30}

    if [ -f "$LOG_FILE" ]; then
        # Keep only recent entries
        local cutoff_date
        cutoff_date=$(date -d "$retention_days days ago" +%Y-%m-%d 2>/dev/null || date -v-"${retention_days}d" +%Y-%m-%d 2>/dev/null || echo "")

        if [ -n "$cutoff_date" ]; then
            sed -i.bak "/^\\[$cutoff_date /d" "$LOG_FILE" 2>/dev/null || true
        fi
    fi
}

# Function to show alert status
show_alert_status() {
    echo "Synthetic Monitoring Alert Status"
    echo "=================================="

    if [ -f "$LOG_FILE" ]; then
        echo "Recent alerts:"
        tail -n 10 "$LOG_FILE" | while read -r line; do
            case $line in
                *"ERROR"*)
                    echo -e "${RED}$line${NC}"
                    ;;
                *"WARN"*)
                    echo -e "${YELLOW}$line${NC}"
                    ;;
                *"SUCCESS"*)
                    echo -e "${GREEN}$line${NC}"
                    ;;
                *)
                    echo "$line"
                    ;;
            esac
        done
    else
        echo "No alert log found"
    fi

    echo
    echo "Configuration:"
    echo "  Email recipient: ${ALERT_EMAIL:-"not configured"}"
    echo "  Webhook URL: ${ALERT_WEBHOOK_URL:+"configured"}"
    echo "  Throttle minutes: ${ALERT_THROTTLE_MINUTES:-60}"
    echo "  Log retention: ${ALERT_LOG_RETENTION_DAYS:-30} days"
}

# Main function
main() {
    mkdir -p "$MONITORING_DIR"

    case "${1:-check}" in
        "check")
            # Find the latest results file
            local latest_results
            latest_results=$(find "$MONITORING_DIR" -name "health-check-*.json" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2- || echo "")

            if [ -n "$latest_results" ]; then
                analyze_and_alert "$latest_results"
            else
                log_alert "WARN" "No monitoring results files found"
            fi
            ;;
        "status")
            show_alert_status
            ;;
        "cleanup")
            cleanup_old_logs
            log_alert "INFO" "Old alert logs cleaned up"
            ;;
        "test")
            # Send a test alert
            local test_subject="🧪 CostPilot Alert System Test"
            local test_body="This is a test alert from the CostPilot synthetic monitoring system.

Timestamp: $(date)
System: $(hostname)
User: $(whoami)

If you received this, the alert system is working correctly."

            echo "Sending test alert..."
            send_email_alert "$test_subject" "$test_body"
            send_webhook_alert "{\"alert_type\":\"test\",\"message\":\"Alert system test\",\"timestamp\":\"$(date -Iseconds)\"}"
            log_alert "INFO" "Test alert sent"
            ;;
        *)
            echo "Usage: $0 {check|status|cleanup|test}"
            echo
            echo "Commands:"
            echo "  check   - Check latest monitoring results and send alerts if needed"
            echo "  status  - Show alert system status and recent alerts"
            echo "  cleanup - Clean up old alert logs"
            echo "  test    - Send a test alert to verify configuration"
            echo
            echo "Environment variables:"
            echo "  ALERT_EMAIL              - Email address for alerts"
            echo "  ALERT_SLACK_WEBHOOK      - Slack webhook URL for alerts"
            echo "  ALERT_WEBHOOK_URL        - Generic webhook URL for alerts"
            echo "  ALERT_THROTTLE_MINUTES   - Minutes to wait between similar alerts (default: 60)"
            echo "  ALERT_LOG_RETENTION_DAYS - Days to keep alert logs (default: 30)"
            exit 1
            ;;
    esac
}

main "$@"
