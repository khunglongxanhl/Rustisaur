//! Automatic Rust <-> Lua type conversions.

use mlua::{FromLua, Function, IntoLua, Lua, Table, Value as LuaValue};

use crate::error::{LuaBridgeError, Result};

/// Rustisaur value type for cross-language communication.
#[derive(Debug, Clone)]
pub enum RexValue<'lua> {
    String(String),
    Number(f64),
    Boolean(bool),
    Table(Table<'lua>),
    Function(Function<'lua>),
    Nil,
}

impl<'lua> IntoLua<'lua> for RexValue<'lua> {
    fn into_lua(self, lua: &'lua Lua) -> mlua::Result<LuaValue<'lua>> {
        match self {
            RexValue::String(s) => s.into_lua(lua),
            RexValue::Number(n) => n.into_lua(lua),
            RexValue::Boolean(b) => b.into_lua(lua),
            RexValue::Table(t) => t.into_lua(lua),
            RexValue::Function(f) => f.into_lua(lua),
            RexValue::Nil => Ok(LuaValue::Nil),
        }
    }
}

impl<'lua> FromLua<'lua> for RexValue<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            LuaValue::String(s) => Ok(RexValue::String(s.to_str()?.to_string())),
            LuaValue::Integer(i) => Ok(RexValue::Number(i as f64)),
            LuaValue::Number(n) => Ok(RexValue::Number(n)),
            LuaValue::Boolean(b) => Ok(RexValue::Boolean(b)),
            LuaValue::Table(t) => Ok(RexValue::Table(t)),
            LuaValue::Function(f) => Ok(RexValue::Function(f)),
            LuaValue::Nil => Ok(RexValue::Nil),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "RexValue",
                message: None,
            }),
        }
    }
}

/// Convert a RexValue to mlua Value.
pub fn rex_to_lua<'lua>(value: RexValue<'lua>, lua: &'lua Lua) -> mlua::Result<LuaValue<'lua>> {
    value.into_lua(lua)
}

/// Convert mlua Value to RexValue.
pub fn lua_to_rex<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> Result<RexValue<'lua>> {
    RexValue::from_lua(value, lua).map_err(LuaBridgeError::from)
}
