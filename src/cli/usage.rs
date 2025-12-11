// CLI commands for usage metering and chargeback reporting

use std::collections::HashMap;
use std::path::PathBuf;

/// Usage metering CLI commands
#[derive(Debug)]
pub enum UsageCommand {
    /// Display usage report for team
    Report {
        team_id: String,
        start: Option<String>,
        end: Option<String>,
        format: OutputFormat,
    },
    /// Export billing data
    Export {
        start: String,
        end: String,
        format: ExportFormat,
        output: Option<PathBuf>,
    },
    /// Show PR usage summary
    Pr {
        pr_number: u32,
        repository: Option<String>,
    },
    /// Generate chargeback report
    Chargeback {
        org_id: String,
        start: String,
        end: String,
        format: OutputFormat,
        output: Option<PathBuf>,
    },
    /// Generate team invoice
    Invoice {
        team_id: String,
        start: String,
        end: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(format!("Unknown export format: {}", s)),
        }
    }
}

/// Parse timestamp from string (ISO 8601 or common formats)
pub fn parse_timestamp(s: &str) -> Result<u64, String> {
    // Try common formats
    // For MVP: Accept YYYY-MM-DD and convert to Unix timestamp
    if let Some((year, month, day)) = parse_date(s) {
        // Calculate Unix timestamp (simplified - in production use chrono)
        let days_since_epoch = days_since_epoch(year, month, day);
        Ok(days_since_epoch * 86400)
    } else if s == "now" {
        Ok(current_timestamp())
    } else if s.starts_with("-") {
        // Relative time: "-30d", "-7d", etc.
        let days: u64 = s[1..s.len() - 1]
            .parse()
            .map_err(|e| format!("Invalid relative time: {}", e))?;
        Ok(current_timestamp() - (days * 86400))
    } else {
        // Try parsing as Unix timestamp
        s.parse().map_err(|e| format!("Invalid timestamp: {}", e))
    }
}

fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year = parts[0].parse().ok()?;
    let month = parts[1].parse().ok()?;
    let day = parts[2].parse().ok()?;

    Some((year, month, day))
}

fn days_since_epoch(year: u32, month: u32, day: u32) -> u64 {
    // Simplified calculation (in production use chrono)
    let mut days = 0u64;

    // Add years since 1970
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add months
    for m in 1..month {
        days += days_in_month(year, m) as u64;
    }

    // Add days
    days += day as u64 - 1;

    days
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Execute usage command
pub fn execute_usage_command(cmd: UsageCommand) -> Result<String, String> {
    match cmd {
        UsageCommand::Report {
            team_id,
            start,
            end,
            format,
        } => execute_report(&team_id, start, end, format),
        UsageCommand::Export {
            start,
            end,
            format,
            output,
        } => execute_export(&start, &end, format, output),
        UsageCommand::Pr {
            pr_number,
            repository,
        } => execute_pr_report(pr_number, repository),
        UsageCommand::Chargeback {
            org_id,
            start,
            end,
            format,
            output,
        } => execute_chargeback(&org_id, &start, &end, format, output),
        UsageCommand::Invoice {
            team_id,
            start,
            end,
        } => execute_invoice(&team_id, &start, &end),
    }
}

fn execute_report(
    team_id: &str,
    start: Option<String>,
    end: Option<String>,
    format: OutputFormat,
) -> Result<String, String> {
    // Parse timestamps
    let start_ts = if let Some(s) = start {
        parse_timestamp(&s)?
    } else {
        start_of_current_month()
    };

    let end_ts = if let Some(e) = end {
        parse_timestamp(&e)?
    } else {
        current_timestamp()
    };

    // Load usage meter
    let meter = load_usage_meter()?;

    // Get team summary
    let summary = meter
        .team_summary(team_id, start_ts, end_ts)
        .map_err(|e| format!("Failed to generate team summary: {}", e))?;

    // Format output
    match format {
        OutputFormat::Text => Ok(format_team_summary_text(&summary)),
        OutputFormat::Json => serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("JSON serialization failed: {}", e)),
        OutputFormat::Csv => Ok(format_team_summary_csv(&summary)),
    }
}

