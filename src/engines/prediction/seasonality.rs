// Seasonality detection - identify periodic cost patterns over time

use crate::engines::shared::error_model::Result;
use serde::{Deserialize, Serialize};

/// Seasonal pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityAnalysis {
    /// Whether seasonality was detected
    pub has_seasonality: bool,

    /// Detected seasonal patterns
    pub patterns: Vec<SeasonalPattern>,

    /// Overall seasonality strength (0.0 - 1.0)
    pub strength: f64,

    /// Recommended adjustment factor for current period
    pub adjustment_factor: f64,
}

/// Individual seasonal pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    /// Pattern type
    pub pattern_type: PatternType,

    /// Periodicity in days
    pub period_days: u32,

    /// Pattern strength (0.0 - 1.0)
    pub strength: f64,

    /// Peak multiplier (relative to baseline)
    pub peak_multiplier: f64,

    /// Trough multiplier (relative to baseline)
    pub trough_multiplier: f64,

    /// Description
    pub description: String,
}

/// Type of seasonal pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Weekly pattern (7 days)
    Weekly,
    /// Monthly pattern (~30 days)
    Monthly,
    /// Quarterly pattern (~90 days)
    Quarterly,
    /// Annual pattern (~365 days)
    Annual,
    /// Custom period
    Custom,
}

/// Historical cost data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDataPoint {
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    /// Cost value
    pub cost: f64,
}

/// Seasonality detector
pub struct SeasonalityDetector {
    /// Historical cost data
    data_points: Vec<CostDataPoint>,

    /// Minimum data points required
    min_data_points: usize,

    /// Significance threshold for pattern detection
    significance_threshold: f64,
}

impl SeasonalityDetector {
    /// Create new seasonality detector
    pub fn new() -> Self {
        Self {
            data_points: Vec::new(),
            min_data_points: 30,          // Need at least 30 data points
            significance_threshold: 0.15, // 15% variation
        }
    }

    /// Add historical data points
    pub fn with_data(mut self, data: Vec<CostDataPoint>) -> Self {
        self.data_points = data;
        self
    }

    /// Set minimum data points required
    pub fn with_min_data_points(mut self, min: usize) -> Self {
        self.min_data_points = min;
        self
    }

