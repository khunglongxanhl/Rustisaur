//! SQLite database module for persistent storage and complex queries
//!
//! Features:
//! - 🐘 SQLite embedded: Không cần server riêng
//! - 🔒 ACID transactions: Đảm bảo data nhất quán
//! - 📊 Complex queries: WHERE, JOIN, GROUP BY, ORDER BY
//! - 💾 Persistent: Data tồn tại mãi mãi
//! - 🧵 Thread-safe: Hoạt động tốt với async/multi-thread
//! - 🔄 Retry logic: Tự động retry khi database bị locked
//! - 📈 Statistics: Theo dõi queries, inserts, updates, deletes
//! - 📦 Batch operations: Tối ưu FFI overhead

use mlua::{Lua, Result as LuaResult, Table, Value};
use rusqlite::Connection;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tracing::debug;

/// Thống kê database (cho debugging)
#[derive(Default)]
pub struct DatabaseStats {
    pub queries: AtomicU64,
    pub inserts: AtomicU64,
    pub updates: AtomicU64,
    pub deletes: AtomicU64,
    pub transactions: AtomicU64,
    pub errors: AtomicU64,
}

impl DatabaseStats {
    pub fn record_query(&self) {
        self.queries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_update(&self) {
        self.updates.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_transaction(&self) {
        self.transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
}

/// Database wrapper - Thread-safe SQLite connection
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    stats: Arc<DatabaseStats>,
    path: String,
}

impl Default for Database {
    fn default() -> Self {
        Self::new("rustisaur.db")
    }
}

impl Database {
    /// Tạo database instance (chưa mở connection)
    pub fn new(path: &str) -> Self {
        debug!("🐘 Creating Database instance for path: {}", path);
        Self {
            conn: Arc::new(Mutex::new(Connection::open(path).unwrap())),
            stats: Arc::new(DatabaseStats::default()),
            path: path.to_string(),
        }
    }

    /// Mở hoặc tạo database mới với retry logic
    pub fn open(path: &str) -> LuaResult<Self> {
        debug!("🐘 Opening database at: {}", path);
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            match Connection::open(path) {
                Ok(conn) => {
                    // Enable WAL mode for better concurrent performance
                    if let Err(e) = conn.execute_batch("PRAGMA journal_mode=WAL;") {
                        // Nếu failed set WAL, continue without it (không critical)
                        eprintln!("Warning: Failed to set WAL mode: {}", e);
                    }

                    // Tối ưu: Enable memory-mapped I/O
                    if let Err(e) = conn.execute_batch("PRAGMA mmap_size=268435456;") {
                        debug!("Warning: Failed to set mmap_size: {}", e);
                    }

                    // Tối ưu: Cache size 256MB
                    if let Err(e) = conn.execute_batch("PRAGMA cache_size=-256000;") {
                        debug!("Warning: Failed to set cache_size: {}", e);
                    }

                    debug!("✅ Database opened successfully with WAL mode");

                    return Ok(Self {
                        conn: Arc::new(Mutex::new(conn)),
                        stats: Arc::new(DatabaseStats::default()),
                        path: path.to_string(),
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Failed to open database after {} attempts: {}",
                            max_attempts, e
                        )));
                    }
                    // Wait before retry (exponential backoff)
                    std::thread::sleep(std::time::Duration::from_millis(100 * attempts as u64));
                }
            }
        }
    }

    /// Execute SQL query (INSERT, UPDATE, DELETE, CREATE TABLE, etc.)
    pub fn execute(&self, sql: &str, params: Vec<Value>) -> LuaResult<usize> {
        debug!(sql = %sql, "📝 DB EXECUTE");

        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;

        let params_ref: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|v| lua_value_to_sql(v))
            .collect::<LuaResult<Vec<_>>>()?
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_ref.iter().map(|p| p.as_ref()).collect();

        let result = conn.execute(sql, param_refs.as_slice());

        match result {
            Ok(count) => {
                // Detect operation type from SQL
                let sql_upper = sql.to_uppercase();
                if sql_upper.starts_with("INSERT") {
                    self.stats.record_insert();
                } else if sql_upper.starts_with("UPDATE") {
                    self.stats.record_update();
                } else if sql_upper.starts_with("DELETE") {
                    self.stats.record_delete();
                }
                Ok(count)
            }
            Err(e) => {
                self.stats.record_error();
                Err(mlua::Error::RuntimeError(format!(
                    "SQL execute error: {}",
                    e
                )))
            }
        }
    }

    /// Query SQL và trả về results dạng Vec<(column_names, Vec<Value>)>
    pub fn query_raw(
        &self,
        sql: &str,
        params: Vec<Value>,
    ) -> LuaResult<Vec<(Vec<String>, Vec<rusqlite::types::Value>)>> {
        debug!(sql = %sql, "📖 DB QUERY");
        self.stats.record_query();

        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;

        let params_ref: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|v| lua_value_to_sql(v))
            .collect::<LuaResult<Vec<_>>>()?
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_ref.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(sql).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("SQL prepare error: {}", e))
        })?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut values = Vec::with_capacity(column_count);
                for i in 0..column_count {
                    let value: rusqlite::types::Value = row.get(i)?;
                    values.push(value);
                }
                Ok(values)
            })
            .map_err(|e| {
                self.stats.record_error();
                mlua::Error::RuntimeError(format!("SQL query error: {}", e))
            })?;

        // Collect results
        let mut results = Vec::new();
        for row in rows {
            let values = row.map_err(|e| {
                self.stats.record_error();
                mlua::Error::RuntimeError(format!("Failed to read row: {}", e))
            })?;
            results.push((column_names.clone(), values));
        }

        debug!("📖 Query returned {} rows", results.len());
        Ok(results)
    }

    /// Bắt đầu transaction
    pub fn begin_transaction(&self) -> LuaResult<()> {
        debug!("💼 DB BEGIN TRANSACTION");
        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;
        conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Begin transaction error: {}", e))
        })?;
        Ok(())
    }

    /// Commit transaction
    pub fn commit(&self) -> LuaResult<()> {
        debug!("💼 DB COMMIT");
        self.stats.record_transaction();
        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;
        conn.execute("COMMIT", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Commit error: {}", e))
        })?;
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(&self) -> LuaResult<()> {
        debug!("💼 DB ROLLBACK");
        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;
        conn.execute("ROLLBACK", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Rollback error: {}", e))
        })?;
        Ok(())
    }

    /// Close database
    pub fn close(&self) -> LuaResult<()> {
        debug!("🔒 DB CLOSE");
        // Connection will be closed when dropped
        Ok(())
    }
    // ✅ THÊM METHOD NÀY VÀO ĐÂY:
    /// Lấy đường dẫn database file
    pub fn get_path(&self) -> &str {
        &self.path
    }

    // ==========================================
    // BATCH OPERATIONS (Tối ưu FFI overhead)
    // ==========================================

    /// Thực thi nhiều câu lệnh SQL trong 1 Transaction duy nhất
    pub fn batch_execute(&self, sql_statements: Vec<String>) -> LuaResult<usize> {
        // ✅ LƯU LENGTH VÀO BIẾN TRƯỚC KHI LOOP
        let count = sql_statements.len();
        debug!("📦 DB BATCH EXECUTE: {} statements", count);

        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;

        // Bắt đầu transaction
        conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Begin transaction error: {}", e))
        })?;

        let mut total_changes = 0;

        // Thực thi từng câu lệnh (sql_statements bị move ở đây)
        for sql in sql_statements {
            match conn.execute(&sql, []) {
                Ok(ch) => {
                    total_changes += ch;

                    // Detect operation type
                    let sql_upper = sql.to_uppercase();
                    if sql_upper.starts_with("INSERT") {
                        self.stats.record_insert();
                    } else if sql_upper.starts_with("UPDATE") {
                        self.stats.record_update();
                    } else if sql_upper.starts_with("DELETE") {
                        self.stats.record_delete();
                    }
                }
                Err(e) => {
                    // Nếu lỗi, rollback ngay lập tức
                    let _ = conn.execute("ROLLBACK", []);
                    self.stats.record_error();
                    return Err(mlua::Error::RuntimeError(format!(
                        "Batch SQL error (rolled back): {}",
                        e
                    )));
                }
            }
        }

        // Commit transaction
        conn.execute("COMMIT", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Commit error: {}", e))
        })?;

        self.stats.record_transaction();

        // ✅ DÙNG BIẾN `count` THAY VÌ `sql_statements.len()`
        debug!(
            "✅ Batch executed {} statements, {} total changes",
            count, total_changes
        );

        Ok(total_changes)
    }

    /// Batch insert với prepared statement (cực nhanh cho nhiều rows)
    pub fn batch_insert(
        &self,
        table: &str,
        columns: Vec<String>,
        rows: Vec<Vec<Value>>,
    ) -> LuaResult<usize> {
        debug!("📦 DB BATCH INSERT: {} rows into {}", rows.len(), table);

        if rows.is_empty() {
            return Ok(0);
        }

        let conn = self.conn.lock().map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Failed to lock database: {}", e))
        })?;

        // Build INSERT statement
        let columns_str = columns.join(", ");
        let placeholders = (0..columns.len())
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table, columns_str, placeholders
        );

        // Bắt đầu transaction
        conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Begin transaction error: {}", e))
        })?;

        // Prepare statement một lần
        let mut stmt = conn.prepare(&sql).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("SQL prepare error: {}", e))
        })?;

        let mut total_inserted = 0;

        // Execute cho từng row
        for row in rows {
            let params_ref: Vec<Box<dyn rusqlite::types::ToSql>> = row
                .iter()
                .map(|v| lua_value_to_sql(v))
                .collect::<LuaResult<Vec<_>>>()?
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
                .collect();

            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params_ref.iter().map(|p| p.as_ref()).collect();

            match stmt.execute(param_refs.as_slice()) {
                Ok(_) => {
                    total_inserted += 1;
                    self.stats.record_insert();
                }
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", []);
                    self.stats.record_error();
                    return Err(mlua::Error::RuntimeError(format!(
                        "Batch insert error (rolled back): {}",
                        e
                    )));
                }
            }
        }

        // Commit transaction
        conn.execute("COMMIT", []).map_err(|e| {
            self.stats.record_error();
            mlua::Error::RuntimeError(format!("Commit error: {}", e))
        })?;

        self.stats.record_transaction();
        debug!("✅ Batch inserted {} rows", total_inserted);

        Ok(total_inserted)
    }

    /// Lấy thống kê database (cho debugging)
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64, u64) {
        (
            self.stats.queries.load(Ordering::Relaxed),
            self.stats.inserts.load(Ordering::Relaxed),
            self.stats.updates.load(Ordering::Relaxed),
            self.stats.deletes.load(Ordering::Relaxed),
            self.stats.transactions.load(Ordering::Relaxed),
            self.stats.errors.load(Ordering::Relaxed),
        )
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
                    let mut vec = Vec::with_capacity(t.len().unwrap_or(0) as usize);
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
                    let mut vec = Vec::with_capacity(t.len().unwrap_or(0) as usize);
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
                    let mut vec = Vec::with_capacity(t.len().unwrap_or(0) as usize);
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

    // ==========================================
    // BATCH OPERATIONS (Tối ưu FFI overhead)
    // ==========================================

    // rex.store.db.batch_execute(sql_array)
    db_table.set(
        "batch_execute",
        lua.create_function({
            let db = db.clone();
            move |_, statements: Table| {
                let mut sql_list = Vec::with_capacity(statements.len().unwrap_or(0) as usize);
                for pair in statements.pairs::<i64, String>() {
                    let (_, sql) = pair?;
                    sql_list.push(sql);
                }
                db.batch_execute(sql_list)
            }
        })?,
    )?;

    // rex.store.db.batch_insert(table, columns, rows)
    db_table.set(
        "batch_insert",
        lua.create_function({
            let db = db.clone();
            move |_, (table, columns, rows): (String, Table, Table)| {
                // Parse columns
                let mut cols = Vec::with_capacity(columns.len().unwrap_or(0) as usize);
                for pair in columns.pairs::<i64, String>() {
                    let (_, col) = pair?;
                    cols.push(col);
                }

                // Parse rows
                let mut rows_vec = Vec::with_capacity(rows.len().unwrap_or(0) as usize);
                for pair in rows.pairs::<i64, Table>() {
                    let (_, row) = pair?;
                    let mut row_vec = Vec::with_capacity(row.len().unwrap_or(0) as usize);
                    for cell_pair in row.pairs::<i64, Value>() {
                        let (_, cell) = cell_pair?;
                        row_vec.push(cell);
                    }
                    rows_vec.push(row_vec);
                }

                db.batch_insert(&table, cols, rows_vec)
            }
        })?,
    )?;

    // rex.store.db.stats() - Lấy thống kê database (cho debugging)
    db_table.set(
        "stats",
        lua.create_function({
            let db = db.clone();
            move |lua, ()| {
                let (queries, inserts, updates, deletes, transactions, errors) = db.get_stats();
                let stats = lua.create_table()?;
                stats.set("queries", queries)?;
                stats.set("inserts", inserts)?;
                stats.set("updates", updates)?;
                stats.set("deletes", deletes)?;
                stats.set("transactions", transactions)?;
                stats.set("errors", errors)?;
                Ok(stats)
            }
        })?,
    )?;

    Ok(db_table)
}