fn execute_export(
    start: &str,
    end: &str,
    format: ExportFormat,
    output: Option<PathBuf>,
) -> Result<String, String> {
    let start_ts = parse_timestamp(start)?;
    let end_ts = parse_timestamp(end)?;

    let meter = load_usage_meter()?;

    let billing_export = meter
        .export_billing_data(start_ts, end_ts)
        .map_err(|e| format!("Failed to export billing data: {}", e))?;

    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&billing_export)
            .map_err(|e| format!("JSON serialization failed: {}", e))?,
        ExportFormat::Csv => format_billing_export_csv(&billing_export),
    };

    if let Some(path) = output {
        std::fs::write(&path, &content).map_err(|e| format!("Failed to write to file: {}", e))?;
        Ok(format!("Billing data exported to: {}", path.display()))
    } else {
        Ok(content)
    }
}

fn execute_pr_report(pr_number: u32, repository: Option<String>) -> Result<String, String> {
    let repo = repository.ok_or_else(|| "Repository is required".to_string())?;

    // Load PR tracker
    let tracker = load_pr_tracker(&repo)?;

    // Get PR summary
    let pricing = load_pricing_model()?;
    let price_per_resource = pricing.price_per_resource;

    let summary = tracker
        .get_pr_summary(pr_number, price_per_resource)
        .map_err(|e| e.message)?;

    Ok(format_pr_summary_text(&summary))
}

fn execute_chargeback(
    org_id: &str,
    start: &str,
    end: &str,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<String, String> {
    let start_ts = parse_timestamp(start)?;
    let end_ts = parse_timestamp(end)?;

    let meter = load_usage_meter()?;
    let teams = load_organization_teams(org_id)?;

    // Build chargeback report
    use crate::engines::metering::ChargebackReportBuilder;

    let mut builder = ChargebackReportBuilder::new(org_id.to_string(), start_ts, end_ts);

    for team in teams {
        let summary = meter
            .team_summary(&team, start_ts, end_ts)
            .map_err(|e| format!("Failed to get team summary for {}: {}", team, e))?;
        builder.add_team(summary);
    }

    let report = builder
        .build()
        .map_err(|e| format!("Failed to build chargeback report: {}", e))?;

    let content = match format {
        OutputFormat::Text => report.format_text(),
        OutputFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("JSON serialization failed: {}", e))?,
        OutputFormat::Csv => report.to_csv(),
    };

    if let Some(path) = output {
        std::fs::write(&path, &content).map_err(|e| format!("Failed to write to file: {}", e))?;
        Ok(format!("Chargeback report saved to: {}", path.display()))
    } else {
        Ok(content)
    }
}

fn execute_invoice(team_id: &str, start: &str, end: &str) -> Result<String, String> {
    let start_ts = parse_timestamp(start)?;
    let end_ts = parse_timestamp(end)?;

    let meter = load_usage_meter()?;
    let org_id = load_organization_id()?;

    // Build minimal chargeback report for this team
    use crate::engines::metering::ChargebackReportBuilder;

    let mut builder = ChargebackReportBuilder::new(org_id, start_ts, end_ts);

    let summary = meter
        .team_summary(team_id, start_ts, end_ts)
        .map_err(|e| format!("Failed to get team summary: {}", e))?;

    builder.add_team(summary);

    let report = builder
        .build()
        .map_err(|e| format!("Failed to build report: {}", e))?;

    report
        .generate_invoice(team_id)
        .ok_or_else(|| format!("Team {} not found in report", team_id))
}

// Helper functions for loading data
// In production, these would load from database or configuration

