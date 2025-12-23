// Unified grouping engine for cost allocation and attribution

use crate::engines::grouping::{
    attribution::{AttributionPipeline, AttributionReport},
    by_environment::{generate_environment_report, group_by_environment, EnvironmentGroup},
    by_module::{generate_module_tree, group_by_module, ModuleGroup},
    by_service::{generate_service_report, group_by_service, ServiceGroup},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for resource tuple: (address, type, service, tags, cost)
pub type ResourceTuple = (String, String, String, HashMap<String, String>, f64);

/// Main grouping engine for organizing and analyzing resource costs
pub struct GroupingEngine {
    attribution_pipeline: AttributionPipeline,
}

impl GroupingEngine {
    pub fn new() -> Self {
        Self {
            attribution_pipeline: AttributionPipeline::new(),
        }
    }

    pub fn with_pipeline(attribution_pipeline: AttributionPipeline) -> Self {
        Self {
            attribution_pipeline,
        }
    }

    /// Group resources by module and return results
    pub fn group_by_module(
        &self,
        resources: &[(String, String, f64)], // (address, type, cost)
    ) -> Vec<ModuleGroup> {
        group_by_module(resources)
    }

    /// Group resources by service and return results
    pub fn group_by_service(
        &self,
        resources: &[(String, String, f64)], // (address, type, cost)
    ) -> Vec<ServiceGroup> {
        group_by_service(resources)
    }

    /// Group resources by environment and return results
    pub fn group_by_environment(&self, resources: &[ResourceTuple]) -> Vec<EnvironmentGroup> {
        group_by_environment(resources)
    }

    /// Generate attribution report for cost allocation
    pub fn generate_attribution_report(
        &self,
        resources: &[(String, String, f64, HashMap<String, String>)], // (address, type, cost, tags)
    ) -> AttributionReport {
        self.attribution_pipeline
            .generate_attribution_report(resources)
    }

    /// Generate comprehensive grouping report with all dimensions
    pub fn generate_comprehensive_report(
        &self,
        resources: &[(String, String, HashMap<String, String>, f64)], // (address, type, tags, cost)
    ) -> ComprehensiveReport {
        // Prepare data for module grouping
        let module_resources: Vec<(String, String, f64)> = resources
            .iter()
            .map(|(addr, ty, _, cost)| (addr.clone(), ty.clone(), *cost))
            .collect();

        // Prepare data for service grouping
        let service_resources = module_resources.clone();

        // Prepare data for environment grouping
        let env_resources: Vec<ResourceTuple> = resources
            .iter()
            .map(|(addr, ty, tags, cost)| {
                let (service, _) = crate::engines::grouping::by_service::extract_service_info(ty);
                (addr.clone(), ty.clone(), service, tags.clone(), *cost)
            })
            .collect();

        // Prepare data for attribution
        let attr_resources: Vec<(String, String, f64, HashMap<String, String>)> = resources
            .iter()
            .map(|(addr, ty, tags, cost)| (addr.clone(), ty.clone(), *cost, tags.clone()))
            .collect();

        let module_groups = self.group_by_module(&module_resources);
        let service_groups = self.group_by_service(&service_resources);
        let environment_groups = self.group_by_environment(&env_resources);
        let attribution_report = self.generate_attribution_report(&attr_resources);

        ComprehensiveReport {
            module_groups,
            service_groups,
            environment_groups,
            attribution_report,
            total_resources: resources.len(),
            total_cost: resources.iter().map(|(_, _, _, cost)| cost).sum(),
        }
    }

    /// Get reference to attribution pipeline
    pub fn attribution_pipeline(&self) -> &AttributionPipeline {
        &self.attribution_pipeline
    }

    /// Get mutable reference to attribution pipeline
    pub fn attribution_pipeline_mut(&mut self) -> &mut AttributionPipeline {
        &mut self.attribution_pipeline
    }
}

impl Default for GroupingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive grouping report across all dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    pub module_groups: Vec<ModuleGroup>,
    pub service_groups: Vec<ServiceGroup>,
    pub environment_groups: Vec<EnvironmentGroup>,
    pub attribution_report: AttributionReport,
    pub total_resources: usize,
    pub total_cost: f64,
}

