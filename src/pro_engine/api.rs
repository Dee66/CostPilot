// ProEngine API - Unified request/response enums for WASM boundary

use crate::engines::autofix::{AutofixMode, AutofixResult};
use crate::engines::detection::Detection;
use crate::engines::detection::ResourceChange;
use crate::engines::explain::Explanation;
use crate::engines::mapping::DependencyGraph;
use crate::engines::policy::PolicyResult;
use crate::engines::prediction::CostEstimate;
use crate::engines::slo::SloReport;
use crate::engines::trend::CostSnapshot;

#[derive(Debug, Clone)]
pub enum ProEngineRequest {
    Predict {
        changes: Vec<ResourceChange>,
    },
    Explain {
        detections: Vec<Detection>,
        changes: Vec<ResourceChange>,
        estimates: Vec<CostEstimate>,
    },
    Autofix {
        detections: Vec<Detection>,
        changes: Vec<ResourceChange>,
        estimates: Vec<CostEstimate>,
        mode: AutofixMode,
    },
    MapDeep {
        changes: Vec<ResourceChange>,
        max_depth: u32,
    },
    TrendSnapshot {
        changes: Vec<ResourceChange>,
        metadata: Option<String>,
    },
    PolicyEnforce {
        changes: Vec<ResourceChange>,
        estimates: Vec<CostEstimate>,
    },
    SloEnforce {
        snapshot: Box<CostSnapshot>,
    },
}

#[derive(Debug, Clone)]
pub enum ProEngineResponse {
    Predict(Vec<CostEstimate>),
    Explain(Vec<Explanation>),
    Autofix(AutofixResult),
    MapDeep(DependencyGraph),
    TrendSnapshot(CostSnapshot),
    PolicyEnforce(PolicyResult),
    SloEnforce(SloReport),
}

/// Trait for ProEngine executor implementation
pub trait ProEngineExecutor {
    fn execute(&self, req: ProEngineRequest) -> Result<ProEngineResponse, String>;
}
