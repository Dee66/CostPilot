// CLI commands for performance monitoring

use crate::engines::performance::{PerformanceBudgets, PerformanceMonitor, PerformanceReport};
use std::path::PathBuf;

/// Performance CLI commands
#[derive(Debug)]
pub enum PerformanceCommand {
    /// Show current performance budgets
    Budgets,

    /// Set performance baseline
    SetBaseline { from_file: Option<PathBuf> },

    /// Show performance statistics
    Stats,

    /// Check for performance regressions
    CheckRegressions { report_file: PathBuf },

    /// Show performance history
    History {
        engine: Option<String>,
        limit: Option<usize>,
    },
}

/// Execute performance command
pub fn execute_performance_command(cmd: PerformanceCommand) -> Result<String, String> {
    match cmd {
        PerformanceCommand::Budgets => execute_budgets(),
        PerformanceCommand::SetBaseline { from_file } => execute_set_baseline(from_file),
        PerformanceCommand::Stats => execute_stats(),
        PerformanceCommand::CheckRegressions { report_file } => {
            execute_check_regressions(&report_file)
        }
        PerformanceCommand::History { engine, limit } => execute_history(engine, limit),
    }
}

fn execute_budgets() -> Result<String, String> {
    let budgets = PerformanceBudgets::default();

    let mut output = String::new();
    output.push_str("⚡ Performance Budgets\n");
    output.push_str("=====================\n\n");

    output.push_str(&"Prediction Engine:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.prediction.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n",
        budgets.prediction.max_memory_mb
    ));
    output.push_str(&format!(
        "  Max File Size: {}MB\n\n",
        budgets.prediction.max_file_size_mb
    ));

    output.push_str(&"Mapping Engine:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.mapping.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n",
        budgets.mapping.max_memory_mb
    ));
    output.push_str(&format!(
        "  Max File Size: {}MB\n\n",
        budgets.mapping.max_file_size_mb
    ));

    output.push_str(&"Autofix Engine:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.autofix.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n",
        budgets.autofix.max_memory_mb
    ));
    output.push_str(&format!(
        "  Max File Size: {}MB\n\n",
        budgets.autofix.max_file_size_mb
    ));

    output.push_str(&"Total Scan:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.total_scan.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n",
        budgets.total_scan.max_memory_mb
    ));
    output.push_str(&format!(
        "  Max File Size: {}MB\n\n",
        budgets.total_scan.max_file_size_mb
    ));

    output.push_str(&"SLO Engine:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.slo.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n\n",
        budgets.slo.max_memory_mb
    ));

    output.push_str(&"Policy Engine:\n".to_string());
    output.push_str(&format!(
        "  Max Latency: {}ms\n",
        budgets.policy.max_latency_ms
    ));
    output.push_str(&format!(
        "  Max Memory: {}MB\n\n",
        budgets.policy.max_memory_mb
    ));

    output.push_str(&"WASM Sandbox:\n".to_string());
    output.push_str(&format!("  Max Memory: {}MB\n", budgets.wasm.max_memory_mb));
    output.push_str(&format!(
        "  Max Execution: {}ms\n",
        budgets.wasm.max_execution_ms
    ));
    output.push_str(&format!(
        "  Max Stack Depth: {}\n",
        budgets.wasm.max_stack_depth
    ));
    output.push_str(&format!(
        "  Max Bytecode: {}MB\n\n",
        budgets.wasm.max_bytecode_mb
    ));

    output.push_str(&"Circuit Breaker:\n".to_string());
    output.push_str(&format!(
        "  Failure Threshold: {}\n",
        budgets.circuit_breaker.failure_threshold
    ));
    output.push_str(&format!(
        "  Success Threshold: {}\n",
        budgets.circuit_breaker.success_threshold
    ));
    output.push_str(&format!(
        "  Timeout: {}s\n",
        budgets.circuit_breaker.timeout_seconds
    ));
    output.push_str(&format!(
        "  Max Consecutive Failures: {}\n",
        budgets.circuit_breaker.max_consecutive_failures
    ));

    Ok(output)
}

fn execute_set_baseline(from_file: Option<PathBuf>) -> Result<String, String> {
    let history_path = get_history_path()?;
    let mut monitor = PerformanceMonitor::load(&history_path)?;

    if let Some(file) = from_file {
        // Load report from file
        let content = std::fs::read_to_string(&file)
            .map_err(|e| format!("Failed to read report file: {}", e))?;
        let report: PerformanceReport =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse report: {}", e))?;

        monitor.set_baseline(&report);
        monitor.save(&history_path)?;

        Ok(format!(
            "✅ Performance baseline set from: {}",
            file.display()
        ))
    } else {
        Err("--from-file is required".to_string())
    }
}

fn execute_stats() -> Result<String, String> {
    let history_path = get_history_path()?;
    let monitor = PerformanceMonitor::load(&history_path)?;

    let stats = monitor.get_statistics();
    Ok(stats.format_text())
}

fn execute_check_regressions(report_file: &PathBuf) -> Result<String, String> {
    let history_path = get_history_path()?;
    let monitor = PerformanceMonitor::load(&history_path)?;

    // Load report
    let content = std::fs::read_to_string(report_file)
        .map_err(|e| format!("Failed to read report file: {}", e))?;
    let report: PerformanceReport =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse report: {}", e))?;

    let regressions = monitor.detect_regressions(&report);

    if regressions.is_empty() {
        Ok("✅ No performance regressions detected".to_string())
    } else {
        let mut output = String::new();
        output.push_str("⚠️  Performance Regressions Detected\n");
        output.push_str("==================================\n\n");

        for regression in &regressions {
            output.push_str(&format!("{}\n", regression.format_text()));
        }

        Ok(output)
    }
}

fn execute_history(engine: Option<String>, _limit: Option<usize>) -> Result<String, String> {
    let history_path = get_history_path()?;
    let monitor = PerformanceMonitor::load(&history_path)?;

    // This would display performance history
    // For now, return summary
    let stats = monitor.get_statistics();

    let mut output = String::new();
    output.push_str("📈 Performance History\n");
    output.push_str("=====================\n\n");

    if let Some(engine_name) = engine {
        if let Some(engine_stats) = stats.engines.get(&engine_name) {
            output.push_str(&format!("{}\n", engine_stats.format_text()));
        } else {
            output.push_str(&format!("No data for engine: {}\n", engine_name));
        }
    } else {
        output.push_str(&stats.format_text());
    }

    Ok(output)
}

fn get_history_path() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home)
        .join(".costpilot")
        .join("performance.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budgets_command() {
        let result = execute_budgets();
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Performance Budgets"));
        assert!(output.contains("Prediction Engine"));
        assert!(output.contains("300ms"));
    }
}
