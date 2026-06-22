//! Lua state management with safety guarantees.

use mlua::{Lua, MultiValue, Value};
use tokio::runtime::Handle;

use crate::async_support;
use crate::error::{LuaBridgeError, Result};
use crate::sandbox::Sandbox;

/// Manages Lua state with safety guarantees.
pub struct LuaStateManager {
    state: Lua,
    sandbox: Sandbox,
    async_runtime: Option<Handle>,
    sandbox_enabled: bool,
}

impl LuaStateManager {
    /// Create a new Lua state with mlua.
    pub fn new() -> Result<Self> {
        let state = Lua::new();
        Ok(Self {
            state,
            sandbox: Sandbox::new(),
            async_runtime: None,
            sandbox_enabled: false,
        })
    }

    /// Access the underlying Lua state.
    pub fn lua(&self) -> &Lua {
        &self.state
    }

    /// Access mutable Lua state (for registration).
    pub fn lua_mut(&mut self) -> &mut Lua {
        &mut self.state
    }

    /// Execute Lua code and return all returned values.
    pub fn execute(&self, code: &str) -> Result<Vec<Value<'_>>> {
        let chunk = self.state.load(code);
        let results: MultiValue = chunk.eval()?;
        Ok(results.into_vec())
    }

    /// Execute a Lua file.
    pub fn execute_file(&self, path: &std::path::Path) -> Result<Vec<Value<'_>>> {
        let code = std::fs::read_to_string(path).map_err(|e| {
            LuaBridgeError::Conversion(format!("Failed to read {}: {e}", path.display()))
        })?;
        self.execute(&code)
    }

    /// Register Rust functions as Lua globals.
    pub fn register_globals(&mut self) -> Result<()> {
        let globals = self.state.globals();
        globals.set(
            "rex_version",
            self.state.create_function(|_, ()| Ok(env!("CARGO_PKG_VERSION")))?,
        )?;
        Ok(())
    }

    /// Enable async/await support in Lua.
    pub fn enable_async(&mut self, handle: Handle) -> Result<()> {
        self.async_runtime = Some(handle);
        async_support::install(&self.state)?;
        Ok(())
    }

    /// Apply sandbox restrictions.
    pub fn sandbox(&mut self) -> Result<()> {
        self.sandbox.apply(&self.state)?;
        self.sandbox_enabled = true;
        Ok(())
    }

    /// Returns whether sandbox mode is active.
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox_enabled
    }

    /// Register a Rust function as a Lua global.
    pub fn register_function<F>(&mut self, name: &str, func: F) -> Result<()>
    where
        F: for<'lua> Fn(&'lua Lua, Value<'lua>) -> mlua::Result<Value<'lua>> + Send + 'static,
    {
        let lua_func = self.state.create_function(move |lua, arg: Value| func(lua, arg))?;
        self.state.globals().set(name, lua_func)?;
        Ok(())
    }

    /// Register a function in a nested table path (e.g. "rex.print").
    pub fn register_in_table<F>(&mut self, path: &str, func: F) -> Result<()>
    where
        F: for<'lua> Fn(&'lua Lua, Value<'lua>) -> mlua::Result<Value<'lua>> + Send + 'static,
    {
        let parts: Vec<&str> = path.rsplitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(LuaBridgeError::Conversion(format!(
                "Invalid table path: {path}"
            )));
        }

        let (func_name, table_path) = (parts[0], parts[1]);
        let table: mlua::Table = self.state.globals().get(table_path)?;
        let lua_func = self
            .state
            .create_function(move |lua, arg: Value| func(lua, arg))?;
        table.set(func_name, lua_func)?;
        Ok(())
    }

    /// Get async runtime handle if available.
    pub fn async_handle(&self) -> Option<&Handle> {
        self.async_runtime.as_ref()
    }

    /// Configure sandbox settings.
    pub fn configure_sandbox(&mut self, sandbox: Sandbox) {
        self.sandbox = sandbox;
    }
}

impl Default for LuaStateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua state")
    }
}
