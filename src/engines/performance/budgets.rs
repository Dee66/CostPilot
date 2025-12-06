// Performance budget enforcement for CostPilot engines
// Ensures SLA compliance and prevents resource exhaustion

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance budgets for all engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudgets {
    /// Prediction engine budget
    pub prediction: EngineBudget,
    
    /// Mapping engine budget
    pub mapping: EngineBudget,
    
    /// Autofix engine budget
    pub autofix: EngineBudget,
    
    /// Total scan budget
    pub total_scan: EngineBudget,
    
    /// SLO engine budget
    pub slo: EngineBudget,
    
    /// Policy engine budget
    pub policy: EngineBudget,
    
    /// WASM sandbox limits
    pub wasm: WasmLimits,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Budget for individual engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineBudget {
    /// Engine name
    pub name: String,
    
    /// Maximum execution time (milliseconds)
    pub max_latency_ms: u64,
    
    /// Maximum memory usage (MB)
    pub max_memory_mb: usize,
    
    /// Maximum file size to process (MB)
    pub max_file_size_mb: usize,
    
    /// Timeout action
    pub timeout_action: TimeoutAction,
    
    /// Warning threshold (percentage of max_latency_ms)
    pub warning_threshold: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Return partial results
    PartialResults,
    
    /// Return error
    Error,
    
    /// Fail fast with circuit breaker
    CircuitBreak,
}

/// WASM sandbox limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLimits {
    /// Maximum memory (MB)
    pub max_memory_mb: usize,
    
    /// Maximum execution time (milliseconds)
    pub max_execution_ms: u64,
    
    /// Maximum stack depth
    pub max_stack_depth: usize,
    
    /// Maximum bytecode size (MB)
    pub max_bytecode_mb: usize,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: usize,
    
    /// Success threshold to close circuit
    pub success_threshold: usize,
    
    /// Timeout before attempting to close (seconds)
    pub timeout_seconds: u64,
    
    /// Maximum consecutive failures
    pub max_consecutive_failures: usize,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit closed, normal operation
    Closed,
    
    /// Circuit open, rejecting requests
    Open,
    
    /// Circuit half-open, testing recovery
    HalfOpen,
}

/// Circuit breaker
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: usize,
    success_count: usize,
    consecutive_failures: usize,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            consecutive_failures: 0,
            last_failure_time: None,
        }
    }
    
    /// Check if request should be allowed
    pub fn allow_request(&mut self) -> Result<(), String> {
        match self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = self.last_failure_time {
                    let elapsed = last_failure.elapsed();
                    if elapsed.as_secs() >= self.config.timeout_seconds {
                        // Transition to half-open
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        Ok(())
                    } else {
                        Err("Circuit breaker is open".to_string())
                    }
                } else {
                    Err("Circuit breaker is open".to_string())
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }
    
    /// Record successful execution
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        
        match self.state {
            CircuitState::Closed => {
                // Stay closed
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    // Transition to closed
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Open => {
                // Should not happen
            }
        }
    }
    
    /// Record failed execution
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        self.last_failure_time = Some(Instant::now());
        
        // Check if we should open circuit
        if self.consecutive_failures >= self.config.max_consecutive_failures
            || self.failure_count >= self.config.failure_threshold
        {
            self.state = CircuitState::Open;
        }
    }
    
    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state
    }
    
    /// Get failure statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state,
            failure_count: self.failure_count,
            success_count: self.success_count,
            consecutive_failures: self.consecutive_failures,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: usize,
    pub success_count: usize,
    pub consecutive_failures: usize,
}

/// Performance tracker for engine execution
pub struct PerformanceTracker {
    budget: EngineBudget,
    start_time: Instant,
    circuit_breaker: Option<CircuitBreaker>,
}

