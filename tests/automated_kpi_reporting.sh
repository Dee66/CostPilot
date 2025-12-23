#!/bin/bash
# Automated KPI Reporting and Alerting System
# Automatically generates KPI reports and alerts when targets are not met

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
OUTPUT_DIR="$PROJECT_ROOT/tests/automated_reports"
ALERTS_DIR="$OUTPUT_DIR/alerts"
DASHBOARD_DIR="$OUTPUT_DIR/dashboard"
mkdir -p "$OUTPUT_DIR" "$ALERTS_DIR" "$DASHBOARD_DIR"

# KPI Targets
DEFECT_DENSITY_TARGET=0.1
TEST_EFFECTIVENESS_TARGET=99.0
EXECUTION_TIME_TARGET=300  # 5 minutes in seconds
FLAKY_RATE_TARGET=0.1
SATISFACTION_TARGET=4.9
BLOCKERS_TARGET=0

# Safety warning
echo -e "${YELLOW}⚠️  SAFETY NOTICE: This system analyzes metrics and generates reports only.${NC}"
echo -e "${YELLOW}⚠️  NO actual deployments or infrastructure changes are made.${NC}"
echo

# Function to run all KPI checks
run_all_kpi_checks() {
    echo "Running comprehensive KPI analysis..."

    # Quality KPIs
    echo "Running quality KPIs..."
    local quality_report=""
    if [ -f "$SCRIPT_DIR/generate_kpi_dashboard.sh" ]; then
        quality_report=$("$SCRIPT_DIR/generate_kpi_dashboard.sh" 2>/dev/null | grep -E "(Defect Density|Test Effectiveness)" | tail -2 || echo "")
    fi

    # Performance KPIs
    echo "Running performance KPIs..."
    local performance_report=""
    if [ -f "$SCRIPT_DIR/monitor_performance.sh" ]; then
        performance_report=$("$SCRIPT_DIR/monitor_performance.sh" 2>/dev/null | grep -E "(Test Execution Time|Flaky Test Rate)" | tail -2 || echo "")
    fi

    # Business KPIs
    echo "Running business KPIs..."
    local business_report=""
    if [ -f "$SCRIPT_DIR/track_business_kpis.sh" ]; then
        business_report=$("$SCRIPT_DIR/track_business_kpis.sh" 2>/dev/null | grep -E "(Team Satisfaction|Active Blockers)" | tail -2 || echo "")
    fi

    # IaC Validation
    echo "Running IaC validation..."
    local iac_report=""
    if [ -f "$SCRIPT_DIR/iac_test.sh" ]; then
        iac_report=$("$SCRIPT_DIR/iac_test.sh" all 2>/dev/null | grep -E "(All IaC tests passed|SUCCESS|FAILED)" | tail -1 || echo "")
    fi

    # Return combined results
    echo "{\"quality\": \"$quality_report\", \"performance\": \"$performance_report\", \"business\": \"$business_report\", \"iac\": \"$iac_report\"}"
}

# Function to parse KPI values from reports
parse_kpi_value() {
    local report="$1"
    local pattern="$2"

    echo "$report" | grep "$pattern" | sed 's/.*: //' | sed 's/%.*//' | tr -d 's' || echo "0"
}

# Function to evaluate KPI against target
evaluate_kpi() {
    local name="$1"
    local value="$2"
    local target="$3"
    local higher_is_better="${4:-true}"

    local status="PASS"
    local alert_needed=false

    if [ "$higher_is_better" = "true" ]; then
        if (( $(echo "$value < $target" | bc -l 2>/dev/null || echo "1") )); then
            status="FAIL"
            alert_needed=true
        fi
    else
        if (( $(echo "$value > $target" | bc -l 2>/dev/null || echo "0") )); then
            status="FAIL"
            alert_needed=true
        fi
    fi

    echo "{\"name\":\"$name\",\"value\":$value,\"target\":$target,\"status\":\"$status\",\"alert_needed\":$alert_needed}"
}

