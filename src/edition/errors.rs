// Edition-specific errors

use super::EditionContext;

/// Error when Premium feature is used in Free edition
#[derive(Debug)]
pub struct UpgradeRequired {
    pub feature: &'static str,
}

impl std::fmt::Display for UpgradeRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} requires CostPilot Premium", self.feature)
    }
}

impl std::error::Error for UpgradeRequired {}

/// Require Premium edition for a feature
pub fn require_premium(
    edition: &EditionContext,
    feature: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if edition.is_free() {
        return Err(Box::new(UpgradeRequired { feature }));
    }
    Ok(())
}
