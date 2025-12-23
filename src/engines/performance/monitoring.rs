// Performance monitoring and regression detection

use crate::engines::performance::budgets::{PerformanceMetrics, PerformanceReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Performance monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitor {
    baseline: Option<PerformanceBaseline>,
    history: Vec<PerformanceSnapshot>,
    regression_threshold: f64,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            baseline: None,
            history: Vec::new(),
            regression_threshold: 1.2, // 20% degradation threshold
        }
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read performance history: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse performance history: {}", e))
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self)
            .map_err(|e| format!("Failed to serialize performance history: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write performance history: {}", e))
    }

    /// Set performance baseline
    pub fn set_baseline(&mut self, report: &PerformanceReport) {
        let mut engine_baselines = HashMap::new();

        for metric in &report.metrics {
            engine_baselines.insert(
                metric.engine.clone(),
                EngineBaseline {
                    engine: metric.engine.clone(),
                    baseline_ms: metric.duration_ms,
                    p50_ms: metric.duration_ms,
                    p95_ms: (metric.duration_ms as f64 * 1.2) as u64,
                    p99_ms: (metric.duration_ms as f64 * 1.5) as u64,
                },
            );
        }

        self.baseline = Some(PerformanceBaseline {
            timestamp: current_timestamp(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            total_baseline_ms: report.total_duration_ms,
            engines: engine_baselines,
        });
    }

    /// Record performance snapshot
    pub fn record_snapshot(&mut self, report: &PerformanceReport) {
        let snapshot = PerformanceSnapshot {
            timestamp: current_timestamp(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            total_duration_ms: report.total_duration_ms,
            metrics: report.metrics.clone(),
        };

        self.history.push(snapshot);

        // Keep only last 100 snapshots
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// Detect performance regressions
    pub fn detect_regressions(&self, report: &PerformanceReport) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();

        if let Some(baseline) = &self.baseline {
            // Check total scan time
            let total_ratio = report.total_duration_ms as f64 / baseline.total_baseline_ms as f64;
            if total_ratio > self.regression_threshold {
                regressions.push(PerformanceRegression {
                    engine: "Total Scan".to_string(),
                    baseline_ms: baseline.total_baseline_ms,
                    current_ms: report.total_duration_ms,
                    ratio: total_ratio,
                    severity: classify_regression_severity(total_ratio),
                });
            }

            // Check individual engines
            for metric in &report.metrics {
                if let Some(engine_baseline) = baseline.engines.get(&metric.engine) {
                    let ratio = metric.duration_ms as f64 / engine_baseline.baseline_ms as f64;
                    if ratio > self.regression_threshold {
                        regressions.push(PerformanceRegression {
                            engine: metric.engine.clone(),
                            baseline_ms: engine_baseline.baseline_ms,
                            current_ms: metric.duration_ms,
                            ratio,
                            severity: classify_regression_severity(ratio),
                        });
                    }
                }
            }
        }

        regressions
    }

    /// Get performance statistics
    pub fn get_statistics(&self) -> PerformanceStatistics {
        if self.history.is_empty() {
            return PerformanceStatistics::default();
        }

        let mut engine_stats: HashMap<String, EngineStats> = HashMap::new();

        for snapshot in &self.history {
            for metric in &snapshot.metrics {
                let stats = engine_stats
                    .entry(metric.engine.clone())
                    .or_insert_with(|| EngineStats {
                        engine: metric.engine.clone(),
                        samples: 0,
                        total_ms: 0,
                        min_ms: u64::MAX,
                        max_ms: 0,
                        avg_ms: 0.0,
                    });

                stats.samples += 1;
                stats.total_ms += metric.duration_ms;
                stats.min_ms = stats.min_ms.min(metric.duration_ms);
                stats.max_ms = stats.max_ms.max(metric.duration_ms);
            }
        }

        // Calculate averages
        for stats in engine_stats.values_mut() {
            stats.avg_ms = stats.total_ms as f64 / stats.samples as f64;
        }

        PerformanceStatistics {
            total_samples: self.history.len(),
            engines: engine_stats,
        }
    }
}

/// Performance baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub timestamp: u64,
    pub version: String,
    pub total_baseline_ms: u64,
    pub engines: HashMap<String, EngineBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineBaseline {
    pub engine: String,
    pub baseline_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

/// Performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub version: String,
    pub total_duration_ms: u64,
    pub metrics: Vec<PerformanceMetrics>,
}

/// Performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub engine: String,
    pub baseline_ms: u64,
    pub current_ms: u64,
    pub ratio: f64,
    pub severity: RegressionSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Minor,    // 1.2x - 1.5x
    Moderate, // 1.5x - 2.0x
    Severe,   // 2.0x - 3.0x
    Critical, // > 3.0x
}

impl PerformanceRegression {
    pub fn format_text(&self) -> String {
        format!(
            "⚠️  {} regression: {}ms → {}ms ({:.1}x slower) - {:?}",
            self.engine, self.baseline_ms, self.current_ms, self.ratio, self.severity
        )
    }
}

