//! Lua bindings for the standard library.

use mlua::{Lua, Result as LuaResult, Table, Value, Variadic};
use serde_json;

use crate::error::StdlibError;

/// Register all standard library functions into the Lua state.
pub fn register_all(lua: &Lua) -> Result<(), StdlibError> {
    let rex = lua.create_table()?;

    // ========================================
    // rex.print - Print output
    // ========================================
    rex.set(
        "print",
        lua.create_function(|_, msg: String| {
            println!("{}", msg);
            Ok(())
        })?,
    )?;

    // ========================================
    // rex.input - Read input
    // ========================================
    rex.set(
        "input",
        lua.create_function(|_, prompt: String| {
            println!("{}", prompt);
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| mlua::Error::RuntimeError(format!("Input error: {}", e)))?;
            Ok(input.trim().to_string())
        })?,
    )?;

    // ========================================
    // rex.json - JSON operations
    // ========================================
    let json = lua.create_table()?;

    json.set(
        "parse",
        lua.create_function(|lua, json_str: String| {
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;
            json_value_to_lua(lua, &value)
        })?,
    )?;

    json.set(
        "stringify",
        lua.create_function(|_, value: Value| {
            let json_value = lua_value_to_json(value)?;
            serde_json::to_string(&json_value)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON stringify error: {}", e)))
        })?,
    )?;

    rex.set("json", json)?;

    // ========================================
    // rex.table - Table operations (ENHANCED)
    // ========================================
    let table = lua.create_table()?;

    // Get table length
    table.set(
        "length",
        lua.create_function(|_, t: Table| Ok(t.len().unwrap_or(0)))?,
    )?;

    // Get all keys
    table.set(
        "keys",
        lua.create_function(|lua, t: Table| {
            let keys = lua.create_table()?;
            for (i, pair) in t.clone().pairs::<Value, Value>().enumerate() {
                let (key, _) = pair?;
                keys.set(i + 1, key)?;
            }
            Ok(keys)
        })?,
    )?;

    // Get all values
    table.set(
        "values",
        lua.create_function(|lua, t: Table| {
            let values = lua.create_table()?;
            for (i, pair) in t.clone().pairs::<Value, Value>().enumerate() {
                let (_, value) = pair?;
                values.set(i + 1, value)?;
            }
            Ok(values)
        })?,
    )?;

    // Merge two tables
    table.set(
        "merge",
        lua.create_function(|lua, (t1, t2): (Table, Table)| {
            let result = lua.create_table()?;
            for pair in t1.clone().pairs::<Value, Value>() {
                let (key, value) = pair?;
                result.set(key, value)?;
            }
            for pair in t2.clone().pairs::<Value, Value>() {
                let (key, value) = pair?;
                result.set(key, value)?;
            }
            Ok(result)
        })?,
    )?;

    // Filter table by predicate
    table.set(
        "filter",
        lua.create_function(|lua, (t, func): (Table, mlua::Function)| {
            let result = lua.create_table()?;
            let mut index = 1;
            for pair in t.clone().pairs::<Value, Value>() {
                let (_, value) = pair?;
                let keep: bool = func.call(value.clone())?;
                if keep {
                    result.set(index, value)?;
                    index += 1;
                }
            }
            Ok(result)
        })?,
    )?;

    // Map table values
    table.set(
        "map",
        lua.create_function(|lua, (t, func): (Table, mlua::Function)| {
            let result = lua.create_table()?;
            for pair in t.clone().pairs::<i64, Value>() {
                let (key, value) = pair?;
                let mapped: Value = func.call(value)?;
                result.set(key, mapped)?;
            }
            Ok(result)
        })?,
    )?;

    // Check if table contains value
    table.set(
        "contains",
        lua.create_function(|_, (t, value): (Table, Value)| {
            for pair in t.clone().pairs::<Value, Value>() {
                let (_, v) = pair?;
                if v == value {
                    return Ok(true);
                }
            }
            Ok(false)
        })?,
    )?;

    // Reverse table
    table.set(
        "reverse",
        lua.create_function(|lua, t: Table| {
            let result = lua.create_table()?;
            let len = t.len().unwrap_or(0);
            for i in 1..=len {
                let value: Value = t.get(len - i + 1)?;
                result.set(i, value)?;
            }
            Ok(result)
        })?,
    )?;

    // Sort table
    table.set(
        "sort",
        lua.create_function(|lua, t: Table| {
            let mut values: Vec<Value> = t.sequence_values().filter_map(|v| v.ok()).collect();
            values.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Number(x), Value::Number(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => {
                    let xs = x.to_str().unwrap_or("");
                    let ys = y.to_str().unwrap_or("");
                    xs.cmp(ys)
                }
                _ => std::cmp::Ordering::Equal,
            });
            let result = lua.create_table()?;
            for (i, value) in values.into_iter().enumerate() {
                result.set(i + 1, value)?;
            }
            Ok(result)
        })?,
    )?;

    // Get unique values
    table.set(
        "unique",
        lua.create_function(|lua, t: Table| {
            let result = lua.create_table()?;
            let mut seen = Vec::new();
            let mut index = 1;
            for pair in t.clone().pairs::<Value, Value>() {
                let (_, value) = pair?;
                if !seen.contains(&value) {
                    seen.push(value.clone());
                    result.set(index, value)?;
                    index += 1;
                }
            }
            Ok(result)
        })?,
    )?;

    rex.set("table", table)?;

    // ========================================
    // rex.fs - File system operations (ENHANCED)
    // ========================================
    let fs = lua.create_table()?;

    // Read file
    fs.set(
        "read",
        lua.create_function(|_, path: String| {
            std::fs::read_to_string(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("File read error: {}", e)))
        })?,
    )?;

    // Write file
    fs.set(
        "write",
        lua.create_function(|_, (path, content): (String, String)| {
            std::fs::write(&path, content)
                .map_err(|e| mlua::Error::RuntimeError(format!("File write error: {}", e)))
        })?,
    )?;

    // Check if file/directory exists
    fs.set(
        "exists",
        lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).exists()))?,
    )?;

    // Check if path is a file
    fs.set(
        "is_file",
        lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).is_file()))?,
    )?;

    // Check if path is a directory
    fs.set(
        "is_dir",
        lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).is_dir()))?,
    )?;

    // Delete file or empty directory
    fs.set(
        "delete",
        lua.create_function(|_, path: String| {
            let path = std::path::Path::new(&path);
            if path.is_dir() {
                std::fs::remove_dir(path).map_err(|e| {
                    mlua::Error::RuntimeError(format!("Directory delete error: {}", e))
                })
            } else {
                std::fs::remove_file(path)
                    .map_err(|e| mlua::Error::RuntimeError(format!("File delete error: {}", e)))
            }
        })?,
    )?;

    // Create directory
    fs.set(
        "mkdir",
        lua.create_function(|_, path: String| {
            std::fs::create_dir(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("Directory create error: {}", e)))
        })?,
    )?;

    // Create directory and all parent directories
    fs.set(
        "mkdir_all",
        lua.create_function(|_, path: String| {
            std::fs::create_dir_all(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("Directory create error: {}", e)))
        })?,
    )?;

    // List directory contents
    fs.set(
        "list",
        lua.create_function(|lua, path: String| {
            let entries = std::fs::read_dir(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("Directory list error: {}", e)))?;

            let table = lua.create_table()?;

            for (index, entry) in entries.flatten().enumerate() {
                let name = entry.file_name().to_string_lossy().to_string();
                table.set(index + 1, name)?; // Lua arrays start at 1
            }

            Ok(table)
        })?,
    )?;

    // Rename/move file or directory
    fs.set(
        "rename",
        lua.create_function(|_, (from, to): (String, String)| {
            std::fs::rename(&from, &to)
                .map_err(|e| mlua::Error::RuntimeError(format!("Rename error: {}", e)))
        })?,
    )?;

    // Copy file
    fs.set(
        "copy",
        lua.create_function(|_, (from, to): (String, String)| {
            std::fs::copy(&from, &to)
                .map_err(|e| mlua::Error::RuntimeError(format!("Copy error: {}", e)))?;
            Ok(())
        })?,
    )?;

    // Get file metadata
    fs.set(
        "metadata",
        lua.create_function(|lua, path: String| {
            let metadata = std::fs::metadata(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("Metadata error: {}", e)))?;
            let table = lua.create_table()?;
            table.set("size", metadata.len())?;
            table.set("is_file", metadata.is_file())?;
            table.set("is_dir", metadata.is_dir())?;
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    table.set("modified", duration.as_secs())?;
                }
            }
            Ok(table)
        })?,
    )?;

    // Get absolute path
    fs.set(
        "absolute",
        lua.create_function(|_, path: String| {
            let abs_path = std::fs::canonicalize(&path)
                .map_err(|e| mlua::Error::RuntimeError(format!("Absolute path error: {}", e)))?;
            Ok(abs_path.to_string_lossy().to_string())
        })?,
    )?;

    // Get file extension
    fs.set(
        "extension",
        lua.create_function(|_, path: String| {
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            Ok(ext)
        })?,
    )?;

    // Get file name
    fs.set(
        "file_name",
        lua.create_function(|_, path: String| {
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            Ok(name)
        })?,
    )?;

    // Get parent directory
    fs.set(
        "parent",
        lua.create_function(|_, path: String| {
            let parent = std::path::Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            Ok(parent)
        })?,
    )?;

    rex.set("fs", fs)?;

    // ========================================
    // rex.string - String operations (ENHANCED)
    // ========================================
    let string = lua.create_table()?;

    // Basic transformations
    string.set(
        "upper",
        lua.create_function(|_, s: String| Ok(s.to_uppercase()))?,
    )?;

    string.set(
        "lower",
        lua.create_function(|_, s: String| Ok(s.to_lowercase()))?,
    )?;

    string.set(
        "trim",
        lua.create_function(|_, s: String| Ok(s.trim().to_string()))?,
    )?;

    string.set(
        "trim_left",
        lua.create_function(|_, s: String| Ok(s.trim_start().to_string()))?,
    )?;

    string.set(
        "trim_right",
        lua.create_function(|_, s: String| Ok(s.trim_end().to_string()))?,
    )?;

    // Split and join
    string.set(
        "split",
        lua.create_function(|lua, (s, delim): (String, String)| {
            let parts = s.split(&delim).collect::<Vec<&str>>();
            let table = lua.create_table()?;
            for (i, part) in parts.iter().enumerate() {
                table.set(i + 1, *part)?;
            }
            Ok(table)
        })?,
    )?;

    string.set(
        "join",
        lua.create_function(|_, (table, delim): (Table, String)| {
            let mut parts = Vec::new();
            for pair in table.pairs::<i64, String>() {
                let (_, value) = pair?;
                parts.push(value);
            }
            Ok(parts.join(&delim))
        })?,
    )?;

    // Replace
    string.set(
        "replace",
        lua.create_function(|_, (s, from, to): (String, String, String)| {
            Ok(s.replacen(&from, &to, 1))
        })?,
    )?;

    string.set(
        "replace_all",
        lua.create_function(
            |_, (s, from, to): (String, String, String)| Ok(s.replace(&from, &to)),
        )?,
    )?;

    // Check functions
    string.set(
        "starts_with",
        lua.create_function(|_, (s, prefix): (String, String)| Ok(s.starts_with(&prefix)))?,
    )?;

    string.set(
        "ends_with",
        lua.create_function(|_, (s, suffix): (String, String)| Ok(s.ends_with(&suffix)))?,
    )?;

    string.set(
        "contains",
        lua.create_function(|_, (s, pattern): (String, String)| Ok(s.contains(&pattern)))?,
    )?;

    // Transform functions
    string.set(
        "capitalize",
        lua.create_function(|_, s: String| {
            let mut chars = s.chars();
            match chars.next() {
                None => Ok(String::new()),
                Some(c) => {
                    let capitalized = c.to_uppercase().to_string();
                    let rest: String = chars.collect::<String>().to_lowercase();
                    Ok(capitalized + &rest)
                }
            }
        })?,
    )?;

    string.set(
        "repeat",
        lua.create_function(|_, (s, count): (String, usize)| Ok(s.repeat(count)))?,
    )?;

    string.set(
        "slice",
        lua.create_function(|_, (s, start, end): (String, usize, usize)| {
            let sliced: String = s
                .chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect();
            Ok(sliced)
        })?,
    )?;

    string.set(
        "reverse",
        lua.create_function(|_, s: String| Ok(s.chars().rev().collect::<String>()))?,
    )?;

    // Pad functions
    string.set(
        "pad_left",
        lua.create_function(|_, (s, width, ch): (String, usize, String)| {
            let pad_char = ch.chars().next().unwrap_or(' ');
            let current_len = s.chars().count();
            if current_len >= width {
                Ok(s)
            } else {
                let padding = pad_char.to_string().repeat(width - current_len);
                Ok(padding + &s)
            }
        })?,
    )?;

    string.set(
        "pad_right",
        lua.create_function(|_, (s, width, ch): (String, usize, String)| {
            let pad_char = ch.chars().next().unwrap_or(' ');
            let current_len = s.chars().count();
            if current_len >= width {
                Ok(s)
            } else {
                let padding = pad_char.to_string().repeat(width - current_len);
                Ok(s + &padding)
            }
        })?,
    )?;

    // Length and check functions
    string.set(
        "len",
        lua.create_function(|_, s: String| Ok(s.chars().count()))?,
    )?;

    string.set(
        "is_empty",
        lua.create_function(|_, s: String| Ok(s.trim().is_empty()))?,
    )?;

    rex.set("string", string)?;

    // ========================================
    // rex.math - Math operations (ENHANCED)
    // ========================================
    let math = lua.create_table()?;

    // Max value
    math.set(
        "max",
        lua.create_function(|_, nums: Variadic<f64>| {
            Ok(nums.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        })?,
    )?;

    // Min value
    math.set(
        "min",
        lua.create_function(|_, nums: Variadic<f64>| {
            Ok(nums.iter().copied().fold(f64::INFINITY, f64::min))
        })?,
    )?;

    // Absolute value
    math.set("abs", lua.create_function(|_, n: f64| Ok(n.abs()))?)?;

    // Round to nearest integer
    math.set("round", lua.create_function(|_, n: f64| Ok(n.round()))?)?;

    // Floor (round down)
    math.set("floor", lua.create_function(|_, n: f64| Ok(n.floor()))?)?;

    // Ceil (round up)
    math.set("ceil", lua.create_function(|_, n: f64| Ok(n.ceil()))?)?;

    // Power
    math.set(
        "pow",
        lua.create_function(|_, (base, exp): (f64, f64)| Ok(base.powf(exp)))?,
    )?;

    // Square root
    math.set("sqrt", lua.create_function(|_, n: f64| Ok(n.sqrt()))?)?;

    // Random number between min and max
    math.set(
        "random",
        lua.create_function(|_, (min, max): (f64, f64)| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as f64;
            let random = (seed.sin() * 10000.0).fract().abs();
            Ok(min + random * (max - min))
        })?,
    )?;

    // Pi constant
    math.set("pi", lua.create_function(|_, ()| Ok(std::f64::consts::PI))?)?;

    // E constant
    math.set("e", lua.create_function(|_, ()| Ok(std::f64::consts::E))?)?;

    // Sum of numbers
    math.set(
        "sum",
        lua.create_function(|_, nums: Variadic<f64>| Ok(nums.iter().sum::<f64>()))?,
    )?;

    rex.set("math", math)?;

    // ========================================
    // rex.os - OS operations (NEW)
    // ========================================
    let os = lua.create_table()?;

    // Get current timestamp
    os.set(
        "time",
        lua.create_function(|_, ()| {
            let duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            Ok(duration.as_secs() as f64)
        })?,
    )?;

    // Sleep for milliseconds
    os.set(
        "sleep",
        lua.create_function(|_, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?,
    )?;

    // Get environment variable
    os.set(
        "env",
        lua.create_function(|_, name: String| Ok(std::env::var(&name).unwrap_or_default()))?,
    )?;

    // Get current working directory
    os.set(
        "cwd",
        lua.create_function(|_, ()| {
            let cwd = std::env::current_dir()
                .map_err(|e| mlua::Error::RuntimeError(format!("CWD error: {}", e)))?;
            Ok(cwd.to_string_lossy().to_string())
        })?,
    )?;

    // Get command line arguments
    os.set(
        "args",
        lua.create_function(|lua, ()| {
            let table = lua.create_table()?;
            for (i, arg) in std::env::args().enumerate() {
                table.set(i + 1, arg)?;
            }
            Ok(table)
        })?,
    )?;

    rex.set("os", os)?;

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
    lua_value_to_json_with_depth(value, 0, 100)
}

