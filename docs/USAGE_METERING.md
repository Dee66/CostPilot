# Usage Metering & Chargeback Documentation

## Overview

CostPilot's Usage Metering system provides comprehensive tracking, attribution, and chargeback capabilities for enterprise deployments. It enables organizations to:

- **Track usage** across teams, users, and projects
- **Attribute costs** to appropriate cost centers
- **Generate chargebacks** for internal billing
- **Monitor PR-based usage** in CI/CD pipelines
- **Calculate ROI** from cost issues prevented

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Usage Metering System                            │
│                                                                       │
│  ┌──────────────┐  ┌───────────────┐  ┌─────────────────────┐     │
│  │ Usage Meter  │  │  PR Tracker   │  │ Chargeback Reporter │     │
│  │              │  │               │  │                     │     │
│  │ • Events     │  │ • PR-based    │  │ • Team summaries    │     │
│  │ • Attribution│  │   billing     │  │ • Cost centers      │     │
│  │ • Metrics    │  │ • ROI calc    │  │ • Invoices          │     │
│  │ • Pricing    │  │ • CI/CD usage │  │ • CSV export        │     │
│  └──────┬───────┘  └───────┬───────┘  └──────────┬──────────┘     │
│         │                  │                      │                 │
│         └──────────────────┼──────────────────────┘                 │
│                            │                                        │
│               ┌────────────▼────────────┐                           │
│               │   Usage Database        │                           │
│               │  • Event storage        │                           │
│               │  • Team aggregation     │                           │
│               │  • Billing calculation  │                           │
│               └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Usage Meter

**Purpose**: Track all CostPilot usage events with full attribution.

**Usage Event Structure**:
```rust
pub struct UsageEvent {
    event_id: String,          // Unique identifier
    timestamp: u64,            // Unix epoch
    event_type: UsageEventType,// Scan, PolicyCheck, etc.
    attribution: Attribution,  // User, team, cost center
    resources_analyzed: u32,   // Number of resources
    cost_impact: f64,          // Estimated cost impact detected
    duration_ms: u64,          // Analysis duration
    context: UsageContext,     // Repository, PR, commit info
    metadata: HashMap<String, String>,
}
```

**Event Types**:
- `Scan` - Workspace scan
- `PlanAnalysis` - Terraform plan analysis
- `PolicyCheck` - Policy evaluation
- `SloCheck` - SLO compliance check
- `AutofixGeneration` - Autofix creation
- `DependencyMap` - Dependency mapping
- `TrendAnalysis` - Trend analysis
- `AdvancedPrediction` - Probabilistic prediction

**Example Usage**:
```rust
use costpilot::engines::metering::{UsageMeter, UsageEvent, PricingModel};

// Create meter with pricing
let pricing = PricingModel::default(); // Pro tier
let mut meter = UsageMeter::new(pricing);

// Record event
let event = UsageEvent {
    event_id: uuid::Uuid::new_v4().to_string(),
    timestamp: current_timestamp(),
    event_type: UsageEventType::Scan,
    attribution: Attribution {
        user_id: "alice@company.com".to_string(),
        team_id: Some("platform-team".to_string()),
        org_id: Some("acme-corp".to_string()),
        cost_center: Some("engineering".to_string()),
        project_id: Some("api-gateway".to_string()),
    },
    resources_analyzed: 150,
    cost_impact: 5000.0,
    duration_ms: 450,
    context: UsageContext {
        repository: "acme-corp/api-gateway".to_string(),
        branch: Some("main".to_string()),
        commit: Some("abc123".to_string()),
        pr_number: None,
        ci_system: Some("github-actions".to_string()),
        environment: Some("production".to_string()),
    },
    metadata: HashMap::new(),
};

meter.record_event(event)?;
```

**Retrieve Metrics**:
```rust
// Get metrics for last month
let start = start_of_month();
let end = end_of_month();
let metrics = meter.get_metrics(start, end);

println!("Total events: {}", metrics.total_events);
println!("Resources analyzed: {}", metrics.total_resources);
println!("Cost impact: ${:.2}", metrics.total_cost_impact);
println!("Unique users: {}", metrics.unique_users);
println!("Unique teams: {}", metrics.unique_teams);
```

**Output**:
```
Total events: 1,247
Resources analyzed: 45,890
Cost impact: $234,567.00
Unique users: 23
Unique teams: 5
```

### 2. PR Tracker

**Purpose**: Track CostPilot usage per Pull Request for CI/CD billing.

**Key Features**:
- PR-level usage tracking
- Commit-by-commit analysis
- ROI calculation (cost prevented / charge)
- Status tracking (Open, Merged, Closed, Draft)

