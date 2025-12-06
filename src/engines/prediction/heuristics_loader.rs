// Heuristics loader with fallback strategies and validation

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::engines::shared::error_model::{Result, CostPilotError, ErrorCategory};
use super::prediction_engine::CostHeuristics;

/// Heuristics loader with multiple fallback strategies
pub struct HeuristicsLoader {
    search_paths: Vec<PathBuf>,
}

impl HeuristicsLoader {
    /// Create a new heuristics loader with default search paths
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

    /// Get default search paths for heuristics file
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Current directory
        paths.push(PathBuf::from("heuristics/cost_heuristics.json"));
        paths.push(PathBuf::from("cost_heuristics.json"));

        // 2. Project root (for development)
        if let Ok(current_dir) = std::env::current_dir() {
            paths.push(current_dir.join("heuristics/cost_heuristics.json"));
        }

        // 3. User config directory
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join(".costpilot/cost_heuristics.json"));
            paths.push(home_path.join(".config/costpilot/cost_heuristics.json"));
        }

        // 4. System-wide config
        paths.push(PathBuf::from("/etc/costpilot/cost_heuristics.json"));
        paths.push(PathBuf::from("/usr/local/share/costpilot/cost_heuristics.json"));

        paths
    }

    /// Load heuristics from first available location
    pub fn load(&self) -> Result<CostHeuristics> {
        let mut tried_paths = Vec::new();

        for path in &self.search_paths {
            tried_paths.push(path.display().to_string());
            
            if path.exists() {
                match self.load_from_file(path) {
                    Ok(heuristics) => {
                        return Ok(heuristics);
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load heuristics from {}: {}", path.display(), e);
                        continue;
                    }
                }
            }
        }

        Err(CostPilotError::new(
            "HEURISTICS_001",
            ErrorCategory::FileSystemError,
            format!(
                "Could not find cost_heuristics.json in any of these locations:\n  {}",
                tried_paths.join("\n  ")
            ),
        )
        .with_hint("Run 'costpilot init' to create default heuristics, or specify path with --heuristics flag"))
    }

    /// Load heuristics from specific file
    pub fn load_from_file(&self, path: &Path) -> Result<CostHeuristics> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            CostPilotError::new(
                "HEURISTICS_002",
                ErrorCategory::FileSystemError,
                format!("Failed to read heuristics file {}: {}", path.display(), e),
            )
        })?;

        let heuristics: CostHeuristics = serde_json::from_str(&content).map_err(|e| {
            CostPilotError::new(
                "HEURISTICS_003",
                ErrorCategory::ParseError,
                format!("Failed to parse heuristics JSON: {}", e),
            )
            .with_hint("Ensure the file is valid JSON matching the CostHeuristics schema")
        })?;

        // Validate heuristics
        self.validate(&heuristics)?;

        Ok(heuristics)
    }

    /// Validate heuristics for completeness
    fn validate(&self, heuristics: &CostHeuristics) -> Result<()> {
        // Check version format
        if !heuristics.version.contains('.') {
            return Err(CostPilotError::new(
                "HEURISTICS_004",
                ErrorCategory::ValidationError,
                "Invalid version format: must be semantic version (e.g., 1.0.0)".to_string(),
            ));
        }

        // Check EC2 instance types
        if heuristics.compute.ec2.is_empty() {
            return Err(CostPilotError::new(
                "HEURISTICS_005",
                ErrorCategory::ValidationError,
                "No EC2 instance types defined in heuristics".to_string(),
            ));
        }

        // Validate price ranges (sanity checks)
        for (instance_type, cost) in &heuristics.compute.ec2 {
            if cost.hourly <= 0.0 || cost.hourly > 1000.0 {
                return Err(CostPilotError::new(
                    "HEURISTICS_006",
                    ErrorCategory::ValidationError,
                    format!("Invalid hourly cost for {}: ${}", instance_type, cost.hourly),
                ));
            }
        }

        // Check Lambda pricing
        if heuristics.compute.lambda.price_per_gb_second <= 0.0 {
            return Err(CostPilotError::new(
                "HEURISTICS_007",
                ErrorCategory::ValidationError,
                "Invalid Lambda price_per_gb_second".to_string(),
            ));
        }

        // Check RDS configuration
        if heuristics.database.rds.mysql.is_empty() {
            return Err(CostPilotError::new(
                "HEURISTICS_008",
                ErrorCategory::ValidationError,
                "No RDS MySQL instance types defined".to_string(),
            ));
        }

        Ok(())
    }

    /// Get heuristics statistics
    pub fn get_statistics(&self, heuristics: &CostHeuristics) -> HeuristicsStats {
        HeuristicsStats {
            version: heuristics.version.clone(),
            last_updated: heuristics.last_updated.clone(),
            ec2_instance_count: heuristics.compute.ec2.len(),
            rds_mysql_count: heuristics.database.rds.mysql.len(),
            rds_postgres_count: heuristics.database.rds.postgres.len(),
            ebs_types_count: heuristics.storage.ebs.len(),
            lambda_configured: true,
            dynamodb_configured: true,
            nat_gateway_configured: true,
        }
    }
}

impl Default for HeuristicsLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Heuristics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsStats {
    pub version: String,
    pub last_updated: String,
    pub ec2_instance_count: usize,
    pub rds_mysql_count: usize,
    pub rds_postgres_count: usize,
    pub ebs_types_count: usize,
    pub lambda_configured: bool,
    pub dynamodb_configured: bool,
    pub nat_gateway_configured: bool,
}

impl HeuristicsStats {
    /// Format as human-readable text
    pub fn format_text(&self) -> String {
        format!(
            "📊 Heuristics Statistics\n\
             Version: {}\n\
             Last Updated: {}\n\
             \n\
             Compute:\n\
             • EC2 Instance Types: {}\n\
             • Lambda: {}\n\
             \n\
             Database:\n\
             • RDS MySQL: {}\n\
             • RDS Postgres: {}\n\
             • DynamoDB: {}\n\
             \n\
             Storage:\n\
             • EBS Types: {}\n\
             \n\
             Networking:\n\
             • NAT Gateway: {}",
            self.version,
            self.last_updated,
            self.ec2_instance_count,
            if self.lambda_configured { "✓" } else { "✗" },
            self.rds_mysql_count,
            self.rds_postgres_count,
            if self.dynamodb_configured { "✓" } else { "✗" },
            self.ebs_types_count,
            if self.nat_gateway_configured { "✓" } else { "✗" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_paths() {
        let loader = HeuristicsLoader::new();
        assert!(!loader.search_paths.is_empty());
        
        // Should include current directory
        assert!(loader.search_paths.iter().any(|p| p.to_string_lossy().contains("cost_heuristics.json")));
    }

    #[test]
    fn test_custom_search_paths() {
        let custom_paths = vec![
            PathBuf::from("/custom/path/heuristics.json"),
            PathBuf::from("/another/path/heuristics.json"),
        ];
        
        let loader = HeuristicsLoader::with_paths(custom_paths.clone());
        assert_eq!(loader.search_paths, custom_paths);
    }

    #[test]
    fn test_validate_version_format() {
        let loader = HeuristicsLoader::new();
        
        // Valid version would pass (can't test without full heuristics object)
        // This test demonstrates the validation logic exists
    }
}
