// Detection engine module

pub mod cdk;
pub mod classifier;
pub mod cloudformation;
pub mod detection_engine;
pub mod severity;
pub mod terraform;

pub use crate::engines::shared::models::{Detection, ResourceChange};
pub use classifier::{classify_regression, RegressionClassifier};
pub use detection_engine::DetectionEngine;
pub use severity::calculate_severity_score;