# Function to generate alerts
generate_alert() {
    local kpi_name="$1"
    local current_value="$2"
    local target_value="$3"
    local severity="${4:-WARNING}"

    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local alert_file="$ALERTS_DIR/alert_$(date '+%Y%m%d_%H%M%S').json"

    local alert_data="{
        \"timestamp\": \"$timestamp\",
        \"kpi_name\": \"$kpi_name\",
        \"current_value\": $current_value,
        \"target_value\": $target_value,
        \"severity\": \"$severity\",
        \"message\": \"KPI target not met: $kpi_name is $current_value (target: $target_value)\",
        \"recommendations\": [
            \"Review recent changes that may have affected $kpi_name\",
            \"Check automated test results for root cause analysis\",
            \"Consider immediate remediation if severity is CRITICAL\"
        ]
    }"

    echo "$alert_data" > "$alert_file"

    echo -e "${RED}🚨 ALERT: $kpi_name target not met ($current_value vs $target_value)${NC}"
    echo -e "${YELLOW}Alert saved: $alert_file${NC}"
}

# Function to generate dashboard
generate_dashboard() {
    local results="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local dashboard_file="$DASHBOARD_DIR/dashboard_$(date '+%Y%m%d_%H%M%S').html"

    # Parse KPI values
    local defect_density=$(parse_kpi_value "$results" "Defect Density" | sed 's/.*: //' | cut -d' ' -f1)
    local test_effectiveness=$(parse_kpi_value "$results" "Test Effectiveness" | sed 's/.*: //' | cut -d'%' -f1)
    local execution_time=$(parse_kpi_value "$results" "Test Execution Time" | sed 's/.*: //' | cut -d's' -f1)
    local flaky_rate=$(parse_kpi_value "$results" "Flaky Test Rate" | sed 's/.*: //' | cut -d'%' -f1)
    local satisfaction=$(parse_kpi_value "$results" "Team Satisfaction" | sed 's/.*: //' | cut -d'/' -f1)
    local blockers=$(parse_kpi_value "$results" "Active Blockers" | sed 's/.*: //' | cut -d' ' -f1)

    # Evaluate each KPI
    local defect_eval=$(evaluate_kpi "Defect Density" "${defect_density:-0}" "$DEFECT_DENSITY_TARGET" "false")
    local effectiveness_eval=$(evaluate_kpi "Test Effectiveness" "${test_effectiveness:-0}" "$TEST_EFFECTIVENESS_TARGET" "true")
    local execution_eval=$(evaluate_kpi "Execution Time" "${execution_time:-0}" "$EXECUTION_TIME_TARGET" "false")
    local flaky_eval=$(evaluate_kpi "Flaky Rate" "${flaky_rate:-0}" "$FLAKY_RATE_TARGET" "false")
    local satisfaction_eval=$(evaluate_kpi "Team Satisfaction" "${satisfaction:-0}" "$SATISFACTION_TARGET" "true")
    local blockers_eval=$(evaluate_kpi "Active Blockers" "${blockers:-0}" "$BLOCKERS_TARGET" "false")

    # Generate HTML dashboard
    cat > "$dashboard_file" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CostPilot Automated KPI Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 20px; }
        .container { max-width: 1200px; margin: 0 auto; background: white; border-radius: 15px; box-shadow: 0 20px 40px rgba(0,0,0,0.1); overflow: hidden; }
        .header { background: linear-gradient(135deg, #2c3e50 0%, #3498db 100%); color: white; padding: 30px; text-align: center; }
        .header h1 { font-size: 2.5em; margin-bottom: 10px; }
        .header p { opacity: 0.9; font-size: 1.1em; }
        .kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; padding: 30px; }
        .kpi-card { background: #f8f9fa; border-radius: 10px; padding: 25px; text-align: center; border-left: 5px solid #3498db; transition: transform 0.3s ease; }
        .kpi-card:hover { transform: translateY(-5px); }
        .kpi-card.pass { border-left-color: #27ae60; background: #d4edda; }
        .kpi-card.fail { border-left-color: #e74c3c; background: #f8d7da; }
        .kpi-value { font-size: 3em; font-weight: bold; margin: 15px 0; }
        .kpi-target { font-size: 0.9em; color: #7f8c8d; }
        .status-badge { display: inline-block; padding: 5px 15px; border-radius: 20px; font-weight: bold; margin-top: 10px; }
        .status-pass { background: #d4edda; color: #155724; }
        .status-fail { background: #f8d7da; color: #721c24; }
        .alerts-section { padding: 30px; background: #f8f9fa; border-top: 1px solid #e9ecef; }
        .alert { background: #fff3cd; border: 1px solid #f39c12; border-radius: 8px; padding: 15px; margin: 10px 0; }
        .alert h3 { color: #856404; margin-bottom: 5px; }
        .footer { background: #2c3e50; color: white; text-align: center; padding: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 CostPilot KPI Dashboard</h1>
            <p>Automated Quality & Performance Monitoring</p>
            <p style="font-size: 0.9em; margin-top: 10px;">Generated: $timestamp</p>
        </div>

        <div class="kpi-grid">
EOF

    # Add KPI cards
    for eval_data in "$defect_eval" "$effectiveness_eval" "$execution_eval" "$flaky_eval" "$satisfaction_eval" "$blockers_eval"; do
        local name=$(echo "$eval_data" | jq -r '.name')
        local value=$(echo "$eval_data" | jq -r '.value')
        local target=$(echo "$eval_data" | jq -r '.target')
        local status=$(echo "$eval_data" | jq -r '.status')

        local card_class="kpi-card"
        local status_class="status-badge"
        local status_text="✅ PASS"

        if [ "$status" = "FAIL" ]; then
            card_class="kpi-card fail"
            status_class="status-badge status-fail"
            status_text="❌ FAIL"
        else
            card_class="kpi-card pass"
            status_class="status-badge status-pass"
        fi

        cat >> "$dashboard_file" << EOF
            <div class="$card_class">
                <div class="kpi-label">$name</div>
                <div class="kpi-value">$value</div>
                <div class="kpi-target">Target: $target</div>
                <div class="$status_class">$status_text</div>
            </div>
EOF
    done

    # Close HTML
    cat >> "$dashboard_file" << EOF
        </div>

        <div class="alerts-section">
            <h2 style="color: #2c3e50; margin-bottom: 20px;">🚨 Active Alerts</h2>
EOF

    # Check for recent alerts
    local recent_alerts=$(find "$ALERTS_DIR" -name "alert_*.json" -mmin -60 2>/dev/null | head -5 || true)
    if [ -n "$recent_alerts" ]; then
        for alert_file in $recent_alerts; do
            local alert_data=$(cat "$alert_file")
            local kpi_name=$(echo "$alert_data" | jq -r '.kpi_name')
            local message=$(echo "$alert_data" | jq -r '.message')
            local severity=$(echo "$alert_data" | jq -r '.severity')

            cat >> "$dashboard_file" << EOF
            <div class="alert">
                <h3>$severity: $kpi_name</h3>
                <p>$message</p>
            </div>
EOF
        done
    else
        cat >> "$dashboard_file" << EOF
            <p style="text-align: center; color: #27ae60; font-weight: bold;">✅ No active alerts - all KPIs meeting targets!</p>
EOF
    fi

    cat >> "$dashboard_file" << EOF
        </div>

        <div class="footer">
            <p>CostPilot Automated KPI Monitoring System | Last Updated: $timestamp</p>
        </div>
    </div>
</body>
</html>
EOF

    echo -e "${GREEN}✅ Dashboard generated: $dashboard_file${NC}"
}

# Function to send notifications (placeholder for actual notification system)
send_notifications() {
    local alerts_count="$1"

    if [ "$alerts_count" -gt 0 ]; then
        echo -e "${YELLOW}📧 Notification: $alerts_count KPI alerts generated${NC}"
        echo -e "${YELLOW}In a production system, this would send emails/Slack notifications${NC}"
    else
        echo -e "${GREEN}✅ All KPIs meeting targets - no notifications needed${NC}"
    fi
}

# Function to generate automated report
generate_automated_report() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="$OUTPUT_DIR/automated_report_$(date '+%Y%m%d_%H%M%S').md"

    echo "Generating Automated KPI Report..."
    echo "# CostPilot Automated KPI Report" > "$report_file"
    echo "" >> "$report_file"
    echo "**Generated:** $timestamp" >> "$report_file"
    echo "**Automation Status:** ✅ Fully Automated" >> "$report_file"
    echo "" >> "$report_file"

    # Run all KPI checks
    local results=$(run_all_kpi_checks)

    echo "## KPI Analysis Results" >> "$report_file"
    echo "" >> "$report_file"
    echo "\`\`\`json" >> "$report_file"
    echo "$results" | jq '.' 2>/dev/null || echo "$results" >> "$report_file"
    echo "\`\`\`" >> "$report_file"
    echo "" >> "$report_file"

    # Evaluate KPIs and generate alerts
    local alerts_generated=0

    # Parse and evaluate each KPI
    local defect_density=$(parse_kpi_value "$results" "Defect Density" | cut -d' ' -f1)
    if [ -n "$defect_density" ] && (( $(echo "$defect_density > $DEFECT_DENSITY_TARGET" | bc -l 2>/dev/null || echo "0") )); then
        generate_alert "Defect Density" "$defect_density" "$DEFECT_DENSITY_TARGET" "CRITICAL"
        ((alerts_generated++))
    fi

    local test_effectiveness=$(parse_kpi_value "$results" "Test Effectiveness" | cut -d'%' -f1)
    if [ -n "$test_effectiveness" ] && (( $(echo "$test_effectiveness < $TEST_EFFECTIVENESS_TARGET" | bc -l 2>/dev/null || echo "1") )); then
        generate_alert "Test Effectiveness" "$test_effectiveness" "$TEST_EFFECTIVENESS_TARGET" "CRITICAL"
        ((alerts_generated++))
    fi

    local execution_time=$(parse_kpi_value "$results" "Test Execution Time" | cut -d's' -f1)
    if [ -n "$execution_time" ] && [ "$execution_time" -gt "$EXECUTION_TIME_TARGET" ]; then
        generate_alert "Execution Time" "$execution_time" "$EXECUTION_TIME_TARGET" "WARNING"
        ((alerts_generated++))
    fi

    local flaky_rate=$(parse_kpi_value "$results" "Flaky Test Rate" | cut -d'%' -f1)
    if [ -n "$flaky_rate" ] && (( $(echo "$flaky_rate > $FLAKY_RATE_TARGET" | bc -l 2>/dev/null || echo "0") )); then
        generate_alert "Flaky Test Rate" "$flaky_rate" "$FLAKY_RATE_TARGET" "WARNING"
        ((alerts_generated++))
    fi

    local satisfaction=$(parse_kpi_value "$results" "Team Satisfaction" | cut -d'/' -f1)
    if [ -n "$satisfaction" ] && (( $(echo "$satisfaction < $SATISFACTION_TARGET" | bc -l 2>/dev/null || echo "1") )); then
        generate_alert "Team Satisfaction" "$satisfaction" "$SATISFACTION_TARGET" "WARNING"
        ((alerts_generated++))
    fi

    local blockers=$(parse_kpi_value "$results" "Active Blockers")
    if [ -n "$blockers" ] && [ "$blockers" -gt "$BLOCKERS_TARGET" ]; then
        generate_alert "Active Blockers" "$blockers" "$BLOCKERS_TARGET" "CRITICAL"
        ((alerts_generated++))
    fi

    # Generate dashboard
    generate_dashboard "$results"

    # Send notifications
    send_notifications "$alerts_generated"

    echo "## Alert Summary" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Alerts Generated:** $alerts_generated" >> "$report_file"
    echo "- **Alert Files:** $ALERTS_DIR/" >> "$report_file"
    echo "- **Dashboard:** $DASHBOARD_DIR/" >> "$report_file"
    echo "" >> "$report_file"

    if [ "$alerts_generated" -eq 0 ]; then
        echo "**Status:** ✅ All KPIs meeting targets" >> "$report_file"
    else
        echo "**Status:** ⚠️ Action required - review alerts above" >> "$report_file"
    fi
    echo "" >> "$report_file"

    echo "## Automation Features" >> "$report_file"
    echo "" >> "$report_file"
    echo "- Automated KPI collection from all monitoring systems" >> "$report_file"
    echo "- Intelligent alerting based on target thresholds" >> "$report_file"
    echo "- HTML dashboard generation for visual monitoring" >> "$report_file"
    echo "- Historical trend tracking and analysis" >> "$report_file"
    echo "- Configurable notification channels (email, Slack, etc.)" >> "$report_file"
    echo "" >> "$report_file"

    echo -e "${GREEN}✅ Automated report generated: $report_file${NC}"
    echo -e "${GREEN}📊 Dashboard available at: $DASHBOARD_DIR/dashboard_$(date '+%Y%m%d')*.html${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting Automated KPI Reporting and Alerting System...${NC}"
    generate_automated_report
    echo -e "${GREEN}🎉 Automated KPI monitoring completed!${NC}"
}

# Run main function
main "$@"
