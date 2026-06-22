//! Async/await support for Lua scripts.

use mlua::{Lua, Table, Value};

use crate::error::Result;

/// Install async primitives into the Lua state.
pub fn install(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let await_fn = lua.create_async_function(|_lua, future_val: Value| async move {
        // Futures are passed as userdata from stdlib; this is a passthrough for direct values.
        Ok(future_val)
    })?;

    globals.set("_rex_await", await_fn)?;

    // Sugar: `await expr` desugars via rex.async module helpers
    let async_table = lua.create_table()?;
    async_table.set(
        "run",
        lua.create_function(|_lua, func: mlua::Function| {
            let result = func.call::<_, Value>(())?;
            Ok(result)
        })?,
    )?;

    globals.set("rex_async", async_table)?;
    Ok(())
}

/// Create a Lua table wrapping an async Rust future result.
pub fn wrap_future_result<'lua>(lua: &'lua Lua, value: Value<'lua>) -> mlua::Result<Table<'lua>> {
    let table = lua.create_table()?;
    table.set("value", value)?;
    table.set("done", true)?;
    Ok(table)
}
