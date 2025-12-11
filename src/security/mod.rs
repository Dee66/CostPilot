// Zero-IAM security validation module

mod sandbox;
mod validator;

pub use sandbox::{SandboxLimits, SandboxViolation};
pub use validator::SecurityValidator;
