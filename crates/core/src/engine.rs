//! Main Rustisaur engine.

use std::path::Path;

use mlua::{Lua, Value};
use tracing_subscriber::EnvFilter;

use rustisaur_lua_bridge::LuaStateManager;
use rustisaur_stdlib::register_all;

use crate::config::EngineConfig;
use crate::error::{EngineError, RexError, Result};
use crate::runtime::RexRuntime;

/// Main Rustisaur engine managing Lua state, runtime, and resources.
pub struct RustisaurEngine {
    lua_manager: LuaStateManager,
    config: EngineConfig,
    runtime: RexRuntime,
    active: bool,
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

        let runtime = RexRuntime::new().map_err(|e| RexError::Engine(EngineError::RuntimeInit(e.to_string())))?;
        let mut lua_manager = LuaStateManager::new()?;

        if config.enable_async {
            lua_manager.enable_async(runtime.handle())?;
        }

        if config.sandbox_mode {
            let sandbox = rustisaur_lua_bridge::Sandbox::new()
                .with_memory_limit(config.max_memory_mb);
            lua_manager.configure_sandbox(sandbox);
            lua_manager.sandbox()?;
        }

        lua_manager.register_globals()?;
        register_all(lua_manager.lua())?;

        tracing::info!("Rustisaur engine initialized");

        Ok(Self {
            lua_manager,
            config,
            runtime,
            active: true,
        })
    }

    /// Execute a Lua/Rustisaur source string.
    pub fn execute_script(&self, script: &str) -> Result<Value<'_>> {
        self.ensure_active()?;
        let results = self.lua_manager.execute(script)?;
        Ok(results.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Execute a script file (.rex or .lua).
    pub fn execute_file(&self, path: &Path) -> Result<Value<'_>> {
        self.ensure_active()?;
        if !path.exists() {
            return Err(RexError::Engine(EngineError::FileNotFound(path.display().to_string())));
        }
        let results = self.lua_manager.execute_file(path)?;
        Ok(results.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Execute a script file asynchronously (for scripts using async I/O).
    pub async fn execute_file_async(&self, path: &Path) -> Result<Value<'_>> {
        self.ensure_active()?;
        if !path.exists() {
            return Err(RexError::Engine(EngineError::FileNotFound(path.display().to_string())));
        }
        let path = path.to_path_buf();
        let code = tokio::fs::read_to_string(&path).await.map_err(|e| {
            RexError::Engine(EngineError::ExecutionFailed(format!("Failed to read {}: {e}", path.display())))
        })?;
        self.execute_script(&code)
    }

    /// Register a custom Rust function as a Lua global.
    pub fn register_function<F>(&mut self, name: &str, func: F) -> Result<()>
    where
        F: for<'lua> Fn(&'lua Lua, Value<'lua>) -> mlua::Result<Value<'lua>> + Send + 'static,
    {
        self.ensure_active()?;
        self.lua_manager.register_function(name, func)?;
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

    /// Reload a script file (hot-reload support).
    pub fn reload_file(&self, path: &Path) -> Result<Value<'_>> {
        tracing::info!("Hot-reloading script: {}", path.display());
        self.execute_file(path)
    }

    /// Shut down the engine and release resources.
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.active {
            return Err(RexError::Engine(EngineError::ShutDown));
        }
        self.active = false;
        tracing::info!("Rustisaur engine shut down");
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
}
