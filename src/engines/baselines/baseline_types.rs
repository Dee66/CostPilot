use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::engines::shared::models::RegressionType;

/// A baseline cost expectation for a module or resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Name/identifier of the module or resource
    pub name: String,

    /// Expected monthly cost
    pub expected_monthly_cost: f64,

    /// Acceptable variance percentage (e.g., 10.0 for ±10%)
    #[serde(default = "default_variance")]
    pub acceptable_variance_percent: f64,

    /// ISO 8601 timestamp when baseline was last updated
    pub last_updated: String,

    /// Justification for this baseline
    pub justification: String,

    /// Who set or approved this baseline
    pub owner: String,

    /// Optional reference to ticket or decision document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Tags for categorization
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub tags: HashMap<String, String>,
}

fn default_variance() -> f64 {
    10.0
}

/// Container for all baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselinesConfig {
    /// Schema version
    pub version: String,

    /// Global baseline for total monthly cost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<Baseline>,

    /// Module-level baselines
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub modules: HashMap<String, Baseline>,

    /// Service-level baselines
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub services: HashMap<String, Baseline>,

    /// Configuration metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BaselineMetadata>,
}

/// Metadata about baselines configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetadata {
    /// When baselines were last reviewed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reviewed: Option<String>,

    /// Review cadence in days
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_cadence_days: Option<u32>,

    /// Team or organization owning baselines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_team: Option<String>,
}

/// Result of baseline comparison
#[derive(Debug, Clone, PartialEq)]
pub enum BaselineStatus {
    /// Within acceptable variance
    Within,

    /// Exceeds upper bound
    Exceeded {
        expected: f64,
        actual: f64,
        variance_percent: f64,
    },

    /// Below lower bound (potentially over-optimized or misconfigured)
    Below {
        expected: f64,
        actual: f64,
        variance_percent: f64,
    },

    /// No baseline defined
    NoBaseline,
}

/// Baseline violation for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineViolation {
    /// Name of module/resource/service
    pub name: String,

    /// Type of baseline (global, module, service)
    pub baseline_type: String,

    /// Expected cost
    pub expected_cost: f64,

    /// Actual cost
    pub actual_cost: f64,

    /// Variance percentage
    pub variance_percent: f64,

    /// Acceptable variance threshold
    pub acceptable_variance: f64,

    /// Severity level
    pub severity: String,

    /// Regression type classification
    pub regression_type: RegressionType,

    /// Baseline owner for escalation
    pub owner: String,

    /// Justification from baseline
    pub justification: String,
}

impl Baseline {
    /// Create a new baseline
    pub fn new(
        name: String,
        expected_monthly_cost: f64,
        justification: String,
        owner: String,
    ) -> Self {
        Self {
            name,
            expected_monthly_cost,
            acceptable_variance_percent: default_variance(),
            last_updated: Utc::now().to_rfc3339(),
            justification,
            owner,
            reference: None,
            tags: HashMap::new(),
        }
    }

    /// Check if actual cost is within acceptable variance
    pub fn check_variance(&self, actual_cost: f64) -> BaselineStatus {
        let variance =
            ((actual_cost - self.expected_monthly_cost) / self.expected_monthly_cost).abs() * 100.0;

        if variance <= self.acceptable_variance_percent {
            BaselineStatus::Within
        } else if actual_cost > self.expected_monthly_cost {
            BaselineStatus::Exceeded {
                expected: self.expected_monthly_cost,
                actual: actual_cost,
                variance_percent: variance,
            }
        } else {
            BaselineStatus::Below {
                expected: self.expected_monthly_cost,
                actual: actual_cost,
                variance_percent: variance,
            }
        }
    }

    /// Get upper bound (expected + variance)
    pub fn upper_bound(&self) -> f64 {
        self.expected_monthly_cost * (1.0 + self.acceptable_variance_percent / 100.0)
    }

    /// Get lower bound (expected - variance)
    pub fn lower_bound(&self) -> f64 {
        self.expected_monthly_cost * (1.0 - self.acceptable_variance_percent / 100.0)
    }

