// Rollback patch generator for drift-safe autofix

use crate::engines::autofix::patch_generator::{PatchFile, PatchHunk, PatchLine, PatchLineType, PatchMetadata};
use crate::engines::autofix::drift_safety::drift_checksum::DriftChecksum;
use crate::engines::shared::models::{Detection, ResourceChange};

/// Rollback patch generator for reverting infrastructure drift
pub struct RollbackPatchGenerator;

impl RollbackPatchGenerator {
    /// Create new rollback patch generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a rollback patch to revert drifted infrastructure to expected state
    pub fn generate_rollback_patch(
        &self,
        detection: &Detection,
        change: &ResourceChange,
        drift_result: &DriftChecksum,
    ) -> Result<PatchFile, Box<dyn std::error::Error>> {
        let filename = self.generate_filename(change);
        let hunks = self.generate_rollback_hunks(change, drift_result)?;
        let metadata = self.generate_patch_metadata(detection, change, drift_result);

        Ok(PatchFile {
            resource_id: detection.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            filename,
            hunks,
            metadata,
        })
    }

    /// Generate appropriate filename for the patch
    fn generate_filename(&self, change: &ResourceChange) -> String {
        match change.resource_type.as_str() {
            "aws_instance" => format!("{}.tf", Self::resource_name_from_id(&change.resource_id)),
            "aws_s3_bucket" => format!("{}.tf", Self::resource_name_from_id(&change.resource_id)),
            "aws_db_instance" => format!("{}.tf", Self::resource_name_from_id(&change.resource_id)),
            "aws_lambda_function" => format!("{}.tf", Self::resource_name_from_id(&change.resource_id)),
            _ => format!("{}.tf", Self::resource_name_from_id(&change.resource_id)),
        }
    }

    /// Extract resource name from resource ID (e.g., "aws_instance.web" -> "web")
    fn resource_name_from_id(resource_id: &str) -> String {
        resource_id.split('.').next_back().unwrap_or("resource").to_string()
    }

    /// Generate rollback hunks to revert drifted attributes
    fn generate_rollback_hunks(
        &self,
        change: &ResourceChange,
        drift_result: &DriftChecksum,
    ) -> Result<Vec<PatchHunk>, Box<dyn std::error::Error>> {
        let mut hunks = Vec::new();

        // Generate hunks for each drifted attribute
        for drifted_attr in &drift_result.drifted_attributes {
            let hunk = self.generate_attribute_rollback_hunk(change, drifted_attr)?;
            hunks.push(hunk);
        }

        Ok(hunks)
    }

    /// Generate a hunk to rollback a single drifted attribute
    fn generate_attribute_rollback_hunk(
        &self,
        change: &ResourceChange,
        drifted_attr: &crate::engines::autofix::drift_safety::drift_checksum::DriftedAttribute,
    ) -> Result<PatchHunk, Box<dyn std::error::Error>> {
        let resource_name = Self::resource_name_from_id(&change.resource_id);

        // Generate Terraform resource block
        let old_lines = self.generate_resource_block(&change.resource_type, &resource_name, &drifted_attr.actual_value, &drifted_attr.path);
        let new_lines = self.generate_resource_block(&change.resource_type, &resource_name, &drifted_attr.expected_value, &drifted_attr.path);

        // Convert to patch lines
        let old_patch_lines = old_lines.into_iter().map(|line| PatchLine {
            line_type: PatchLineType::Deletion,
            content: line,
            indent_level: 0,
        }).collect::<Vec<_>>();

        let new_patch_lines = new_lines.into_iter().map(|line| PatchLine {
            line_type: PatchLineType::Addition,
            content: line,
            indent_level: 0,
        }).collect::<Vec<_>>();

        Ok(PatchHunk {
            old_start: 1,
            old_count: old_patch_lines.len(),
            new_start: 1,
            new_count: new_patch_lines.len(),
            lines: [old_patch_lines, new_patch_lines].concat(),
            context_before: vec![],
            context_after: vec![],
        })
    }

    /// Generate Terraform resource block with specific attribute value
    fn generate_resource_block(&self, resource_type: &str, resource_name: &str, attr_value: &str, attr_path: &str) -> Vec<String> {
        vec![
            format!("resource \"{}\" \"{}\" {{", resource_type, resource_name),
            format!("  {} = {}", attr_path, attr_value),
            "}".to_string(),
        ]
    }

    /// Generate patch metadata
    fn generate_patch_metadata(
        &self,
        detection: &Detection,
        _change: &ResourceChange,
        drift_result: &DriftChecksum,
    ) -> PatchMetadata {
        PatchMetadata {
            cost_before: detection.estimated_cost.unwrap_or(0.0),
            cost_after: 0.0, // Rollback should reduce cost
            monthly_savings: detection.estimated_cost.unwrap_or(0.0),
            confidence: 0.95, // High confidence for drift rollback
            anti_patterns: vec!["infrastructure_drift".to_string()],
            rationale: format!(
                "Rollback infrastructure drift for {} - reverting {} drifted attributes to expected state",
                detection.resource_id,
                drift_result.drifted_attributes.len()
            ),
            simulation_required: true, // Always require simulation for infrastructure changes
            beta: true, // Drift-safe autofix is still in beta
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::autofix::drift_safety::drift_checksum::{DriftChecksum, DriftedAttribute, DriftSeverity};
    use crate::engines::shared::models::{ChangeAction, RegressionType, Severity};

    #[test]
    fn test_generate_rollback_patch() {
        let generator = RollbackPatchGenerator::new();

        let detection = Detection {
            rule_id: "drift_detected".to_string(),
            resource_id: "aws_instance.web".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::High,
            severity_score: 80,
            message: "Infrastructure drift detected".to_string(),
            estimated_cost: Some(100.0),
            fix_snippet: None,
        };

        let change = ResourceChange::builder()
            .resource_id("aws_instance.web".to_string())
            .resource_type("aws_instance".to_string())
            .action(ChangeAction::Update)
            .old_config(serde_json::json!({"instance_type": "t3.medium"}))
            .new_config(serde_json::json!({"instance_type": "t3.xlarge"}))
            .build();

        let drift_result = DriftChecksum {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            current_checksum: "current".to_string(),
            expected_checksum: "expected".to_string(),
            drift_detected: true,
            checked_at: "2024-01-01T00:00:00Z".to_string(),
            drifted_attributes: vec![DriftedAttribute {
                path: "instance_type".to_string(),
                expected_value: "\"t3.medium\"".to_string(),
                actual_value: "\"t3.xlarge\"".to_string(),
                severity: DriftSeverity::High,
            }],
        };

        let result = generator.generate_rollback_patch(&detection, &change, &drift_result);
        assert!(result.is_ok());

        let patch = result.unwrap();
        assert_eq!(patch.resource_id, "aws_instance.web");
        assert_eq!(patch.resource_type, "aws_instance");
        assert!(!patch.hunks.is_empty());
        assert!(patch.metadata.simulation_required);
        assert!(patch.metadata.beta);
    }
}
