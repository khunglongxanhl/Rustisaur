//! Main Rustisaur engine.

use std::cell::Cell;
use std::path::Path;
use std::time::Instant;

use mlua::{Lua, Value};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use rustisaur_lua_bridge::LuaStateManager;
use rustisaur_stdlib::register_all;

use crate::cache::ScriptCache;
use crate::config::EngineConfig;
use crate::error::{EngineError, Result, RexError};
use crate::runtime::RexRuntime;

/// Main Rustisaur engine managing Lua state, runtime, and resources.
pub struct RustisaurEngine {
    lua_manager: LuaStateManager,
    config: EngineConfig,
    runtime: RexRuntime,
    active: bool,
    script_cache: ScriptCache,
    execution_count: Cell<u64>,
    total_execution_time_ms: Cell<u64>,
}

impl RustisaurEngine {
    /// Create a new Rustisaur engine with the given configuration.
    pub fn new(config: EngineConfig) -> Result<Self> {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.log_level.to_string()));

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .try_init();

        let runtime = RexRuntime::new()
            .map_err(|e| RexError::Engine(EngineError::RuntimeInit(e.to_string())))?;
        let mut lua_manager = LuaStateManager::new()?;

        if config.enable_async {
            lua_manager.enable_async(runtime.handle())?;
        }

        if config.sandbox_mode {
            let sandbox =
                rustisaur_lua_bridge::Sandbox::new().with_memory_limit(config.max_memory_mb);
            lua_manager.configure_sandbox(sandbox);
            lua_manager.sandbox()?;
        }

        lua_manager.register_globals()?;
        register_all(lua_manager.lua())?;

        // Initialize script cache with capacity of 100 scripts
        let script_cache = ScriptCache::new(100);

        info!("Rustisaur engine initialized");
        debug!("Script cache capacity: 100 scripts");

        Ok(Self {
            lua_manager,
            config,
            runtime,
            active: true,
            script_cache,
            execution_count: Cell::new(0),
            total_execution_time_ms: Cell::new(0),
        })
    }

    /// Execute a Lua/Rustisaur source string with caching and profiling.
    pub fn execute_script(&self, script: &str) -> Result<Value<'_>> {
        self.ensure_active()?;

        let start_time = Instant::now();

        // Validate script for dangerous patterns (security check)
        if self.config.sandbox_mode {
            if let Err(e) = self.validate_script_security(script) {
                warn!("Security validation failed: {}", e);
                return Err(RexError::Engine(EngineError::ExecutionFailed(e)));
            }
        }

        // Try to get from cache first (performance optimization)
        let lua = self.lua_manager.lua();
        let cached = self
            .script_cache
            .get_or_compile(lua, script)
            .map_err(|e| RexError::Engine(EngineError::ExecutionFailed(e.to_string())))?;

        // Execute cached bytecode
        let results = self
            .script_cache
            .execute_cached(lua, &cached)
            .map_err(|e| RexError::Engine(EngineError::ExecutionFailed(e.to_string())))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update execution statistics (using Cell for interior mutability)
        self.execution_count.set(self.execution_count.get() + 1);
        self.total_execution_time_ms
            .set(self.total_execution_time_ms.get() + execution_time);

        debug!(
            "Script executed in {}ms (cache: {})",
            execution_time,
            if !cached.bytecode.is_empty() {
                "HIT"
            } else {
                "MISS"
            }
        );

        Ok(results.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Execute a script file (.rex or .lua) with caching.
    pub fn execute_file(&self, path: &Path) -> Result<Value<'_>> {
        self.ensure_active()?;

        let start_time = Instant::now();

        if !path.exists() {
            return Err(RexError::Engine(EngineError::FileNotFound(
                path.display().to_string(),
            )));
        }

        let results = self.lua_manager.execute_file(path)?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update execution statistics
        self.execution_count.set(self.execution_count.get() + 1);
        self.total_execution_time_ms
            .set(self.total_execution_time_ms.get() + execution_time);

        debug!("File executed in {}ms: {}", execution_time, path.display());

        Ok(results.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Execute a script file asynchronously (for scripts using async I/O).
    pub async fn execute_file_async(&self, path: &Path) -> Result<Value<'_>> {
        self.ensure_active()?;

        let start_time = Instant::now();

        if !path.exists() {
            return Err(RexError::Engine(EngineError::FileNotFound(
                path.display().to_string(),
            )));
        }

        let path = path.to_path_buf();
        let code = tokio::fs::read_to_string(&path).await.map_err(|e| {
            RexError::Engine(EngineError::ExecutionFailed(format!(
                "Failed to read {}: {e}",
                path.display()
            )))
        })?;

        let result = self.execute_script(&code)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        debug!(
            "Async file executed in {}ms: {}",
            execution_time,
            path.display()
        );

        Ok(result)
    }

    /// Register a custom Rust function as a Lua global.
    pub fn register_function<F>(&mut self, name: &str, func: F) -> Result<()>
    where
        F: for<'lua> Fn(&'lua Lua, Value<'lua>) -> mlua::Result<Value<'lua>> + Send + 'static,
    {
        self.ensure_active()?;
        self.lua_manager.register_function(name, func)?;
        debug!("Registered custom function: {}", name);
        Ok(())
    }

    /// Access the underlying Lua state manager.
    pub fn lua_manager(&self) -> &LuaStateManager {
        &self.lua_manager
    }

    /// Access mutable Lua state manager.
    pub fn lua_manager_mut(&mut self) -> &mut LuaStateManager {
        &mut self.lua_manager
    }

    /// Get engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get the Tokio runtime handle.
    pub fn runtime(&self) -> &RexRuntime {
        &self.runtime
    }

    /// Get script cache statistics.
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.script_cache.stats()
    }

    /// Clear the script cache.
    pub fn clear_cache(&self) {
        self.script_cache.clear();
        info!("Script cache cleared");
    }

    /// Reset performance statistics.
    pub fn reset_stats(&self) {
        self.execution_count.set(0);
        self.total_execution_time_ms.set(0);
        info!("Performance statistics reset");
    }

    /// Get performance statistics.
    pub fn performance_stats(&self) -> PerformanceStats {
        let total_executions = self.execution_count.get();
        let total_time_ms = self.total_execution_time_ms.get();

        let avg_time = total_time_ms.checked_div(total_executions).unwrap_or(0);

        PerformanceStats {
            total_executions,
            total_time_ms,
            average_time_ms: avg_time,
            cache_stats: self.script_cache.stats(),
        }
    }

    /// Reload a script file (hot-reload support).
    pub fn reload_file(&self, path: &Path) -> Result<Value<'_>> {
        info!("Hot-reloading script: {}", path.display());

        // Clear cache for this specific file
        self.script_cache.clear();

        self.execute_file(path)
    }

    /// Shut down the engine and release resources.
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.active {
            return Err(RexError::Engine(EngineError::ShutDown));
        }

        // Log performance stats before shutdown
        let stats = self.performance_stats();
        info!("Engine shutting down. Performance: {}", stats);

        self.active = false;
        info!("Rustisaur engine shut down");
        Ok(())
    }

    /// Validate script for security violations (sandbox mode).
    fn validate_script_security(&self, script: &str) -> std::result::Result<(), String> {
        // Check for dangerous patterns
        let dangerous_patterns = vec![
            ("os.execute", "Process execution"),
            ("io.popen", "Process execution"),
            ("loadfile", "File loading"),
            ("dofile", "File execution"),
            ("os.remove", "File deletion"),
            ("os.rename", "File renaming"),
        ];

        for (pattern, description) in dangerous_patterns {
            if script.contains(pattern) {
                return Err(format!(
                    "Security violation: {} detected ({})",
                    pattern, description
                ));
            }
        }

        Ok(())
    }

    fn ensure_active(&self) -> Result<()> {
        if !self.active {
            Err(RexError::Engine(EngineError::ShutDown))
        } else {
            Ok(())
        }
    }
}