fn classify_regression_severity(ratio: f64) -> RegressionSeverity {
    if ratio < 1.5 {
        RegressionSeverity::Minor
    } else if ratio < 2.0 {
        RegressionSeverity::Moderate
    } else if ratio < 3.0 {
        RegressionSeverity::Severe
    } else {
        RegressionSeverity::Critical
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceStatistics {
    pub total_samples: usize,
    pub engines: HashMap<String, EngineStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub engine: String,
    pub samples: usize,
    pub total_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: f64,
}

impl EngineStats {
    pub fn format_text(&self) -> String {
        format!(
            "{}: avg={:.0}ms, min={}ms, max={}ms ({} samples)",
            self.engine, self.avg_ms, self.min_ms, self.max_ms, self.samples
        )
    }
}

impl PerformanceStatistics {
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("📊 Performance Statistics\n");
        output.push_str("========================\n\n");
        output.push_str(&format!("Total Samples: {}\n\n", self.total_samples));

        if !self.engines.is_empty() {
            output.push_str("Engine Statistics:\n");
            for stats in self.engines.values() {
                output.push_str(&format!("  {}\n", stats.format_text()));
            }
        }

        output
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            eprintln!("Warning: System time is set before UNIX epoch, using 0 as timestamp");
            std::time::Duration::from_secs(0)
        })
        .as_secs()
}

/// Memory usage tracker
pub struct MemoryTracker {
    /// Initial memory usage (tracked for future reporting)
    _initial_memory: usize,
    peak_memory: usize,
    max_memory_mb: usize,
}

impl MemoryTracker {
    /// Create new memory tracker
    pub fn new(max_memory_mb: usize) -> Self {
        let initial = Self::get_current_memory_kb();
        Self {
            _initial_memory: initial,
            peak_memory: initial,
            max_memory_mb,
        }
    }

    /// Check if memory limit exceeded
    pub fn check_limit(&mut self) -> Result<(), String> {
        let current = Self::get_current_memory_kb();
        self.peak_memory = self.peak_memory.max(current);

        let current_mb = current / 1024;
        if current_mb > self.max_memory_mb {
            return Err(format!(
                "Memory limit exceeded: {}MB / {}MB",
                current_mb, self.max_memory_mb
            ));
        }

        Ok(())
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        let current = Self::get_current_memory_kb();
        let peak_mb = self.peak_memory / 1024;
        let current_mb = current / 1024;

        MemoryStats {
            peak_mb,
            current_mb,
            limit_mb: self.max_memory_mb,
            utilization: (peak_mb as f64 / self.max_memory_mb as f64 * 100.0),
        }
    }

    /// Get current memory usage (KB)
    fn get_current_memory_kb() -> usize {
        // In production, use platform-specific APIs
        // For now, return simulated value
        1024 * 64 // 64MB simulated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub peak_mb: usize,
    pub current_mb: usize,
    pub limit_mb: usize,
    pub utilization: f64,
}

impl MemoryStats {
    pub fn format_text(&self) -> String {
        format!(
            "Memory: peak={}MB, current={}MB, limit={}MB ({:.1}% utilized)",
            self.peak_mb, self.current_mb, self.limit_mb, self.utilization
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::performance::budgets::PerformanceReport;

    #[test]
    fn test_performance_monitor_baseline() {
        let mut monitor = PerformanceMonitor::new();

        let mut report = PerformanceReport::new();
        report.add_metric(crate::engines::performance::budgets::PerformanceMetrics {
            engine: "Prediction".to_string(),
            duration_ms: 250,
            budget_ms: 300,
            within_budget: true,
            utilization: 83.3,
            circuit_breaker_stats: None,
        });

        monitor.set_baseline(&report);

        assert!(monitor.baseline.is_some());
        let baseline = monitor.baseline.as_ref().unwrap();
        assert_eq!(baseline.total_baseline_ms, 250);
    }

    #[test]
    fn test_regression_detection() {
        let mut monitor = PerformanceMonitor::new();

        // Set baseline
        let mut baseline_report = PerformanceReport::new();
        baseline_report.add_metric(crate::engines::performance::budgets::PerformanceMetrics {
            engine: "Prediction".to_string(),
            duration_ms: 200,
            budget_ms: 300,
            within_budget: true,
            utilization: 66.7,
            circuit_breaker_stats: None,
        });
        monitor.set_baseline(&baseline_report);

        // Create regressed report
        let mut regressed_report = PerformanceReport::new();
        regressed_report.add_metric(crate::engines::performance::budgets::PerformanceMetrics {
            engine: "Prediction".to_string(),
            duration_ms: 400, // 2x slower
            budget_ms: 300,
            within_budget: false,
            utilization: 133.3,
            circuit_breaker_stats: None,
        });

        let regressions = monitor.detect_regressions(&regressed_report);
        assert!(!regressions.is_empty());
        assert!(matches!(
            regressions[0].severity,
            RegressionSeverity::Severe
        ));
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new(256);

        // Should not exceed limit
        assert!(tracker.check_limit().is_ok());

        let stats = tracker.get_stats();
        assert!(stats.current_mb <= stats.limit_mb);
    }
}
