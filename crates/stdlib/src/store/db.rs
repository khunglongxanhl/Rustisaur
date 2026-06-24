//! SQLite database module for persistent storage and complex queries
//!
//! Features:
//! - 🐘 SQLite embedded: Không cần server riêng
//! - 🔒 ACID transactions: Đảm bảo data nhất quán
//! - 📊 Complex queries: WHERE, JOIN, GROUP BY, ORDER BY
//! - 💾 Persistent: Data tồn tại mãi mãi
//! - 🧵 Thread-safe: Hoạt động tốt với async/multi-thread

use mlua::{Lua, Result as LuaResult, Table, Value};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Database wrapper - Thread-safe SQLite connection
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Mở hoặc tạo database mới
    pub fn open(path: &str) -> LuaResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to open database: {}", e)))?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to set WAL mode: {}", e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Execute SQL query (INSERT, UPDATE, DELETE, CREATE TABLE, etc.)
    pub fn execute(&self, sql: &str, params: Vec<Value>) -> LuaResult<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock database: {}", e)))?;

        let params_ref: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|v| lua_value_to_sql(v))
            .collect::<LuaResult<Vec<_>>>()?
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_ref.iter().map(|p| p.as_ref()).collect();

        conn.execute(sql, param_refs.as_slice())
            .map_err(|e| mlua::Error::RuntimeError(format!("SQL execute error: {}", e)))
    }

    /// Query SQL và trả về results dạng Vec<(column_names, Vec<Value>)>
    pub fn query_raw(
        &self,
        sql: &str,
        params: Vec<Value>,
    ) -> LuaResult<Vec<(Vec<String>, Vec<rusqlite::types::Value>)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock database: {}", e)))?;

        let params_ref: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|v| lua_value_to_sql(v))
            .collect::<LuaResult<Vec<_>>>()?
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_ref.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| mlua::Error::RuntimeError(format!("SQL prepare error: {}", e)))?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut values = Vec::new();
                for i in 0..column_count {
                    let value: rusqlite::types::Value = row.get(i)?;
                    values.push(value);
                }
                Ok(values)
            })
            .map_err(|e| mlua::Error::RuntimeError(format!("SQL query error: {}", e)))?;

        // Collect results
        let mut results = Vec::new();
        for row in rows {
            let values =
                row.map_err(|e| mlua::Error::RuntimeError(format!("Failed to read row: {}", e)))?;
            results.push((column_names.clone(), values));
        }

        Ok(results)
    }

    /// Bắt đầu transaction
    pub fn begin_transaction(&self) -> LuaResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock database: {}", e)))?;
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| mlua::Error::RuntimeError(format!("Begin transaction error: {}", e)))?;
        Ok(())
    }

    /// Commit transaction
    pub fn commit(&self) -> LuaResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock database: {}", e)))?;
        conn.execute("COMMIT", [])
            .map_err(|e| mlua::Error::RuntimeError(format!("Commit error: {}", e)))?;
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(&self) -> LuaResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock database: {}", e)))?;
        conn.execute("ROLLBACK", [])
            .map_err(|e| mlua::Error::RuntimeError(format!("Rollback error: {}", e)))?;
        Ok(())
    }

    /// Close database
    pub fn close(&self) -> LuaResult<()> {
        // Connection will be closed when dropped
        Ok(())
    }
}

/// Convert Lua Value to SQLite Value
fn lua_value_to_sql(value: &Value) -> LuaResult<rusqlite::types::Value> {
    match value {
        Value::Nil => Ok(rusqlite::types::Value::Null),
        Value::Boolean(b) => Ok(rusqlite::types::Value::Integer(if *b { 1 } else { 0 })),
        Value::Integer(i) => Ok(rusqlite::types::Value::Integer(*i)),
        Value::Number(n) => Ok(rusqlite::types::Value::Real(*n)),
        Value::String(s) => Ok(rusqlite::types::Value::Text(s.to_str()?.to_string())),
        _ => Err(mlua::Error::RuntimeError(
            "Unsupported type for SQL parameter".to_string(),
        )),
    }
}