impl PerformanceTracker {
    /// Create new performance tracker
    pub fn new(budget: EngineBudget) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            circuit_breaker: None,
        }
    }
    
    /// Create tracker with circuit breaker
    pub fn with_circuit_breaker(budget: EngineBudget, breaker: CircuitBreaker) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            circuit_breaker: Some(breaker),
        }
    }
    
    /// Check if execution should continue
    pub fn check_budget(&self) -> Result<(), BudgetViolation> {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        
        // Check if exceeded budget
        if elapsed_ms > self.budget.max_latency_ms {
            return Err(BudgetViolation {
                engine: self.budget.name.clone(),
                violation_type: ViolationType::Timeout,
                budget_value: self.budget.max_latency_ms,
                actual_value: elapsed_ms,
                action: self.budget.timeout_action,
            });
        }
        
        // Check if approaching warning threshold
        let warning_threshold_ms = (self.budget.max_latency_ms as f64 * self.budget.warning_threshold) as u64;
        if elapsed_ms > warning_threshold_ms {
            // Log warning but don't fail
            eprintln!("⚠️  Performance warning: {} at {}ms ({}% of budget)",
                self.budget.name, elapsed_ms,
                (elapsed_ms as f64 / self.budget.max_latency_ms as f64 * 100.0) as u64);
        }
        
        Ok(())
    }
    
    /// Complete execution and record metrics
    pub fn complete(mut self) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;
        
        let within_budget = duration_ms <= self.budget.max_latency_ms;
        
        // Update circuit breaker
        if let Some(breaker) = &mut self.circuit_breaker {
            if within_budget {
                breaker.record_success();
            } else {
                breaker.record_failure();
            }
        }
        
        PerformanceMetrics {
            engine: self.budget.name.clone(),
            duration_ms,
            budget_ms: self.budget.max_latency_ms,
            within_budget,
            utilization: (duration_ms as f64 / self.budget.max_latency_ms as f64 * 100.0),
            circuit_breaker_stats: self.circuit_breaker.as_ref().map(|b| b.stats()),
        }
    }
    
    /// Complete with failure
    pub fn complete_with_failure(mut self, error: &str) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;
        
        // Update circuit breaker
        if let Some(breaker) = &mut self.circuit_breaker {
            breaker.record_failure();
        }
        
        PerformanceMetrics {
            engine: self.budget.name.clone(),
            duration_ms,
            budget_ms: self.budget.max_latency_ms,
            within_budget: false,
            utilization: (duration_ms as f64 / self.budget.max_latency_ms as f64 * 100.0),
            circuit_breaker_stats: self.circuit_breaker.as_ref().map(|b| b.stats()),
        }
    }
}

/// Budget violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub engine: String,
    pub violation_type: ViolationType,
    pub budget_value: u64,
    pub actual_value: u64,
    pub action: TimeoutAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ViolationType {
    Timeout,
    MemoryExceeded,
    FileSizeExceeded,
    CircuitBreakerOpen,
}

impl BudgetViolation {
    pub fn format_error(&self) -> String {
        match self.violation_type {
            ViolationType::Timeout => {
                format!("⏱️  {} exceeded time budget: {}ms (limit: {}ms)",
                    self.engine, self.actual_value, self.budget_value)
            }
            ViolationType::MemoryExceeded => {
                format!("💾 {} exceeded memory budget: {}MB (limit: {}MB)",
                    self.engine, self.actual_value, self.budget_value)
            }
            ViolationType::FileSizeExceeded => {
                format!("📄 {} input file too large: {}MB (limit: {}MB)",
                    self.engine, self.actual_value, self.budget_value)
            }
            ViolationType::CircuitBreakerOpen => {
                format!("🔌 {} circuit breaker is open (too many failures)",
                    self.engine)
            }
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub engine: String,
    pub duration_ms: u64,
    pub budget_ms: u64,
    pub within_budget: bool,
    pub utilization: f64,
    pub circuit_breaker_stats: Option<CircuitBreakerStats>,
}

impl PerformanceMetrics {
    pub fn format_text(&self) -> String {
        let status = if self.within_budget { "✅" } else { "❌" };
        format!("{} {} - {}ms / {}ms ({}% utilized)",
            status, self.engine, self.duration_ms, self.budget_ms,
            self.utilization as u64)
    }
}

/// Performance report for all engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metrics: Vec<PerformanceMetrics>,
    pub total_duration_ms: u64,
    pub all_within_budget: bool,
}

impl PerformanceReport {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            total_duration_ms: 0,
            all_within_budget: true,
        }
    }
    
    pub fn add_metric(&mut self, metric: PerformanceMetrics) {
        if !metric.within_budget {
            self.all_within_budget = false;
        }
        self.total_duration_ms += metric.duration_ms;
        self.metrics.push(metric);
    }
    
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str("⚡ Performance Report\n");
        output.push_str("===================\n\n");
        
        if self.all_within_budget {
            output.push_str("✅ All engines within budget\n\n");
        } else {
            output.push_str("❌ Some engines exceeded budget\n\n");
        }
        
        output.push_str("Engine Performance:\n");
        for metric in &self.metrics {
            output.push_str(&format!("  {}\n", metric.format_text()));
        }
        
        output.push_str(&format!("\nTotal Duration: {}ms\n", self.total_duration_ms));
        
        output
    }
}