**Example Usage**:
```rust
use costpilot::engines::metering::{CiUsageTracker, PrStatus};

let mut tracker = CiUsageTracker::new("acme-corp/api-gateway".to_string());

// Start tracking PR
tracker.track_pr(
    123,                           // PR number
    "alice@company.com".to_string(),// Author
    "Add caching layer".to_string(),// Title
    "feature/caching".to_string(), // Branch
)?;

// Record analysis events
for commit in commits {
    let event = create_usage_event(commit);
    tracker.record_pr_event(123, event)?;
}

// Update status when merged
tracker.update_pr_status(123, PrStatus::Merged)?;

// Get summary for billing
let summary = tracker.get_pr_summary(123, 0.01)?;

println!("PR #{}: {} resources analyzed", summary.pr_number, summary.resources_analyzed);
println!("Scans: {}", summary.scan_count);
println!("Cost prevented: ${:.2}", summary.cost_prevented);
println!("Charge: ${:.2}", summary.estimated_charge);

if let Some(roi) = summary.roi {
    println!("ROI: {:.1}x", roi);
}
```

**Output**:
```
PR #123: 245 resources analyzed
Scans: 8
Cost prevented: $12,450.00
Charge: $2.45
ROI: 5081.6x
```

**PR Usage Report**:
```rust
// Generate report for all PRs in period
let report = tracker.generate_report(start_of_month(), end_of_month());

println!("{}", report.format_text());
```

**Output**:
```
📊 PR Usage Report
==================

Period: 1701388800 - 1704067199

Summary:
  Total PRs: 47
  Total Scans: 189
  Resources Analyzed: 8,934
  Cost Prevented: $445,670.00
  Estimated Charge: $89.34
  Avg Resources/PR: 190
  Average ROI: 4989.2x

Top PRs by Resources Analyzed:
  1. PR #145 - 542 resources ($5.42)
  2. PR #138 - 387 resources ($3.87)
  3. PR #142 - 356 resources ($3.56)
  ...
```

### 3. Chargeback Reporter

**Purpose**: Generate team-level chargeback reports for internal billing.

**Key Features**:
- Team cost attribution
- Cost center aggregation
- User-level breakdown
- Project-level breakdown
- Top cost drivers identification
- Invoice generation
- CSV export

**Example Usage**:
```rust
use costpilot::engines::metering::ChargebackReportBuilder;

// Build report
let mut builder = ChargebackReportBuilder::new(
    "acme-corp".to_string(),
    start_of_month(),
    end_of_month(),
);

// Add team summaries
for team in teams {
    let summary = meter.team_summary(&team.id, start, end)?;
    builder.add_team(summary);
}

let report = builder.build()?;

// Display report
println!("{}", report.format_text());
```

**Output**:
```
💰 Chargeback Report
====================

Organization: acme-corp
Period: 1701388800 - 1704067199

Total Charge: $487.50

Team Breakdown:
  Platform Team - $245.00 (50.3%)
    Resources: 12,450
    Events: 89
    Value Delivered: $187,600.00
    ROI: 765.7x

  Data Team - $158.00 (32.4%)
    Resources: 8,900
    Events: 67
    Value Delivered: $134,200.00
    ROI: 849.4x

  Frontend Team - $84.50 (17.3%)
    Resources: 4,200
    Events: 34
    Value Delivered: $58,900.00
    ROI: 697.0x

Top Cost Drivers:
  Team: Platform Team (Rank #1) - $245.00 (50.3%)
  Team: Data Team (Rank #2) - $158.00 (32.4%)
  Team: Frontend Team (Rank #3) - $84.50 (17.3%)
```

**Generate Team Invoice**:
```rust
if let Some(invoice) = report.generate_invoice("platform-team") {
    println!("{}", invoice);
}
```

**Output**:
```
┌────────────────────────────────────────────┐
│           CostPilot Invoice                │
└────────────────────────────────────────────┘

Organization: acme-corp
Team: Platform Team
Period: 1701388800 - 1704067199

Usage Summary:
  Resources Analyzed: 12,450
  Events Performed: 89

Charges:
  Total: $245.00

Value Delivered:
  Cost Issues Detected: $187,600.00
  ROI: 765.7x return on investment

Top Users:
  alice@company.com - 4,567 resources ($90.12)
  bob@company.com - 3,234 resources ($63.78)
  charlie@company.com - 2,890 resources ($56.98)
```

**CSV Export**:
```rust
let csv = report.to_csv();
// Save to file or send to external billing system
std::fs::write("chargeback.csv", csv)?;
```

## Pricing Models

### Free Tier
```rust
PricingModel {
    tier: PricingTier::Free,
    price_per_resource: 0.0,
    price_per_scan: 0.0,
    price_per_advanced: 0.0,
    monthly_minimum: 0.0,
    free_tier_resources: 1000,
}
```
- 1,000 resources/month free
- Single user
- Basic features only

