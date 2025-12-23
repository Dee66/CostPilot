// Monte Carlo simulation for cost uncertainty quantification

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Monte Carlo simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// Number of simulation runs
    pub num_simulations: u32,

    /// Mean cost from simulations
    pub mean_cost: f64,

    /// Median cost (P50)
    pub median_cost: f64,

    /// Standard deviation
    pub std_dev: f64,

    /// Percentile results
    pub percentiles: HashMap<u8, f64>,

    /// Value at Risk (VaR) at 95% confidence
    pub var_95: f64,

    /// Conditional Value at Risk (CVaR) - expected cost in worst 5%
    pub cvar_95: f64,

    /// Distribution of simulated costs
    pub distribution: CostDistribution,
}

/// Cost distribution from simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDistribution {
    /// Histogram bins
    pub bins: Vec<DistributionBin>,

    /// Minimum simulated cost
    pub min: f64,

    /// Maximum simulated cost
    pub max: f64,

    /// Distribution shape
    pub shape: DistributionShape,
}

/// Single histogram bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBin {
    /// Lower bound of bin
    pub lower: f64,

    /// Upper bound of bin
    pub upper: f64,

    /// Count of simulations in this bin
    pub count: u32,

    /// Frequency (normalized to 0-1)
    pub frequency: f64,
}

/// Shape classification of distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionShape {
    /// Symmetric around mean
    Normal,

    /// Long tail toward higher costs
    RightSkewed,

    /// Long tail toward lower costs
    LeftSkewed,

    /// Two distinct peaks
    Bimodal,

    /// Flat distribution
    Uniform,
}

/// Uncertainty input for simulation
#[derive(Debug, Clone)]
pub struct UncertaintyInput {
    /// Base value
    pub base_value: f64,

    /// Uncertainty type
    pub uncertainty_type: UncertaintyType,

    /// Weight in overall simulation
    pub weight: f64,
}

/// Type of uncertainty distribution
#[derive(Debug, Clone, Copy)]
pub enum UncertaintyType {
    /// Normal distribution (mean, std_dev)
    Normal { std_dev_ratio: f64 },

    /// Log-normal distribution (for costs that can't be negative)
    LogNormal { std_dev_ratio: f64 },

    /// Uniform distribution (min_ratio, max_ratio from base)
    Uniform { min_ratio: f64, max_ratio: f64 },

    /// Triangular distribution (min, mode, max)
    Triangular { min_ratio: f64, max_ratio: f64 },
}

/// Monte Carlo simulator
pub struct MonteCarloSimulator {
    /// Number of simulation runs
    num_simulations: u32,

    /// Random seed for reproducibility
    seed: u64,

    /// Number of histogram bins
    num_bins: usize,
}