    /// Parse last_updated timestamp
    pub fn get_last_updated(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.last_updated).map(|dt| dt.with_timezone(&Utc))
    }

    /// Check if baseline is stale (older than review cadence)
    pub fn is_stale(&self, review_cadence_days: u32) -> bool {
        if let Ok(last_updated) = self.get_last_updated() {
            let now = Utc::now();
            let days_since_update = (now - last_updated).num_days();
            days_since_update > review_cadence_days as i64
        } else {
            true // Invalid timestamp = stale
        }
    }
}

impl BaselinesConfig {
    /// Create a new baselines configuration
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            global: None,
            modules: HashMap::new(),
            services: HashMap::new(),
            metadata: Some(BaselineMetadata {
                last_reviewed: Some(Utc::now().to_rfc3339()),
                review_cadence_days: Some(90),
                owner_team: None,
            }),
        }
    }

    /// Add a global baseline
    pub fn set_global(&mut self, baseline: Baseline) {
        self.global = Some(baseline);
    }

    /// Add a module baseline
    pub fn add_module(&mut self, name: String, baseline: Baseline) {
        self.modules.insert(name, baseline);
    }

    /// Add a service baseline
    pub fn add_service(&mut self, name: String, baseline: Baseline) {
        self.services.insert(name, baseline);
    }

    /// Get baseline for a module
    pub fn get_module_baseline(&self, module_name: &str) -> Option<&Baseline> {
        self.modules.get(module_name)
    }

    /// Get baseline for a service
    pub fn get_service_baseline(&self, service_name: &str) -> Option<&Baseline> {
        self.services.get(service_name)
    }

    /// Get all stale baselines
    pub fn get_stale_baselines(&self) -> Vec<(&str, &Baseline)> {
        let review_cadence = self
            .metadata
            .as_ref()
            .and_then(|m| m.review_cadence_days)
            .unwrap_or(90);

        let mut stale = Vec::new();

        if let Some(global) = &self.global {
            if global.is_stale(review_cadence) {
                stale.push(("global", global));
            }
        }

        for (name, baseline) in &self.modules {
            if baseline.is_stale(review_cadence) {
                stale.push((name.as_str(), baseline));
            }
        }

        for (name, baseline) in &self.services {
            if baseline.is_stale(review_cadence) {
                stale.push((name.as_str(), baseline));
            }
        }

        stale
    }
}

