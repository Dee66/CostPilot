use serde::{Deserialize, Serialize};

/// Sandbox limits for WASM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLimits {
    /// Maximum file size in megabytes
    pub max_file_size_mb: u32,
    /// Maximum memory allocation in megabytes
    pub max_memory_mb: u32,
    /// Maximum execution timeout in milliseconds
    pub max_timeout_ms: u32,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_file_size_mb: 20,
            max_memory_mb: 256,
            max_timeout_ms: 2000,
        }
    }
}

impl SandboxLimits {
    /// Create custom sandbox limits
    pub fn new(max_file_size_mb: u32, max_memory_mb: u32, max_timeout_ms: u32) -> Self {
        Self {
            max_file_size_mb,
            max_memory_mb,
            max_timeout_ms,
        }
    }

    /// Validate file size against limits
    pub fn check_file_size(&self, size_bytes: u64) -> Result<(), SandboxViolation> {
        let max_bytes = (self.max_file_size_mb as u64) * 1024 * 1024;
        if size_bytes > max_bytes {
            return Err(SandboxViolation::FileSizeExceeded {
                actual_mb: (size_bytes / 1024 / 1024) as u32,
                limit_mb: self.max_file_size_mb,
            });
        }
        Ok(())
    }

    /// Validate memory allocation against limits
    pub fn check_memory(&self, allocated_mb: u32) -> Result<(), SandboxViolation> {
        if allocated_mb > self.max_memory_mb {
            return Err(SandboxViolation::MemoryLimitExceeded {
                actual_mb: allocated_mb,
                limit_mb: self.max_memory_mb,
            });
        }
        Ok(())
    }

    /// Check if execution time exceeds timeout
    pub fn check_timeout(&self, elapsed_ms: u64) -> Result<(), SandboxViolation> {
        if elapsed_ms > self.max_timeout_ms as u64 {
            return Err(SandboxViolation::TimeoutExceeded {
                actual_ms: elapsed_ms,
                limit_ms: self.max_timeout_ms,
            });
        }
        Ok(())
    }
}

/// Sandbox violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxViolation {
    /// File size exceeded limit
    FileSizeExceeded { actual_mb: u32, limit_mb: u32 },

    /// Memory allocation exceeded limit
    MemoryLimitExceeded { actual_mb: u32, limit_mb: u32 },

    /// Execution timeout exceeded
    TimeoutExceeded { actual_ms: u64, limit_ms: u32 },

    /// Network access detected
    NetworkAccessDetected { operation: String },

    /// AWS SDK usage detected
    AwsSdkDetected { service: String },

    /// Secret or token detected in output
    SecretLeakage { pattern: String },
}

impl std::fmt::Display for SandboxViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxViolation::FileSizeExceeded {
                actual_mb,
                limit_mb,
            } => {
                write!(
                    f,
                    "File size {}MB exceeds limit of {}MB",
                    actual_mb, limit_mb
                )
            }
            SandboxViolation::MemoryLimitExceeded {
                actual_mb,
                limit_mb,
            } => {
                write!(
                    f,
                    "Memory allocation {}MB exceeds limit of {}MB",
                    actual_mb, limit_mb
                )
            }
            SandboxViolation::TimeoutExceeded {
                actual_ms,
                limit_ms,
            } => {
                write!(
                    f,
                    "Execution time {}ms exceeds timeout of {}ms",
                    actual_ms, limit_ms
                )
            }
            SandboxViolation::NetworkAccessDetected { operation } => {
                write!(f, "Network access detected: {}", operation)
            }
            SandboxViolation::AwsSdkDetected { service } => {
                write!(f, "AWS SDK usage detected: {}", service)
            }
            SandboxViolation::SecretLeakage { pattern } => {
                write!(f, "Potential secret leakage detected: {}", pattern)
            }
        }
    }
}

impl std::error::Error for SandboxViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = SandboxLimits::default();
        assert_eq!(limits.max_file_size_mb, 20);
        assert_eq!(limits.max_memory_mb, 256);
        assert_eq!(limits.max_timeout_ms, 2000);
    }

    #[test]
    fn test_file_size_check() {
        let limits = SandboxLimits::default();

        // Within limit
        assert!(limits.check_file_size(10 * 1024 * 1024).is_ok());

        // Exceeds limit
        assert!(limits.check_file_size(30 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_memory_check() {
        let limits = SandboxLimits::default();

        // Within limit
        assert!(limits.check_memory(128).is_ok());

        // Exceeds limit
        assert!(limits.check_memory(512).is_err());
    }

    #[test]
    fn test_timeout_check() {
        let limits = SandboxLimits::default();

        // Within limit
        assert!(limits.check_timeout(1000).is_ok());

        // Exceeds limit
        assert!(limits.check_timeout(3000).is_err());
    }
}