impl MonteCarloSimulator {
    /// Create new Monte Carlo simulator
    pub fn new(num_simulations: u32) -> Self {
        Self {
            num_simulations,
            seed: 42, // Default seed for deterministic results
            num_bins: 20,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set number of histogram bins
    pub fn with_bins(mut self, num_bins: usize) -> Self {
        self.num_bins = num_bins;
        self
    }

    /// Run simulation with uncertainty inputs
    pub fn simulate(&self, inputs: &[UncertaintyInput]) -> Result<MonteCarloResult> {
        if inputs.is_empty() {
            return Err(CostPilotError::new(
                "MC_001",
                ErrorCategory::ValidationError,
                "No uncertainty inputs provided for Monte Carlo simulation".to_string(),
            ));
        }

        let mut simulated_costs = Vec::with_capacity(self.num_simulations as usize);

        // Run simulations
        for i in 0..self.num_simulations {
            let mut total_cost = 0.0;

            for input in inputs {
                let sample = self.sample_distribution(input, i);
                total_cost += sample * input.weight;
            }

            simulated_costs.push(total_cost.max(0.0)); // Ensure non-negative
        }

        // Sort for percentile calculations
        simulated_costs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate statistics
        let mean = simulated_costs.iter().sum::<f64>() / simulated_costs.len() as f64;
        let median = self.percentile(&simulated_costs, 50);
        let std_dev = self.calculate_std_dev(&simulated_costs, mean);

        // Calculate percentiles
        let mut percentiles = HashMap::new();
        for p in [1, 5, 10, 25, 50, 75, 90, 95, 99] {
            percentiles.insert(p, self.percentile(&simulated_costs, p));
        }

        // Calculate VaR and CVaR at 95%
        let var_95 = self.percentile(&simulated_costs, 95);
        let cvar_95 = self.calculate_cvar(&simulated_costs, 0.95);

        // Build distribution
        let distribution = self.build_distribution(&simulated_costs);

        Ok(MonteCarloResult {
            num_simulations: self.num_simulations,
            mean_cost: mean,
            median_cost: median,
            std_dev,
            percentiles,
            var_95,
            cvar_95,
            distribution,
        })
    }

    /// Sample from uncertainty distribution (deterministic with seed)
    fn sample_distribution(&self, input: &UncertaintyInput, iteration: u32) -> f64 {
        // Use simple deterministic pseudo-random generation for reproducibility
        let random_value = self.deterministic_random(iteration);

        match input.uncertainty_type {
            UncertaintyType::Normal { std_dev_ratio } => {
                // Box-Muller transform for normal distribution
                let u1 = random_value;
                let u2 = self.deterministic_random(iteration + 1000);
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                input.base_value + z * input.base_value * std_dev_ratio
            }

            UncertaintyType::LogNormal { std_dev_ratio } => {
                // Log-normal: exp(normal)
                let u1 = random_value;
                let u2 = self.deterministic_random(iteration + 1000);
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                let log_mean = (input.base_value).ln();
                (log_mean + z * std_dev_ratio).exp()
            }

            UncertaintyType::Uniform {
                min_ratio,
                max_ratio,
            } => {
                let range = max_ratio - min_ratio;
                input.base_value * (min_ratio + random_value * range)
            }

            UncertaintyType::Triangular {
                min_ratio,
                max_ratio,
            } => {
                // Triangular distribution with mode at base_value
                let min_val = input.base_value * min_ratio;
                let max_val = input.base_value * max_ratio;
                let mode_val = input.base_value;

                if random_value < (mode_val - min_val) / (max_val - min_val) {
                    min_val + ((max_val - min_val) * (mode_val - min_val) * random_value).sqrt()
                } else {
                    max_val
                        - ((max_val - min_val) * (max_val - mode_val) * (1.0 - random_value)).sqrt()
                }
            }
        }
    }

    /// Deterministic pseudo-random number (0.0 - 1.0)
    fn deterministic_random(&self, iteration: u32) -> f64 {
        // Linear congruential generator
        let a = 1664525_u64;
        let c = 1013904223_u64;
        let m = 2_u64.pow(32);

        let value = ((a * (self.seed + iteration as u64) + c) % m) as f64 / m as f64;
        value.clamp(0.001, 0.999) // Avoid exact 0 or 1
    }

    /// Calculate percentile from sorted data
    fn percentile(&self, sorted_data: &[f64], percentile: u8) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let index = (sorted_data.len() as f64 * percentile as f64 / 100.0) as usize;
        let index = index.min(sorted_data.len() - 1);
        sorted_data[index]
    }

    /// Calculate standard deviation
    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }

        let variance =
            data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;