    /// Detect seasonal patterns
    pub fn detect_seasonality(&self) -> Result<SeasonalityAnalysis> {
        if self.data_points.len() < self.min_data_points {
            return Ok(SeasonalityAnalysis {
                has_seasonality: false,
                patterns: Vec::new(),
                strength: 0.0,
                adjustment_factor: 1.0,
            });
        }

        let mut patterns = Vec::new();

        // Detect weekly pattern
        if let Some(weekly) = self.detect_weekly_pattern()? {
            patterns.push(weekly);
        }

        // Detect monthly pattern
        if let Some(monthly) = self.detect_monthly_pattern()? {
            patterns.push(monthly);
        }

        // Calculate overall seasonality strength
        let strength = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.strength).sum::<f64>() / patterns.len() as f64
        };

        // Calculate adjustment factor for current period
        let adjustment_factor = self.calculate_current_adjustment(&patterns);

        Ok(SeasonalityAnalysis {
            has_seasonality: !patterns.is_empty(),
            patterns,
            strength,
            adjustment_factor,
        })
    }

    /// Detect weekly pattern (business days vs weekends)
    fn detect_weekly_pattern(&self) -> Result<Option<SeasonalPattern>> {
        // Group data by day of week
        let mut weekday_costs: Vec<f64> = Vec::new();
        let mut weekend_costs: Vec<f64> = Vec::new();

        for point in &self.data_points {
            let day_of_week = self.get_day_of_week(point.timestamp);
            if day_of_week < 5 {
                // Monday-Friday (0-4)
                weekday_costs.push(point.cost);
            } else {
                // Saturday-Sunday (5-6)
                weekend_costs.push(point.cost);
            }
        }

        if weekday_costs.is_empty() || weekend_costs.is_empty() {
            return Ok(None);
        }

        let weekday_avg = weekday_costs.iter().sum::<f64>() / weekday_costs.len() as f64;
        let weekend_avg = weekend_costs.iter().sum::<f64>() / weekend_costs.len() as f64;
        let overall_avg = (weekday_avg * weekday_costs.len() as f64
            + weekend_avg * weekend_costs.len() as f64)
            / (weekday_costs.len() + weekend_costs.len()) as f64;

        // Check if variation is significant
        let max_variation = ((weekday_avg - weekend_avg).abs() / overall_avg).max(0.0);

        if max_variation < self.significance_threshold {
            return Ok(None);
        }

        let (peak_mult, trough_mult) = if weekday_avg > weekend_avg {
            (weekday_avg / overall_avg, weekend_avg / overall_avg)
        } else {
            (weekend_avg / overall_avg, weekday_avg / overall_avg)
        };

        Ok(Some(SeasonalPattern {
            pattern_type: PatternType::Weekly,
            period_days: 7,
            strength: max_variation,
            peak_multiplier: peak_mult,
            trough_multiplier: trough_mult,
            description: if weekday_avg > weekend_avg {
                "Higher costs on weekdays (business hours)".to_string()
            } else {
                "Higher costs on weekends".to_string()
            },
        }))
    }

    /// Detect monthly pattern (end-of-month spikes)
    fn detect_monthly_pattern(&self) -> Result<Option<SeasonalPattern>> {
        // Group data by day of month
        let mut early_month: Vec<f64> = Vec::new(); // Days 1-10
        let mut mid_month: Vec<f64> = Vec::new(); // Days 11-20
        let mut late_month: Vec<f64> = Vec::new(); // Days 21-31

        for point in &self.data_points {
            let day_of_month = self.get_day_of_month(point.timestamp);
            if day_of_month <= 10 {
                early_month.push(point.cost);
            } else if day_of_month <= 20 {
                mid_month.push(point.cost);
            } else {
                late_month.push(point.cost);
            }
        }

        if early_month.is_empty() || mid_month.is_empty() || late_month.is_empty() {
            return Ok(None);
        }

        let early_avg = early_month.iter().sum::<f64>() / early_month.len() as f64;
        let mid_avg = mid_month.iter().sum::<f64>() / mid_month.len() as f64;
        let late_avg = late_month.iter().sum::<f64>() / late_month.len() as f64;
        let overall_avg = (early_avg * early_month.len() as f64
            + mid_avg * mid_month.len() as f64
            + late_avg * late_month.len() as f64)
            / (early_month.len() + mid_month.len() + late_month.len()) as f64;

        // Find max variation
        let max_cost = early_avg.max(mid_avg).max(late_avg);
        let min_cost = early_avg.min(mid_avg).min(late_avg);
        let variation = (max_cost - min_cost) / overall_avg;

        if variation < self.significance_threshold {
            return Ok(None);
        }

        Ok(Some(SeasonalPattern {
            pattern_type: PatternType::Monthly,
            period_days: 30,
            strength: variation,
            peak_multiplier: max_cost / overall_avg,
            trough_multiplier: min_cost / overall_avg,
            description: "Monthly cost variation detected".to_string(),
        }))
    }

    /// Get day of week (0=Monday, 6=Sunday)
    fn get_day_of_week(&self, timestamp: u64) -> u32 {
        // Simplified calculation: Unix epoch (Jan 1, 1970) was Thursday (3)
        let days_since_epoch = timestamp / 86400;
        ((days_since_epoch + 3) % 7) as u32
    }

    /// Get day of month (1-31)
    fn get_day_of_month(&self, timestamp: u64) -> u32 {
        // Simplified - assume 30-day months for pattern detection
        let days_since_epoch = timestamp / 86400;
        ((days_since_epoch % 30) + 1) as u32
    }

    /// Calculate adjustment factor for current time period
    fn calculate_current_adjustment(&self, patterns: &[SeasonalPattern]) -> f64 {
        if patterns.is_empty() {
            return 1.0;
        }

        // Use current timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut total_adjustment = 0.0;
        let mut weight_sum = 0.0;

        for pattern in patterns {
            let phase = self.calculate_phase(now, pattern.period_days);
            // Peak at phase 0.5, trough at phase 0.0/1.0
            let phase_factor = (phase * 2.0 * std::f64::consts::PI).sin();
            let adjustment =
                1.0 + (pattern.peak_multiplier - 1.0) * phase_factor * pattern.strength;

            total_adjustment += adjustment * pattern.strength;
            weight_sum += pattern.strength;
        }

        if weight_sum > 0.0 {
            total_adjustment / weight_sum
        } else {
            1.0
        }
    }

    /// Calculate phase within pattern (0.0 - 1.0)
    fn calculate_phase(&self, timestamp: u64, period_days: u32) -> f64 {
        let days_since_epoch = timestamp / 86400;
        let position_in_period = days_since_epoch % period_days as u64;
        position_in_period as f64 / period_days as f64
    }
}

