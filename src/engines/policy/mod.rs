// Policy evaluation engine module

pub mod approval_workflow;
mod audit_log;
mod compliance;
mod exemption_ci;
mod exemption_types;
mod exemption_validator;
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

// Lifecycle exports - PolicyLifecycle from lifecycle module (state machine)
pub use lifecycle::{
    PolicyState, StateTransition, ApprovalConfig, ApprovalRequest,
    ApprovalStatus, LifecycleSummary, LifecycleError,
    PolicyLifecycle as LifecycleStateMachine
};

// Metadata engine exports - PolicyRule from metadata_engine
pub use metadata_engine::{
    MetadataPolicyEngine, PolicyRule as MetadataPolicyRule,
    MetadataPolicyResult, MetadataPolicyViolation
};

// Parser DSL exports - PolicyRule from parser
pub use parser::{
    PolicyRule as DslPolicyRule, RuleSeverity, Condition, ConditionType,
    Operator, ConditionValue, RuleAction, DslParser, ParseError,
    RuleEvaluator, EvaluationContext, EvaluationResult, RuleMatch,
    PolicyRuleLoader, RuleStatistics, LoadError
};

pub use policy_engine::*;

// Policy history exports - PolicyVersion from policy_history
pub use policy_history::{
    PolicyVersion as HistoryPolicyVersion, PolicyContent,
    VersionMetadata as HistoryVersionMetadata, PolicyHistory,
    VersionDiff, HistoryError
};

pub use policy_loader::*;

// Policy metadata exports - PolicyLifecycle from policy_metadata (metadata struct)
pub use policy_metadata::{
    PolicyMetadata, PolicyCategory, Severity, PolicyStatus,
    PolicyOwnership, PolicyLifecycle as MetadataLifecycle,
    DeprecationInfo, PolicyRevision, PolicyLinks, PolicyMetrics,
    PolicyWithMetadata
};

pub use policy_repository::*;
pub use policy_types::*;

// Policy version exports - PolicyVersion from policy_version (version metadata)
pub use policy_version::{
    PolicyVersion as VersionInfo, PolicyVersionManager
};

pub use zero_network::*;
