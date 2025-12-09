// CLI commands for audit log and compliance reporting

use crate::engines::policy::{
    AuditEvent, AuditEventType, AuditLog, AuditSeverity, ComplianceAnalyzer, ComplianceFramework,
    AuditQuery,
};
use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

const AUDIT_LOG_PATH: &str = ".costpilot/audit_log.json";

/// Load audit log from file
fn load_audit_log(path: Option<PathBuf>) -> Result<AuditLog, Box<dyn std::error::Error>> {
    let log_path = path
        .unwrap_or_else(|| PathBuf::from(AUDIT_LOG_PATH));

    if log_path.exists() {
        let contents = fs::read_to_string(&log_path)?;
        let log: AuditLog = serde_json::from_str(&contents)?;
        Ok(log)
    } else {
        Ok(AuditLog::new())
    }
}

/// Save audit log to file
fn save_audit_log(
    log: &AuditLog,
    path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = path.unwrap_or_else(|| PathBuf::from(AUDIT_LOG_PATH));

    // Create directory if needed
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(log)?;
    fs::write(&log_path, json)?;

    Ok(())
}

/// View audit log entries
pub fn cmd_audit_view(
    event_type: Option<String>,
    actor: Option<String>,
    resource: Option<String>,
    severity: Option<String>,
    last_n: Option<usize>,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = load_audit_log(None)?;

    // Build query
    let mut query = AuditQuery::new(&log);

    if let Some(et) = event_type {
        let event_type = match et.to_lowercase().as_str() {
            "policy_state_change" => AuditEventType::PolicyStateChange,
            "policy_approval" => AuditEventType::PolicyApproval,
            "policy_version_created" => AuditEventType::PolicyVersionCreated,
            "policy_activated" => AuditEventType::PolicyActivated,
            "slo_violation" => AuditEventType::SloViolation,
            _ => return Err(format!("Unknown event type: {}", et).into()),
        };
        query = query.with_event_type(event_type);
    }

    if let Some(a) = actor {
        query = query.with_actor(a);
    }

    if let Some(r) = resource {
        query = query.with_resource(r);
    }

    if let Some(s) = severity {
        let severity = match s.to_lowercase().as_str() {
            "low" => AuditSeverity::Low,
            "medium" => AuditSeverity::Medium,
            "high" => AuditSeverity::High,
            "critical" => AuditSeverity::Critical,
            _ => return Err(format!("Unknown severity: {}", s).into()),
        };
        query = query.with_severity(severity);
    }

    let mut entries = query.execute();

    // Take last N entries if specified
    if let Some(n) = last_n {
        let total = entries.len();
        if total > n {
            entries = entries.into_iter().skip(total - n).collect();
        }
    }

    if format == "json" {
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{}", json);
    } else {
        println!("{}", "📋 Audit Log Entries".bright_blue().bold());
        println!("{}", "━".repeat(80).bright_black());
        println!();

        if entries.is_empty() {
            println!("{}", "No entries found matching criteria.".yellow());
            return Ok(());
        }

        let entries_len = entries.len();
        
        for entry in &entries {
            let severity_color = match entry.event.severity {
                AuditSeverity::Critical => "critical".red().bold(),
                AuditSeverity::High => "high".bright_red(),
                AuditSeverity::Medium => "medium".yellow(),
                AuditSeverity::Low => "low".bright_black(),
            };

            let status_icon = if entry.event.success {
                "✅".green()
            } else {
                "❌".red()
            };

            println!("  {} Sequence #{}", status_icon, entry.sequence);
            println!("  Type: {:?}", entry.event.event_type);
            println!("  Severity: {}", severity_color);
            println!("  Timestamp: {}", entry.event.timestamp.to_rfc3339());
            println!("  Actor: {}", entry.event.actor.bright_cyan());
            println!("  Resource: {} ({})", entry.event.resource_id, entry.event.resource_type);
            println!("  Description: {}", entry.event.description);

            if let Some(old) = &entry.event.old_value {
                println!("  {} Old: {}", "←".bright_black(), old);
            }
            if let Some(new) = &entry.event.new_value {
                println!("  {} New: {}", "→".bright_green(), new);
            }

            if !entry.event.success {
                if let Some(err) = &entry.event.error_message {
                    println!("  {} Error: {}", "⚠".red(), err);
                }
            }

            println!("  Hash: {}", &entry.hash[..16].bright_black());
            println!();
        }

        println!("{}", "━".repeat(80).bright_black());
        println!("Total entries: {}", entries_len);
    }

    Ok(())
}

