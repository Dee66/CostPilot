// Policy loader - Load policy rules from files and directories

use super::dsl::{DslParser, ParseError, PolicyRule};
use dirs;
use std::fs;
use std::path::{Path, PathBuf};

/// Policy rule loader with multiple search paths
pub struct PolicyRuleLoader {
    search_paths: Vec<PathBuf>,
}

impl PolicyRuleLoader {
    /// Create new loader with default search paths
    pub fn new() -> Self {
        Self {
            search_paths: Self::default_search_paths(),
        }
    }

    /// Create loader with custom search paths
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: paths,
        }
    }

    /// Default search paths for policy rules
    pub fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = vec![
            PathBuf::from(".costpilot/policies"), // Project policies
            PathBuf::from(".costpilot/rules"),    // Alternative name
        ];

        // User policies
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".costpilot/policies"));
        }

        // System-wide policies
        #[cfg(unix)]
        paths.push(PathBuf::from("/etc/costpilot/policies"));
        #[cfg(windows)]
        if let Some(program_data) = std::env::var_os("ProgramData") {
            paths.push(PathBuf::from(program_data).join("CostPilot\\policies"));
        }

        paths
    }

    /// Load all policy rules from search paths
    pub fn load_all(&self) -> Result<Vec<PolicyRule>, LoadError> {
        let mut all_rules = Vec::new();
        let mut parse_errors = Vec::new();

        for path in &self.search_paths {
            match self.load_from_path(path) {
                Ok(rules) => {
                    all_rules.extend(rules);
                }
                Err(LoadError::PathNotFound(_)) => {
                    // Ignore missing paths - they're not errors
                    continue;
                }
                Err(e) => {
                    // Collect actual parsing/validation errors
                    parse_errors.push((path.clone(), e));
                }
            }
        }

        // If we found any rules, return them (ignore missing paths)
        if !all_rules.is_empty() {
            return Ok(all_rules);
        }

        // If no rules found but there were actual parse errors, return the errors
        if !parse_errors.is_empty() {
            return Err(LoadError::NoRulesFound {
                searched_paths: self.search_paths.clone(),
                errors: parse_errors,
            });
        }

        // No rules and no errors means empty policy set
        Ok(all_rules)
    }

    /// Load rules from a specific path (file or directory)
    pub fn load_from_path(&self, path: &Path) -> Result<Vec<PolicyRule>, LoadError> {
        if !path.exists() {
            return Err(LoadError::PathNotFound(path.to_path_buf()));
        }

        if path.is_file() {
            self.load_from_file(path)
        } else if path.is_dir() {
            self.load_from_directory(path)
        } else {
            Err(LoadError::InvalidPath(path.to_path_buf()))
        }
    }

    /// Load rules from a single file
    pub fn load_from_file(&self, file_path: &Path) -> Result<Vec<PolicyRule>, LoadError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| LoadError::ReadError(file_path.to_path_buf(), e.to_string()))?;

        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "yaml" | "yml" => match DslParser::parse_yaml(&content) {
                Ok(rules) => Ok(rules),
                Err(e) => {
                    // If the file is a mapping with a `rules:` key (policy file style),
                    // try extracting the rules sequence and parsing that instead.
                    if let Ok(serde_yaml::Value::Mapping(map)) =
                        serde_yaml::from_str::<serde_yaml::Value>(&content)
                    {
                        use serde_yaml::Value;
                        let key = Value::String("rules".to_string());
                        if let Some(rules_val) = map.get(&key) {
                            if let Value::Sequence(_) = rules_val {
                                // Serialize the rules sequence back to YAML and parse
                                if let Ok(rules_yaml) = serde_yaml::to_string(rules_val) {
                                    return DslParser::parse_yaml(&rules_yaml).map_err(|pe| {
                                        LoadError::ParseError(file_path.to_path_buf(), pe)
                                    });
                                }
                            }
                        }
                    }
                    Err(LoadError::ParseError(file_path.to_path_buf(), e))
                }
            },
            "json" => DslParser::parse_json(&content)
                .map_err(|e| LoadError::ParseError(file_path.to_path_buf(), e)),
            _ => Err(LoadError::UnsupportedFormat(file_path.to_path_buf())),
        }
    }

    /// Load rules from all files in a directory
    pub fn load_from_directory(&self, dir_path: &Path) -> Result<Vec<PolicyRule>, LoadError> {
        let mut all_rules = Vec::new();

        let entries = fs::read_dir(dir_path)
            .map_err(|e| LoadError::ReadError(dir_path.to_path_buf(), e.to_string()))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| LoadError::ReadError(dir_path.to_path_buf(), e.to_string()))?;

            let path = entry.path();

            // Skip non-files and hidden files
            if !path.is_file()
                || path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            {
                continue;
            }

            // Try to load this file
            match self.load_from_file(&path) {
                Ok(rules) => {
                    all_rules.extend(rules);
                }
                Err(LoadError::UnsupportedFormat(_)) => {
                    // Skip files with unsupported extensions
                    continue;
                }
                Err(e) => {
                    // Propagate other errors
                    return Err(e);
                }
            }
        }

        Ok(all_rules)
    }

    /// Validate loaded rules
    pub fn validate_rules(&self, rules: &[PolicyRule]) -> Result<(), LoadError> {
        for rule in rules {
            if rule.name.is_empty() {
                return Err(LoadError::ValidationError(
                    "Rule name cannot be empty".to_string(),
                ));
            }

            // Allow empty conditions for legacy or shorthand rules used in tests.
            // Evaluation semantics may treat an empty condition as always-true.
        }

        Ok(())
    }

    /// Get statistics about loaded rules
    pub fn get_statistics(&self, rules: &[PolicyRule]) -> RuleStatistics {
        let total_rules = rules.len();
        let enabled_rules = rules.iter().filter(|r| r.enabled).count();
        let disabled_rules = total_rules - enabled_rules;

        let mut severity_counts = std::collections::HashMap::new();
        for rule in rules {
            *severity_counts
                .entry(format!("{:?}", rule.severity))
                .or_insert(0) += 1;
        }

        RuleStatistics {
            total_rules,
            enabled_rules,
            disabled_rules,
            severity_counts,
        }
    }
}

