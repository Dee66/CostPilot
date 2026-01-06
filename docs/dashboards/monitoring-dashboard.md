# CostPilot Synthetic Monitoring Dashboard

Generated: <!-- DASHBOARD_TIMESTAMP -->

## System Status
- **Last Check**: <!-- LAST_CHECK_TIME -->
- **Overall Health**: <!-- OVERALL_STATUS -->
- **Uptime**: <!-- UPTIME_PERCENTAGE -->

## Recent Health Checks

| Timestamp | Status | Passed/Total | Failures | Warnings |
|-----------|--------|-------------|----------|----------|
<!-- RECENT_CHECKS_TABLE -->

## Key Metrics

### Performance
- **Average Response Time**: <!-- AVG_RESPONSE_TIME -->ms
- **95th Percentile**: <!-- P95_RESPONSE_TIME -->ms
- **Error Rate**: <!-- ERROR_RATE -->%

### Reliability
- **Availability**: <!-- AVAILABILITY -->%
- **MTTR**: <!-- MTTR -->minutes
- **False Positives**: <!-- FALSE_POSITIVES -->

## Active Alerts
<!-- ACTIVE_ALERTS -->

## Alert History
<!-- ALERT_HISTORY -->

## Configuration
- **Check Interval**: 15 minutes
- **Alert Throttling**: 60 minutes
- **Log Retention**: 30 days
- **Notification Channels**: Email, Webhook

## Quick Actions
- [Run Manual Check](scripts/synthetic_monitoring.sh)
- [View Latest Results](synthetic-monitoring/)
- [Configure Alerts](configs/alerting.yml)

---
*Dashboard automatically updated by CI/CD pipeline*