fn load_usage_meter() -> Result<crate::engines::metering::UsageMeter, String> {
    use crate::engines::metering::UsageMeter;

    // Load from file or database
    let storage_path = get_storage_path()?;
    let meter_path = storage_path.join("usage_events.ndjson");

    if meter_path.exists() {
        // Load existing meter
        UsageMeter::load_from_file(&meter_path, load_pricing_model()?)
            .map_err(|e| format!("Failed to load usage meter: {}", e))
    } else {
        // Create new meter
        Ok(UsageMeter::new(load_pricing_model()?))
    }
}

fn load_pr_tracker(repository: &str) -> Result<crate::engines::metering::CiUsageTracker, String> {
    use crate::engines::metering::CiUsageTracker;

    let storage_path = get_storage_path()?;
    let tracker_path = storage_path.join(format!(
        "pr_tracker_{}.json",
        sanitize_repo_name(repository)
    ));

    if tracker_path.exists() {
        let content = std::fs::read_to_string(&tracker_path)
            .map_err(|e| format!("Failed to read PR tracker: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse PR tracker: {}", e))
    } else {
        Ok(CiUsageTracker::new(repository.to_string()))
    }
}

fn load_pricing_model() -> Result<crate::engines::metering::PricingModel, String> {
    // Load from configuration or use default
    Ok(crate::engines::metering::PricingModel::default())
}

fn load_organization_teams(org_id: &str) -> Result<Vec<String>, String> {
    // Load from configuration or database
    // For MVP, return hardcoded list
    let storage_path = get_storage_path()?;
    let teams_path = storage_path.join(format!("org_{}_teams.json", org_id));

    if teams_path.exists() {
        let content = std::fs::read_to_string(&teams_path)
            .map_err(|e| format!("Failed to read teams: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse teams: {}", e))
    } else {
        Err("Organization teams not configured".to_string())
    }
}

fn load_organization_id() -> Result<String, String> {
    // Load from configuration
    let storage_path = get_storage_path()?;
    let config_path = storage_path.join("org_config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: HashMap<String, String> =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;
        config
            .get("org_id")
            .cloned()
            .ok_or_else(|| "Organization ID not configured".to_string())
    } else {
        Err("Organization not configured".to_string())
    }
}

fn get_storage_path() -> Result<PathBuf, String> {
    // Use XDG data directory or fallback to ~/.costpilot
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let storage = PathBuf::from(home).join(".costpilot").join("data");

    // Create if doesn't exist
    std::fs::create_dir_all(&storage)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;

    Ok(storage)
}

fn sanitize_repo_name(repo: &str) -> String {
    repo.replace(['/', '\\'], "_")
}

fn start_of_current_month() -> u64 {
    // Simplified: calculate start of current month
    // In production use chrono
    let now = current_timestamp();
    let days_this_month = (now % (30 * 86400)) / 86400;
    now - (days_this_month * 86400)
}

// Formatting functions

fn format_team_summary_text(summary: &crate::engines::metering::TeamUsageSummary) -> String {
    let mut output = String::new();

    output.push_str(&"📊 Team Usage Summary\n".to_string());
    output.push_str(&"====================\n\n".to_string());
    output.push_str(&format!(
        "Team: {} ({})\n",
        summary.team_name, summary.team_id
    ));
    output.push_str(&format!(
        "Period: {} - {}\n\n",
        summary.period_start, summary.period_end
    ));

    output.push_str(&"Usage:\n".to_string());
    output.push_str(&format!("  Total Events: {}\n", summary.total_events));
    output.push_str(&format!(
        "  Resources Analyzed: {}\n",
        summary.resources_analyzed
    ));
    output.push_str(&format!(
        "  Cost Impact Detected: ${:.2}\n\n",
        summary.cost_impact_detected
    ));

    output.push_str(&"Billing:\n".to_string());
    output.push_str(&format!("  Billable Units: {}\n", summary.billable_units));
    output.push_str(&format!(
        "  Estimated Charge: ${:.2}\n\n",
        summary.estimated_charge
    ));

    if !summary.top_users.is_empty() {
        output.push_str("Top Users:\n");
        for user in summary.top_users.iter().take(5) {
            output.push_str(&format!(
                "  {} - {} events ({:.1}%)\n",
                user.user_id, user.events, user.percentage_of_team
            ));
        }
        output.push('\n');
    }

    if !summary.top_projects.is_empty() {
        output.push_str("Top Projects:\n");
        for project in summary.top_projects.iter().take(5) {
            output.push_str(&format!(
                "  {} - {} resources (${:.2} impact)\n",
                project.project_id, project.resources_analyzed, project.cost_impact
            ));
        }
    }

    output
}

