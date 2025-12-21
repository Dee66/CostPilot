/// SLO burn rate prediction and time-to-breach analysis
///
/// This module provides enterprise-grade burn rate prediction using linear
/// regression on historical cost snapshots. It predicts when SLO budgets
/// will be exhausted based on current spend velocity.
///
/// # Key Features
///
/// - Linear regression on historical snapshots
/// - Time-to-breach prediction
/// - Burn risk classification (Low/Medium/High/Critical)
/// - Multi-SLO analysis
/// - Confidence scoring
///
/// # Zero-Network Guarantee
///
/// All burn rate calculations are deterministic and require no network access.
/// Historical data comes from local snapshot files only.
use super::slo_types::{Slo, SloType};
use crate::engines::trend::snapshot_types::CostSnapshot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Burn rate analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnAnalysis {
    /// SLO being analyzed
    pub slo_id: String,

    /// SLO name
    pub slo_name: String,

    /// Current spend rate (cost per day)
    pub burn_rate: f64,

    /// Projected end-of-period cost
    pub projected_cost: f64,

    /// SLO limit
    pub slo_limit: f64,

    /// Days until SLO breach (None if no breach predicted)
    pub days_to_breach: Option<f64>,

    /// Risk level
    pub risk: BurnRisk,

    /// Confidence in prediction (0.0-1.0)
    pub confidence: f64,

    /// Linear regression slope (dollars per day)
    pub trend_slope: f64,

    /// Linear regression intercept
    pub trend_intercept: f64,

    /// R² value for regression fit quality
    pub r_squared: f64,

    /// Analysis timestamp
    pub analyzed_at: String,
}

/// Burn risk classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BurnRisk {
    /// No breach predicted
    Low,

    /// Breach possible but not imminent (>14 days)
    Medium,

    /// Breach likely within 14 days
    High,

    /// Breach imminent (<7 days)
    Critical,
}

impl BurnRisk {
    /// Get numeric severity (0-3)
    pub fn severity(&self) -> u8 {
        match self {
            BurnRisk::Low => 0,
            BurnRisk::Medium => 1,
            BurnRisk::High => 2,
            BurnRisk::Critical => 3,
        }
    }

    /// Check if action is required
    pub fn requires_action(&self) -> bool {
        matches!(self, BurnRisk::High | BurnRisk::Critical)
    }
}

/// Aggregated burn rate report for multiple SLOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnReport {
    /// Individual burn analyses
    pub analyses: Vec<BurnAnalysis>,

    /// Overall risk level (highest individual risk)
    pub overall_risk: BurnRisk,

    /// Number of SLOs analyzed
    pub total_slos: usize,

    /// Number at risk
    pub slos_at_risk: usize,

    /// Report generation timestamp
    pub generated_at: String,
}

impl BurnReport {
    /// Create a new burn report
    pub fn new(analyses: Vec<BurnAnalysis>) -> Self {
        let overall_risk = analyses
            .iter()
            .map(|a| a.risk.clone())
            .max_by_key(|r| r.severity())
            .unwrap_or(BurnRisk::Low);

        let slos_at_risk = analyses.iter().filter(|a| a.risk.requires_action()).count();

        Self {
            total_slos: analyses.len(),
            slos_at_risk,
            overall_risk,
            analyses,
            generated_at: Utc::now().to_rfc3339(),
        }
    }

    /// Check if any SLO requires immediate action
    pub fn requires_action(&self) -> bool {
        self.overall_risk.requires_action()
    }

    /// Get critical SLOs only
    pub fn critical_slos(&self) -> Vec<&BurnAnalysis> {
        self.analyses
            .iter()
            .filter(|a| a.risk == BurnRisk::Critical)
            .collect()
    }
}

/// Burn rate calculator using linear regression
pub struct BurnRateCalculator {
    /// Minimum snapshots required for analysis
    min_snapshots: usize,

    /// Minimum R² for reliable predictions
    min_r_squared: f64,
}

