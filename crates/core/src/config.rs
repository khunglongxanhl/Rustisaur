//! Engine configuration.

use std::path::PathBuf;
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
    /// Detailed sandbox configuration.
    pub sandbox: SandboxConfig,
}

/// Detailed sandbox configuration for security.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    // File system restrictions
    /// Allow file read operations
    pub allow_read: bool,
    /// Allow file write operations
    pub allow_write: bool,
    /// Allowed read directories (empty = allow all if allow_read is true)
    pub allowed_read_dirs: Vec<PathBuf>,
    /// Allowed write directories (empty = allow all if allow_write is true)
    pub allowed_write_dirs: Vec<PathBuf>,
    
    // Network restrictions
    /// Allow network operations
    pub allow_network: bool,
    /// Allowed network hosts (empty = allow all if allow_network is true)
    pub allowed_hosts: Vec<String>,
    
    // System restrictions
    /// Allow process execution
    pub allow_process: bool,
    /// Allow environment variable access
    pub allow_env: bool,
    /// Allow file system watch
    pub allow_fs_watch: bool,
    
    // Resource limits
    /// Maximum number of iterations in loops (prevents infinite loops)
    pub max_loop_iterations: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            script_timeout_secs: 30,
            enable_async: true,
            sandbox_mode: false,
            log_level: Level::INFO,
            sandbox: SandboxConfig::default(),
        }
    }
}

impl EngineConfig {
    /// Create a sandboxed configuration suitable for untrusted scripts.
    pub fn sandboxed() -> Self {
        Self {
            sandbox_mode: true,
            max_memory_mb: 64,
            script_timeout_secs: 5,
            sandbox: SandboxConfig::strict(),
            ..Default::default()
        }
    }
    
    /// Create a development configuration with relaxed security.
    pub fn development() -> Self {
        Self {
            sandbox_mode: false,
            max_memory_mb: 1024,
            script_timeout_secs: 60,
            log_level: Level::DEBUG,
            sandbox: SandboxConfig::permissive(),
            ..Default::default()
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allow_read: true,
            allow_write: true,
            allowed_read_dirs: vec![],
            allowed_write_dirs: vec![],
            allow_network: false,
            allowed_hosts: vec![],
            allow_process: false,
            allow_env: false,
            allow_fs_watch: false,
            max_loop_iterations: 1_000_000,
        }
    }
}

impl SandboxConfig {
    /// Create a strict sandbox configuration (maximum security).
    pub fn strict() -> Self {
        Self {
            allow_read: false,
            allow_write: false,
            allowed_read_dirs: vec![],
            allowed_write_dirs: vec![],
            allow_network: false,
            allowed_hosts: vec![],
            allow_process: false,
            allow_env: false,
            allow_fs_watch: false,
            max_loop_iterations: 10_000,
        }
    }
    
    /// Create a permissive sandbox configuration (development mode).
    pub fn permissive() -> Self {
        Self {
            allow_read: true,
            allow_write: true,
            allowed_read_dirs: vec![],
            allowed_write_dirs: vec![],
            allow_network: true,
            allowed_hosts: vec![],
            allow_process: true,
            allow_env: true,
            allow_fs_watch: true,
            max_loop_iterations: 10_000_000,
        }
    }
    
    /// Check if a file path is allowed for reading.
    pub fn is_read_allowed(&self, path: &std::path::Path) -> bool {
        if !self.allow_read {
            return false;
        }
        
        if self.allowed_read_dirs.is_empty() {
            return true; // Allow all if no restrictions
        }
        
        self.allowed_read_dirs.iter().any(|dir| path.starts_with(dir))
    }
    
    /// Check if a file path is allowed for writing.
    pub fn is_write_allowed(&self, path: &std::path::Path) -> bool {
        if !self.allow_write {
            return false;
        }
        
        if self.allowed_write_dirs.is_empty() {
            return true; // Allow all if no restrictions
        }
        
        self.allowed_write_dirs.iter().any(|dir| path.starts_with(dir))
    }
    
    /// Check if a network host is allowed.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.allow_network {
            return false;
        }
        
        if self.allowed_hosts.is_empty() {
            return true; // Allow all if no restrictions
        }
        
        self.allowed_hosts.iter().any(|h| host.contains(h))
    }
}
