// Policy lifecycle CLI commands

use colored::*;
use std::path::{Path, PathBuf};

use crate::engines::policy::{
    ApprovalWorkflowManager, PolicyContent, PolicyHistory, PolicyState,
};
use crate::engines::policy::lifecycle::PolicyLifecycle as PolicyLifecycleManager;


/// Execute policy submit command
pub fn cmd_submit(
    policy_file: PathBuf,
    approvers: Vec<String>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "📝 Submitting policy for approval...".bright_blue().bold());

    if verbose {
        println!("  Policy file: {}", policy_file.display());
        println!("  Approvers: {}", approvers.join(", "));
    }

    // Load policy (simplified - would integrate with actual policy loader)
    let policy_id = policy_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid policy filename")?
        .to_string();

    // Create workflow manager
    let mut manager = ApprovalWorkflowManager::new();
    
    // Setup roles (in production, load from config)
    manager.assign_role(
        "policy-approver".to_string(),
        approvers.clone(),
    );

    // Register policy
    manager.register_policy(policy_id.clone(), None)?;

    // Submit for approval
    let assigned_approvers = manager.submit_for_approval(&policy_id, "system@costpilot".to_string())?;

    match format {
        "json" => {
            let output = serde_json::json!({
                "policy_id": policy_id,
                "status": "submitted",
                "approvers": assigned_approvers,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!();
            println!("{}", "✅ Policy submitted for approval".bright_green().bold());
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("Status: {}", "Review".bright_yellow());
            println!();
            println!("Approval requests sent to:");
            for approver in &assigned_approvers {
                println!("  • {}", approver);
            }
            println!();
            println!("Next steps:");
            println!("  1. Wait for approvers to review");
            println!("  2. Check status: costpilot policy status {}", policy_id);
            println!("  3. Activate when approved: costpilot policy activate {}", policy_id);
        }
    }

    Ok(())
}

/// Execute policy approve command
pub fn cmd_approve(
    policy_id: String,
    approver: String,
    comment: Option<String>,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("✅ Approving policy '{}'...", policy_id).bright_blue().bold());

    // In production, would load from persistence layer
    let mut manager = ApprovalWorkflowManager::new();
    
    // For demo, setup mock data
    manager.assign_role(
        "policy-approver".to_string(),
        vec![approver.clone()],
    );
    manager.register_policy(policy_id.clone(), None)?;
    manager.submit_for_approval(&policy_id, "author".to_string())?;

    // Record approval
    let result = manager.approve(&policy_id, approver.clone(), comment.clone())?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!();
            println!("{}", "✅ Approval recorded".bright_green().bold());
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("Approver: {}", approver);
            if let Some(c) = comment {
                println!("Comment: {}", c);
            }
            println!();
            
            if result.sufficient_approvals {
                println!("{}", "🎉 Policy has sufficient approvals!".bright_green().bold());
                println!();
                println!("Next step: costpilot policy activate {}", policy_id);
            } else {
                println!("Approvals: {}/{}", 
                    result.remaining_approvals,
                    result.remaining_approvals
                );
                println!("Waiting for {} more approval(s)", result.remaining_approvals);
            }
        }
    }

    Ok(())
}

/// Execute policy reject command
pub fn cmd_reject(
    policy_id: String,
    approver: String,
    reason: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("❌ Rejecting policy '{}'...", policy_id).bright_red().bold());

    let mut manager = ApprovalWorkflowManager::new();
    manager.assign_role("policy-approver".to_string(), vec![approver.clone()]);
    manager.register_policy(policy_id.clone(), None)?;
    manager.submit_for_approval(&policy_id, "author".to_string())?;

    let result = manager.reject(&policy_id, approver.clone(), reason.clone())?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!();
            println!("{}", "❌ Policy rejected".bright_red().bold());
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("Reviewer: {}", approver);
            println!("Reason: {}", reason);
            println!();
            println!("The policy has been sent back to draft status.");
            println!("Author should address feedback and resubmit.");
        }
    }

    Ok(())
}

