// PR-based billing tracker for measuring CostPilot usage in CI/CD

use crate::engines::metering::usage_meter::{Attribution, UsageContext, UsageEvent, UsageEventType};
use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// PR (Pull Request) usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrUsageTracker {
    /// Repository identifier
    pub repository: String,

    /// PR number
    pub pr_number: u32,

    /// PR author
    pub author: String,

    /// PR title
    pub title: String,

    /// Branch name
    pub branch: String,

    /// Usage events for this PR
    pub events: Vec<UsageEvent>,

    /// Total resources analyzed
    pub total_resources: u32,

    /// Total cost impact detected
    pub total_cost_impact: f64,

    /// Total analysis time
    pub total_duration_ms: u64,

    /// Number of commits analyzed
    pub commits_analyzed: u32,

    /// PR status
    pub status: PrStatus,
}

/// PR lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrStatus {
    /// PR is open
    Open,

    /// PR has been merged
    Merged,

    /// PR was closed without merging
    Closed,

    /// PR is in draft state
    Draft,
}

/// PR usage summary for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrUsageSummary {
    /// PR identifier
    pub pr_number: u32,

    /// Repository
    pub repository: String,

    /// Author and team attribution
    pub attribution: Attribution,

    /// Number of scans performed
    pub scan_count: u32,

    /// Resources analyzed
    pub resources_analyzed: u32,

    /// Cost issues detected
    pub issues_detected: u32,

    /// Cost impact prevented (if issues were fixed)
    pub cost_prevented: f64,

    /// Billable units for this PR
    pub billable_units: u32,

    /// Estimated charge
    pub estimated_charge: f64,

    /// ROI (return on investment)
    pub roi: Option<f64>,
}

/// CI/CD usage tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiUsageTracker {
    /// Tracked PRs
    prs: HashMap<u32, PrUsageTracker>,

    /// Repository identifier
    repository: String,
}

impl CiUsageTracker {
    /// Create new CI usage tracker
    pub fn new(repository: String) -> Self {
        Self {
            prs: HashMap::new(),
            repository,
        }
    }

    /// Start tracking a PR
    pub fn track_pr(
        &mut self,
        pr_number: u32,
        author: String,
        title: String,
        branch: String,
    ) -> Result<()> {
        let tracker = PrUsageTracker {
            repository: self.repository.clone(),
            pr_number,
            author,
            title,
            branch,
            events: Vec::new(),
            total_resources: 0,
            total_cost_impact: 0.0,
            total_duration_ms: 0,
            commits_analyzed: 0,
            status: PrStatus::Open,
        };

        self.prs.insert(pr_number, tracker);
        Ok(())
    }

    /// Record usage event for a PR
    pub fn record_pr_event(&mut self, pr_number: u32, event: UsageEvent) -> Result<()> {
        let tracker = self.prs.get_mut(&pr_number).ok_or_else(|| {
            CostPilotError::new(
                "PR_001",
                ErrorCategory::NotFound,
                format!("PR {} not being tracked", pr_number),
            )
        })?;

        tracker.total_resources += event.resources_analyzed;
        tracker.total_cost_impact += event.cost_impact;
        tracker.total_duration_ms += event.duration_ms;

        if event.context.commit.is_some() {
            tracker.commits_analyzed += 1;
        }

        tracker.events.push(event);
        Ok(())
    }

    /// Update PR status
    pub fn update_pr_status(&mut self, pr_number: u32, status: PrStatus) -> Result<()> {
        let tracker = self.prs.get_mut(&pr_number).ok_or_else(|| {
            CostPilotError::new(
                "PR_002",
                ErrorCategory::NotFound,
                format!("PR {} not being tracked", pr_number),
            )
        })?;

        tracker.status = status;
        Ok(())
    }

