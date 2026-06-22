//! Security sandbox for untrusted Lua scripts.

use std::collections::HashSet;

use mlua::{HookTriggers, Lua, Table};

use crate::error::Result;

/// Sandbox configuration and enforcement.
#[derive(Debug, Clone)]
pub struct Sandbox {
    pub max_memory_mb: usize,
    pub max_instructions: u64,
    pub allowed_modules: HashSet<String>,
    pub disabled_functions: HashSet<String>,
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
            allowed_modules,
            disabled_functions: disabled,
        }
    }

    /// Apply sandbox restrictions to a Lua state.
    pub fn apply(&self, lua: &Lua) -> Result<()> {
        let globals = lua.globals();

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

        let memory_bytes = (self.max_memory_mb * 1024 * 1024) as usize;
        lua.set_memory_limit(memory_bytes)?;

        if self.max_instructions > 0 {
            lua.set_hook(
                HookTriggers::new().every_nth_instruction(self.max_instructions as u32),
                |_lua, _debug| {
                    Err(mlua::Error::RuntimeError(
                        "Instruction limit exceeded".to_string(),
                    ))
                },
            );
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
}