/// Default performance budgets
impl Default for PerformanceBudgets {
    fn default() -> Self {
        Self {
            prediction: EngineBudget {
                name: "Prediction".to_string(),
                max_latency_ms: 300,
                max_memory_mb: 128,
                max_file_size_mb: 10,
                timeout_action: TimeoutAction::PartialResults,
                warning_threshold: 0.8,
            },
            mapping: EngineBudget {
                name: "Mapping".to_string(),
                max_latency_ms: 500,
                max_memory_mb: 256,
                max_file_size_mb: 20,
                timeout_action: TimeoutAction::PartialResults,
                warning_threshold: 0.8,
            },
            autofix: EngineBudget {
                name: "Autofix".to_string(),
                max_latency_ms: 400,
                max_memory_mb: 128,
                max_file_size_mb: 10,
                timeout_action: TimeoutAction::Error,
                warning_threshold: 0.8,
            },
            total_scan: EngineBudget {
                name: "Total Scan".to_string(),
                max_latency_ms: 2000,
                max_memory_mb: 512,
                max_file_size_mb: 20,
                timeout_action: TimeoutAction::Error,
                warning_threshold: 0.9,
            },
            slo: EngineBudget {
                name: "SLO".to_string(),
                max_latency_ms: 150,
                max_memory_mb: 64,
                max_file_size_mb: 5,
                timeout_action: TimeoutAction::Error,
                warning_threshold: 0.8,
            },
            policy: EngineBudget {
                name: "Policy".to_string(),
                max_latency_ms: 200,
                max_memory_mb: 128,
                max_file_size_mb: 10,
                timeout_action: TimeoutAction::Error,
                warning_threshold: 0.8,
            },
            wasm: WasmLimits {
                max_memory_mb: 256,
                max_execution_ms: 2000,
                max_stack_depth: 1024,
                max_bytecode_mb: 10,
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_seconds: 60,
                max_consecutive_failures: 3,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_performance_tracker_within_budget() {
        let budget = EngineBudget {
            name: "Test".to_string(),
            max_latency_ms: 100,
            max_memory_mb: 128,
            max_file_size_mb: 10,
            timeout_action: TimeoutAction::Error,
            warning_threshold: 0.8,
        };
        
        let tracker = PerformanceTracker::new(budget);
        thread::sleep(Duration::from_millis(10));
        
        let metrics = tracker.complete();
        assert!(metrics.within_budget);
        assert!(metrics.duration_ms < 100);
    }
    
    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_seconds: 1,
            max_consecutive_failures: 3,
        };
        
        let mut breaker = CircuitBreaker::new(config);
        
        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();
        
        // Circuit should be open
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(breaker.allow_request().is_err());
    }
    
    #[test]
    fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 0, // Immediate timeout for testing
            max_consecutive_failures: 2,
        };
        
        let mut breaker = CircuitBreaker::new(config);
        
        // Open circuit
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
        
        // Wait a bit
        thread::sleep(Duration::from_millis(10));
        
        // Should transition to half-open
        assert!(breaker.allow_request().is_ok());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }
    
    #[test]
    fn test_circuit_breaker_closes_on_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 0,
            max_consecutive_failures: 2,
        };
        
        let mut breaker = CircuitBreaker::new(config);
        
        // Open circuit
        breaker.record_failure();
        breaker.record_failure();
        
        // Transition to half-open
        thread::sleep(Duration::from_millis(10));
        breaker.allow_request().unwrap();
        
        // Record successes
        breaker.record_success();
        breaker.record_success();
        
        // Should be closed
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
    
    #[test]
    fn test_performance_report() {
        let mut report = PerformanceReport::new();
        
        report.add_metric(PerformanceMetrics {
            engine: "Prediction".to_string(),
            duration_ms: 250,
            budget_ms: 300,
            within_budget: true,
            utilization: 83.3,
            circuit_breaker_stats: None,
        });
        
        report.add_metric(PerformanceMetrics {
            engine: "Mapping".to_string(),
            duration_ms: 450,
            budget_ms: 500,
            within_budget: true,
            utilization: 90.0,
            circuit_breaker_stats: None,
        });
        
        assert!(report.all_within_budget);
        assert_eq!(report.total_duration_ms, 700);
        
        let text = report.format_text();
        assert!(text.contains("All engines within budget"));
    }
    
    #[test]
    fn test_budget_violation_format() {
        let violation = BudgetViolation {
            engine: "Test".to_string(),
            violation_type: ViolationType::Timeout,
            budget_value: 300,
            actual_value: 500,
            action: TimeoutAction::Error,
        };
        
        let error = violation.format_error();
        assert!(error.contains("exceeded time budget"));
        assert!(error.contains("500ms"));
        assert!(error.contains("300ms"));
    }
}
