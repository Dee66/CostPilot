// Host bridge - call ProEngine from host Rust code

use crate::edition::EditionContext;
use crate::engines::shared::error_model::CostPilotError;

/// Call ProEngine with a request
/// Returns UpgradeRequired if Premium engine not available
pub fn call_pro_engine(
    edition: &EditionContext,
    req: super::api::ProEngineRequest,
) -> Result<super::api::ProEngineResponse, CostPilotError> {
    let handle = edition
        .pro
        .as_ref()
        .ok_or_else(|| CostPilotError::upgrade_required("Premium engine not available"))?;

    handle.execute(req).map_err(|e| {
        CostPilotError::new(
            "E_PRO_ENGINE",
            crate::engines::shared::error_model::ErrorCategory::InternalError,
            e,
        )
    })
}

/// Check if ProEngine is available (Premium mode)
pub fn is_pro_available(edition: &EditionContext) -> bool {
    edition.pro.is_some()
}
