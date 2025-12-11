// Grouping engine module exports

pub mod attribution;
pub mod by_environment;
pub mod by_module;
pub mod by_service;
pub mod grouping_engine;

// Re-export main types
pub use attribution::{Attribution, AttributionPipeline, AttributionReport};
pub use by_environment::{
    calculate_environment_ratios, detect_anomalies, generate_environment_report,
    group_by_environment, infer_environment, normalize_environment, AnomalyType,
    EnvironmentAnomaly, EnvironmentGroup, Severity,
};
pub use by_module::{
    aggregate_module_hierarchy, generate_module_tree, group_by_module, ModuleGroup,
};
pub use by_service::{
    cost_by_category, generate_service_report, group_by_category, group_by_service,
    ServiceCategory, ServiceGroup,
};
pub use grouping_engine::{ComprehensiveReport, GroupingEngine, GroupingOptions, SortBy};
