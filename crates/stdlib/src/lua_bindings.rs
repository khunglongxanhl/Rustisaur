//! Lua bindings for the standard library.

use mlua::{Lua, Result as LuaResult, Table, Value};
use serde_json;

use crate::error::StdlibError;

/// Register all standard library functions into the Lua state.
pub fn register_all(lua: &Lua) -> Result<(), StdlibError> {
    let rex = lua.create_table()?;
    
    // Register rex.print
    rex.set("print", lua.create_function(|_, msg: String| {
        println!("{}", msg);
        Ok(())
    })?)?;
    
    // Register rex.input
    rex.set("input", lua.create_function(|_, prompt: String| {
        println!("{}", prompt);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| {
            mlua::Error::RuntimeError(format!("Input error: {}", e))
        })?;
        Ok(input.trim().to_string())
    })?)?;
    
    // Register rex.json module
    let json = lua.create_table()?;
    
    json.set("parse", lua.create_function(|lua, json_str: String| {
        let value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            mlua::Error::RuntimeError(format!("JSON parse error: {}", e))
        })?;
        json_value_to_lua(lua, &value)
    })?)?;
    
    json.set("stringify", lua.create_function(|_, value: Value| {
        let json_value = lua_value_to_json(value)?;
        serde_json::to_string(&json_value).map_err(|e| {
            mlua::Error::RuntimeError(format!("JSON stringify error: {}", e))
        })
    })?)?;
    
    rex.set("json", json)?;
    
    // Register rex.table module
    let table = lua.create_table()?;
    
    table.set("length", lua.create_function(|_, t: Table| {
        Ok(t.len().unwrap_or(0))
    })?)?;
    
    table.set("keys", lua.create_function(|lua, t: Table| {
        let keys = lua.create_table()?;
        let mut i = 1;
        for pair in t.clone().pairs::<Value, Value>() {
            let (key, _) = pair?;
            keys.set(i, key)?;
            i += 1;
        }
        Ok(keys)
    })?)?;
    
    rex.set("table", table)?;
    
    // Set rex as global
    lua.globals().set("rex", rex)?;
    
    Ok(())
}

/// Convert serde_json::Value to Lua Value.
fn json_value_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> LuaResult<Value<'lua>> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => lua.create_string(s).map(Value::String),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                table.set(i + 1, json_value_to_lua(lua, val)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                table.set(key.clone(), json_value_to_lua(lua, val)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

/// Convert Lua Value to serde_json::Value.
fn lua_value_to_json(value: Value) -> Result<serde_json::Value, mlua::Error> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
        Value::Number(n) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
        )),
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        Value::Table(t) => {
            // Try to convert as array first
            let mut arr = Vec::new();
            let mut i = 1;
            loop {
                match t.get::<_, Value>(i) {
                    Ok(val) => {
                        arr.push(lua_value_to_json(val)?);
                        i += 1;
                    }
                    Err(_) => break,
                }
            }
            
            if !arr.is_empty() {
                Ok(serde_json::Value::Array(arr))
            } else {
                // Convert as object
                let mut obj = serde_json::Map::new();
                for pair in t.clone().pairs::<String, Value>() {
                    let (key, val) = pair?;
                    obj.insert(key, lua_value_to_json(val)?);
                }
                Ok(serde_json::Value::Object(obj))
            }
        }
        _ => Err(mlua::Error::RuntimeError("Unsupported Lua type".to_string())),
    }
}