### Solo Tier
```rust
PricingModel {
    tier: PricingTier::Solo,
    price_per_resource: 0.005,  // $0.005/resource
    price_per_scan: 0.0,
    price_per_advanced: 0.05,   // $0.05/advanced analysis
    monthly_minimum: 19.0,
    free_tier_resources: 5000,
}
```
- $19/month base
- 5,000 resources included
- $0.005 per additional resource
- Advanced predictions: $0.05 each

### Pro Tier (Default)
```rust
PricingModel {
    tier: PricingTier::Pro,
    price_per_resource: 0.01,   // $0.01/resource
    price_per_scan: 0.05,       // $0.05/scan
    price_per_advanced: 0.10,   // $0.10/advanced analysis
    monthly_minimum: 49.0,
    free_tier_resources: 1000,
}
```
- $49/month base
- 1,000 resources included
- $0.01 per additional resource
- $0.05 per scan event
- Team collaboration

### Enterprise Tier
```rust
PricingModel {
    tier: PricingTier::Enterprise,
    price_per_resource: 0.008,  // Volume discount
    price_per_scan: 0.03,
    price_per_advanced: 0.08,
    monthly_minimum: 499.0,
    free_tier_resources: 10000,
}
```
- $499/month base
- 10,000 resources included
- Volume discounts
- Full chargeback reporting
- SSO, audit logs, escrow

## Integration Patterns

### 1. GitHub Actions Integration

```yaml
# .github/workflows/costpilot.yml
name: CostPilot Cost Analysis

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run CostPilot
        run: |
          # Record usage event
          costpilot meter record \
            --event-type scan \
            --user "${{ github.actor }}" \
            --team "${{ github.repository_owner }}" \
            --repository "${{ github.repository }}" \
            --pr "${{ github.event.pull_request.number }}" \
            --commit "${{ github.sha }}" \

          # Run scan
          costpilot scan --format json > scan-result.json

          # Update PR tracker
          costpilot meter pr-update \
            --pr "${{ github.event.pull_request.number }}" \
            --result scan-result.json
```

### 2. Monthly Billing Export

```rust
use costpilot::engines::metering::UsageMeter;

// Export billing data for external systems
let meter = UsageMeter::load_from_db()?;

let start = start_of_last_month();
let end = end_of_last_month();

let billing_export = meter.export_billing_data(start, end)?;

// Send to Stripe, AWS Marketplace, or internal billing
send_to_billing_system(&billing_export)?;
```

### 3. Real-Time Dashboard

```rust
// REST API endpoint for usage dashboard
#[get("/api/usage/current")]
async fn current_usage() -> Json<UsageMetrics> {
    let meter = get_usage_meter();
    let now = current_timestamp();
    let start_of_period = start_of_current_billing_period();

    let metrics = meter.get_metrics(start_of_period, now);
    Json(metrics)
}
```

### 4. Cost Center Reporting

```rust
// Generate reports by cost center
fn cost_center_report(cost_center: &str) -> Result<ChargebackReport> {
    let meter = load_meter();
    let teams = get_teams_by_cost_center(cost_center);

    let mut builder = ChargebackReportBuilder::new(
        get_org_id(),
        start_of_month(),
        end_of_month(),
    );

    for team in teams {
        let summary = meter.team_summary(&team.id, start, end)?;
        builder.add_team(summary);
    }

    builder.build()
}
```

## CLI Commands

### Record Usage Event
```bash
costpilot meter record \
  --event-type scan \
  --user alice@company.com \
  --team platform-team \
  --resources 150 \
  --cost-impact 5000.0 \
  --repository acme-corp/api
```

### Get Usage Metrics
```bash
# Current month
costpilot meter metrics

# Specific period
costpilot meter metrics \
  --start 2024-01-01 \
  --end 2024-01-31
```

### Team Summary
```bash
costpilot meter team-summary platform-team \
  --start 2024-01-01 \
  --end 2024-01-31
```

### PR Report
```bash
costpilot meter pr-report \
  --repository acme-corp/api \
  --start 2024-01-01 \
  --end 2024-01-31 \
  --format text
```

### Chargeback Report
```bash
# Generate chargeback report
costpilot meter chargeback \
  --org acme-corp \
  --start 2024-01-01 \
  --end 2024-01-31 \
  --format csv > chargeback.csv

# Generate team invoice
costpilot meter invoice platform-team \
  --month 2024-01
```

## Storage Backend

### File-Based (Development)
```rust
// Store events in NDJSON file
impl UsageMeter {
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let file = std::fs::File::create(path)?;
        for event in &self.events {
            serde_json::to_writer(&file, event)?;
            writeln!(&file)?;
        }
        Ok(())
    }

    pub fn load_from_file(path: &Path, pricing: PricingModel) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let events = reader.lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        Ok(Self { events, pricing })
    }
}
```

