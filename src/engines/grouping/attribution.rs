// Attribution pipeline for cost allocation and chargeback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tag extraction and normalization for cost attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionPipeline {
    /// Tag key mappings (e.g., "cost_center" -> ["CostCenter", "cost-center", "costcenter"])
    pub tag_mappings: HashMap<String, Vec<String>>,
    /// Default environment if none can be inferred
    pub default_environment: String,
    /// Whether to use strict matching (case-sensitive)
    pub strict_matching: bool,
}

impl Default for AttributionPipeline {
    fn default() -> Self {
        let mut tag_mappings = HashMap::new();
        
        // Environment mappings
        tag_mappings.insert(
            "environment".to_string(),
            vec![
                "Environment".to_string(),
                "environment".to_string(),
                "Env".to_string(),
                "env".to_string(),
                "ENVIRONMENT".to_string(),
            ],
        );
        
        // Cost center mappings
        tag_mappings.insert(
            "cost_center".to_string(),
            vec![
                "CostCenter".to_string(),
                "cost_center".to_string(),
                "cost-center".to_string(),
                "costcenter".to_string(),
                "COST_CENTER".to_string(),
            ],
        );
        
        // Team/Owner mappings
        tag_mappings.insert(
            "owner".to_string(),
            vec![
                "Owner".to_string(),
                "owner".to_string(),
                "Team".to_string(),
                "team".to_string(),
                "OWNER".to_string(),
            ],
        );
        
        // Project mappings
        tag_mappings.insert(
            "project".to_string(),
            vec![
                "Project".to_string(),
                "project".to_string(),
                "ProjectName".to_string(),
                "project_name".to_string(),
                "PROJECT".to_string(),
            ],
        );
        
        // Application mappings
        tag_mappings.insert(
            "application".to_string(),
            vec![
                "Application".to_string(),
                "application".to_string(),
                "App".to_string(),
                "app".to_string(),
                "APPLICATION".to_string(),
            ],
        );

        Self {
            tag_mappings,
            default_environment: "unknown".to_string(),
            strict_matching: false,
        }
    }
}

impl AttributionPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract normalized tags from raw resource tags
    pub fn extract_tags(&self, raw_tags: &HashMap<String, String>) -> HashMap<String, String> {
        let mut normalized = HashMap::new();

        for (canonical_key, variants) in &self.tag_mappings {
            for variant in variants {
                if let Some(value) = raw_tags.get(variant) {
                    normalized.insert(canonical_key.clone(), value.clone());
                    break;
                }
            }
        }

        normalized
    }

    /// Normalize tag casing (lowercase keys, preserve values)
    pub fn normalize_tag_casing(tags: &HashMap<String, String>) -> HashMap<String, String> {
        tags.iter()
            .map(|(k, v)| (k.to_lowercase(), v.clone()))
            .collect()
    }

    /// Infer environment from tags and resource address
    pub fn infer_environment(
        &self,
        address: &str,
        tags: &HashMap<String, String>,
    ) -> String {
        crate::engines::grouping::by_environment::infer_environment(address, tags)
    }

    /// Generate attribution report for resources
    pub fn generate_attribution_report(
        &self,
        resources: &[(String, String, f64, HashMap<String, String>)], // (address, type, cost, tags)
    ) -> AttributionReport {
        let mut report = AttributionReport::new();

        for (address, resource_type, cost, raw_tags) in resources {
            let normalized_tags = self.extract_tags(raw_tags);
            
            let environment = self.infer_environment(address, raw_tags);
            let cost_center = normalized_tags.get("cost_center").cloned().unwrap_or_else(|| "untagged".to_string());
            let owner = normalized_tags.get("owner").cloned().unwrap_or_else(|| "untagged".to_string());
            let project = normalized_tags.get("project").cloned().unwrap_or_else(|| "untagged".to_string());
            let application = normalized_tags.get("application").cloned().unwrap_or_else(|| "untagged".to_string());

            report.add_allocation(Attribution {
                resource_address: address.clone(),
                resource_type: resource_type.clone(),
                environment,
                cost_center,
                owner,
                project,
                application,
                monthly_cost: *cost,
                tags: normalized_tags,
            });
        }

        report
    }

    /// Add custom tag mapping
    pub fn add_tag_mapping(&mut self, canonical_key: String, variants: Vec<String>) {
        self.tag_mappings.insert(canonical_key, variants);
    }
}