/// Verify audit log integrity
pub fn cmd_audit_verify(
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = load_audit_log(None)?;

    if format == "json" {
        let result = match log.verify_chain() {
            Ok(_) => serde_json::json!({
                "verified": true,
                "entry_count": log.entry_count(),
                "status": "PASS"
            }),
            Err(e) => serde_json::json!({
                "verified": false,
                "entry_count": log.entry_count(),
                "status": "FAIL",
                "error": e.to_string()
            }),
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", "🔐 Verifying Audit Log Integrity".bright_blue().bold());
        println!("{}", "━".repeat(80).bright_black());
        println!();

        match log.verify_chain() {
            Ok(_) => {
                println!("{}", "✅ Audit log verification: PASS".green().bold());
                println!();
                println!("The audit log chain is intact and tamper-proof.");
                println!("Total entries verified: {}", log.entry_count());
                println!("Genesis hash: {}", log.genesis_hash[..16].bright_black());
                if let Some(last) = log.last_entry() {
                    println!("Last entry hash: {}", &last.hash[..16].bright_black());
                }
            }
            Err(e) => {
                println!("{}", "❌ Audit log verification: FAIL".red().bold());
                println!();
                println!("{} Integrity violation detected!", "⚠".red());
                println!("Error: {}", e);
                println!();
                println!("The audit log chain has been tampered with or corrupted.");
                println!("This indicates a serious security issue.");
            }
        }
    }

    Ok(())
}

/// Generate compliance report
pub fn cmd_audit_compliance(
    framework: String,
    days: u32,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = load_audit_log(None)?;

    let compliance_framework = match framework.to_uppercase().as_str() {
        "SOC2" => ComplianceFramework::Soc2,
        "ISO27001" => ComplianceFramework::Iso27001,
        "GDPR" => ComplianceFramework::Gdpr,
        "HIPAA" => ComplianceFramework::Hipaa,
        "PCI_DSS" => ComplianceFramework::PciDss,
        _ => return Err(format!("Unknown compliance framework: {}", framework).into()),
    };

    let end = Utc::now();
    let start = end - Duration::days(days as i64);

    let analyzer = ComplianceAnalyzer::new(log);
    let report = analyzer.generate_report(compliance_framework, start, end)?;

    if format == "json" {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
    } else {
        println!(
            "{}",
            format!("📊 {} Compliance Report", framework.to_uppercase())
                .bright_blue()
                .bold()
        );
        println!("{}", "━".repeat(80).bright_black());
        println!();

        let status_display = match report.overall_status {
            crate::engines::policy::ComplianceStatus::Compliant => "✅ COMPLIANT".green().bold(),
            crate::engines::policy::ComplianceStatus::NonCompliant => {
                "❌ NON-COMPLIANT".red().bold()
            }
            crate::engines::policy::ComplianceStatus::PartiallyCompliant => {
                "⚠ PARTIALLY COMPLIANT".yellow().bold()
            }
            crate::engines::policy::ComplianceStatus::NotApplicable => "N/A".bright_black(),
        };

        println!("Overall Status: {}", status_display);
        println!("Report Period: {} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
        println!(
            "Audit Log Verified: {}",
            if report.audit_log_verified {
                "✅ Yes".green()
            } else {
                "❌ No".red()
            }
        );
        println!();

        println!("{}", "Summary".bright_cyan().bold());
        println!("  Total Requirements: {}", report.summary.total_requirements);
        println!("  {} Compliant: {}", "✅".green(), report.summary.compliant);
        println!("  {} Non-Compliant: {}", "❌".red(), report.summary.non_compliant);
        println!(
            "  {} Partially Compliant: {}",
            "⚠".yellow(),
            report.summary.partially_compliant
        );
        println!(
            "  Compliance Rate: {:.1}%",
            report.summary.compliance_percentage
        );
        println!();

        println!("{}", "Requirement Checks".bright_cyan().bold());
        for check in &report.checks {
            let status_icon = match check.status {
                crate::engines::policy::ComplianceStatus::Compliant => "✅",
                crate::engines::policy::ComplianceStatus::NonCompliant => "❌",
                crate::engines::policy::ComplianceStatus::PartiallyCompliant => "⚠",
                crate::engines::policy::ComplianceStatus::NotApplicable => "○",
            };

            println!("  {} {}", status_icon, check.requirement.bold());
            println!("     {}", check.description);

            if !check.evidence.is_empty() {
                println!("     Evidence:");
                for evidence in &check.evidence {
                    println!("       • {}", evidence.bright_black());
                }
            }

            if let Some(recommendations) = &check.recommendations {
                println!("     {} Recommendations:", "💡".yellow());
                for rec in recommendations {
                    println!("       • {}", rec.yellow());
                }
            }
            println!();
        }

        println!("{}", "━".repeat(80).bright_black());
        println!(
            "Report generated at: {}",
            report.generated_at.to_rfc3339()
        );
    }

    Ok(())
}

/// Export audit log
pub fn cmd_audit_export(
    output_format: String,
    output_path: PathBuf,
    _format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = load_audit_log(None)?;

    let export_data = match output_format.to_lowercase().as_str() {
        "ndjson" => log.export_ndjson()?,
        "csv" => log.export_csv()?,
        "json" => serde_json::to_string_pretty(&log)?,
        _ => return Err(format!("Unknown export format: {}", output_format).into()),
    };

    fs::write(&output_path, export_data)?;

    println!("{}", "✅ Export complete".green().bold());
    println!();
    println!("Format: {}", output_format);
    println!("Entries exported: {}", log.entry_count());
    println!("Output file: {}", output_path.display());

    Ok(())
}

/// Get audit log statistics
pub fn cmd_audit_stats(
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = load_audit_log(None)?;
    let stats = log.get_statistics()?;

    if format == "json" {
        let json = serde_json::to_string_pretty(&stats)?;
        println!("{}", json);
    } else {
        println!("{}", "📊 Audit Log Statistics".bright_blue().bold());
        println!("{}", "━".repeat(80).bright_black());
        println!();

        println!("{}", "Overview".bright_cyan().bold());
        println!("  Total Events: {}", stats.total_events);
        println!("  Unique Actors: {}", stats.unique_actors);
        println!("  Unique Resources: {}", stats.unique_resources);
        println!("  Failed Events: {}", stats.failed_events);
        println!(
            "  Chain Verified: {}",
            if stats.chain_verified {
                "✅ Yes".green()
            } else {
                "❌ No".red()
            }
        );
        println!();

        println!("{}", "Time-Based Activity".bright_cyan().bold());
        println!("  Last 24 hours: {}", stats.events_last_24h);
        println!("  Last 7 days: {}", stats.events_last_7d);
        println!("  Last 30 days: {}", stats.events_last_30d);
        println!();

        println!("{}", "Events by Type".bright_cyan().bold());
        let mut types: Vec<_> = stats.events_by_type.iter().collect();
        types.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for (event_type, count) in types {
            println!("  {:30} {}", event_type, count);
        }
        println!();

        println!("{}", "Events by Severity".bright_cyan().bold());
        for (severity, count) in &stats.events_by_severity {
            let severity_colored = match severity.as_str() {
                "Critical" => severity.red().bold(),
                "High" => severity.bright_red(),
                "Medium" => severity.yellow(),
                "Low" => severity.bright_black(),
                _ => severity.normal(),
            };
            println!("  {:30} {}", severity_colored, count);
        }
    }

    Ok(())
}

/// Record a manual audit event
pub fn cmd_audit_record(
    event_type: String,
    actor: String,
    resource_id: String,
    resource_type: String,
    description: String,
    format: &str,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut log = load_audit_log(None)?;

    let audit_event_type = match event_type.to_lowercase().as_str() {
        "policy_state_change" => AuditEventType::PolicyStateChange,
        "policy_approval" => AuditEventType::PolicyApproval,
        "policy_version_created" => AuditEventType::PolicyVersionCreated,
        "policy_activated" => AuditEventType::PolicyActivated,
        "configuration_change" => AuditEventType::ConfigurationChange,
        _ => return Err(format!("Unknown event type: {}", event_type).into()),
    };

    let event = AuditEvent::new(
        audit_event_type,
        actor,
        resource_id,
        resource_type,
        description,
    );

    let sequence = log.append(event)?;
    save_audit_log(&log, None)?;

    if format == "json" {
        let result = serde_json::json!({
            "success": true,
            "sequence": sequence,
            "total_entries": log.entry_count()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", "✅ Audit event recorded".green().bold());
        println!();
        println!("Sequence: #{}", sequence);
        println!("Total entries: {}", log.entry_count());
    }

    Ok(())
}