    /// Get PR summary for billing
    pub fn get_pr_summary(
        &self,
        pr_number: u32,
        price_per_resource: f64,
    ) -> Result<PrUsageSummary> {
        let tracker = self.prs.get(&pr_number).ok_or_else(|| {
            CostPilotError::new(
                "PR_003",
                ErrorCategory::NotFound,
                format!("PR {} not found", pr_number),
            )
        })?;

        // Get attribution from first event (or use PR author)
        let attribution = tracker
            .events
            .first()
            .map(|e| e.attribution.clone())
            .unwrap_or_else(|| Attribution {
                user_id: tracker.author.clone(),
                team_id: None,
                org_id: None,
                cost_center: None,
                project_id: Some(self.repository.clone()),
            });

        let scan_count = tracker
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type,
                    UsageEventType::Scan | UsageEventType::PlanAnalysis
                )
            })
            .count() as u32;

        // Count issues detected (would come from event metadata in practice)
        let issues_detected = tracker
            .events
            .iter()
            .filter_map(|e| e.metadata.get("issues_detected"))
            .filter_map(|s| s.parse::<u32>().ok())
            .sum();

        let billable_units = tracker.total_resources;
        let estimated_charge = billable_units as f64 * price_per_resource;

        // Calculate ROI if cost was prevented
        let roi = if estimated_charge > 0.0 && tracker.total_cost_impact > 0.0 {
            Some(tracker.total_cost_impact / estimated_charge)
        } else {
            None
        };

        Ok(PrUsageSummary {
            pr_number,
            repository: self.repository.clone(),
            attribution,
            scan_count,
            resources_analyzed: tracker.total_resources,
            issues_detected,
            cost_prevented: tracker.total_cost_impact,
            billable_units,
            estimated_charge,
            roi,
        })
    }

    /// Get all PR summaries for a time period
    pub fn get_all_summaries(
        &self,
        start: u64,
        end: u64,
        price_per_resource: f64,
    ) -> Vec<PrUsageSummary> {
        self.prs
            .values()
            .filter(|pr| {
                pr.events
                    .iter()
                    .any(|e| e.timestamp >= start && e.timestamp <= end)
            })
            .filter_map(|pr| self.get_pr_summary(pr.pr_number, price_per_resource).ok())
            .collect()
    }

    /// Generate PR usage report
    pub fn generate_report(&self, start: u64, end: u64) -> PrUsageReport {
        let summaries = self.get_all_summaries(start, end, 0.01);

        let total_prs = summaries.len() as u32;
        let total_scans = summaries.iter().map(|s| s.scan_count).sum();
        let total_resources = summaries.iter().map(|s| s.resources_analyzed).sum();
        let total_cost_prevented: f64 = summaries.iter().map(|s| s.cost_prevented).sum();
        let total_estimated_charge: f64 = summaries.iter().map(|s| s.estimated_charge).sum();

        let avg_resources_per_pr = if total_prs > 0 {
            total_resources / total_prs
        } else {
            0
        };

        let avg_roi = if !summaries.is_empty() {
            let total_roi: f64 = summaries.iter().filter_map(|s| s.roi).sum();
            let roi_count = summaries.iter().filter(|s| s.roi.is_some()).count();
            if roi_count > 0 {
                Some(total_roi / roi_count as f64)
            } else {
                None
            }
        } else {
            None
        };

        PrUsageReport {
            period_start: start,
            period_end: end,
            total_prs,
            total_scans,
            total_resources,
            total_cost_prevented,
            total_estimated_charge,
            avg_resources_per_pr,
            avg_roi,
            pr_summaries: summaries,
        }
    }
}

/// PR usage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrUsageReport {
    /// Time period
    pub period_start: u64,
    pub period_end: u64,

    /// Total PRs analyzed
    pub total_prs: u32,

    /// Total scans performed
    pub total_scans: u32,

    /// Total resources analyzed
    pub total_resources: u32,

    /// Total cost prevented
    pub total_cost_prevented: f64,

    /// Total estimated charge
    pub total_estimated_charge: f64,

    /// Average resources per PR
    pub avg_resources_per_pr: u32,

    /// Average ROI
    pub avg_roi: Option<f64>,

    /// Individual PR summaries
    pub pr_summaries: Vec<PrUsageSummary>,
}

impl PrUsageReport {
    /// Format report as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("📊 PR Usage Report\n");
        output.push_str("==================\n\n");

        output.push_str(&format!(
            "Period: {} - {}\n\n",
            self.period_start, self.period_end
        ));