impl BurnRateCalculator {
    /// Create a new burn rate calculator
    pub fn new() -> Self {
        Self {
            min_snapshots: 3,
            min_r_squared: 0.7,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(min_snapshots: usize, min_r_squared: f64) -> Self {
        Self {
            min_snapshots,
            min_r_squared,
        }
    }

    /// Analyze burn rate for a single SLO
    pub fn analyze_slo(&self, slo: &Slo, snapshots: &[CostSnapshot]) -> Option<BurnAnalysis> {
        if snapshots.len() < self.min_snapshots {
            return None;
        }

        // Extract cost values based on SLO type
        let data_points = self.extract_data_points(slo, snapshots);
        if data_points.len() < self.min_snapshots {
            return None;
        }

        // Perform linear regression
        let (slope, intercept, r_squared) = self.linear_regression(&data_points);

        // Calculate confidence based on R²
        let confidence = if r_squared >= self.min_r_squared {
            r_squared
        } else {
            r_squared * 0.7 // Penalize low R²
        };

        // Get current cost and SLO limit
        let current_cost = data_points.last().unwrap().1;
        let slo_limit = slo.threshold.max_value;

        // Project to end of period (30 days from now)
        let days_ahead = 30.0;
        let current_day = data_points.last().unwrap().0;
        let projected_cost = slope * (current_day + days_ahead) + intercept;

        // Calculate days to breach
        let days_to_breach = if slope > 0.0 && current_cost < slo_limit {
            let days = (slo_limit - intercept) / slope - current_day;
            if days > 0.0 {
                Some(days)
            } else {
                None // Already exceeded
            }
        } else {
            None // No breach predicted (negative or zero slope)
        };

        // Determine risk level
        let risk = self.classify_risk(days_to_breach, current_cost, slo_limit);

        Some(BurnAnalysis {
            slo_id: slo.id.clone(),
            slo_name: slo.name.clone(),
            burn_rate: slope, // dollars per day
            projected_cost,
            slo_limit,
            days_to_breach,
            risk,
            confidence,
            trend_slope: slope,
            trend_intercept: intercept,
            r_squared,
            analyzed_at: Utc::now().to_rfc3339(),
        })
    }

    /// Analyze all SLOs and generate report
    pub fn analyze_all(&self, slos: &[Slo], snapshots: &[CostSnapshot]) -> BurnReport {
        let analyses: Vec<BurnAnalysis> = slos
            .iter()
            .filter_map(|slo| self.analyze_slo(slo, snapshots))
            .collect();

        BurnReport::new(analyses)
    }

    /// Extract (day, cost) data points for SLO from snapshots
    fn extract_data_points(&self, slo: &Slo, snapshots: &[CostSnapshot]) -> Vec<(f64, f64)> {
        let mut points = Vec::new();

        // Sort snapshots by timestamp
        let mut sorted = snapshots.to_vec();
        sorted.sort_by_key(|s| s.timestamp.clone());

        // Use first snapshot as day 0
        if sorted.is_empty() {
            return points;
        }

        let base_time = DateTime::parse_from_rfc3339(&sorted[0].timestamp)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        for snapshot in &sorted {
            let timestamp = DateTime::parse_from_rfc3339(&snapshot.timestamp)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let days = (timestamp - base_time).num_days() as f64;

            let cost = match slo.slo_type {
                SloType::MonthlyBudget => snapshot.total_monthly_cost,
                SloType::ModuleBudget => {
                    // Extract module cost
                    snapshot
                        .modules
                        .iter()
                        .find(|m| slo.target == format!("module.{}", m.1.name))
                        .map(|m| m.1.monthly_cost)
                        .unwrap_or(0.0)
                }
                SloType::ServiceBudget => {
                    // Sum service costs
                    snapshot
                        .modules
                        .iter()
                        .flat_map(|m| &m.1.services)
                        .filter(|s| slo.target.contains(&s.name))
                        .map(|s| s.monthly_cost)
                        .sum()
                }
                _ => continue, // Skip unsupported types
            };

            points.push((days, cost));
        }

        points
    }

    /// Perform linear regression on data points
    /// Returns (slope, intercept, r_squared)
    fn linear_regression(&self, points: &[(f64, f64)]) -> (f64, f64, f64) {
        let n = points.len() as f64;

        // Calculate means
        let mean_x: f64 = points.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y: f64 = points.iter().map(|(_, y)| y).sum::<f64>() / n;

        // Calculate slope
        let numerator: f64 = points
            .iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let denominator: f64 = points.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();

        let slope = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Calculate intercept
        let intercept = mean_y - slope * mean_x;

        // Calculate R²
        let ss_tot: f64 = points.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = points
            .iter()
            .map(|(x, y)| {
                let predicted = slope * x + intercept;
                (y - predicted).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, intercept, r_squared.max(0.0).min(1.0))
    }

    /// Classify risk based on time to breach
    fn classify_risk(&self, days_to_breach: Option<f64>, current: f64, limit: f64) -> BurnRisk {
        // Already exceeded
        if current >= limit {
            return BurnRisk::Critical;
        }

        match days_to_breach {
            None => BurnRisk::Low, // No breach predicted
            Some(days) if days < 7.0 => BurnRisk::Critical,
            Some(days) if days < 14.0 => BurnRisk::High,
            Some(days) if days < 30.0 => BurnRisk::Medium,
            Some(_) => BurnRisk::Low,
        }
    }
}

impl Default for BurnRateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_snapshots() -> Vec<CostSnapshot> {
        vec![
            CostSnapshot {
                id: "snap1".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                commit_hash: None,
                branch: None,
                total_monthly_cost: 1000.0,
                modules: std::collections::HashMap::from([(
                    "vpc".to_string(),
                    crate::engines::trend::ModuleCost {
                        name: "vpc".to_string(),
                        monthly_cost: 500.0,
                        resource_count: 5,
                        change_from_previous: None,
                        change_percent: None,
                        services: vec![],
                    },
                )]),
                services: std::collections::HashMap::new(),
                regressions: vec![],
                slo_violations: vec![],
                metadata: None,
            },
            CostSnapshot {
                id: "snap2".to_string(),
                timestamp: "2024-01-08T00:00:00Z".to_string(),
                commit_hash: None,
                branch: None,
                total_monthly_cost: 2000.0,
                modules: std::collections::HashMap::from([(
                    "vpc".to_string(),
                    crate::engines::trend::ModuleCost {
                        name: "vpc".to_string(),
                        monthly_cost: 1000.0,
                        resource_count: 6,
                        change_from_previous: None,
                        change_percent: None,
                        services: vec![],
                    },
                )]),
                services: std::collections::HashMap::new(),
                regressions: vec![],
                slo_violations: vec![],
                metadata: None,
            },
            CostSnapshot {
                id: "snap3".to_string(),
                timestamp: "2024-01-15T00:00:00Z".to_string(),
                commit_hash: None,
                branch: None,
                total_monthly_cost: 3000.0,
                modules: std::collections::HashMap::from([(
                    "vpc".to_string(),
                    crate::engines::trend::ModuleCost {
                        name: "vpc".to_string(),
                        monthly_cost: 1500.0,
                        resource_count: 7,
                        change_from_previous: None,
                        change_percent: None,
                        services: vec![],
                    },
                )]),
                services: std::collections::HashMap::new(),
                regressions: vec![],
                slo_violations: vec![],
                metadata: None,
            },
        ]
    }

    fn create_test_slo() -> Slo {
        Slo {
            id: "test-slo".to_string(),
            name: "Monthly Budget".to_string(),
            description: "Test SLO".to_string(),
            slo_type: SloType::MonthlyBudget,
            target: "global".to_string(),
            threshold: super::super::slo_types::SloThreshold {
                max_value: 5000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            enforcement: super::super::slo_types::EnforcementLevel::Block,
            owner: "test@example.com".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: None,
            tags: HashMap::new(),
        }
    }

    #[test]
    fn test_linear_regression() {
        let calculator = BurnRateCalculator::new();

        // Perfect linear data: y = 2x + 1
        let points = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 5.0), (3.0, 7.0)];

        let (slope, intercept, r_squared) = calculator.linear_regression(&points);

        assert!((slope - 2.0).abs() < 0.01);
        assert!((intercept - 1.0).abs() < 0.01);
        assert!((r_squared - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_burn_rate_analysis() {
        let calculator = BurnRateCalculator::new();
        let slo = create_test_slo();
        let snapshots = create_test_snapshots();

        let analysis = calculator.analyze_slo(&slo, &snapshots);

        assert!(analysis.is_some());
        let analysis = analysis.unwrap();

        // Should detect increasing burn rate
        assert!(analysis.burn_rate > 0.0);
        assert_eq!(analysis.slo_limit, 5000.0);
        assert!(analysis.projected_cost > 3000.0); // Last snapshot was 3000
    }

    #[test]
    fn test_risk_classification() {
        let calculator = BurnRateCalculator::new();

        // Critical: breach in 5 days
        let risk = calculator.classify_risk(Some(5.0), 4000.0, 5000.0);
        assert_eq!(risk, BurnRisk::Critical);

        // High: breach in 10 days
        let risk = calculator.classify_risk(Some(10.0), 3000.0, 5000.0);
        assert_eq!(risk, BurnRisk::High);

        // Medium: breach in 20 days
        let risk = calculator.classify_risk(Some(20.0), 2000.0, 5000.0);
        assert_eq!(risk, BurnRisk::Medium);

        // Low: no breach predicted
        let risk = calculator.classify_risk(None, 1000.0, 5000.0);
        assert_eq!(risk, BurnRisk::Low);

        // Critical: already exceeded
        let risk = calculator.classify_risk(Some(5.0), 6000.0, 5000.0);
        assert_eq!(risk, BurnRisk::Critical);
    }

    #[test]
    fn test_burn_report() {
        let calculator = BurnRateCalculator::new();
        let slo = create_test_slo();
        let snapshots = create_test_snapshots();

        let analysis = calculator.analyze_slo(&slo, &snapshots).unwrap();
        let report = BurnReport::new(vec![analysis]);

        assert_eq!(report.total_slos, 1);
        assert!(!report.generated_at.is_empty());
    }

    #[test]
    fn test_insufficient_data() {
        let calculator = BurnRateCalculator::new();
        let slo = create_test_slo();

        // Only 2 snapshots (need 3)
        let snapshots = vec![
            create_test_snapshots()[0].clone(),
            create_test_snapshots()[1].clone(),
        ];

        let analysis = calculator.analyze_slo(&slo, &snapshots);
        assert!(analysis.is_none());
    }

    #[test]
    fn test_burn_risk_severity() {
        assert_eq!(BurnRisk::Low.severity(), 0);
        assert_eq!(BurnRisk::Medium.severity(), 1);
        assert_eq!(BurnRisk::High.severity(), 2);
        assert_eq!(BurnRisk::Critical.severity(), 3);
    }

    #[test]
    fn test_burn_risk_requires_action() {
        assert!(!BurnRisk::Low.requires_action());
        assert!(!BurnRisk::Medium.requires_action());
        assert!(BurnRisk::High.requires_action());
        assert!(BurnRisk::Critical.requires_action());
    }
}
