// Detection engine module

pub mod terraform;
pub mod cdk;
pub mod cloudformation;
pub mod classifier;
pub mod severity;
pub mod detection_engine;

pub use detection_engine::DetectionEngine;
pub use classifier::{classify_regression, RegressionClassifier};
pub use severity::calculate_severity_score;
