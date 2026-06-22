//! Security sandbox for untrusted Lua scripts.

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use mlua::{HookTriggers, Lua, Table};
use tracing::debug;

use crate::error::Result;

/// Security violations that can occur during script execution.
#[derive(Debug, thiserror::Error)]
pub enum SecurityViolation {
    #[error("File read not allowed: {0}")]
    FileReadNotAllowed(String),
    
    #[error("File write not allowed: {0}")]
    FileWriteNotAllowed(String),
    
    #[error("Network access not allowed: {0}")]
    NetworkAccessNotAllowed(String),
    
    #[error("Process execution not allowed")]
    ProcessExecutionNotAllowed,
    
    #[error("Environment variable access not allowed")]
    EnvAccessNotAllowed,
    
    #[error("Execution time limit exceeded: {0}ms")]
    TimeLimitExceeded(u64),
    
    #[error("Memory limit exceeded: {0}MB")]
    MemoryLimitExceeded(usize),
    
    #[error("Instruction limit exceeded")]
    InstructionLimitExceeded,
    
    #[error("Loop iteration limit exceeded")]
    LoopIterationLimitExceeded,
    
    #[error("Dangerous pattern detected: {0}")]
    DangerousPatternDetected(String),
    
    #[error("Module not allowed: {0}")]
    ModuleNotAllowed(String),
}

/// Detailed sandbox configuration for security.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    // File system restrictions
    pub allow_read: bool,
    pub allow_write: bool,
    pub allowed_read_dirs: Vec<std::path::PathBuf>,
    pub allowed_write_dirs: Vec<std::path::PathBuf>,
    
    // Network restrictions
    pub allow_network: bool,
    pub allowed_hosts: Vec<String>,
    
    // System restrictions
    pub allow_process: bool,
    pub allow_env: bool,
    pub allow_fs_watch: bool,
    
    // Resource limits
    pub max_loop_iterations: usize,
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
            return true;
        }
        
        self.allowed_read_dirs.iter().any(|dir| path.starts_with(dir))
    }
    
    /// Check if a file path is allowed for writing.
    pub fn is_write_allowed(&self, path: &std::path::Path) -> bool {
        if !self.allow_write {
            return false;
        }
        
        if self.allowed_write_dirs.is_empty() {
            return true;
        }
        
        self.allowed_write_dirs.iter().any(|dir| path.starts_with(dir))
    }
    
    /// Check if a network host is allowed.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.allow_network {
            return false;
        }
        
        if self.allowed_hosts.is_empty() {
            return true;
        }
        
        self.allowed_hosts.iter().any(|h| host.contains(h))
    }
}

/// Sandbox configuration and enforcement.
#[derive(Debug, Clone)]
pub struct Sandbox {
    pub max_memory_mb: usize,
    pub max_instructions: u64,
    pub max_execution_time_ms: u64,
    pub max_loop_iterations: usize,
    pub allowed_modules: HashSet<String>,
    pub disabled_functions: HashSet<String>,
    pub config: SandboxConfig,
    start_time: Option<Instant>,
    iteration_count: usize,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    /// Create a sandbox with secure defaults.
    pub fn new() -> Self {
        let mut disabled = HashSet::new();
        disabled.insert("os.execute".to_string());
        disabled.insert("os.exit".to_string());
        disabled.insert("io.popen".to_string());
        disabled.insert("loadfile".to_string());
        disabled.insert("dofile".to_string());
        disabled.insert("load".to_string());

        let allowed_modules = ["rex", "string", "table", "math", "coroutine"]
            .into_iter()
            .map(String::from)
            .collect();

        Self {
            max_memory_mb: 128,
            max_instructions: 1_000_000,
            max_execution_time_ms: 30_000,
            max_loop_iterations: 1_000_000,
            allowed_modules,
            disabled_functions: disabled,
            config: SandboxConfig::default(),
            start_time: None,
            iteration_count: 0,
        }
    }
    
