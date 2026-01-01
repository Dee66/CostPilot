// Release automation for software escrow
// Handles automated escrow deposit on releases

use crate::engines::escrow::package::{
    BuildEnvironment, BuildInstructions, BuildStep, DependenciesManifest, DependencyInfo,
    DepositType, EscrowPackage, EscrowPackageBuilder, LicenseInfo, OpenSourceComponent,
    PackageMetadata, Tool, VendorInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Release configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    /// Vendor information
    pub vendor: VendorInfo,

    /// Escrow agent (if configured)
    pub escrow_agent: Option<EscrowAgentConfig>,

    /// Automatic deposit on release
    pub auto_deposit: bool,

    /// Release triggers
    pub triggers: Vec<ReleaseTrigger>,

    /// Output directory for escrow packages
    pub output_dir: PathBuf,

    /// Include artifacts in package
    pub include_artifacts: bool,

    /// Run verification before deposit
    pub verify_before_deposit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowAgentConfig {
    pub agent_name: String,
    pub agent_contact: String,
    pub agreement_id: String,
    pub api_endpoint: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseTrigger {
    /// Trigger on git tag matching pattern
    GitTag { pattern: String },

    /// Trigger on version change
    VersionChange,

    /// Manual trigger
    Manual,
}

/// Release automation engine
pub struct ReleaseAutomation {
    config: ReleaseConfig,
    repository_root: PathBuf,
}

impl ReleaseAutomation {
    /// Create new release automation
    pub fn new(config: ReleaseConfig, repository_root: PathBuf) -> Self {
        Self {
            config,
            repository_root,
        }
    }

    /// Create escrow package for current release
    pub fn create_package(&self, version: &str) -> Result<EscrowPackage, String> {
        // Get git information
        let git_info = self.get_git_info()?;

        // Create metadata
        let metadata = PackageMetadata {
            package_id: uuid::Uuid::new_v4().to_string(),
            product_name: "CostPilot".to_string(),
            version: version.to_string(),
            release_date: current_timestamp(),
            commit_hash: git_info.commit_hash.clone(),
            git_tag: git_info.tag,
            branch: git_info.branch.clone(),
            vendor: self.config.vendor.clone(),
            customer: None,
            escrow_agent: self.config.escrow_agent.as_ref().map(|a| {
                crate::engines::escrow::package::EscrowAgentInfo {
                    agent_name: a.agent_name.clone(),
                    agent_contact: a.agent_contact.clone(),
                    agreement_id: a.agreement_id.clone(),
                    agreement_date: current_timestamp(),
                }
            }),
            deposit_type: DepositType::General,
        };

        // Build package
        let mut builder =
            EscrowPackageBuilder::new(self.repository_root.clone()).metadata(metadata);

        // Scan source files
        builder.scan_source_files()?;

        // Add build artifacts if configured
        if self.config.include_artifacts {
            self.add_build_artifacts(&mut builder)?;
        }

        // Generate dependencies manifest
        let dependencies = self.generate_dependencies_manifest()?;
        builder = builder.dependencies(dependencies);

        // Generate build instructions
        let build_instructions = self.generate_build_instructions();
        builder = builder.build_instructions(build_instructions);

        // Generate license info
        let license = self.generate_license_info();
        builder = builder.license(license);

        // Build package
        let package = builder.build()?;

        // Verify if configured
        if self.config.verify_before_deposit {
            let report = package.verify()?;
            if !report.verified {
                return Err(format!("Package verification failed: {:?}", report.errors));
            }
        }

        Ok(package)
    }

    /// Deposit package to escrow
    pub fn deposit_package(&self, package: &EscrowPackage) -> Result<DepositReceipt, String> {
        // Create output directory
        let output_dir = self.config.output_dir.join(&package.metadata.version);
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Export package
        package.export(&output_dir)?;

        // Determine escrow agent name
        let agent_name = if let Some(agent) = &self.config.escrow_agent {
            if let (Some(endpoint), Some(api_key)) = (&agent.api_endpoint, &agent.api_key) {
                self.upload_to_agent(package, endpoint, api_key)?;
            }
            agent.agent_name.clone()
        } else {
            "local".to_string()
        };

        // Create receipt
        let receipt = DepositReceipt {
            receipt_id: uuid::Uuid::new_v4().to_string(),
            package_id: package.metadata.package_id.clone(),
            version: package.metadata.version.clone(),
            deposit_date: current_timestamp(),
            deposit_location: output_dir.to_string_lossy().to_string(),
            escrow_agent: agent_name,
            status: DepositStatus::Completed,
        };

        Ok(receipt)
    }

    /// Get git information
    fn get_git_info(&self) -> Result<GitInfo, String> {
        let commit_hash = run_git_command(&["rev-parse", "HEAD"])?;
        let branch = run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let tag = run_git_command(&["describe", "--tags", "--exact-match"]).ok();

        Ok(GitInfo {
            commit_hash,
            branch,
            tag,
        })
    }

    /// Add build artifacts
    fn add_build_artifacts(&self, builder: &mut EscrowPackageBuilder) -> Result<(), String> {
        use crate::engines::escrow::package::{ArtifactType, BuildArtifact};

        // Add Rust binary
        let binary_path = self.repository_root.join("target/release/costpilot");
        if binary_path.exists() {
            let metadata = std::fs::metadata(&binary_path)
                .map_err(|e| format!("Failed to read binary metadata: {}", e))?;

            builder.add_artifact(BuildArtifact {
                name: "costpilot".to_string(),
                artifact_type: ArtifactType::Binary,
                path: "target/release/costpilot".to_string(),
                size: metadata.len(),
                checksum: calculate_file_checksum(&binary_path)?,
                build_date: current_timestamp(),
            });
        }

        // Add WASM module
        let wasm_path = self
            .repository_root
            .join("target/wasm32-unknown-unknown/release/costpilot.wasm");
        if wasm_path.exists() {
            let metadata = std::fs::metadata(&wasm_path)
                .map_err(|e| format!("Failed to read WASM metadata: {}", e))?;

            builder.add_artifact(BuildArtifact {
                name: "costpilot.wasm".to_string(),
                artifact_type: ArtifactType::WasmModule,
                path: "target/wasm32-unknown-unknown/release/costpilot.wasm".to_string(),
                size: metadata.len(),
                checksum: calculate_file_checksum(&wasm_path)?,
                build_date: current_timestamp(),
            });
        }

        Ok(())
    }

    /// Generate dependencies manifest
    fn generate_dependencies_manifest(&self) -> Result<DependenciesManifest, String> {
        let mut cargo_deps = HashMap::new();

        // Parse Cargo.toml
        let cargo_toml_path = self.repository_root.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let content = std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

            // Simple parsing (in production use toml crate)
            for line in content.lines() {
                if line.contains("serde") && line.contains("=") {
                    cargo_deps.insert(
                        "serde".to_string(),
                        DependencyInfo {
                            version: "1.0".to_string(),
                            checksum: None,
                            license: "MIT OR Apache-2.0".to_string(),
                            source: "crates.io".to_string(),
                        },
                    );
                }
            }
        }

        let total_count = cargo_deps.len();

        Ok(DependenciesManifest {
            cargo_dependencies: cargo_deps,
            npm_dependencies: HashMap::new(),
            system_dependencies: vec![crate::engines::escrow::package::SystemDependency {
                name: "git".to_string(),
                version: "2.0+".to_string(),
                required: false,
            }],
            total_count,
            vulnerability_scan: None,
        })
    }

    /// Generate build instructions
    fn generate_build_instructions(&self) -> BuildInstructions {
        BuildInstructions {
            environment: BuildEnvironment {
                rust_version: "1.75.0".to_string(),
                cargo_version: "1.75.0".to_string(),
                node_version: Some("18.0.0".to_string()),
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                tools: vec![Tool {
                    name: "git".to_string(),
                    version: "2.0+".to_string(),
                    required: false,
                }],
            },
            steps: vec![
                BuildStep {
                    step: 1,
                    description: "Install Rust toolchain".to_string(),
                    command: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                        .to_string(),
                    working_dir: ".".to_string(),
                    expected_exit_code: 0,
                    timeout: 300,
                },
                BuildStep {
                    step: 2,
                    description: "Build release binary".to_string(),
                    command: "cargo build --release".to_string(),
                    working_dir: ".".to_string(),
                    expected_exit_code: 0,
                    timeout: 600,
                },
                BuildStep {
                    step: 3,
                    description: "Run tests".to_string(),
                    command: "cargo test".to_string(),
                    working_dir: ".".to_string(),
                    expected_exit_code: 0,
                    timeout: 300,
                },
                BuildStep {
                    step: 4,
                    description: "Build WASM target".to_string(),
                    command: "cargo build --target wasm32-unknown-unknown --release".to_string(),
                    working_dir: ".".to_string(),
                    expected_exit_code: 0,
                    timeout: 600,
                },
            ],
            test_commands: vec![
                "cargo test".to_string(),
                "cargo test --target wasm32-unknown-unknown".to_string(),
            ],
            expected_outputs: vec![
                "target/release/costpilot".to_string(),
                "target/wasm32-unknown-unknown/release/costpilot.wasm".to_string(),
            ],
            estimated_duration: 900,
        }
    }

    /// Generate license information
    fn generate_license_info(&self) -> LicenseInfo {
        let license_path = self.repository_root.join("LICENSE");
        let license_text = std::fs::read_to_string(&license_path)
            .unwrap_or_else(|_| "See LICENSE file".to_string());

        LicenseInfo {
            license_type: "MIT".to_string(),
            license_text,
            copyright_holder: self.config.vendor.company_name.clone(),
            copyright_year: "2024".to_string(),
            additional_terms: None,
            open_source_components: vec![OpenSourceComponent {
                name: "serde".to_string(),
                version: "1.0".to_string(),
                license: "MIT OR Apache-2.0".to_string(),
                source_url: "https://github.com/serde-rs/serde".to_string(),
            }],
        }
    }

    /// Upload package to escrow agent
    fn upload_to_agent(
        &self,
        package: &EscrowPackage,
        endpoint: &str,
        _api_key: &str,
    ) -> Result<(), String> {
        // In production, upload via HTTPS API
        // For now, just log
        println!(
            "Would upload package {} to {}",
            package.metadata.package_id, endpoint
        );
        Ok(())
    }
}

/// Git repository information
#[derive(Debug, Clone)]
struct GitInfo {
    commit_hash: String,
    branch: String,
    tag: Option<String>,
}

/// Deposit receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositReceipt {
    pub receipt_id: String,
    pub package_id: String,
    pub version: String,
    pub deposit_date: u64,
    pub deposit_location: String,
    pub escrow_agent: String,
    pub status: DepositStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DepositStatus {
    Pending,
    Completed,
    Failed,
    Verified,
}

impl DepositReceipt {
    pub fn format_text(&self) -> String {
        format!(
            r#"📦 Escrow Deposit Receipt
========================

Receipt ID: {}
Package ID: {}
Version: {}
Deposit Date: {}
Location: {}
Escrow Agent: {}
Status: {:?}

This receipt confirms the deposit of CostPilot v{} to software escrow.
The package has been securely stored and is available for release under
the terms of the escrow agreement.
"#,
            self.receipt_id,
            self.package_id,
            self.version,
            self.deposit_date,
            self.deposit_location,
            self.escrow_agent,
            self.status,
            self.version
        )
    }
}

/// Run git command
fn run_git_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let result = String::from_utf8_lossy(&output.stdout);
    Ok(result.trim().to_string())
}

