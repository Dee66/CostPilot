// Usage metering and attribution system for team chargeback and billing

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Usage event representing a CostPilot analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Unique event ID
    pub event_id: String,

    /// Timestamp (Unix epoch)
    pub timestamp: u64,

    /// Event type
    pub event_type: UsageEventType,

    /// User/team attribution
    pub attribution: Attribution,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Total estimated cost impact
    pub cost_impact: f64,

    /// Analysis duration in milliseconds
    pub duration_ms: u64,

    /// Repository/project context
    pub context: UsageContext,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Type of usage event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UsageEventType {
    /// Workspace scan
    Scan,

    /// Terraform plan analysis
    PlanAnalysis,

    /// Policy evaluation
    PolicyCheck,

    /// SLO compliance check
    SloCheck,

    /// Autofix generation
    AutofixGeneration,

    /// Dependency mapping
    DependencyMap,

    /// Trend analysis
    TrendAnalysis,

    /// Advanced prediction (probabilistic)
    AdvancedPrediction,
}

/// Attribution information for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    /// User identifier (email, username, etc.)
    pub user_id: String,

    /// Team identifier
    pub team_id: Option<String>,

    /// Organization identifier
    pub org_id: Option<String>,

    /// Cost center or department
    pub cost_center: Option<String>,

    /// Project identifier
    pub project_id: Option<String>,
}

/// Context for usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageContext {
    /// Repository URL or identifier
    pub repository: String,

    /// Branch name
    pub branch: Option<String>,

    /// Commit SHA
    pub commit: Option<String>,

    /// PR number (if applicable)
    pub pr_number: Option<u32>,

    /// CI/CD system (GitHub Actions, GitLab CI, etc.)
    pub ci_system: Option<String>,

    /// Environment (dev, staging, prod)
    pub environment: Option<String>,
}

/// Usage metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Time period for metrics
    pub period_start: u64,
    pub period_end: u64,

    /// Total events
    pub total_events: u32,

    /// Events by type
    pub events_by_type: HashMap<UsageEventType, u32>,

    /// Total resources analyzed
    pub total_resources: u32,

    /// Total cost impact detected
    pub total_cost_impact: f64,

    /// Average analysis duration
    pub avg_duration_ms: u64,

    /// Unique users
    pub unique_users: u32,

    /// Unique teams
    pub unique_teams: u32,
}

/// Team usage summary for chargeback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUsageSummary {
    /// Team identifier
    pub team_id: String,

    /// Team name
    pub team_name: String,

    /// Time period
    pub period_start: u64,
    pub period_end: u64,

    /// Total events
    pub total_events: u32,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Cost impact detected
    pub cost_impact_detected: f64,

    /// Billable units (based on pricing model)
    pub billable_units: u32,

    /// Estimated charge
    pub estimated_charge: f64,

    /// Top users by usage
    pub top_users: Vec<UserUsage>,

    /// Top projects by usage
    pub top_projects: Vec<ProjectUsage>,
}

/// User usage within a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsage {
    pub user_id: String,
    pub events: u32,
    pub resources_analyzed: u32,
    pub percentage_of_team: f64,
}

/// Project usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUsage {
    pub project_id: String,
    pub events: u32,
    pub resources_analyzed: u32,
    pub cost_impact: f64,
}

/// Pricing model for usage billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    /// Pricing tier
    pub tier: PricingTier,

    /// Price per resource analyzed
    pub price_per_resource: f64,

    /// Price per scan event
    pub price_per_scan: f64,

    /// Price per advanced analysis
    pub price_per_advanced: f64,

    /// Monthly minimum charge
    pub monthly_minimum: f64,

    /// Free tier included resources
    pub free_tier_resources: u32,
}

/// Pricing tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingTier {
    /// Free tier - limited usage
    Free,

    /// Solo tier - individual developers
    Solo,

    /// Pro tier - small teams
    Pro,

    /// Enterprise tier - large organizations
    Enterprise,
}

/// Usage metering system
pub struct UsageMeter {
    /// Storage backend for events
    events: Vec<UsageEvent>,

    /// Pricing model
    pricing: PricingModel,
}

impl UsageMeter {
    /// Create new usage meter
    pub fn new(pricing: PricingModel) -> Self {
        Self {
            events: Vec::new(),
            pricing,
        }
    }

    /// Load usage meter from file
    pub fn load_from_file(_path: &std::path::Path, pricing: PricingModel) -> Result<Self> {
        // Stub: return new meter for now
        Ok(Self::new(pricing))
    }

    /// Record usage event
    pub fn record_event(&mut self, event: UsageEvent) -> Result<()> {
        self.events.push(event);
        Ok(())
    }