### Database (Production)
```sql
-- PostgreSQL schema
CREATE TABLE usage_events (
    event_id UUID PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    team_id VARCHAR(255),
    org_id VARCHAR(255),
    cost_center VARCHAR(255),
    project_id VARCHAR(255),
    resources_analyzed INTEGER NOT NULL,
    cost_impact DECIMAL(12,2) NOT NULL,
    duration_ms BIGINT NOT NULL,
    repository VARCHAR(500) NOT NULL,
    branch VARCHAR(255),
    commit_sha VARCHAR(64),
    pr_number INTEGER,
    ci_system VARCHAR(100),
    environment VARCHAR(100),
    metadata JSONB
);

CREATE INDEX idx_usage_timestamp ON usage_events(timestamp);
CREATE INDEX idx_usage_team ON usage_events(team_id, timestamp);
CREATE INDEX idx_usage_org ON usage_events(org_id, timestamp);
CREATE INDEX idx_usage_pr ON usage_events(repository, pr_number);
```

## Best Practices

### 1. Attribution Hierarchy
```
Organization → Cost Center → Team → User → Project
```
Always provide as much context as possible for accurate chargeback.

### 2. Event Metadata
Include rich metadata for debugging and analysis:
```rust
let mut metadata = HashMap::new();
metadata.insert("terraform_version".to_string(), "1.6.0".to_string());
metadata.insert("issues_detected".to_string(), "12".to_string());
metadata.insert("ci_job_id".to_string(), "12345".to_string());
```

### 3. ROI Tracking
Always calculate and report ROI to demonstrate value:
```
ROI = Cost Issues Prevented / CostPilot Charge
```

### 4. Billing Periods
Align with your organization's billing cycles:
- Monthly: Most common
- Quarterly: For budget planning
- Annual: For enterprise contracts

### 5. Free Tier Management
```rust
// Check if within free tier
if total_resources <= pricing.free_tier_resources {
    charge = 0.0; // Still free
} else {
    let billable = total_resources - pricing.free_tier_resources;
    charge = billable * pricing.price_per_resource;
}
```

## Security & Privacy

### PII Handling
- User IDs should be email addresses or anonymized IDs
- No sensitive data in event metadata
- GDPR-compliant data retention policies

### Access Control
- Team summaries only visible to team members or admins
- Org-wide reports require admin privileges
- Cost center reports available to finance team

### Audit Trail
All metering events are immutable and auditable:
```rust
// Every event has unique ID and timestamp
event.event_id = uuid::Uuid::new_v4().to_string();
event.timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

## Testing

### Unit Tests
```bash
cargo test --package costpilot --lib engines::metering
```

### Integration Tests
```bash
# Test PR tracking flow
cargo test --test pr_tracking_integration

# Test chargeback generation
cargo test --test chargeback_generation
```

### Load Testing
```rust
#[test]
fn test_high_volume_events() {
    let mut meter = UsageMeter::new(PricingModel::default());

    // Simulate 10,000 events
    for i in 0..10000 {
        let event = create_test_event(i);
        meter.record_event(event).unwrap();
    }

    let metrics = meter.get_metrics(0, u64::MAX);
    assert_eq!(metrics.total_events, 10000);
}
```

## Future Enhancements

### 1. Real-Time Streaming
- Kafka/Kinesis integration for real-time event processing
- Live dashboard updates
- Instant billing updates

### 2. Machine Learning
- Anomaly detection (unusual usage patterns)
- Cost forecasting (predict next month's usage)
- Team clustering (identify similar usage patterns)

### 3. Advanced Attribution
- Multi-dimensional attribution (team + project + environment)
- Weighted attribution models
- Tag-based allocation

### 4. External Integrations
- Stripe for automated billing
- AWS Marketplace for reseller billing
- Salesforce for CRM integration

### 5. Budgets & Alerts
```rust
// Team budget tracking
pub struct TeamBudget {
    team_id: String,
    monthly_limit: f64,
    current_usage: f64,
    alert_threshold: f64, // 0.8 = 80%
}

// Alert when approaching limit
if current_usage / monthly_limit > alert_threshold {
    send_alert(&team_id, "Approaching monthly budget limit")?;
}
```

## Conclusion

The Usage Metering system transforms CostPilot from a cost analysis tool into a comprehensive FinOps platform with:

- **Full visibility** into who uses what and when
- **Fair chargeback** based on actual usage
- **ROI demonstration** with cost issues prevented
- **PR-level tracking** for CI/CD billing
- **Enterprise-ready** reporting and invoicing

All while maintaining CostPilot's core principles: **deterministic**, **zero-IAM**, and **offline-capable**.
