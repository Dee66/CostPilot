// Chargeback reporting for team cost attribution

use crate::engines::metering::usage_meter::TeamUsageSummary;
use crate::engines::shared::error_model::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chargeback report for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargebackReport {
    /// Report period
    pub period_start: u64,
    pub period_end: u64,

    /// Organization identifier
    pub org_id: String,

    /// Total charge for organization
    pub total_charge: f64,

    /// Team breakdowns
    pub team_charges: Vec<TeamChargeback>,

    /// Cost center breakdowns
    pub cost_center_charges: HashMap<String, f64>,

    /// Top cost drivers
    pub top_cost_drivers: Vec<CostDriver>,
}

/// Team chargeback details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamChargeback {
    /// Team identifier
    pub team_id: String,

    /// Team name
    pub team_name: String,

    /// Cost center
    pub cost_center: Option<String>,

    /// Total charge for team
    pub charge: f64,

    /// Percentage of total org charge
    pub percentage_of_org: f64,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Events performed
    pub events: u32,

    /// Cost impact detected (value delivered)
    pub value_delivered: f64,

    /// ROI (value delivered / charge)
    pub roi: f64,

    /// Top users in team
    pub top_users: Vec<UserChargeback>,

    /// Top projects
    pub top_projects: Vec<ProjectChargeback>,
}

/// User chargeback within team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChargeback {
    /// User identifier
    pub user_id: String,

    /// User name
    pub user_name: String,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Events
    pub events: u32,

    /// Allocated charge
    pub charge: f64,

    /// Percentage of team charge
    pub percentage_of_team: f64,
}

/// Project chargeback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectChargeback {
    /// Project identifier
    pub project_id: String,

    /// Project name
    pub project_name: String,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Allocated charge
    pub charge: f64,

    /// Cost impact detected
    pub cost_impact: f64,
}

/// Cost driver identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDriver {
    /// Driver type (team, user, project, resource_type)
    pub driver_type: String,

    /// Driver identifier
    pub driver_id: String,

    /// Charge amount
    pub charge: f64,

    /// Percentage of total
    pub percentage: f64,

    /// Description
    pub description: String,
}

/// Chargeback report builder
pub struct ChargebackReportBuilder {
    org_id: String,
    period_start: u64,
    period_end: u64,
    team_summaries: Vec<TeamUsageSummary>,
}

impl ChargebackReportBuilder {
    /// Create new chargeback report builder
    pub fn new(org_id: String, period_start: u64, period_end: u64) -> Self {
        Self {
            org_id,
            period_start,
            period_end,
            team_summaries: Vec::new(),
        }
    }

    /// Add team usage summary
    pub fn add_team(&mut self, summary: TeamUsageSummary) {
        self.team_summaries.push(summary);
    }