        output.push_str("Summary:\n");
        output.push_str(&format!("  Total PRs: {}\n", self.total_prs));
        output.push_str(&format!("  Total Scans: {}\n", self.total_scans));
        output.push_str(&format!("  Resources Analyzed: {}\n", self.total_resources));
        output.push_str(&format!(
            "  Cost Prevented: ${:.2}\n",
            self.total_cost_prevented
        ));
        output.push_str(&format!(
            "  Estimated Charge: ${:.2}\n",
            self.total_estimated_charge
        ));
        output.push_str(&format!(
            "  Avg Resources/PR: {}\n",
            self.avg_resources_per_pr
        ));

        if let Some(roi) = self.avg_roi {
            output.push_str(&format!("  Average ROI: {:.1}x\n", roi));
        }

        output.push_str("\nTop PRs by Resources Analyzed:\n");
        let mut sorted = self.pr_summaries.clone();
        sorted.sort_by(|a, b| b.resources_analyzed.cmp(&a.resources_analyzed));

        for (i, pr) in sorted.iter().take(10).enumerate() {
            output.push_str(&format!(
                "  {}. PR #{} - {} resources (${:.2})\n",
                i + 1,
                pr.pr_number,
                pr.resources_analyzed,
                pr.estimated_charge
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_event(resources: u32, cost_impact: f64) -> UsageEvent {
        UsageEvent {
            event_id: "test".to_string(),
            timestamp: 1000,
            event_type: UsageEventType::Scan,
            attribution: Attribution {
                user_id: "user1".to_string(),
                team_id: Some("team1".to_string()),
                org_id: Some("org1".to_string()),
                cost_center: None,
                project_id: Some("proj1".to_string()),
            },
            resources_analyzed: resources,
            cost_impact,
            duration_ms: 500,
            context: UsageContext {
                repository: "test/repo".to_string(),
                branch: Some("feature/test".to_string()),
                commit: Some("abc123".to_string()),
                pr_number: Some(1),
                ci_system: Some("github-actions".to_string()),
                environment: Some("dev".to_string()),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_track_pr() {
        let mut tracker = CiUsageTracker::new("test/repo".to_string());

        tracker
            .track_pr(
                1,
                "user1".to_string(),
                "Test PR".to_string(),
                "feature/test".to_string(),
            )
            .unwrap();

        let event = create_test_event(100, 1000.0);
        tracker.record_pr_event(1, event).unwrap();

        let summary = tracker.get_pr_summary(1, 0.01).unwrap();
        assert_eq!(summary.pr_number, 1);
        assert_eq!(summary.resources_analyzed, 100);
        assert_eq!(summary.estimated_charge, 1.0);
    }

    #[test]
    fn test_pr_roi_calculation() {
        let mut tracker = CiUsageTracker::new("test/repo".to_string());

        tracker
            .track_pr(
                1,
                "user1".to_string(),
                "Test PR".to_string(),
                "feature/test".to_string(),
            )
            .unwrap();

        // Detected $5000 in cost issues, charged $1.00
        let event = create_test_event(100, 5000.0);
        tracker.record_pr_event(1, event).unwrap();

        let summary = tracker.get_pr_summary(1, 0.01).unwrap();
        assert!(summary.roi.is_some());
        assert!(summary.roi.unwrap() > 4999.0); // ROI of 5000x
    }

    #[test]
    fn test_usage_report() {
        let mut tracker = CiUsageTracker::new("test/repo".to_string());

        tracker
            .track_pr(
                1,
                "user1".to_string(),
                "PR 1".to_string(),
                "branch1".to_string(),
            )
            .unwrap();
        tracker
            .track_pr(
                2,
                "user2".to_string(),
                "PR 2".to_string(),
                "branch2".to_string(),
            )
            .unwrap();

        tracker
            .record_pr_event(1, create_test_event(100, 1000.0))
            .unwrap();
        tracker
            .record_pr_event(2, create_test_event(200, 2000.0))
            .unwrap();

        let report = tracker.generate_report(0, 2000);
        assert_eq!(report.total_prs, 2);
        assert_eq!(report.total_resources, 300);
        assert_eq!(report.total_cost_prevented, 3000.0);
    }
}
