// Edition gating - enforce Premium requirements

use super::{EditionContext, EditionMode};
use crate::engines::shared::error_model::CostPilotError;

/// Require Premium edition for a feature
pub fn require_premium(feature: &str, edition: &EditionContext) -> Result<(), CostPilotError> {
    if edition.mode == EditionMode::Premium {
        Ok(())
    } else {
        Err(CostPilotError::upgrade_required(format!(
            "{} requires CostPilot Premium",
            feature
        )))
    }
}