    /// Build chargeback report
    pub fn build(self) -> Result<ChargebackReport> {
        let total_charge: f64 = self.team_summaries.iter().map(|s| s.estimated_charge).sum();

        // Build team chargebacks
        let mut team_charges = Vec::new();
        for summary in &self.team_summaries {
            let percentage_of_org = if total_charge > 0.0 {
                (summary.estimated_charge / total_charge) * 100.0
            } else {
                0.0
            };

            let roi = if summary.estimated_charge > 0.0 {
                summary.cost_impact_detected / summary.estimated_charge
            } else {
                0.0
            };

            // Convert user usage to user chargeback
            let top_users = summary
                .top_users
                .iter()
                .map(|u| {
                    let user_charge = (u.percentage_of_team / 100.0) * summary.estimated_charge;
                    UserChargeback {
                        user_id: u.user_id.clone(),
                        user_name: u.user_id.clone(), // TODO: Lookup real name
                        resources_analyzed: u.resources_analyzed,
                        events: u.events,
                        charge: user_charge,
                        percentage_of_team: u.percentage_of_team,
                    }
                })
                .collect();

            // Convert project usage to project chargeback
            let total_project_resources: u32 = summary
                .top_projects
                .iter()
                .map(|p| p.resources_analyzed)
                .sum();

            let top_projects = summary
                .top_projects
                .iter()
                .map(|p| {
                    let project_percentage = if total_project_resources > 0 {
                        (p.resources_analyzed as f64 / total_project_resources as f64) * 100.0
                    } else {
                        0.0
                    };
                    let project_charge = (project_percentage / 100.0) * summary.estimated_charge;

                    ProjectChargeback {
                        project_id: p.project_id.clone(),
                        project_name: p.project_id.clone(), // TODO: Lookup real name
                        resources_analyzed: p.resources_analyzed,
                        charge: project_charge,
                        cost_impact: p.cost_impact,
                    }
                })
                .collect();

            team_charges.push(TeamChargeback {
                team_id: summary.team_id.clone(),
                team_name: summary.team_name.clone(),
                cost_center: None, // TODO: Get from metadata
                charge: summary.estimated_charge,
                percentage_of_org,
                resources_analyzed: summary.resources_analyzed,
                events: summary.total_events,
                value_delivered: summary.cost_impact_detected,
                roi,
                top_users,
                top_projects,
            });
        }

        // Sort teams by charge
        team_charges.sort_by(|a, b| b.charge.partial_cmp(&a.charge).unwrap());

        // Build cost center breakdown (TODO: implement)
        let cost_center_charges = HashMap::new();

        // Identify top cost drivers
        let mut top_cost_drivers = Vec::new();

        // Top teams by charge
        for (i, team) in team_charges.iter().take(5).enumerate() {
            top_cost_drivers.push(CostDriver {
                driver_type: "team".to_string(),
                driver_id: team.team_id.clone(),
                charge: team.charge,
                percentage: team.percentage_of_org,
                description: format!("Team: {} (Rank #{})", team.team_name, i + 1),
            });
        }

        Ok(ChargebackReport {
            period_start: self.period_start,
            period_end: self.period_end,
            org_id: self.org_id,
            total_charge,
            team_charges,
            cost_center_charges,
            top_cost_drivers,
        })
    }
}

impl ChargebackReport {
    /// Format report as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("💰 Chargeback Report\n");
        output.push_str("====================\n\n");

        output.push_str(&format!("Organization: {}\n", self.org_id));
        output.push_str(&format!(
            "Period: {} - {}\n\n",
            self.period_start, self.period_end
        ));

        output.push_str(&format!("Total Charge: ${:.2}\n\n", self.total_charge));

        output.push_str("Team Breakdown:\n");
        for team in &self.team_charges {
            output.push_str(&format!(
                "  {} - ${:.2} ({:.1}%)\n",
                team.team_name, team.charge, team.percentage_of_org
            ));
            output.push_str(&format!("    Resources: {}\n", team.resources_analyzed));
            output.push_str(&format!("    Events: {}\n", team.events));
            output.push_str(&format!(
                "    Value Delivered: ${:.2}\n",
                team.value_delivered
            ));
            output.push_str(&format!("    ROI: {:.1}x\n\n", team.roi));
        }

        output.push_str("Top Cost Drivers:\n");
        for driver in &self.top_cost_drivers {
            output.push_str(&format!(
                "  {} - ${:.2} ({:.1}%)\n",
                driver.description, driver.charge, driver.percentage
            ));
        }

