//! Engine configuration.

use tracing::Level;

/// Runtime configuration for the Rustisaur engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum memory usage in megabytes.
    pub max_memory_mb: usize,
    /// Script execution timeout in seconds (0 = no timeout).
    pub script_timeout_secs: u64,
    /// Enable async I/O support.
    pub enable_async: bool,
    /// Enable sandbox mode for untrusted scripts.
    pub sandbox_mode: bool,
    /// Logging level.
    pub log_level: Level,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            script_timeout_secs: 30,
            enable_async: true,
            sandbox_mode: false,
            log_level: Level::INFO,
        }
    }
}

impl EngineConfig {
    /// Create a sandboxed configuration suitable for untrusted scripts.
    pub fn sandboxed() -> Self {
        Self {
            sandbox_mode: true,
            ..Default::default()
        }
    }
}