    /// Get metrics for time period
    pub fn get_metrics(&self, start: u64, end: u64) -> UsageMetrics {
        let period_events: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect();

        let mut events_by_type: HashMap<UsageEventType, u32> = HashMap::new();
        let mut unique_users = std::collections::HashSet::new();
        let mut unique_teams = std::collections::HashSet::new();

        let mut total_resources = 0;
        let mut total_cost_impact = 0.0;
        let mut total_duration = 0;

        for event in &period_events {
            *events_by_type.entry(event.event_type).or_insert(0) += 1;
            unique_users.insert(event.attribution.user_id.clone());
            if let Some(team) = &event.attribution.team_id {
                unique_teams.insert(team.clone());
            }
            total_resources += event.resources_analyzed;
            total_cost_impact += event.cost_impact;
            total_duration += event.duration_ms;
        }

        let avg_duration_ms = if !period_events.is_empty() {
            total_duration / period_events.len() as u64
        } else {
            0
        };

        UsageMetrics {
            period_start: start,
            period_end: end,
            total_events: period_events.len() as u32,
            events_by_type,
            total_resources,
            total_cost_impact,
            avg_duration_ms,
            unique_users: unique_users.len() as u32,
            unique_teams: unique_teams.len() as u32,
        }
    }

    /// Generate team usage summary for chargeback
    pub fn team_summary(&self, team_id: &str, start: u64, end: u64) -> Result<TeamUsageSummary> {
        let team_events: Vec<_> = self
            .events
            .iter()
            .filter(|e| {
                e.timestamp >= start
                    && e.timestamp <= end
                    && e.attribution.team_id.as_deref() == Some(team_id)
            })
            .collect();

        if team_events.is_empty() {
            return Err(CostPilotError::new(
                "METER_001",
                ErrorCategory::NotFound,
                format!("No usage events found for team {}", team_id),
            ));
        }

        // Aggregate by user
        let mut user_stats: HashMap<String, (u32, u32)> = HashMap::new();
        for event in &team_events {
            let entry = user_stats
                .entry(event.attribution.user_id.clone())
                .or_insert((0, 0));
            entry.0 += 1; // events
            entry.1 += event.resources_analyzed; // resources
        }

        // Aggregate by project
        let mut project_stats: HashMap<String, (u32, u32, f64)> = HashMap::new();
        for event in &team_events {
            if let Some(project) = &event.attribution.project_id {
                let entry = project_stats.entry(project.clone()).or_insert((0, 0, 0.0));
                entry.0 += 1; // events
                entry.1 += event.resources_analyzed; // resources
                entry.2 += event.cost_impact; // cost impact
            }
        }

        let total_events = team_events.len() as u32;
        let total_resources: u32 = team_events.iter().map(|e| e.resources_analyzed).sum();
        let total_cost_impact: f64 = team_events.iter().map(|e| e.cost_impact).sum();

        // Calculate billable units and charge
        let (billable_units, estimated_charge) =
            self.calculate_charge(total_resources, total_events);

        // Top users
        let mut top_users: Vec<_> = user_stats
            .into_iter()
            .map(|(user_id, (events, resources))| UserUsage {
                user_id,
                events,
                resources_analyzed: resources,
                percentage_of_team: (events as f64 / total_events as f64) * 100.0,
            })
            .collect();
        top_users.sort_by(|a, b| b.events.cmp(&a.events));
        top_users.truncate(10);

        // Top projects
        let mut top_projects: Vec<_> = project_stats
            .into_iter()
            .map(
                |(project_id, (events, resources, cost_impact))| ProjectUsage {
                    project_id,
                    events,
                    resources_analyzed: resources,
                    cost_impact,
                },
            )
            .collect();
        top_projects.sort_by(|a, b| b.resources_analyzed.cmp(&a.resources_analyzed));
        top_projects.truncate(10);

        Ok(TeamUsageSummary {
            team_id: team_id.to_string(),
            team_name: team_id.to_string(), // TODO: Lookup from metadata
            period_start: start,
            period_end: end,
            total_events,
            resources_analyzed: total_resources,
            cost_impact_detected: total_cost_impact,
            billable_units,
            estimated_charge,
            top_users,
            top_projects,
        })
    }

    /// Calculate billable units and charge
    fn calculate_charge(&self, resources: u32, events: u32) -> (u32, f64) {
        // Apply free tier
        let billable_resources = resources.saturating_sub(self.pricing.free_tier_resources);

        // Calculate charge
        let resource_charge = billable_resources as f64 * self.pricing.price_per_resource;
        let event_charge = events as f64 * self.pricing.price_per_scan;
        let total_charge = (resource_charge + event_charge).max(self.pricing.monthly_minimum);

        (billable_resources, total_charge)
    }