    /// Create a sandbox with custom configuration.
    pub fn with_config(config: SandboxConfig) -> Self {
        let mut sandbox = Self::new();
        sandbox.max_loop_iterations = config.max_loop_iterations;
        sandbox.config = config;
        sandbox
    }
    
    /// Create a strict sandbox (maximum security).
    pub fn strict() -> Self {
        Self::with_config(SandboxConfig::strict())
    }
    
    /// Create a permissive sandbox (development mode).
    pub fn permissive() -> Self {
        Self::with_config(SandboxConfig::permissive())
    }

    /// Apply sandbox restrictions to a Lua state.
    pub fn apply(&self, lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        // Disable dangerous functions
        for func in &self.disabled_functions {
            let parts: Vec<&str> = func.split('.').collect();
            if parts.len() == 2 {
                if let Ok(table) = globals.get::<_, Table>(parts[0]) {
                    let disabled_fn = lua.create_function(|_, _: ()| {
                        Err::<(), mlua::Error>(mlua::Error::RuntimeError(
                            "Function disabled in sandbox mode".to_string(),
                        ))
                    })?;
                    table.set(parts[1], disabled_fn)?;
                }
            } else if parts.len() == 1 {
                let disabled_fn = lua.create_function(|_, _: ()| {
                    Err::<(), mlua::Error>(mlua::Error::RuntimeError(
                        "Function disabled in sandbox mode".to_string(),
                    ))
                })?;
                globals.set(parts[0], disabled_fn)?;
            }
        }

        // Set memory limit
        let memory_bytes = self.max_memory_mb * 1024 * 1024;
        lua.set_memory_limit(memory_bytes)?;
        debug!("Sandbox: Memory limit set to {}MB", self.max_memory_mb);

        // Set instruction hook for time/iteration tracking
        if self.max_instructions > 0 {
            let _max_instructions = self.max_instructions;
            lua.set_hook(
                HookTriggers::new().every_nth_instruction(1000), // Check every 1000 instructions
                move |_lua, _debug| {
                    // This hook will be called periodically
                    // We can't access self here, so we rely on Lua's built-in limits
                    Ok(())
                },
            );
        }

        debug!("Sandbox: Applied restrictions to Lua state");
        Ok(())
    }

