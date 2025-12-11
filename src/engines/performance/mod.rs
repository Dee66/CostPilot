// Performance module exports

pub mod budgets;
pub mod monitoring;

pub use budgets::{
    BudgetViolation, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState,
    EngineBudget, PerformanceBudgets, PerformanceMetrics, PerformanceReport, PerformanceTracker,
    TimeoutAction, ViolationType, WasmLimits,
};

pub use monitoring::{
    EngineStats, MemoryStats, MemoryTracker, PerformanceBaseline, PerformanceMonitor,
    PerformanceRegression, PerformanceSnapshot, PerformanceStatistics, RegressionSeverity,
};
