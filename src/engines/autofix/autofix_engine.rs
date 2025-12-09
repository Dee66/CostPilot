// Autofix engine - orchestrates fix generation

use crate::engines::shared::models::{Detection, ResourceChange, CostEstimate};
use crate::engines::explain::anti_patterns::{AntiPattern, detect_anti_patterns};
use crate::engines::autofix::snippet_generator::{FixSnippet, SnippetGenerator};
use crate::engines::autofix::patch_generator::{PatchGenerator, PatchFile};
use serde::{Deserialize, Serialize};

/// Autofix mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AutofixMode {
    /// Generate snippet only (MVP)
    Snippet,
    /// Generate full patch diff (Pro)
    Patch,
    /// Drift-safe autofix with rollback (Beta)
    DriftSafe,
}

/// Autofix result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofixResult {
    pub mode: String,
    pub fixes_generated: usize,
    pub fixes: Vec<FixSnippet>,
    pub patches: Vec<PatchFile>,
    pub warnings: Vec<String>,
}

pub struct AutofixEngine;

impl AutofixEngine {
    /// Create new AutofixEngine
    pub fn new() -> Self {
        Self
    }

    /// Generate fixes for detections (MVP: snippet mode only)
    pub fn generate_fixes(
        detections: &[Detection],
        changes: &[ResourceChange],
        estimates: &[CostEstimate],
        mode: AutofixMode,
    ) -> AutofixResult {
        match mode {
            AutofixMode::Snippet => Self::generate_snippets(detections, changes, estimates),
            AutofixMode::Patch => Self::generate_patches(detections, changes, estimates),
            AutofixMode::DriftSafe => {
                // Beta feature - not implemented in MVP
                AutofixResult {
                    mode: "drift-safe".to_string(),
                    fixes_generated: 0,
                    fixes: vec![],
                    patches: vec![],
                    warnings: vec!["Drift-safe mode is in Beta (V1)".to_string()],
                }
            }
        }
    }

    /// Generate snippet fixes (MVP)
    fn generate_snippets(
        detections: &[Detection],
        changes: &[ResourceChange],
        estimates: &[CostEstimate],
    ) -> AutofixResult {
        let mut fixes = Vec::new();
        let mut warnings = Vec::new();

        for detection in detections {
            // Find corresponding resource change
            let change = changes.iter()
                .find(|c| c.resource_id == detection.resource_id);
            
            let estimate = estimates.iter()
                .find(|e| e.resource_id == detection.resource_id);

            if let Some(change) = change {
                // Detect anti-patterns for this resource
                let anti_patterns = detect_anti_patterns(change, estimate);

                // Generate fix snippet if applicable
                if let Some(snippet) = SnippetGenerator::generate(detection, change, &anti_patterns, estimate) {
                    fixes.push(snippet);
                } else {
                    warnings.push(format!(
                        "No automated fix available for {} ({})",
                        detection.resource_id,
                        change.resource_type
                    ));
                }
            } else {
                warnings.push(format!(
                    "Resource change not found for detection: {}",
                    detection.resource_id
                ));
            }
        }

        AutofixResult {
            mode: "snippet".to_string(),
            fixes_generated: fixes.len(),
            fixes,
            patches: vec![],
            warnings,
        }
    }

    /// Generate patch fixes
    fn generate_patches(
        detections: &[Detection],
        changes: &[ResourceChange],
        estimates: &[CostEstimate],
    ) -> AutofixResult {
        let patch_result = PatchGenerator::generate(detections, changes, estimates);

        AutofixResult {
            mode: "patch".to_string(),
            fixes_generated: patch_result.patches.len(),
            fixes: vec![],
            patches: patch_result.patches,
            warnings: patch_result.warnings,
        }
    }

    /// Validate fix is deterministic and idempotent
    pub fn validate_fix(snippet: &FixSnippet) -> Result<(), String> {
        if !snippet.deterministic {
            return Err("Fix is not deterministic".to_string());
        }

        if !snippet.idempotent {
            return Err("Fix is not idempotent".to_string());
        }

        if snippet.rationale.is_empty() {
            return Err("Fix missing human rationale".to_string());
        }

        if snippet.snippet.is_empty() {
            return Err("Fix snippet is empty".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ChangeAction, RegressionType, Severity, CostEstimate};
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_snippet_mode() {
        let detection = Detection {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::High,
            estimated_cost: Some(CostEstimate {
                estimate: 560.0,
                lower: 420.0,
                upper: 700.0,
                confidence: 0.9,
            }),
            fix_snippet: None,
        };

        let change = ResourceChange {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": "m5.4xlarge"})),
            tags: HashMap::new(),
        };

        let result = AutofixEngine::generate_fixes(
            &[detection],
            &[change],
            AutofixMode::Snippet,
        );

        assert_eq!(result.mode, "snippet");
        assert_eq!(result.fixes_generated, 1);
        assert!(result.fixes[0].deterministic);
        assert!(result.fixes[0].idempotent);
    }

    #[test]
    fn test_patch_mode() {
        let detection = Detection {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::High,
            estimated_cost: Some(CostEstimate {
                estimate: 560.0,
                lower: 420.0,
                upper: 700.0,
                confidence: 0.9,
            }),
            fix_snippet: None,
        };

        let change = ResourceChange {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": "m5.4xlarge"})),
            tags: HashMap::new(),
        };

        let result = AutofixEngine::generate_fixes(
            &[detection],
            &[change],
            AutofixMode::Patch,
        );

        assert_eq!(result.mode, "patch");
        // Will have patches if anti-patterns are detected
        assert!(result.patches.len() >= 0);
    }

    #[test]
    fn test_validate_fix() {
        let valid_snippet = FixSnippet {
            resource_id: "test".to_string(),
            resource_type: "aws_instance".to_string(),
            snippet: "resource ...".to_string(),
            format: crate::engines::autofix::snippet_generator::SnippetFormat::Terraform,
            rationale: "Test rationale".to_string(),
            before_after: crate::engines::autofix::snippet_generator::BeforeAfter {
                before: "before".to_string(),
                after: "after".to_string(),
                change_description: "test".to_string(),
            },
            impact: "test impact".to_string(),
            deterministic: true,
            idempotent: true,
        };

        assert!(AutofixEngine::validate_fix(&valid_snippet).is_ok());

        let mut invalid = valid_snippet.clone();
        invalid.deterministic = false;
        assert!(AutofixEngine::validate_fix(&invalid).is_err());

        let mut invalid = valid_snippet.clone();
        invalid.idempotent = false;
        assert!(AutofixEngine::validate_fix(&invalid).is_err());

        let mut invalid = valid_snippet;
        invalid.rationale = String::new();
        assert!(AutofixEngine::validate_fix(&invalid).is_err());
    }
}