/// Execute policy activate command
pub fn cmd_activate(
    policy_id: String,
    actor: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("🚀 Activating policy '{}'...", policy_id).bright_blue().bold());

    let mut manager = ApprovalWorkflowManager::new();
    manager.register_policy(policy_id.clone(), None)?;
    
    // Simulate approved state (in production, load from storage)
    let lifecycle = manager.get_lifecycle_mut(&policy_id).unwrap();
    lifecycle.current_state = PolicyState::Approved;

    manager.activate_policy(&policy_id, actor.clone())?;

    match format {
        "json" => {
            let output = serde_json::json!({
                "policy_id": policy_id,
                "status": "active",
                "activated_by": actor,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!();
            println!("{}", "✅ Policy activated successfully!".bright_green().bold());
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("Status: {}", "Active".bright_green());
            println!("Activated by: {}", actor);
            println!();
            println!("The policy is now enforced in all evaluations.");
        }
    }

    Ok(())
}

/// Execute policy deprecate command
pub fn cmd_deprecate(
    policy_id: String,
    actor: String,
    reason: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("⚠️  Deprecating policy '{}'...", policy_id).bright_yellow().bold());

    let mut manager = ApprovalWorkflowManager::new();
    manager.register_policy(policy_id.clone(), None)?;
    
    let lifecycle = manager.get_lifecycle_mut(&policy_id).unwrap();
    lifecycle.current_state = PolicyState::Active;

    manager.deprecate_policy(&policy_id, actor.clone(), reason.clone())?;

    match format {
        "json" => {
            let output = serde_json::json!({
                "policy_id": policy_id,
                "status": "deprecated",
                "deprecated_by": actor,
                "reason": reason,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!();
            println!("{}", "⚠️  Policy deprecated".bright_yellow().bold());
            println!();
            println!("Policy ID: {}", policy_id.bright_white());
            println!("Status: {}", "Deprecated".bright_yellow());
            println!("Reason: {}", reason);
            println!();
            println!("The policy is still active but marked for removal.");
            println!("Plan migration to replacement policy.");
        }
    }

    Ok(())
}

/// Execute policy status command
pub fn cmd_status(
    policy_id: String,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("📊 Policy status for '{}'...", policy_id).bright_blue().bold());

    // Mock lifecycle for demo
    let lifecycle = PolicyLifecycleManager::new(policy_id.clone());
    let summary = lifecycle.summary();

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        _ => {
            println!();
            println!("{}", "Policy Lifecycle Status".bright_white().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();
            println!("Policy ID: {}", summary.policy_id.bright_white());
            
            let state_colored = match summary.current_state {
                PolicyState::Draft => format!("{:?}", summary.current_state).bright_black(),
                PolicyState::Review => format!("{:?}", summary.current_state).bright_yellow(),
                PolicyState::Approved => format!("{:?}", summary.current_state).bright_cyan(),
                PolicyState::Active => format!("{:?}", summary.current_state).bright_green(),
                PolicyState::Deprecated => format!("{:?}", summary.current_state).yellow(),
                PolicyState::Archived => format!("{:?}", summary.current_state).bright_black(),
            };
            println!("Status: {}", state_colored);
            println!("Description: {}", summary.state_description);
            println!();

            println!("Permissions:");
            println!("  Editable: {}", if summary.is_editable { "✅ Yes" } else { "❌ No" });
            println!("  Enforceable: {}", if summary.is_enforceable { "✅ Yes" } else { "❌ No" });
            println!();

            if summary.requires_approval {
                println!("Approvals:");
                println!("  Received: {}/{}", summary.approvals_received, summary.approvals_required);
                if summary.has_rejections {
                    println!("  Rejections: {}", "⚠️  Yes".bright_yellow());
                }
                if summary.is_expired {
                    println!("  Status: {}", "⏰ Expired".bright_red());
                }
                println!();
            }

            if verbose {
                println!("History:");
                println!("  Transitions: {}", summary.transition_count);
                if let Some(last) = summary.last_transition {
                    println!("  Last change: {}", last);
                }
            }
        }
    }

    Ok(())
}

/// Execute policy history command
pub fn cmd_history(
    policy_id: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("📚 Policy history for '{}'...", policy_id).bright_blue().bold());

    // Mock history for demo
    use serde_json::json;
    use std::collections::HashMap;
    
    let content = PolicyContent {
        id: policy_id.clone(),
        name: "Test Policy".to_string(),
        description: "Test policy description".to_string(),
        rules: json!({"test": "rule"}),
        config: HashMap::new(),
    };
    
    let history = PolicyHistory::new(policy_id.clone(), content, "author@example.com".to_string());

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&history)?);
        }
        _ => {
            println!();
            println!("{}", "Policy Version History".bright_white().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            for (i, version) in history.get_all_versions().iter().enumerate() {
                let marker = if version.version == history.current_version {
                    "→".bright_green()
                } else {
                    " ".normal()
                };

                println!("{} Version {}", marker, version.version.bright_white().bold());
                println!("  Author: {}", version.author);
                println!("  Created: {}", version.created_at);
                println!("  Description: {}", version.change_description);
                
                if !version.metadata.tags.is_empty() {
                    println!("  Tags: {}", version.metadata.tags.join(", "));
                }

                if i < history.get_all_versions().len() - 1 {
                    println!();
                }
            }

            println!();
            println!("Current version: {}", history.current_version.bright_green());
            println!("Total versions: {}", history.version_count());
        }
    }

    Ok(())
}

/// Execute policy diff command
pub fn cmd_diff(
    policy_id: String,
    from_version: String,
    to_version: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("🔍 Comparing versions {} → {}...", from_version, to_version).bright_blue().bold());

    // Mock for demo
    use serde_json::json;
    use std::collections::HashMap;
    
    let content = PolicyContent {
        id: policy_id.clone(),
        name: "Test Policy".to_string(),
        description: "Test policy description".to_string(),
        rules: json!({"test": "rule"}),
        config: HashMap::new(),
    };
    
    let history = PolicyHistory::new(policy_id, content, "author@example.com".to_string());
    let diff = history.diff(&from_version, &to_version)?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&diff)?);
        }
        _ => {
            println!();
            println!("{}", "Version Diff".bright_white().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();
            println!("From: {} ({})", from_version, diff.author_from);
            println!("To:   {} ({})", to_version, diff.author_to);
            println!();

            if diff.has_changes() {
                println!("{}", "Changes:".bright_white().bold());
                if diff.name_changed {
                    println!("  {} Name", "✓".bright_yellow());
                }
                if diff.description_changed {
                    println!("  {} Description", "✓".bright_yellow());
                }
                if diff.rules_changed {
                    println!("  {} Rules", "✓".bright_yellow());
                }
                if diff.config_changed {
                    println!("  {} Configuration", "✓".bright_yellow());
                }
                println!();
                println!("Summary: {}", diff.summary());
            } else {
                println!("{}", "No changes detected".bright_green());
            }

            println!();
            println!("Checksums:");
            println!("  From: {}", &diff.checksum_from[..16]);
            println!("  To:   {}", &diff.checksum_to[..16]);
        }
    }

    Ok(())
}
