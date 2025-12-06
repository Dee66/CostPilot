// CLI commands for software escrow management

use crate::engines::escrow::{
    EscrowPackage, ReleaseAutomation, ReleaseConfig, RecoveryOrchestrator,
    RecoveryPlaybook, VendorInfo,
};
use std::path::PathBuf;

/// Escrow CLI commands
#[derive(Debug)]
pub enum EscrowCommand {
    /// Create escrow package for release
    Create {
        version: String,
        output_dir: Option<PathBuf>,
        include_artifacts: bool,
    },
    
    /// Verify escrow package integrity
    Verify {
        package_dir: PathBuf,
    },
    
    /// Generate recovery playbook
    Playbook {
        package_dir: PathBuf,
        output: Option<PathBuf>,
    },
    
    /// Recover from escrow package
    Recover {
        package_dir: PathBuf,
        working_dir: PathBuf,
    },
    
    /// Configure escrow settings
    Configure {
        vendor_name: String,
        contact_email: String,
        support_url: String,
    },
    
    /// List escrow packages
    List {
        escrow_dir: Option<PathBuf>,
    },
}

/// Execute escrow command
pub fn execute_escrow_command(cmd: EscrowCommand) -> Result<String, String> {
    match cmd {
        EscrowCommand::Create { version, output_dir, include_artifacts } => {
            execute_create(&version, output_dir, include_artifacts)
        }
        EscrowCommand::Verify { package_dir } => {
            execute_verify(&package_dir)
        }
        EscrowCommand::Playbook { package_dir, output } => {
            execute_playbook(&package_dir, output)
        }
        EscrowCommand::Recover { package_dir, working_dir } => {
            execute_recover(&package_dir, &working_dir)
        }
        EscrowCommand::Configure { vendor_name, contact_email, support_url } => {
            execute_configure(&vendor_name, &contact_email, &support_url)
        }
        EscrowCommand::List { escrow_dir } => {
            execute_list(escrow_dir)
        }
    }
}

fn execute_create(
    version: &str,
    output_dir: Option<PathBuf>,
    include_artifacts: bool,
) -> Result<String, String> {
    // Load configuration
    let config = load_escrow_config()?;
    
    // Get repository root
    let repo_root = get_repository_root()?;
    
    // Create release automation
    let mut release_config = ReleaseConfig {
        vendor: config.vendor,
        escrow_agent: config.escrow_agent,
        auto_deposit: false,
        triggers: vec![],
        output_dir: output_dir.unwrap_or_else(|| PathBuf::from("./escrow-packages")),
        include_artifacts,
        verify_before_deposit: true,
    };
    
    let automation = ReleaseAutomation::new(release_config, repo_root);
    
    // Create package
    let package = automation.create_package(version)?;
    
    // Deposit package
    let receipt = automation.deposit_package(&package)?;
    
    Ok(format!("✅ Escrow package created successfully\n\n{}", receipt.format_text()))
}

fn execute_verify(package_dir: &Path) -> Result<String, String> {
    // Load package
    let package = EscrowPackage::load(package_dir)?;
    
    // Verify
    let report = package.verify()?;
    
    Ok(report.format_text())
}

fn execute_playbook(
    package_dir: &Path,
    output: Option<PathBuf>,
) -> Result<String, String> {
    // Load package
    let package = EscrowPackage::load(package_dir)?;
    
    // Generate playbook
    let playbook = RecoveryPlaybook::new(package);
    let content = playbook.generate();
    
    if let Some(output_path) = output {
        std::fs::write(&output_path, &content)
            .map_err(|e| format!("Failed to write playbook: {}", e))?;
        Ok(format!("📖 Recovery playbook written to: {}", output_path.display()))
    } else {
        Ok(content)
    }
}

fn execute_recover(
    package_dir: &Path,
    working_dir: &Path,
) -> Result<String, String> {
    // Load package
    let package = EscrowPackage::load(package_dir)?;
    
    // Create working directory
    std::fs::create_dir_all(working_dir)
        .map_err(|e| format!("Failed to create working directory: {}", e))?;
    
    // Run recovery
    let orchestrator = RecoveryOrchestrator::new(package, working_dir.to_path_buf());
    let report = orchestrator.recover()?;
    
    Ok(report.format_text())
}

fn execute_configure(
    vendor_name: &str,
    contact_email: &str,
    support_url: &str,
) -> Result<String, String> {
    let config = EscrowConfig {
        vendor: VendorInfo {
            company_name: vendor_name.to_string(),
            contact_email: contact_email.to_string(),
            support_url: support_url.to_string(),
            legal_entity: vendor_name.to_string(),
        },
        escrow_agent: None,
    };
    
    // Save configuration
    save_escrow_config(&config)?;
    
    Ok(format!("✅ Escrow configuration saved\n\nVendor: {}\nContact: {}\nSupport: {}",
        vendor_name, contact_email, support_url))
}

fn execute_list(escrow_dir: Option<PathBuf>) -> Result<String, String> {
    let dir = escrow_dir.unwrap_or_else(|| PathBuf::from("./escrow-packages"));
    
    if !dir.exists() {
        return Ok("No escrow packages found.".to_string());
    }
    
    let mut output = String::new();
    output.push_str("📦 Escrow Packages\n");
    output.push_str("==================\n\n");
    
    // Read directory
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            // Try to load package
            if let Ok(package) = EscrowPackage::load(&path) {
                count += 1;
                output.push_str(&format!("{}. Version: {}\n", count, package.metadata.version));
                output.push_str(&format!("   Date: {}\n", package.metadata.release_date));
                output.push_str(&format!("   Commit: {}\n", package.metadata.commit_hash));
                output.push_str(&format!("   Location: {}\n\n", path.display()));
            }
        }
    }
    
    if count == 0 {
        output.push_str("No escrow packages found.\n");
    }
    
    Ok(output)
}

// Helper functions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EscrowConfig {
    vendor: VendorInfo,
    escrow_agent: Option<crate::engines::escrow::EscrowAgentConfig>,
}

fn load_escrow_config() -> Result<EscrowConfig, String> {
    let config_path = get_config_path()?;
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    } else {
        // Return default config
        Ok(EscrowConfig {
            vendor: VendorInfo {
                company_name: "CostPilot Inc.".to_string(),
                contact_email: "support@costpilot.io".to_string(),
                support_url: "https://costpilot.io/support".to_string(),
                legal_entity: "CostPilot Inc.".to_string(),
            },
            escrow_agent: None,
        })
    }
}

fn save_escrow_config(config: &EscrowConfig) -> Result<(), String> {
    let config_path = get_config_path()?;
    
    // Create parent directory
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

fn get_config_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home).join(".costpilot").join("escrow.json"))
}

fn get_repository_root() -> Result<PathBuf, String> {
    // Try to find git repository root
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    
    if !output.status.success() {
        return Err("Not in a git repository".to_string());
    }
    
    let root = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(root.trim()))
}

use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_escrow_config() {
        let config = EscrowConfig {
            vendor: VendorInfo {
                company_name: "Test Corp".to_string(),
                contact_email: "test@example.com".to_string(),
                support_url: "https://example.com".to_string(),
                legal_entity: "Test Corp".to_string(),
            },
            escrow_agent: None,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: EscrowConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.vendor.company_name, "Test Corp");
    }
}