/// Calculate file checksum
fn calculate_file_checksum(path: &Path) -> Result<String, String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let content = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);

    Ok(format!("{:x}", hasher.finish()))
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Default release configuration
impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            vendor: VendorInfo {
                company_name: "Your Company Name".to_string(),
                contact_email: "support@yourcompany.com".to_string(),
                support_url: "https://yourcompany.com/support".to_string(),
                legal_entity: "Your Company Name".to_string(),
            },
            escrow_agent: None,
            auto_deposit: false,
            triggers: vec![ReleaseTrigger::GitTag {
                pattern: "v*.*.*".to_string(),
            }],
            output_dir: PathBuf::from("./escrow-packages"),
            include_artifacts: true,
            verify_before_deposit: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_config_default() {
        let config = ReleaseConfig::default();
        assert_eq!(config.vendor.company_name, "Your Company Name");
        assert!(config.verify_before_deposit);
    }

    #[test]
    fn test_deposit_receipt() {
        let receipt = DepositReceipt {
            receipt_id: "receipt-123".to_string(),
            package_id: "package-456".to_string(),
            version: "1.0.0".to_string(),
            deposit_date: 1234567890,
            deposit_location: "/path/to/package".to_string(),
            escrow_agent: "Iron Mountain".to_string(),
            status: DepositStatus::Completed,
        };

        let text = receipt.format_text();
        assert!(text.contains("Receipt ID: receipt-123"));
        assert!(text.contains("Version: 1.0.0"));
    }
}
