# SLO Burn Rate CLI Examples

This guide demonstrates using the `costpilot slo burn` command for predictive cost analysis.

## Prerequisites

1. **SLO Configuration** (`.costpilot/slo.json`):
```json
{
  "version": "1.0",
  "slos": [
    {
      "id": "prod-monthly",
      "name": "Production Monthly Budget",
      "description": "Total monthly cost limit for production",
      "slo_type": "monthly_budget",
      "target": "global",
      "threshold": {
        "max_value": 10000.0,
        "warning_threshold_percent": 80.0,
        "time_window": "30d",
        "use_baseline": false
      },
      "enforcement": "block",
      "owner": "devops@company.com",
      "created_at": "2024-12-01T00:00:00Z"
    },
    {
      "id": "vpc-module",
      "name": "VPC Module Budget",
      "description": "Cost limit for VPC infrastructure",
      "slo_type": "module_budget",
      "target": "module.vpc",
      "threshold": {
        "max_value": 3000.0,
        "warning_threshold_percent": 85.0,
        "time_window": "30d",
        "use_baseline": false
      },
      "enforcement": "warn",
      "owner": "network-team@company.com",
      "created_at": "2024-12-01T00:00:00Z"
    }
  ]
}
```

2. **Historical Snapshots** (`.costpilot/snapshots/`):
```bash
# Create snapshots on each PR
costpilot snapshot create --plan plan.json

# Or manually from test data
.costpilot/snapshots/
├── snapshot_2024-11-01.json
├── snapshot_2024-11-08.json
├── snapshot_2024-11-15.json
├── snapshot_2024-11-22.json
└── snapshot_2024-11-29.json
```

## Basic Usage

### Analyze All SLOs

```bash
costpilot slo burn
```

**Output:**
```
🔥 Calculating burn rate...

📊 SLO Burn Rate Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔶 Production Monthly Budget ($10,000/month)
  Burn Rate:      $142.86/day
  Projected Cost: $4,428.60/month
  Time to Breach: 8.5 days
  Risk Level:     High
  Confidence:     95% (R² = 0.950)

✅ VPC Module Budget ($3,000/month)
  Burn Rate:      $35.71/day
  Projected Cost: $1,107.10/month
  Time to Breach: No breach predicted
  Risk Level:     Low
  Confidence:     92% (R² = 0.920)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary
  Total SLOs:     2
  At Risk:        1
  Overall Risk:   High

⚠️  Action Required

Critical SLOs:
  • Production Monthly Budget - 8.5 days to breach

Recommended Actions:
  1. Review cost drivers in critical SLOs
  2. Consider scaling down or optimizing resources
  3. Update SLO limits if growth is expected
  4. Investigate unexpected cost increases
```

## Advanced Usage

### JSON Output for CI/CD

```bash
costpilot slo burn --format json > burn-report.json
```

**Output:**
```json
{
  "analyses": [
    {
      "slo_id": "prod-monthly",
      "slo_name": "Production Monthly Budget",
      "burn_rate": 142.86,
      "projected_cost": 4428.6,
      "slo_limit": 10000.0,
      "days_to_breach": 8.5,
      "risk": "High",
      "confidence": 0.95,
      "trend_slope": 142.86,
      "trend_intercept": 1000.0,
      "r_squared": 0.95,
      "analyzed_at": "2024-12-06T10:00:00Z"
    }
  ],
  "overall_risk": "High",
  "total_slos": 2,
  "slos_at_risk": 1,
  "generated_at": "2024-12-06T10:00:00Z"
}
```

### Markdown Output for PR Comments

```bash
costpilot slo burn --format markdown
```

**Output:**
```markdown
## 📊 SLO Burn Rate Analysis

| SLO | Burn Rate | Projected | Days to Breach | Risk | Confidence |
|-----|-----------|-----------|----------------|------|------------|
| 🔶 Production Monthly Budget | $142.86/day | $4,428.60 | 8.5 days | High | 95% |
| ✅ VPC Module Budget | $35.71/day | $1,107.10 | No breach | Low | 92% |

### Summary

- **Total SLOs:** 2
- **At Risk:** 1
- **Overall Risk:** High

### ⚠️ Action Required

**Critical SLOs:**
- Production Monthly Budget - 8.5 days to breach

**Recommended Actions:**
1. Review cost drivers in critical SLOs
2. Consider scaling down or optimizing resources
```

### Custom Thresholds

```bash
# Require more snapshots for better accuracy
costpilot slo burn --min-snapshots 5

# Higher confidence threshold (R² ≥ 0.85)
costpilot slo burn --min-r-squared 0.85

# Both
costpilot slo burn --min-snapshots 7 --min-r-squared 0.9
```

### Custom Paths

```bash
# Different SLO config location
costpilot slo burn --slo config/production-slo.json

# Different snapshots directory
costpilot slo burn --snapshots /data/cost-snapshots/

# Both
costpilot slo burn \
  --slo .costpilot/slo.json \
  --snapshots .costpilot/snapshots/
```