fn lua_value_to_json_with_depth(
    value: Value,
    depth: usize,
    max_depth: usize,
) -> Result<serde_json::Value, mlua::Error> {
    if depth > max_depth {
        return Err(mlua::Error::RuntimeError(
            "Maximum nesting depth exceeded".to_string(),
        ));
    }

    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
        Value::Number(n) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
        )),
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        Value::Table(t) => {
            // First, check if it's a pure array (sequential integer keys starting from 1)
            let len = t.len().unwrap_or(0);
            if len > 0 {
                let mut arr = Vec::with_capacity(len as usize);
                let mut is_pure_array = true;

                for i in 1..=len {
                    match t.get::<_, Value>(i) {
                        Ok(val) => {
                            arr.push(lua_value_to_json_with_depth(val, depth + 1, max_depth)?);
                        }
                        Err(_) => {
                            is_pure_array = false;
                            break;
                        }
                    }
                }

                if is_pure_array {
                    return Ok(serde_json::Value::Array(arr));
                }
            }

            // Convert as object
            let mut obj = serde_json::Map::new();
            for pair in t.clone().pairs::<Value, Value>() {
                let (key, val) = pair?;
                let key_str = match key {
                    Value::String(s) => s.to_str()?.to_string(),
                    Value::Integer(i) => i.to_string(),
                    _ => continue,
                };
                obj.insert(
                    key_str,
                    lua_value_to_json_with_depth(val, depth + 1, max_depth)?,
                );
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err(mlua::Error::RuntimeError(
            "Unsupported Lua type".to_string(),
        )),
    }
}