impl Default for SeasonalityDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Seasonality-adjusted prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalAdjustedPrediction {
    /// Base prediction (without seasonality)
    pub base_cost: f64,

    /// Seasonal adjustment factor applied
    pub adjustment_factor: f64,

    /// Final seasonally-adjusted cost
    pub adjusted_cost: f64,

    /// Seasonality analysis
    pub seasonality: SeasonalityAnalysis,
}

impl SeasonalAdjustedPrediction {
    /// Create new seasonally-adjusted prediction
    pub fn new(base_cost: f64, seasonality: SeasonalityAnalysis) -> Self {
        let adjustment_factor = seasonality.adjustment_factor;
        let adjusted_cost = base_cost * adjustment_factor;

        Self {
            base_cost,
            adjustment_factor,
            adjusted_cost,
            seasonality,
        }
    }

    /// Get explanation of seasonal adjustment
    pub fn explanation(&self) -> String {
        if !self.seasonality.has_seasonality {
            return "No seasonal patterns detected".to_string();
        }

        let mut explanation = format!(
            "Seasonal adjustment: {:.1}% {} from baseline\n",
            (self.adjustment_factor - 1.0).abs() * 100.0,
            if self.adjustment_factor > 1.0 {
                "above"
            } else {
                "below"
            }
        );

        for pattern in &self.seasonality.patterns {
            explanation.push_str(&format!(
                "  - {}: {:.0}% variation\n",
                match pattern.pattern_type {
                    PatternType::Weekly => "Weekly pattern",
                    PatternType::Monthly => "Monthly pattern",
                    PatternType::Quarterly => "Quarterly pattern",
                    PatternType::Annual => "Annual pattern",
                    PatternType::Custom => "Custom pattern",
                },
                pattern.strength * 100.0
            ));
        }

        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_seasonality_with_insufficient_data() {
        let detector = SeasonalityDetector::new().with_data(vec![
            CostDataPoint {
                timestamp: 1000,
                cost: 100.0,
            },
            CostDataPoint {
                timestamp: 2000,
                cost: 105.0,
            },
        ]);

        let result = detector.detect_seasonality().unwrap();
        assert!(!result.has_seasonality);
    }

    #[test]
    fn test_weekly_pattern_detection() {
        // Generate weekday/weekend pattern
        let mut data = Vec::new();
        for i in 0..60 {
            let timestamp = i * 86400; // Daily data
            let day_of_week = (i + 3) % 7; // Start on Thursday
            let cost = if day_of_week < 5 { 150.0 } else { 80.0 }; // Weekday vs weekend
            data.push(CostDataPoint { timestamp, cost });
        }

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        assert!(result.has_seasonality);
        assert!(result
            .patterns
            .iter()
            .any(|p| p.pattern_type == PatternType::Weekly));
    }

    #[test]
    fn test_seasonal_adjustment() {
        let seasonality = SeasonalityAnalysis {
            has_seasonality: true,
            patterns: vec![SeasonalPattern {
                pattern_type: PatternType::Weekly,
                period_days: 7,
                strength: 0.25,
                peak_multiplier: 1.3,
                trough_multiplier: 0.7,
                description: "Weekly pattern".to_string(),
            }],
            strength: 0.25,
            adjustment_factor: 1.15,
        };

        let prediction = SeasonalAdjustedPrediction::new(100.0, seasonality);

        assert_eq!(prediction.base_cost, 100.0);
        assert_eq!(prediction.adjustment_factor, 1.15);
        assert!((prediction.adjusted_cost - 115.0).abs() < 1e-6);
    }
}