impl Default for BaselinesConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaselineStatus::Within => write!(f, "Within baseline"),
            BaselineStatus::Exceeded {
                expected,
                actual,
                variance_percent,
            } => write!(
                f,
                "Exceeded baseline: ${:.2} vs ${:.2} expected ({:.1}% over)",
                actual, expected, variance_percent
            ),
            BaselineStatus::Below {
                expected,
                actual,
                variance_percent,
            } => write!(
                f,
                "Below baseline: ${:.2} vs ${:.2} expected ({:.1}% under)",
                actual, expected, variance_percent
            ),
            BaselineStatus::NoBaseline => write!(f, "No baseline defined"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_creation() {
        let baseline = Baseline::new(
            "module.vpc".to_string(),
            1000.0,
            "Production VPC baseline".to_string(),
            "platform-team@example.com".to_string(),
        );

        assert_eq!(baseline.name, "module.vpc");
        assert_eq!(baseline.expected_monthly_cost, 1000.0);
        assert_eq!(baseline.acceptable_variance_percent, 10.0);
    }

    #[test]
    fn test_check_variance_within() {
        let baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );

        // Within 10% variance
        assert_eq!(baseline.check_variance(1050.0), BaselineStatus::Within);
        assert_eq!(baseline.check_variance(950.0), BaselineStatus::Within);
        assert_eq!(baseline.check_variance(1000.0), BaselineStatus::Within);
    }

    #[test]
    fn test_check_variance_exceeded() {
        let baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );

        match baseline.check_variance(1200.0) {
            BaselineStatus::Exceeded {
                variance_percent, ..
            } => {
                assert!(variance_percent > 10.0);
            }
            _ => panic!("Expected Exceeded status"),
        }
    }

    #[test]
    fn test_check_variance_below() {
        let baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );

        match baseline.check_variance(800.0) {
            BaselineStatus::Below {
                variance_percent, ..
            } => {
                assert!(variance_percent > 10.0);
            }
            _ => panic!("Expected Below status"),
        }
    }

    #[test]
    fn test_bounds() {
        let baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );

        assert_eq!(baseline.upper_bound(), 1100.0);
        assert_eq!(baseline.lower_bound(), 900.0);
    }

    #[test]
    fn test_custom_variance() {
        let mut baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );
        baseline.acceptable_variance_percent = 20.0;

        assert_eq!(baseline.upper_bound(), 1200.0);
        assert_eq!(baseline.lower_bound(), 800.0);
        assert_eq!(baseline.check_variance(1150.0), BaselineStatus::Within);
    }

    #[test]
    fn test_baselines_config() {
        let mut config = BaselinesConfig::new();

        let baseline = Baseline::new(
            "module.vpc".to_string(),
            1000.0,
            "VPC baseline".to_string(),
            "team".to_string(),
        );

        config.add_module("module.vpc".to_string(), baseline);

        assert!(config.get_module_baseline("module.vpc").is_some());
        assert!(config.get_module_baseline("nonexistent").is_none());
    }

    #[test]
    fn test_global_baseline() {
        let mut config = BaselinesConfig::new();
        let global = Baseline::new(
            "global".to_string(),
            5000.0,
            "Total budget".to_string(),
            "finance".to_string(),
        );
        config.set_global(global);

        assert!(config.global.is_some());
        assert_eq!(
            config.global.as_ref().unwrap().expected_monthly_cost,
            5000.0
        );
    }

    #[test]
    fn test_service_baselines() {
        let mut config = BaselinesConfig::new();
        let nat = Baseline::new(
            "NAT Gateway".to_string(),
            405.0,
            "3 NAT Gateways".to_string(),
            "network".to_string(),
        );
        config.add_service("NAT Gateway".to_string(), nat);

        assert!(config.get_service_baseline("NAT Gateway").is_some());
    }

    #[test]
    fn test_baseline_status_display() {
        assert_eq!(BaselineStatus::Within.to_string(), "Within baseline");
        assert_eq!(
            BaselineStatus::NoBaseline.to_string(),
            "No baseline defined"
        );

        let exceeded = BaselineStatus::Exceeded {
            expected: 1000.0,
            actual: 1200.0,
            variance_percent: 20.0,
        };
        assert!(exceeded.to_string().contains("Exceeded"));
        assert!(exceeded.to_string().contains("1200"));
    }

    #[test]
    fn test_baseline_serialization() {
        let baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test baseline".to_string(),
            "owner".to_string(),
        );

        let json = serde_json::to_string(&baseline).unwrap();
        let deserialized: Baseline = serde_json::from_str(&json).unwrap();

        assert_eq!(baseline.name, deserialized.name);
        assert_eq!(
            baseline.expected_monthly_cost,
            deserialized.expected_monthly_cost
        );
    }

    #[test]
    fn test_config_serialization() {
        let mut config = BaselinesConfig::new();
        config.add_module(
            "module.test".to_string(),
            Baseline::new(
                "module.test".to_string(),
                1000.0,
                "Test".to_string(),
                "owner".to_string(),
            ),
        );

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: BaselinesConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.version, deserialized.version);
        assert!(deserialized.modules.contains_key("module.test"));
    }

    #[test]
    fn test_stale_detection() {
        let mut baseline = Baseline::new(
            "test".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );

        // Set last_updated to 100 days ago
        let past = Utc::now() - chrono::Duration::days(100);
        baseline.last_updated = past.to_rfc3339();

        // Should be stale with 90-day cadence
        assert!(baseline.is_stale(90));
        // Should not be stale with 120-day cadence
        assert!(!baseline.is_stale(120));
    }

    #[test]
    fn test_get_stale_baselines() {
        let mut config = BaselinesConfig::new();

        // Fresh baseline
        let fresh = Baseline::new(
            "fresh".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );
        config.add_module("fresh".to_string(), fresh);

        // Stale baseline
        let mut stale = Baseline::new(
            "stale".to_string(),
            1000.0,
            "Test".to_string(),
            "owner".to_string(),
        );
        let past = Utc::now() - chrono::Duration::days(100);
        stale.last_updated = past.to_rfc3339();
        config.add_module("stale".to_string(), stale);

        let stale_list = config.get_stale_baselines();
        assert_eq!(stale_list.len(), 1);
        assert_eq!(stale_list[0].0, "stale");
    }
}
