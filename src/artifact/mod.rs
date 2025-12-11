// Artifact parsing and normalization module

mod artifact_normalizer;
mod artifact_types;
mod cdk_parser;
mod cloudformation_parser;

pub use artifact_normalizer::*;
pub use artifact_types::*;
pub use cdk_parser::*;
pub use cloudformation_parser::*;

/// Parse an artifact from a file, auto-detecting the format
pub fn parse_artifact_file(path: &str) -> ArtifactResult<Artifact> {
    let content = std::fs::read_to_string(path)?;
    parse_artifact(&content, path)
}

/// Parse an artifact from content, auto-detecting the format
pub fn parse_artifact(content: &str, hint: &str) -> ArtifactResult<Artifact> {
    // Try to detect format from content or filename
    if hint.contains("cdk.out") || hint.ends_with(".template.json") {
        // CDK output
        let parser = CdkParser::new();
        return parser.parse(content);
    }

    if hint.ends_with(".yaml")
        || hint.ends_with(".yml")
        || content.trim_start().starts_with("AWSTemplateFormatVersion")
    {
        // CloudFormation template
        let parser = CloudFormationParser::new();
        return parser.parse(content);
    }

    if hint.ends_with(".json") {
        // Try CloudFormation first, then fall back to others
        let parser = CloudFormationParser::new();
        if let Ok(artifact) = parser.parse(content) {
            return Ok(artifact);
        }
    }

    Err(ArtifactError::UnsupportedFormat(
        "Could not detect artifact format".to_string(),
    ))
}