/// Convert SQLite Value to Lua Value
fn sql_value_to_lua<'lua>(
    lua: &'lua Lua,
    value: &rusqlite::types::Value,
) -> LuaResult<Value<'lua>> {
    match value {
        rusqlite::types::Value::Null => Ok(Value::Nil),
        rusqlite::types::Value::Integer(i) => Ok(Value::Integer(*i)),
        rusqlite::types::Value::Real(f) => Ok(Value::Number(*f)),
        rusqlite::types::Value::Text(s) => Ok(Value::String(lua.create_string(s)?)),
        rusqlite::types::Value::Blob(b) => {
            // Convert blob to string (base64 or hex)
            let hex: String = b.iter().map(|byte| format!("{:02x}", byte)).collect();
            Ok(Value::String(lua.create_string(&hex)?))
        }
    }
}

/// Tạo module `rex.store.db` cho Lua
pub fn create_db_module(lua: &Lua, db: Database) -> LuaResult<Table<'_>> {
    let db_table = lua.create_table()?;

    // rex.store.db.execute(sql, params?)
    db_table.set(
        "execute",
        lua.create_function({
            let db = db.clone();
            move |_, (sql, params): (String, Option<Table>)| {
                let params_vec = if let Some(t) = params {
                    let mut vec = Vec::new();
                    for pair in t.pairs::<i64, Value>() {
                        let (_, value) = pair?;
                        vec.push(value);
                    }
                    vec
                } else {
                    Vec::new()
                };
                db.execute(&sql, params_vec)
            }
        })?,
    )?;

    // rex.store.db.query(sql, params?)
    db_table.set(
        "query",
        lua.create_function({
            let db = db.clone();
            move |lua, (sql, params): (String, Option<Table>)| {
                let params_vec = if let Some(t) = params {
                    let mut vec = Vec::new();
                    for pair in t.pairs::<i64, Value>() {
                        let (_, value) = pair?;
                        vec.push(value);
                    }
                    vec
                } else {
                    Vec::new()
                };

                let results = db.query_raw(&sql, params_vec)?;
                let result_table = lua.create_table()?;

                for (i, (names, values)) in results.into_iter().enumerate() {
                    let row = lua.create_table()?;
                    for (name, value) in names.iter().zip(values.iter()) {
                        let lua_value = sql_value_to_lua(lua, value)?;
                        row.set(name.as_str(), lua_value)?;
                    }
                    result_table.set(i + 1, row)?;
                }

                Ok(result_table)
            }
        })?,
    )?;

    // rex.store.db.query_one(sql, params?)
    db_table.set(
        "query_one",
        lua.create_function({
            let db = db.clone();
            move |lua, (sql, params): (String, Option<Table>)| {
                let params_vec = if let Some(t) = params {
                    let mut vec = Vec::new();
                    for pair in t.pairs::<i64, Value>() {
                        let (_, value) = pair?;
                        vec.push(value);
                    }
                    vec
                } else {
                    Vec::new()
                };

                let results = db.query_raw(&sql, params_vec)?;

                if results.is_empty() {
                    Ok(Value::Nil)
                } else {
                    let (names, values) = &results[0];
                    let row = lua.create_table()?;
                    for (name, value) in names.iter().zip(values.iter()) {
                        let lua_value = sql_value_to_lua(lua, value)?;
                        row.set(name.as_str(), lua_value)?;
                    }
                    Ok(Value::Table(row))
                }
            }
        })?,
    )?;

    // rex.store.db.begin()
    db_table.set(
        "begin",
        lua.create_function({
            let db = db.clone();
            move |_, ()| db.begin_transaction()
        })?,
    )?;

    // rex.store.db.commit()
    db_table.set(
        "commit",
        lua.create_function({
            let db = db.clone();
            move |_, ()| db.commit()
        })?,
    )?;

    // rex.store.db.rollback()
    db_table.set(
        "rollback",
        lua.create_function({
            let db = db.clone();
            move |_, ()| db.rollback()
        })?,
    )?;

    // rex.store.db.close()
    db_table.set(
        "close",
        lua.create_function({
            let db = db.clone();
            move |_, ()| db.close()
        })?,
    )?;

    Ok(db_table)
}