impl Drop for RustisaurEngine {
    fn drop(&mut self) {
        if self.active {
            let _ = self.shutdown();
        }
    }
}

/// Performance statistics for the engine.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_executions: u64,
    pub total_time_ms: u64,
    pub average_time_ms: u64,
    pub cache_stats: crate::cache::CacheStats,
}

impl std::fmt::Display for PerformanceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Executions: {}, Avg time: {}ms, {}",
            self.total_executions, self.average_time_ms, self.cache_stats
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_executes_hello_world() {
        let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
        let result = engine.execute_script("return 'Hello, Rustisaur!'").unwrap();
        match result {
            Value::String(s) => assert_eq!(s.to_str().unwrap(), "Hello, Rustisaur!"),
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn engine_caches_scripts() {
        let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();

        // Execute same script twice
        engine.execute_script("return 42").unwrap();
        engine.execute_script("return 42").unwrap();

        // Check cache stats
        let stats = engine.cache_stats();
        assert!(stats.size > 0, "Cache should have at least 1 entry");
    }

    #[test]
    fn engine_performance_stats() {
        let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();

        engine.execute_script("return 1").unwrap();
        engine.execute_script("return 2").unwrap();

        let stats = engine.performance_stats();
        assert_eq!(stats.total_executions, 2);
    }

    #[test]
    fn engine_reset_stats() {
        let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();

        engine.execute_script("return 1").unwrap();
        engine.execute_script("return 2").unwrap();

        assert_eq!(engine.performance_stats().total_executions, 2);

        engine.reset_stats();
        assert_eq!(engine.performance_stats().total_executions, 0);
    }
}
