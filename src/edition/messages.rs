// User-facing edition upgrade messages

/// Generate upgrade message for gated features
pub fn upgrade_message(feature: &str) -> String {
    format!(
        "{} requires CostPilot Premium.\nUpgrade: https://shieldcraft-ai.com/costpilot/upgrade",
        feature
    )
}

/// Generate feature comparison message
pub fn feature_comparison() -> String {
    r#"
CostPilot Free vs Premium:

FREE:
  ✓ Basic cost prediction (static heuristics)
  ✓ Explain lite (top 5 patterns)
  ✓ Mapping (depth 1)
  ✓ Policy lint-only
  ✓ SLO validation-only

PREMIUM:
  ✓ Advanced prediction (ML-enhanced)
  ✓ Full explanation chains
  ✓ Deep dependency mapping
  ✓ Autofix with drift safety
  ✓ Trend tracking & history
  ✓ Policy enforcement (blocking)
  ✓ SLO enforcement (blocking)

Upgrade: https://shieldcraft-ai.com/costpilot/upgrade
"#
    .to_string()
}