## CI/CD Integration

### GitHub Actions

Add burn rate check to your workflow:

```yaml
name: Cost SLO Burn Rate
on:
  schedule:
    # Run daily at 9 AM UTC
    - cron: '0 9 * * *'
  workflow_dispatch:

jobs:
  burn-rate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install CostPilot
        run: cargo install costpilot
      
      - name: Check Burn Rate
        id: burn
        run: |
          costpilot slo burn --format json > burn-report.json
          cat burn-report.json
        continue-on-error: true
      
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: burn-rate-report
          path: burn-report.json
      
      - name: Alert on Critical
        if: steps.burn.outcome == 'failure'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('burn-report.json', 'utf8'));
            
            if (report.overall_risk === 'Critical' || report.overall_risk === 'High') {
              github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: `🔥 Critical SLO Burn Rate Alert`,
                body: `**Overall Risk:** ${report.overall_risk}\n\n` +
                      `**SLOs at Risk:** ${report.slos_at_risk}/${report.total_slos}\n\n` +
                      `Review required: Some SLOs are approaching breach.`,
                labels: ['cost-alert', 'urgent']
              });
            }
```

### Pull Request Comments

```yaml
name: PR Cost Analysis
on: [pull_request]

jobs:
  cost-burn:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Need history for snapshots
      
      - name: Install CostPilot
        run: cargo install costpilot
      
      - name: Generate Burn Report
        run: |
          costpilot slo burn --format markdown > burn-report.md
      
      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('burn-report.md', 'utf8');
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

### GitLab CI

```yaml
slo_burn_rate:
  stage: test
  script:
    - costpilot slo burn --format json > burn-report.json
    - costpilot slo burn --format markdown > burn-report.md
  artifacts:
    reports:
      metrics: burn-report.json
    paths:
      - burn-report.md
  rules:
    - if: '$CI_PIPELINE_SOURCE == "schedule"'
```

## Exit Codes

The `costpilot slo burn` command returns different exit codes based on risk level:

- **Exit 0**: All SLOs at Low or Medium risk
- **Exit 1**: One or more SLOs at High or Critical risk (requires action)

This allows CI/CD pipelines to fail when cost breaches are imminent:

```bash
# Block deployment if burn rate is critical
if ! costpilot slo burn; then
  echo "❌ Deployment blocked: Critical burn rate detected"
  exit 1
fi
```

## Understanding Risk Levels

| Risk | Days to Breach | Severity | Action |
|------|---------------|----------|--------|
| **Low** | >30 days or no breach | 0 | Monitor |
| **Medium** | 14-30 days | 1 | Plan mitigation |
| **High** | 7-14 days | 2 | Take action |
| **Critical** | <7 days or already exceeded | 3 | Immediate action |

## Troubleshooting

### "No historical snapshots found"

**Problem:** No `.costpilot/snapshots/` directory or empty

**Solution:**
```bash
# Create snapshot directory
mkdir -p .costpilot/snapshots

# Generate first snapshot
costpilot snapshot create --plan plan.json
```

### "Insufficient data for analysis"

**Problem:** Less than 3 snapshots available

**Solution:**
- Collect snapshots over time (daily recommended)
- Lower threshold: `--min-snapshots 2` (less reliable)

### "Low confidence predictions"

**Problem:** R² < 0.7 indicating poor linear fit

**Causes:**
- Volatile cost patterns
- Recent infrastructure changes
- Seasonal variations

**Solutions:**
- Collect more snapshots for better trend
- Investigate cost volatility
- Use `--min-r-squared 0.5` for lower threshold (use with caution)

### "Failed to load SLO config"

**Problem:** Missing or invalid SLO configuration

**Solution:**
```bash
# Initialize CostPilot configuration
costpilot init

# Or specify custom path
costpilot slo burn --slo path/to/slo.json
```

## Best Practices

1. **Daily Snapshot Collection**
   - Run `costpilot snapshot create` on every deployment
   - Maintain 30-90 days of history

2. **Automated Monitoring**
   - Schedule daily burn rate checks in CI/CD
   - Alert on High/Critical risk levels

3. **Threshold Tuning**
   - Start with defaults (3 snapshots, R² ≥ 0.7)
   - Increase for production: 5+ snapshots, R² ≥ 0.85

4. **Risk Response**
   - **Critical**: Investigate immediately, block deployments
   - **High**: Review within 24 hours, plan optimization
   - **Medium**: Track weekly, consider preventive measures
   - **Low**: Monitor normal operations

5. **Integration with Alerts**
   - Send Critical/High alerts to Slack/PagerDuty
   - Create GitHub issues for tracking
   - Include burn report in PR comments

## Related Documentation

- [SLO_ENGINE.md](../SLO_ENGINE.md) - SLO system overview
- [SLO_BURN_ALERTS.md](../SLO_BURN_ALERTS.md) - Burn rate technical details
- [CLI.md](../CLI.md) - Full CLI reference
