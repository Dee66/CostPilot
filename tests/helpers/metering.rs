/// Metering test helpers for chargeback calculations
///
/// Provides fixed unit prices used by tests for deterministic cost calculations.

/// Fixed unit price per resource-hour used in chargeback tests
/// This matches the constant used by metering engine test fixtures
pub const TEST_RESOURCE_UNIT_PRICE: f64 = 0.10;

/// Fixed free tier threshold in USD used for deduction tests
pub const TEST_FREE_TIER_THRESHOLD: f64 = 49.0;

/// Calculate chargeback amount for test resources
pub fn calculate_test_chargeback(resource_count: usize, hours: f64) -> f64 {
    resource_count as f64 * hours * TEST_RESOURCE_UNIT_PRICE
}

/// Calculate expected cost after free tier deduction
pub fn apply_free_tier(gross_cost: f64) -> f64 {
    if gross_cost <= TEST_FREE_TIER_THRESHOLD {
        0.0
    } else {
        gross_cost - TEST_FREE_TIER_THRESHOLD
    }
}
