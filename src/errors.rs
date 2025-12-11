// Re-export shared error model for backward compatibility

pub use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};

// Error variants for different components
impl CostPilotError {
    /// Create a timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new("TIMEOUT", ErrorCategory::Timeout, message.into())
    }

    /// Create a circuit breaker error
    pub fn circuit_breaker(message: impl Into<String>) -> Self {
        Self::new(
            "CIRCUIT_BREAK",
            ErrorCategory::CircuitBreaker,
            message.into(),
        )
    }

    /// Create a performance budget error
    pub fn budget_exceeded(engine: &str, budget_ms: u64, actual_ms: u64) -> Self {
        Self::new(
            "BUDGET_EXCEEDED",
            ErrorCategory::Timeout,
            format!(
                "{} engine exceeded budget: {}ms budget, {}ms actual",
                engine, budget_ms, actual_ms
            ),
        )
    }

    // Legacy error constructors for backward compatibility
    pub fn Timeout(message: String) -> Self {
        Self::timeout(message)
    }

    pub fn InvalidJson(message: String) -> Self {
        Self::invalid_json(message)
    }

    pub fn CircuitBreaker(message: String) -> Self {
        Self::circuit_breaker(message)
    }
}