/// A single resource attribution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    pub resource_address: String,
    pub resource_type: String,
    pub environment: String,
    pub cost_center: String,
    pub owner: String,
    pub project: String,
    pub application: String,
    pub monthly_cost: f64,
    pub tags: HashMap<String, String>,
}

/// Attribution report with cost allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionReport {
    pub allocations: Vec<Attribution>,
    pub total_cost: f64,
    pub cost_by_environment: HashMap<String, f64>,
    pub cost_by_cost_center: HashMap<String, f64>,
    pub cost_by_owner: HashMap<String, f64>,
    pub cost_by_project: HashMap<String, f64>,
    pub cost_by_application: HashMap<String, f64>,
    pub untagged_cost: f64,
}

impl AttributionReport {
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
            total_cost: 0.0,
            cost_by_environment: HashMap::new(),
            cost_by_cost_center: HashMap::new(),
            cost_by_owner: HashMap::new(),
            cost_by_project: HashMap::new(),
            cost_by_application: HashMap::new(),
            untagged_cost: 0.0,
        }
    }

    /// Convert report to JSON string
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize AttributionReport: {}", e))
    }

    pub fn add_allocation(&mut self, attribution: Attribution) {
        self.total_cost += attribution.monthly_cost;

        *self.cost_by_environment
            .entry(attribution.environment.clone())
            .or_insert(0.0) += attribution.monthly_cost;

        *self.cost_by_cost_center
            .entry(attribution.cost_center.clone())
            .or_insert(0.0) += attribution.monthly_cost;

        *self.cost_by_owner
            .entry(attribution.owner.clone())
            .or_insert(0.0) += attribution.monthly_cost;

        *self.cost_by_project
            .entry(attribution.project.clone())
            .or_insert(0.0) += attribution.monthly_cost;

        *self.cost_by_application
            .entry(attribution.application.clone())
            .or_insert(0.0) += attribution.monthly_cost;

        // Track untagged costs
        if attribution.cost_center == "untagged" {
            self.untagged_cost += attribution.monthly_cost;
        }

        self.allocations.push(attribution);
    }

    /// Get top N cost centers
    pub fn top_cost_centers(&self, limit: usize) -> Vec<(String, f64)> {
        let mut entries: Vec<(String, f64)> = self.cost_by_cost_center.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        entries.truncate(limit);
        entries
    }

    /// Get top N owners
    pub fn top_owners(&self, limit: usize) -> Vec<(String, f64)> {
        let mut entries: Vec<(String, f64)> = self.cost_by_owner.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        entries.truncate(limit);
        entries
    }

    /// Get top N projects
    pub fn top_projects(&self, limit: usize) -> Vec<(String, f64)> {
        let mut entries: Vec<(String, f64)> = self.cost_by_project.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        entries.truncate(limit);
        entries
    }

    /// Calculate tagging coverage (percentage of costs with proper tags)
    pub fn tagging_coverage(&self) -> f64 {
        if self.total_cost == 0.0 {
            0.0
        } else {
            ((self.total_cost - self.untagged_cost) / self.total_cost) * 100.0
        }
    }

    /// Generate text report
    pub fn format_text(&self) -> String {
        let mut report = String::new();
        report.push_str("Cost Attribution Report\n");
        report.push_str("======================\n\n");

        report.push_str(&format!("Total Monthly Cost: ${:.2}\n", self.total_cost));
        report.push_str(&format!("Tagging Coverage: {:.1}%\n", self.tagging_coverage()));
        report.push_str(&format!("Untagged Cost: ${:.2}\n\n", self.untagged_cost));

        // Environment breakdown
        report.push_str("By Environment:\n");
        let mut env_sorted: Vec<(&String, &f64)> = self.cost_by_environment.iter().collect();
        env_sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        for (env, cost) in env_sorted {
            let pct = (cost / self.total_cost) * 100.0;
            report.push_str(&format!("  {}: ${:.2}/mo ({:.1}%)\n", env, cost, pct));
        }

        // Top cost centers
        report.push_str("\nTop Cost Centers:\n");
        for (i, (center, cost)) in self.top_cost_centers(5).iter().enumerate() {
            let pct = (cost / self.total_cost) * 100.0;
            report.push_str(&format!("  {}. {}: ${:.2}/mo ({:.1}%)\n", i + 1, center, cost, pct));
        }

        // Top owners
        report.push_str("\nTop Owners:\n");
        for (i, (owner, cost)) in self.top_owners(5).iter().enumerate() {
            let pct = (cost / self.total_cost) * 100.0;
            report.push_str(&format!("  {}. {}: ${:.2}/mo ({:.1}%)\n", i + 1, owner, cost, pct));
        }

        // Top projects
        report.push_str("\nTop Projects:\n");
        for (i, (project, cost)) in self.top_projects(5).iter().enumerate() {
            let pct = (cost / self.total_cost) * 100.0;
            report.push_str(&format!("  {}. {}: ${:.2}/mo ({:.1}%)\n", i + 1, project, cost, pct));
        }

        report
    }

    /// Export to CSV format
    pub fn export_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("Resource,Type,Environment,CostCenter,Owner,Project,Application,MonthlyCost\n");

        for allocation in &self.allocations {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{:.2}\n",
                allocation.resource_address,
                allocation.resource_type,
                allocation.environment,
                allocation.cost_center,
                allocation.owner,
                allocation.project,
                allocation.application,
                allocation.monthly_cost
            ));
        }

        csv
    }
}

