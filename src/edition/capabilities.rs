use super::EditionContext;

/// Feature capabilities determined by edition mode
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub allow_predict: bool,
    pub allow_explain_full: bool,
    pub allow_autofix: bool,
    pub allow_mapping_deep: bool,
    pub allow_trend: bool,
    pub allow_policy_enforce: bool,
    pub allow_slo_enforce: bool,
}

impl Capabilities {
    /// Create capabilities based on edition context
    pub fn from_edition(edition: &EditionContext) -> Self {
        if edition.is_premium() {
            Self {
                allow_predict: true,
                allow_explain_full: true,
                allow_autofix: true,
                allow_mapping_deep: true,
                allow_trend: true,
                allow_policy_enforce: true,
                allow_slo_enforce: true,
            }
        } else {
            // Free edition - basic detect/explain-lite only
            Self {
                allow_predict: false,
                allow_explain_full: false,
                allow_autofix: false,
                allow_mapping_deep: false,
                allow_trend: false,
                allow_policy_enforce: false,
                allow_slo_enforce: false,
            }
        }
    }
}