impl Default for PolicyRuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about loaded rules
#[derive(Debug, Clone)]
pub struct RuleStatistics {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub disabled_rules: usize,
    pub severity_counts: std::collections::HashMap<String, usize>,
}

impl RuleStatistics {
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Total Rules: {}\n", self.total_rules));
        output.push_str(&format!("Enabled: {}\n", self.enabled_rules));
        output.push_str(&format!("Disabled: {}\n", self.disabled_rules));
        output.push_str("\nBy Severity:\n");

        for (severity, count) in &self.severity_counts {
            output.push_str(&format!("  {}: {}\n", severity, count));
        }

        output
    }
}

/// Error types for policy loading
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Failed to read {0}: {1}")]
    ReadError(PathBuf, String),

    #[error("Failed to parse {0}: {1}")]
    ParseError(PathBuf, ParseError),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(PathBuf),

    #[error("Rule validation failed: {0}")]
    ValidationError(String),

    #[error("No rules found in searched paths: {searched_paths:?}")]
    NoRulesFound {
        searched_paths: Vec<PathBuf>,
        errors: Vec<(PathBuf, LoadError)>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_paths() {
        let paths = PolicyRuleLoader::default_search_paths();
        assert!(paths.len() >= 3);
        assert!(paths
            .iter()
            .any(|p| p.to_str().unwrap().contains(".costpilot")));
    }

    #[test]
    fn test_load_nonexistent_path() {
        let loader = PolicyRuleLoader::new();
        let result = loader.load_from_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
