//! Built-in Lua module registration helpers.

use mlua::Lua;

use crate::error::Result;

/// Register placeholder built-in modules.
pub fn register_builtin_modules(_lua: &Lua) -> Result<()> {
    Ok(())
}