fn format_team_summary_csv(summary: &crate::engines::metering::TeamUsageSummary) -> String {
    let mut csv = String::new();
    csv.push_str("team_id,team_name,period_start,period_end,total_events,resources_analyzed,cost_impact_detected,billable_units,estimated_charge\n");
    csv.push_str(&format!(
        "{},{},{},{},{},{},{:.2},{},{:.2}\n",
        summary.team_id,
        summary.team_name,
        summary.period_start,
        summary.period_end,
        summary.total_events,
        summary.resources_analyzed,
        summary.cost_impact_detected,
        summary.billable_units,
        summary.estimated_charge
    ));
    csv
}

fn format_billing_export_csv(export: &crate::engines::metering::BillingExport) -> String {
    let mut csv = String::new();
    csv.push_str("period_start,period_end,total_events,total_resources,team_id,team_charge\n");

    for (team_id, charge) in &export.team_charges {
        csv.push_str(&format!(
            "{},{},{},{},{},{:.2}\n",
            export.period_start,
            export.period_end,
            export.total_events,
            export.total_resources,
            team_id,
            charge
        ));
    }

    csv
}

fn format_pr_summary_text(summary: &crate::engines::metering::PrUsageSummary) -> String {
    let mut output = String::new();

    output.push_str(&"🔍 PR Usage Summary\n".to_string());
    output.push_str(&"===================\n\n".to_string());
    output.push_str(&format!("Repository: {}\n", summary.repository));
    output.push_str(&format!("PR Number: #{}\n\n", summary.pr_number));

    output.push_str(&"Analysis:\n".to_string());
    output.push_str(&format!("  Scans: {}\n", summary.scan_count));
    output.push_str(&format!(
        "  Resources Analyzed: {}\n",
        summary.resources_analyzed
    ));
    output.push_str(&format!("  Issues Detected: {}\n", summary.issues_detected));
    output.push_str(&format!(
        "  Cost Prevented: ${:.2}\n\n",
        summary.cost_prevented
    ));

    output.push_str(&"Billing:\n".to_string());
    output.push_str(&format!("  Billable Units: {}\n", summary.billable_units));
    output.push_str(&format!(
        "  Estimated Charge: ${:.2}\n",
        summary.estimated_charge
    ));

    if let Some(roi) = summary.roi {
        output.push_str(&format!("  ROI: {:.1}x return on investment\n", roi));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2024-01-15"), Some((2024, 1, 15)));
        assert_eq!(parse_date("2024-12-31"), Some((2024, 12, 31)));
        assert_eq!(parse_date("invalid"), None);
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 2), 29); // Leap year
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn test_output_format_parsing() {
        assert!(matches!(
            OutputFormat::from_str("text"),
            Ok(OutputFormat::Text)
        ));
        assert!(matches!(
            OutputFormat::from_str("json"),
            Ok(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("csv"),
            Ok(OutputFormat::Csv)
        ));
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_sanitize_repo_name() {
        assert_eq!(sanitize_repo_name("owner/repo"), "owner_repo");
        assert_eq!(sanitize_repo_name("owner\\repo"), "owner_repo");
    }
}