        variance.sqrt()
    }

    /// Calculate Conditional Value at Risk (expected cost in worst p%)
    fn calculate_cvar(&self, sorted_data: &[f64], threshold: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let cutoff_index = (sorted_data.len() as f64 * threshold) as usize;
        let tail_data = &sorted_data[cutoff_index..];

        if tail_data.is_empty() {
            return sorted_data[sorted_data.len() - 1];
        }

        tail_data.iter().sum::<f64>() / tail_data.len() as f64
    }

    /// Build cost distribution histogram
    fn build_distribution(&self, sorted_data: &[f64]) -> CostDistribution {
        if sorted_data.is_empty() {
            return CostDistribution {
                bins: Vec::new(),
                min: 0.0,
                max: 0.0,
                shape: DistributionShape::Normal,
            };
        }

        let min = sorted_data[0];
        let max = sorted_data[sorted_data.len() - 1];
        let bin_width = (max - min) / self.num_bins as f64;

        let mut bins = Vec::with_capacity(self.num_bins);
        for i in 0..self.num_bins {
            let lower = min + i as f64 * bin_width;
            let upper = min + (i + 1) as f64 * bin_width;

            let count = sorted_data
                .iter()
                .filter(|&&cost| cost >= lower && (i == self.num_bins - 1 || cost < upper))
                .count() as u32;

            let frequency = count as f64 / sorted_data.len() as f64;

            bins.push(DistributionBin {
                lower,
                upper,
                count,
                frequency,
            });
        }

        let shape = self.classify_distribution_shape(sorted_data);

        CostDistribution {
            bins,
            min,
            max,
            shape,
        }
    }

    /// Classify distribution shape
    fn classify_distribution_shape(&self, sorted_data: &[f64]) -> DistributionShape {
        let mean = sorted_data.iter().sum::<f64>() / sorted_data.len() as f64;
        let median = sorted_data[sorted_data.len() / 2];

        // Calculate skewness
        let skew = (mean - median) / self.calculate_std_dev(sorted_data, mean);

        if skew.abs() < 0.2 {
            DistributionShape::Normal
        } else if skew > 0.2 {
            DistributionShape::RightSkewed
        } else {
            DistributionShape::LeftSkewed
        }
    }
}

impl Default for MonteCarloSimulator {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_simulation() {
        let simulator = MonteCarloSimulator::new(1000);

        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.2 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        assert_eq!(result.num_simulations, 1000);
        assert!(result.mean_cost > 0.0);
        assert!(result.median_cost > 0.0);
        assert!(result.std_dev > 0.0);
        assert!(result.var_95 > result.median_cost);
        assert!(result.cvar_95 >= result.var_95);
    }

    #[test]
    fn test_deterministic_simulation() {
        let sim1 = MonteCarloSimulator::new(100).with_seed(42);
        let sim2 = MonteCarloSimulator::new(100).with_seed(42);

        let inputs = vec![UncertaintyInput {
            base_value: 50.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];

        let result1 = sim1.simulate(&inputs).unwrap();
        let result2 = sim2.simulate(&inputs).unwrap();

        // Same seed should produce identical results
        assert_eq!(result1.mean_cost, result2.mean_cost);
        assert_eq!(result1.median_cost, result2.median_cost);
    }

    #[test]
    fn test_percentiles() {
        let simulator = MonteCarloSimulator::new(1000);

        let inputs = vec![UncertaintyInput {
            base_value: 200.0,
            uncertainty_type: UncertaintyType::Normal {
                std_dev_ratio: 0.15,
            },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        // P50 should be close to median
        assert!((result.percentiles[&50] - result.median_cost).abs() < 1.0);

        // P1 < P50 < P99
        assert!(result.percentiles[&1] < result.percentiles[&50]);
        assert!(result.percentiles[&50] < result.percentiles[&99]);
    }

    #[test]
    fn test_distribution_bins() {
        let simulator = MonteCarloSimulator::new(1000).with_bins(10);

        let inputs = vec![UncertaintyInput {
            base_value: 150.0,
            uncertainty_type: UncertaintyType::Uniform {
                min_ratio: 0.8,
                max_ratio: 1.2,
            },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        assert_eq!(result.distribution.bins.len(), 10);

        // Total frequency should sum to ~1.0
        let total_freq: f64 = result.distribution.bins.iter().map(|b| b.frequency).sum();
        assert!((total_freq - 1.0).abs() < 0.01);
    }
}
