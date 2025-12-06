// Grouping engine module exports

pub mod by_module;
pub mod by_service;
pub mod by_environment;
pub mod attribution;
pub mod grouping_engine;

// Re-export main types
pub use by_module::{ModuleGroup, group_by_module, generate_module_tree, aggregate_module_hierarchy};
pub use by_service::{ServiceGroup, ServiceCategory, group_by_service, generate_service_report, group_by_category, cost_by_category};
pub use by_environment::{
    EnvironmentGroup, group_by_environment, infer_environment, normalize_environment,
    calculate_environment_ratios, detect_anomalies, generate_environment_report,
    EnvironmentAnomaly, AnomalyType, Severity,
};
pub use attribution::{
    AttributionPipeline, Attribution, AttributionReport,
};
pub use grouping_engine::{
    GroupingEngine, ComprehensiveReport, GroupingOptions, SortBy,
};
