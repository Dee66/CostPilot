// Performance module exports

pub mod budgets;
pub mod monitoring;

pub use budgets::{
    PerformanceBudgets, EngineBudget, WasmLimits, CircuitBreakerConfig,
    CircuitBreaker, CircuitState, CircuitBreakerStats,
    PerformanceTracker, PerformanceMetrics, PerformanceReport,
    BudgetViolation, ViolationType, TimeoutAction,
};

pub use monitoring::{
    PerformanceMonitor, PerformanceBaseline, PerformanceSnapshot,
    PerformanceRegression, RegressionSeverity, PerformanceStatistics,
    EngineStats, MemoryTracker, MemoryStats,
};
