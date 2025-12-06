// Group resources by environment (dev, staging, prod)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A group of resources organized by environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentGroup {
    /// Environment name (e.g., "production", "staging", "development")
    pub environment: String,
    /// Resource addresses in this environment
    pub resources: Vec<String>,
    /// Total monthly cost for this environment
    pub monthly_cost: f64,
    /// Number of resources
    pub resource_count: usize,
    /// Cost breakdown by resource type
    pub cost_by_type: HashMap<String, f64>,
    /// Cost breakdown by service
    pub cost_by_service: HashMap<String, f64>,
}

impl EnvironmentGroup {
    pub fn new(environment: String) -> Self {
        Self {
            environment,
            resources: Vec::new(),
            monthly_cost: 0.0,
            resource_count: 0,
            cost_by_type: HashMap::new(),
            cost_by_service: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, address: String, resource_type: String, service: String, cost: f64) {
        self.resources.push(address);
        self.monthly_cost += cost;
        self.resource_count += 1;
        *self.cost_by_type.entry(resource_type).or_insert(0.0) += cost;
        *self.cost_by_service.entry(service).or_insert(0.0) += cost;
    }

    pub fn average_cost_per_resource(&self) -> f64 {
        if self.resource_count == 0 {
            0.0
        } else {
            self.monthly_cost / self.resource_count as f64
        }
    }

    pub fn top_services(&self, limit: usize) -> Vec<(String, f64)> {
        let mut services: Vec<(String, f64)> = self.cost_by_service.iter()
            .map(|(s, c)| (s.clone(), *c))
            .collect();
        services.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        services.truncate(limit);
        services
    }
}

/// Group resources by environment extracted from tags or resource names
pub fn group_by_environment(
    resources: &[(String, String, String, HashMap<String, String>, f64)], // (address, type, service, tags, cost)
) -> Vec<EnvironmentGroup> {
    let mut groups: HashMap<String, EnvironmentGroup> = HashMap::new();

    for (address, resource_type, service, tags, cost) in resources {
        let environment = infer_environment(address, tags);
        let group = groups
            .entry(environment.clone())
            .or_insert_with(|| EnvironmentGroup::new(environment));
        group.add_resource(address.clone(), resource_type.clone(), service.clone(), *cost);
    }

    let mut result: Vec<EnvironmentGroup> = groups.into_values().collect();
    result.sort_by(|a, b| b.monthly_cost.partial_cmp(&a.monthly_cost).unwrap());
    result
}

/// Infer environment from tags or resource address
/// Priority:
/// 1. "Environment" tag (exact)
/// 2. "environment" tag (lowercase)
/// 3. "Env" tag
/// 4. "env" tag
/// 5. Address patterns (e.g., "prod-*", "*-staging", "dev_*")
/// 6. Default to "unknown"
pub fn infer_environment(address: &str, tags: &HashMap<String, String>) -> String {
    // Check tags in priority order
    let tag_keys = ["Environment", "environment", "Env", "env", "ENVIRONMENT"];
    for key in &tag_keys {
        if let Some(value) = tags.get(*key) {
            return normalize_environment(value);
        }
    }

    // Check address patterns
    let lower = address.to_lowercase();
    
    // Production patterns
    if lower.contains("prod") || lower.contains("production") || lower.contains("prd") {
        return "production".to_string();
    }
    
    // Staging patterns
    if lower.contains("stag") || lower.contains("stage") || lower.contains("staging") {
        return "staging".to_string();
    }
    
    // Development patterns
    if lower.contains("dev") || lower.contains("development") {
        return "development".to_string();
    }
    
    // QA/Test patterns
    if lower.contains("qa") || lower.contains("test") || lower.contains("testing") {
        return "qa".to_string();
    }
    
    // UAT patterns
    if lower.contains("uat") || lower.contains("acceptance") {
        return "uat".to_string();
    }
    
    // Sandbox patterns
    if lower.contains("sandbox") || lower.contains("sbx") {
        return "sandbox".to_string();
    }

    "unknown".to_string()
}

/// Normalize environment names to standard values
pub fn normalize_environment(env: &str) -> String {
    let normalized = env.trim().to_lowercase();
    
    match normalized.as_str() {
        "prod" | "production" | "prd" | "live" => "production".to_string(),
        "stag" | "stage" | "staging" | "stg" => "staging".to_string(),
        "dev" | "development" | "devel" => "development".to_string(),
        "qa" | "test" | "testing" | "tst" => "qa".to_string(),
        "uat" | "acceptance" | "acc" => "uat".to_string(),
        "sandbox" | "sbx" | "sb" => "sandbox".to_string(),
        "demo" | "demonstration" => "demo".to_string(),
        _ => normalized,
    }
}

/// Calculate environment cost ratios (useful for detecting cost imbalances)
pub fn calculate_environment_ratios(groups: &[EnvironmentGroup]) -> HashMap<String, f64> {
    let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();
    
    groups
        .iter()
        .map(|g| {
            let ratio = if total_cost > 0.0 {
                g.monthly_cost / total_cost
            } else {
                0.0
            };
            (g.environment.clone(), ratio)
        })
        .collect()
}

/// Detect environment cost anomalies (e.g., dev/staging costs exceeding production)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentAnomaly {
    pub anomaly_type: AnomalyType,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    DevExceedsProd,
    StagingExceedsProd,
    UnknownEnvironmentHigh,
    ImbalancedCosts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    High,
    Medium,
    Low,
}