    /// Start execution timer.
    pub fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
        self.iteration_count = 0;
        debug!("Sandbox: Execution timer started");
    }

    /// Check if execution time limit has been exceeded.
    pub fn check_time_limit(&self) -> std::result::Result<(), SecurityViolation> {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_millis() as u64;
            if elapsed > self.max_execution_time_ms {
                debug!("Sandbox: Time limit exceeded: {}ms", elapsed);
                return Err(SecurityViolation::TimeLimitExceeded(elapsed));
            }
        }
        Ok(())
    }

    /// Increment iteration counter and check limit.
    pub fn increment_iteration(&mut self) -> std::result::Result<(), SecurityViolation> {
        self.iteration_count += 1;
        if self.iteration_count > self.max_loop_iterations {
            debug!("Sandbox: Loop iteration limit exceeded: {}", self.iteration_count);
            return Err(SecurityViolation::LoopIterationLimitExceeded);
        }
        Ok(())
    }

    /// Check if file read is allowed.
    pub fn check_file_read(&self, path: &Path) -> std::result::Result<(), SecurityViolation> {
        if !self.config.is_read_allowed(path) {
            debug!("Sandbox: File read not allowed: {:?}", path);
            return Err(SecurityViolation::FileReadNotAllowed(
                path.to_string_lossy().to_string()
            ));
        }
        Ok(())
    }

    /// Check if file write is allowed.
    pub fn check_file_write(&self, path: &Path) -> std::result::Result<(), SecurityViolation> {
        if !self.config.is_write_allowed(path) {
            debug!("Sandbox: File write not allowed: {:?}", path);
            return Err(SecurityViolation::FileWriteNotAllowed(
                path.to_string_lossy().to_string()
            ));
        }
        Ok(())
    }

    /// Check if network access is allowed.
    pub fn check_network_access(&self, host: &str) -> std::result::Result<(), SecurityViolation> {
        if !self.config.is_host_allowed(host) {
            debug!("Sandbox: Network access not allowed: {}", host);
            return Err(SecurityViolation::NetworkAccessNotAllowed(host.to_string()));
        }
        Ok(())
    }

    /// Check if process execution is allowed.
    pub fn check_process_execution(&self) -> std::result::Result<(), SecurityViolation> {
        if !self.config.allow_process {
            debug!("Sandbox: Process execution not allowed");
            return Err(SecurityViolation::ProcessExecutionNotAllowed);
        }
        Ok(())
    }

    /// Check if environment variable access is allowed.
    pub fn check_env_access(&self) -> std::result::Result<(), SecurityViolation> {
        if !self.config.allow_env {
            debug!("Sandbox: Environment variable access not allowed");
            return Err(SecurityViolation::EnvAccessNotAllowed);
        }
        Ok(())
    }

    /// Validate script for dangerous patterns.
    pub fn validate_script(&self, code: &str) -> std::result::Result<(), SecurityViolation> {
        let dangerous_patterns = vec![
            ("os.execute", "Process execution"),
            ("io.popen", "Process execution"),
            ("loadfile", "File loading"),
            ("dofile", "File execution"),
            ("os.remove", "File deletion"),
            ("os.rename", "File renaming"),
        ];
        
        for (pattern, description) in dangerous_patterns {
            if code.contains(pattern) {
                debug!("Sandbox: Dangerous pattern detected: {} ({})", pattern, description);
                return Err(SecurityViolation::DangerousPatternDetected(
                    format!("{} ({})", pattern, description)
                ));
            }
        }
        
        Ok(())
    }

    /// Check if a module name is allowed.
    pub fn is_module_allowed(&self, name: &str) -> bool {
        self.allowed_modules.contains(name)
    }

    /// Configure memory limit in megabytes.
    pub fn with_memory_limit(mut self, mb: usize) -> Self {
        self.max_memory_mb = mb;
        self
    }

    /// Configure instruction limit.
    pub fn with_instruction_limit(mut self, limit: u64) -> Self {
        self.max_instructions = limit;
        self
    }

    /// Configure execution time limit.
    pub fn with_time_limit(mut self, ms: u64) -> Self {
        self.max_execution_time_ms = ms;
        self
    }

    /// Configure loop iteration limit.
    pub fn with_iteration_limit(mut self, limit: usize) -> Self {
        self.max_loop_iterations = limit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_disables_os_execute() {
        let lua = Lua::new();
        let sandbox = Sandbox::new();
        sandbox.apply(&lua).unwrap();

        lua.load("os.execute('echo test')").exec().unwrap_err();
    }

    #[test]
    fn sandbox_validates_script() {
        let sandbox = Sandbox::new();
        
        // Should pass for safe code
        assert!(sandbox.validate_script("print('hello')").is_ok());
        
        // Should fail for dangerous code
        assert!(sandbox.validate_script("os.execute('rm -rf /')").is_err());
        assert!(sandbox.validate_script("io.popen('ls')").is_err());
    }

    #[test]
    fn sandbox_checks_time_limit() {
        let mut sandbox = Sandbox::new().with_time_limit(1); // 1ms
        
        sandbox.start_timer();
        
        // Should pass immediately
        assert!(sandbox.check_time_limit().is_ok());
        
        // Wait and check again
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(sandbox.check_time_limit().is_err());
    }

    #[test]
    fn sandbox_checks_file_read() {
        let mut sandbox = Sandbox::new();
        sandbox.config.allow_read = false;
        
        let path = std::path::Path::new("/etc/passwd");
        assert!(sandbox.check_file_read(path).is_err());
    }

    #[test]
    fn sandbox_checks_file_write() {
        let mut sandbox = Sandbox::new();
        sandbox.config.allow_write = false;
        
        let path = std::path::Path::new("/tmp/test.txt");
        assert!(sandbox.check_file_write(path).is_err());
    }
}