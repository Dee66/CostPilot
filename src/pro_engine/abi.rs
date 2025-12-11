// ProEngine ABI - request/response types for WASM boundary

use crate::engines::explain::explain_engine::Explanation;
use crate::engines::mapping::DependencyGraph;
use crate::engines::policy::PolicyResult;
use crate::engines::shared::models::{CostEstimate, Detection, ResourceChange};
use crate::engines::trend::CostSnapshot;
use serde::{Deserialize, Serialize};

/// Request types for ProEngine operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProEngineRequest {
    Scan {
        resources: Vec<ResourceChange>,
    },
    Predict {
        resources: Vec<ResourceChange>,
    },
    Explain {
        detection: Detection,
        change: ResourceChange,
        estimate: Option<CostEstimate>,
    },
    Map {
        resources: Vec<ResourceChange>,
        max_depth: u32,
    },
    TrendSnapshot {
        resources: Vec<ResourceChange>,
    },
    PolicyEnforce {
        resources: Vec<ResourceChange>,
        costs: Vec<CostEstimate>,
    },
}

/// Response types from ProEngine operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProEngineResponse {
    ScanResult(Vec<Detection>),
    PredictResult(Vec<CostEstimate>),
    ExplainResult(Explanation),
    MapResult(DependencyGraph),
    TrendSnapshotResult(CostSnapshot),
    PolicyEnforceResult(PolicyResult),
    Error(String),
}