impl Default for AttributionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags() {
        let pipeline = AttributionPipeline::new();
        let mut raw_tags = HashMap::new();
        raw_tags.insert("Environment".to_string(), "production".to_string());
        raw_tags.insert("CostCenter".to_string(), "eng-platform".to_string());

        let normalized = pipeline.extract_tags(&raw_tags);
        assert_eq!(normalized.get("environment"), Some(&"production".to_string()));
        assert_eq!(normalized.get("cost_center"), Some(&"eng-platform".to_string()));
    }

    #[test]
    fn test_normalize_tag_casing() {
        let mut tags = HashMap::new();
        tags.insert("Environment".to_string(), "Production".to_string());
        tags.insert("OWNER".to_string(), "TeamA".to_string());

        let normalized = AttributionPipeline::normalize_tag_casing(&tags);
        assert_eq!(normalized.get("environment"), Some(&"Production".to_string()));
        assert_eq!(normalized.get("owner"), Some(&"TeamA".to_string()));
    }

    #[test]
    fn test_generate_attribution_report() {
        let pipeline = AttributionPipeline::new();
        let mut tags1 = HashMap::new();
        tags1.insert("Environment".to_string(), "production".to_string());
        tags1.insert("CostCenter".to_string(), "engineering".to_string());

        let mut tags2 = HashMap::new();
        tags2.insert("env".to_string(), "dev".to_string());

        let resources = vec![
            ("aws_instance.web".to_string(), "aws_instance".to_string(), 100.0, tags1),
            ("aws_s3_bucket.data".to_string(), "aws_s3_bucket".to_string(), 50.0, tags2),
        ];

        let report = pipeline.generate_attribution_report(&resources);
        assert_eq!(report.total_cost, 150.0);
        assert_eq!(report.allocations.len(), 2);
        assert!(report.cost_by_environment.contains_key("production"));
        assert!(report.cost_by_environment.contains_key("development"));
    }

    #[test]
    fn test_tagging_coverage() {
        let mut report = AttributionReport::new();
        
        report.add_allocation(Attribution {
            resource_address: "res1".to_string(),
            resource_type: "type1".to_string(),
            environment: "prod".to_string(),
            cost_center: "eng".to_string(),
            owner: "team-a".to_string(),
            project: "project-x".to_string(),
            application: "app1".to_string(),
            monthly_cost: 100.0,
            tags: HashMap::new(),
        });

        report.add_allocation(Attribution {
            resource_address: "res2".to_string(),
            resource_type: "type2".to_string(),
            environment: "dev".to_string(),
            cost_center: "untagged".to_string(),
            owner: "untagged".to_string(),
            project: "untagged".to_string(),
            application: "untagged".to_string(),
            monthly_cost: 50.0,
            tags: HashMap::new(),
        });

        assert_eq!(report.total_cost, 150.0);
        assert_eq!(report.untagged_cost, 50.0);
        assert!((report.tagging_coverage() - 66.67).abs() < 0.1);
    }
}
