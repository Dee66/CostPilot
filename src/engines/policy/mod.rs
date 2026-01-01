// Policy evaluation engine module

pub mod approval_workflow;
mod audit_log;
mod compliance;
pub mod exemption_ci;
pub mod exemption_types;
pub mod exemption_validator;
pub mod lifecycle;
mod metadata_engine;
mod policy_engine;
mod policy_history;
mod policy_loader;
mod policy_metadata;
mod policy_repository;
mod policy_types;
mod policy_version;
mod zero_network;

pub mod parser;

// Re-export all public items from submodules with explicit names to avoid ambiguity
pub use approval_workflow::*;
pub use audit_log::*;
pub use compliance::*;
pub use exemption_ci::*;
pub use exemption_types::*;
pub use exemption_validator::*;

// Zero-network exports
pub use zero_network::{ZeroNetworkToken, ZeroNetworkValidator, ZeroNetworkViolation};

// Lifecycle exports - PolicyLifecycle from lifecycle module (state machine)
pub use lifecycle::{
    ApprovalConfig, ApprovalRequest, ApprovalStatus, LifecycleError, LifecycleSummary,
    PolicyLifecycle as LifecycleStateMachine, PolicyState, StateTransition,
};

// Metadata engine exports - PolicyRule from metadata_engine
pub use metadata_engine::{
    MetadataPolicyEngine, MetadataPolicyResult, MetadataPolicyViolation,
    PolicyRule as MetadataPolicyRule,
};

// Parser DSL exports - PolicyRule from parser
pub use parser::{
    Condition, ConditionType, ConditionValue, DslParser, EvaluationContext, EvaluationResult,
    LoadError, Operator, ParseError, PolicyRule as DslPolicyRule, PolicyRuleLoader, RuleAction,
    RuleEvaluator, RuleMatch, RuleSeverity, RuleStatistics,
};

pub use policy_engine::*;

// Policy history exports - PolicyVersion from policy_history
pub use policy_history::{
    HistoryError, PolicyContent, PolicyHistory, PolicyVersion as HistoryPolicyVersion, VersionDiff,
    VersionMetadata as HistoryVersionMetadata,
};

pub use policy_loader::*;

// Policy metadata exports - PolicyLifecycle from policy_metadata (metadata struct)
pub use policy_metadata::{
    DeprecationInfo, PolicyCategory, PolicyLifecycle as MetadataLifecycle, PolicyLinks,
    PolicyMetadata, PolicyMetrics, PolicyOwnership, PolicyRevision, PolicyStatus,
    PolicyWithMetadata, Severity,
};

pub use policy_repository::*;
pub use policy_types::*;

// Policy version exports - PolicyVersion from policy_version (version metadata)
pub use policy_version::{PolicyVersion as VersionInfo, PolicyVersionManager};

pub use zero_network::*;
