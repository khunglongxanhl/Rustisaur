//! Check command - syntax validation.

use std::path::Path;

use anyhow::{Context, Result};
use mlua::Lua;

pub fn execute(file: &str) -> Result<()> {
    let path = Path::new(file);
    let code = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {file}"))?;

    let lua = Lua::new();
    lua.load(&code)
        .set_name(file)
        .into_function()
        .with_context(|| format!("Syntax error in {file}"))?;

    println!("✓ {file} - syntax OK");
    Ok(())
}