        output
    }

    /// Export to CSV format
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("Team,Charge,Percentage,Resources,Events,Value Delivered,ROI\n");

        // Data rows
        for team in &self.team_charges {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                team.team_name,
                team.charge,
                team.percentage_of_org,
                team.resources_analyzed,
                team.events,
                team.value_delivered,
                team.roi
            ));
        }

        csv
    }

    /// Generate invoice-style report
    pub fn generate_invoice(&self, team_id: &str) -> Option<String> {
        let team = self.team_charges.iter().find(|t| t.team_id == team_id)?;

        let mut invoice = String::new();

        invoice.push_str("┌────────────────────────────────────────────┐\n");
        invoice.push_str("│           CostPilot Invoice                │\n");
        invoice.push_str("└────────────────────────────────────────────┘\n\n");

        invoice.push_str(&format!("Organization: {}\n", self.org_id));
        invoice.push_str(&format!("Team: {}\n", team.team_name));
        invoice.push_str(&format!(
            "Period: {} - {}\n\n",
            self.period_start, self.period_end
        ));

        invoice.push_str("Usage Summary:\n");
        invoice.push_str(&format!(
            "  Resources Analyzed: {}\n",
            team.resources_analyzed
        ));
        invoice.push_str(&format!("  Events Performed: {}\n\n", team.events));

        invoice.push_str("Charges:\n");
        invoice.push_str(&format!("  Total: ${:.2}\n\n", team.charge));

        invoice.push_str("Value Delivered:\n");
        invoice.push_str(&format!(
            "  Cost Issues Detected: ${:.2}\n",
            team.value_delivered
        ));
        invoice.push_str(&format!("  ROI: {:.1}x return on investment\n\n", team.roi));

        if !team.top_users.is_empty() {
            invoice.push_str("Top Users:\n");
            for user in team.top_users.iter().take(5) {
                invoice.push_str(&format!(
                    "  {} - {} resources (${:.2})\n",
                    user.user_name, user.resources_analyzed, user.charge
                ));
            }
        }

        Some(invoice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::metering::usage_meter::{ProjectUsage, UserUsage};

    fn create_test_summary(team_id: &str, charge: f64, resources: u32) -> TeamUsageSummary {
        TeamUsageSummary {
            team_id: team_id.to_string(),
            team_name: format!("Team {}", team_id),
            period_start: 0,
            period_end: 1000,
            total_events: 10,
            resources_analyzed: resources,
            cost_impact_detected: charge * 100.0, // High ROI
            billable_units: resources,
            estimated_charge: charge,
            top_users: vec![UserUsage {
                user_id: "user1".to_string(),
                events: 5,
                resources_analyzed: resources / 2,
                percentage_of_team: 50.0,
            }],
            top_projects: vec![ProjectUsage {
                project_id: "proj1".to_string(),
                events: 10,
                resources_analyzed: resources,
                cost_impact: charge * 100.0,
            }],
        }
    }

    #[test]
    fn test_chargeback_report() {
        let mut builder = ChargebackReportBuilder::new("org1".to_string(), 0, 1000);

        builder.add_team(create_test_summary("team1", 100.0, 1000));
        builder.add_team(create_test_summary("team2", 200.0, 2000));

        let report = builder.build().unwrap();

        assert_eq!(report.total_charge, 300.0);
        assert_eq!(report.team_charges.len(), 2);

        // Team 2 should be first (higher charge)
        assert_eq!(report.team_charges[0].team_id, "team2");
        assert!((report.team_charges[0].percentage_of_org - 66.66666666666667).abs() < 1e-6);
    }

    #[test]
    fn test_invoice_generation() {
        let mut builder = ChargebackReportBuilder::new("org1".to_string(), 0, 1000);
        builder.add_team(create_test_summary("team1", 150.0, 1500));

        let report = builder.build().unwrap();
        let invoice = report.generate_invoice("team1");

        assert!(invoice.is_some());
        assert!(invoice.unwrap().contains("$150.00"));
    }

    #[test]
    fn test_csv_export() {
        let mut builder = ChargebackReportBuilder::new("org1".to_string(), 0, 1000);
        builder.add_team(create_test_summary("team1", 50.0, 500));

        let report = builder.build().unwrap();
        let csv = report.to_csv();

        assert!(csv.contains("Team,Charge,Percentage"));
        assert!(csv.contains("Team team1"));
    }
}