pub fn detect_anomalies(groups: &[EnvironmentGroup]) -> Vec<EnvironmentAnomaly> {
    let mut anomalies = Vec::new();

    let prod_cost = groups
        .iter()
        .find(|g| g.environment == "production")
        .map(|g| g.monthly_cost)
        .unwrap_or(0.0);

    let dev_cost = groups
        .iter()
        .find(|g| g.environment == "development")
        .map(|g| g.monthly_cost)
        .unwrap_or(0.0);

    let staging_cost = groups
        .iter()
        .find(|g| g.environment == "staging")
        .map(|g| g.monthly_cost)
        .unwrap_or(0.0);

    let unknown_cost = groups
        .iter()
        .find(|g| g.environment == "unknown")
        .map(|g| g.monthly_cost)
        .unwrap_or(0.0);

    let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();

    // Dev exceeds prod
    if prod_cost > 0.0 && dev_cost > prod_cost * 1.2 {
        anomalies.push(EnvironmentAnomaly {
            anomaly_type: AnomalyType::DevExceedsProd,
            message: format!(
                "Development costs (${:.2}/mo) exceed production costs (${:.2}/mo) by {:.0}%",
                dev_cost,
                prod_cost,
                ((dev_cost - prod_cost) / prod_cost) * 100.0
            ),
            severity: Severity::High,
        });
    }

    // Staging exceeds prod
    if prod_cost > 0.0 && staging_cost > prod_cost * 1.5 {
        anomalies.push(EnvironmentAnomaly {
            anomaly_type: AnomalyType::StagingExceedsProd,
            message: format!(
                "Staging costs (${:.2}/mo) exceed production costs (${:.2}/mo) by {:.0}%",
                staging_cost,
                prod_cost,
                ((staging_cost - prod_cost) / prod_cost) * 100.0
            ),
            severity: Severity::Medium,
        });
    }

    // Unknown environment has significant cost
    if total_cost > 0.0 && unknown_cost > total_cost * 0.2 {
        anomalies.push(EnvironmentAnomaly {
            anomaly_type: AnomalyType::UnknownEnvironmentHigh,
            message: format!(
                "Unknown environment represents {:.1}% of total costs (${:.2}/mo). Consider tagging resources.",
                (unknown_cost / total_cost) * 100.0,
                unknown_cost
            ),
            severity: Severity::Medium,
        });
    }

    anomalies
}

/// Generate an environment cost report
pub fn generate_environment_report(groups: &[EnvironmentGroup]) -> String {
    let mut report = String::new();
    report.push_str("Environment Cost Summary\n");
    report.push_str("=======================\n\n");

    let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();
    report.push_str(&format!("Total Monthly Cost: ${:.2}\n\n", total_cost));

    report.push_str("By Environment:\n");
    for group in groups {
        let percentage = if total_cost > 0.0 {
            (group.monthly_cost / total_cost) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!(
            "  {} ${:.2}/mo ({:.1}%, {} resources)\n",
            group.environment, group.monthly_cost, percentage, group.resource_count
        ));

        // Show top 3 services for this environment
        let top_services = group.top_services(3);
        for (service, cost) in top_services {
            report.push_str(&format!("    - {}: ${:.2}/mo\n", service, cost));
        }
    }

    // Detect and report anomalies
    let anomalies = detect_anomalies(groups);
    if !anomalies.is_empty() {
        report.push_str("\n⚠️  Cost Anomalies Detected:\n");
        for anomaly in anomalies {
            let icon = match anomaly.severity {
                Severity::High => "🔴",
                Severity::Medium => "🟡",
                Severity::Low => "🟢",
            };
            report.push_str(&format!("  {} {}\n", icon, anomaly.message));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_environment_from_tags() {
        let mut tags = HashMap::new();
        tags.insert("Environment".to_string(), "production".to_string());
        assert_eq!(infer_environment("aws_instance.web", &tags), "production");

        tags.clear();
        tags.insert("env".to_string(), "dev".to_string());
        assert_eq!(infer_environment("aws_instance.web", &tags), "development");
    }

    #[test]
    fn test_infer_environment_from_address() {
        let tags = HashMap::new();
        assert_eq!(infer_environment("aws_instance.prod-web", &tags), "production");
        assert_eq!(infer_environment("aws_instance.staging-api", &tags), "staging");
        assert_eq!(infer_environment("aws_instance.dev_db", &tags), "development");
        assert_eq!(infer_environment("aws_instance.qa-test", &tags), "qa");
    }

    #[test]
    fn test_normalize_environment() {
        assert_eq!(normalize_environment("prod"), "production");
        assert_eq!(normalize_environment("PRODUCTION"), "production");
        assert_eq!(normalize_environment("stag"), "staging");
        assert_eq!(normalize_environment("dev"), "development");
    }

    #[test]
    fn test_detect_anomalies() {
        let groups = vec![
            EnvironmentGroup {
                environment: "production".to_string(),
                resources: vec![],
                monthly_cost: 100.0,
                resource_count: 10,
                cost_by_type: HashMap::new(),
                cost_by_service: HashMap::new(),
            },
            EnvironmentGroup {
                environment: "development".to_string(),
                resources: vec![],
                monthly_cost: 150.0,
                resource_count: 15,
                cost_by_type: HashMap::new(),
                cost_by_service: HashMap::new(),
            },
        ];

        let anomalies = detect_anomalies(&groups);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, AnomalyType::DevExceedsProd);
    }
}