    /// Export usage data for external billing systems
    pub fn export_billing_data(&self, start: u64, end: u64) -> Result<BillingExport> {
        let period_events: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect();

        // Group by team
        let mut team_charges: HashMap<String, f64> = HashMap::new();
        let mut team_resources: HashMap<String, u32> = HashMap::new();

        for event in &period_events {
            if let Some(team_id) = &event.attribution.team_id {
                *team_resources.entry(team_id.clone()).or_insert(0) += event.resources_analyzed;
            }
        }

        for (team_id, resources) in &team_resources {
            let events_count = period_events
                .iter()
                .filter(|e| e.attribution.team_id.as_deref() == Some(team_id))
                .count() as u32;
            let (_, charge) = self.calculate_charge(*resources, events_count);
            team_charges.insert(team_id.clone(), charge);
        }

        Ok(BillingExport {
            period_start: start,
            period_end: end,
            total_events: period_events.len() as u32,
            total_resources: team_resources.values().sum(),
            team_charges,
            events: period_events,
        })
    }
}

/// Billing export data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingExport {
    pub period_start: u64,
    pub period_end: u64,
    pub total_events: u32,
    pub total_resources: u32,
    pub team_charges: HashMap<String, f64>,
    pub events: Vec<UsageEvent>,
}

impl Default for PricingModel {
    fn default() -> Self {
        // Default Pro tier pricing
        Self {
            tier: PricingTier::Pro,
            price_per_resource: 0.01,  // $0.01 per resource
            price_per_scan: 0.05,      // $0.05 per scan
            price_per_advanced: 0.10,  // $0.10 per advanced analysis
            monthly_minimum: 49.0,     // $49/month minimum
            free_tier_resources: 1000, // 1000 resources free
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(user_id: &str, team_id: Option<&str>, resources: u32) -> UsageEvent {
        UsageEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: 1000,
            event_type: UsageEventType::Scan,
            attribution: Attribution {
                user_id: user_id.to_string(),
                team_id: team_id.map(String::from),
                org_id: Some("org1".to_string()),
                cost_center: None,
                project_id: Some("proj1".to_string()),
            },
            resources_analyzed: resources,
            cost_impact: 1000.0,
            duration_ms: 500,
            context: UsageContext {
                repository: "test/repo".to_string(),
                branch: Some("main".to_string()),
                commit: None,
                pr_number: None,
                ci_system: Some("github-actions".to_string()),
                environment: Some("dev".to_string()),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_record_and_retrieve_metrics() {
        let pricing = PricingModel::default();
        let mut meter = UsageMeter::new(pricing);

        meter
            .record_event(create_test_event("user1", Some("team1"), 100))
            .unwrap();
        meter
            .record_event(create_test_event("user2", Some("team1"), 200))
            .unwrap();

        let metrics = meter.get_metrics(0, 2000);
        assert_eq!(metrics.total_events, 2);
        assert_eq!(metrics.total_resources, 300);
        assert_eq!(metrics.unique_users, 2);
    }

    #[test]
    fn test_team_summary() {
        let pricing = PricingModel::default();
        let mut meter = UsageMeter::new(pricing);

        meter
            .record_event(create_test_event("user1", Some("team1"), 500))
            .unwrap();
        meter
            .record_event(create_test_event("user2", Some("team1"), 700))
            .unwrap();
        meter
            .record_event(create_test_event("user3", Some("team2"), 300))
            .unwrap();

        let summary = meter.team_summary("team1", 0, 2000).unwrap();
        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.resources_analyzed, 1200);
        assert!(summary.estimated_charge > 0.0);
    }

    #[test]
    fn test_free_tier_deduction() {
        let pricing = PricingModel {
            free_tier_resources: 100,
            price_per_resource: 0.01,
            price_per_scan: 0.0,
            monthly_minimum: 0.0,
            ..Default::default()
        };
        let meter = UsageMeter::new(pricing);

        let (billable, charge) = meter.calculate_charge(150, 1);
        assert_eq!(billable, 50); // 150 - 100 free tier
        assert_eq!(charge, 0.50); // 50 * $0.01
    }

    #[test]
    fn test_monthly_minimum() {
        let pricing = PricingModel {
            monthly_minimum: 49.0,
            price_per_resource: 0.01,
            price_per_scan: 0.0,
            ..Default::default()
        };
        let meter = UsageMeter::new(pricing);

        let (_, charge) = meter.calculate_charge(10, 1);
        assert_eq!(charge, 49.0); // Below minimum, charged minimum
    }
}
