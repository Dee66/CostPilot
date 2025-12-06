// Zero-IAM security validation module

mod validator;
mod sandbox;

pub use validator::SecurityValidator;
pub use sandbox::{SandboxLimits, SandboxViolation};