impl ComprehensiveReport {
    /// Generate formatted text report
    pub fn format_text(&self) -> String {
        let mut report = String::new();

        report.push_str("╔═══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║           CostPilot Comprehensive Grouping Report            ║\n");
        report.push_str("╚═══════════════════════════════════════════════════════════════╝\n\n");

        report.push_str(&format!("Total Resources: {}\n", self.total_resources));
        report.push_str(&format!("Total Monthly Cost: ${:.2}\n\n", self.total_cost));

        // Module summary
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("Module Breakdown\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");
        report.push_str(&generate_module_tree(&self.module_groups));

        // Service summary
        report.push_str("\n═══════════════════════════════════════════════════════════════\n");
        report.push_str("Service Breakdown\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");
        report.push_str(&generate_service_report(&self.service_groups));

        // Environment summary
        report.push_str("\n═══════════════════════════════════════════════════════════════\n");
        report.push_str("Environment Breakdown\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");
        report.push_str(&generate_environment_report(&self.environment_groups));

        // Attribution summary
        report.push_str("\n═══════════════════════════════════════════════════════════════\n");
        report.push_str("Cost Attribution\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");
        report.push_str(&self.attribution_report.format_text());

        report
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to CSV (attribution data only)
    pub fn to_csv(&self) -> String {
        self.attribution_report.export_csv()
    }
}

/// Grouping options for customizing grouping behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingOptions {
    /// Minimum cost threshold for groups (groups below this are filtered)
    pub min_cost_threshold: f64,
    /// Maximum number of groups to return
    pub max_groups: Option<usize>,
    /// Sort order (by cost or name)
    pub sort_by: SortBy,
    /// Whether to include zero-cost resources
    pub include_zero_cost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Cost,
    Name,
    ResourceCount,
}

impl Default for GroupingOptions {
    fn default() -> Self {
        Self {
            min_cost_threshold: 0.0,
            max_groups: None,
            sort_by: SortBy::Cost,
            include_zero_cost: true,
        }
    }
}

impl GroupingOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_cost(mut self, threshold: f64) -> Self {
        self.min_cost_threshold = threshold;
        self
    }

    pub fn with_max_groups(mut self, max: usize) -> Self {
        self.max_groups = Some(max);
        self
    }

    pub fn with_sort_by(mut self, sort: SortBy) -> Self {
        self.sort_by = sort;
        self
    }

    pub fn exclude_zero_cost(mut self) -> Self {
        self.include_zero_cost = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grouping_engine_creation() {
        let engine = GroupingEngine::new();
        assert!(engine
            .attribution_pipeline()
            .tag_mappings
            .contains_key("environment"));
    }

    #[test]
    fn test_group_by_module() {
        let engine = GroupingEngine::new();
        let resources = vec![
            (
                "aws_instance.web".to_string(),
                "aws_instance".to_string(),
                100.0,
            ),
            (
                "module.vpc.aws_vpc.main".to_string(),
                "aws_vpc".to_string(),
                50.0,
            ),
        ];

        let groups = engine.group_by_module(&resources);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_comprehensive_report() {
        let engine = GroupingEngine::new();
        let mut tags = HashMap::new();
        tags.insert("Environment".to_string(), "production".to_string());

        let resources = vec![(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            tags.clone(),
            100.0,
        )];

        let report = engine.generate_comprehensive_report(&resources);
        assert_eq!(report.total_resources, 1);
        assert_eq!(report.total_cost, 100.0);
        assert!(!report.module_groups.is_empty());
        assert!(!report.service_groups.is_empty());
        assert!(!report.environment_groups.is_empty());
    }

    #[test]
    fn test_grouping_options() {
        let options = GroupingOptions::new()
            .with_min_cost(10.0)
            .with_max_groups(5)
            .exclude_zero_cost();

        assert_eq!(options.min_cost_threshold, 10.0);
        assert_eq!(options.max_groups, Some(5));
        assert!(!options.include_zero_cost);
    }
}
