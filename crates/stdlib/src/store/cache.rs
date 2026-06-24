//! Redis-like in-memory cache with TTL support
//!
//! Features:
//! - ⚡ Tốc độ phản lực: Lưu trực tiếp trên RAM (DashMap)
//! - ⏰ TTL (Time To Live): Tự động xóa sau X giây
//! - 🧵 Thread-safe: Hoạt động tốt với async/multi-thread
//! - 📦 Hỗ trợ mọi kiểu dữ liệu Lua (string, number, table, boolean)

use dashmap::DashMap;
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Một entry trong cache, bao gồm value và thời gian hết hạn
#[derive(Clone)]
struct CacheEntry {
    value: CachedValue,
    expires_at: Option<Instant>,
}

/// Kiểu dữ liệu được lưu trong cache (tương thích với Lua)
#[derive(Clone)]
pub enum CachedValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Table(Vec<(CachedValue, CachedValue)>),
}

impl CachedValue {
    /// Chuyển từ Lua Value sang CachedValue
    fn from_lua(value: Value) -> LuaResult<Self> {
        match value {
            Value::String(s) => Ok(CachedValue::String(s.to_str()?.to_string())),
            Value::Number(n) => Ok(CachedValue::Number(n)),
            Value::Integer(i) => Ok(CachedValue::Number(i as f64)),
            Value::Boolean(b) => Ok(CachedValue::Boolean(b)),
            Value::Nil => Ok(CachedValue::Nil),
            Value::Table(t) => {
                let mut pairs = Vec::new();
                for pair in t.pairs::<Value, Value>() {
                    let (k, v) = pair?;
                    pairs.push((CachedValue::from_lua(k)?, CachedValue::from_lua(v)?));
                }
                Ok(CachedValue::Table(pairs))
            }
            _ => Err(mlua::Error::RuntimeError(
                "Unsupported type for cache".to_string(),
            )),
        }
    }

    /// Chuyển từ CachedValue sang Lua Value
    fn to_lua<'a>(&self, lua: &'a Lua) -> LuaResult<Value<'a>> {
        match self {
            CachedValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
            CachedValue::Number(n) => Ok(Value::Number(*n)),
            CachedValue::Boolean(b) => Ok(Value::Boolean(*b)),
            CachedValue::Nil => Ok(Value::Nil),
            CachedValue::Table(pairs) => {
                let table = lua.create_table()?;
                for (k, v) in pairs {
                    table.set(k.to_lua(lua)?, v.to_lua(lua)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }
}

/// Cache store - Trái tim của "tốc độ phản lực"
#[derive(Clone)]
pub struct CacheStore {
    inner: Arc<DashMap<String, CacheEntry>>,
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    /// Set value với TTL (time to live) tính bằng giây
    pub fn set(&self, key: String, value: CachedValue, ttl_seconds: Option<u64>) {
        let expires_at = ttl_seconds.map(|secs| Instant::now() + Duration::from_secs(secs));
        self.inner.insert(key, CacheEntry { value, expires_at });
    }

    /// Get value (trả về None nếu hết hạn hoặc không tồn tại)
    pub fn get(&self, key: &str) -> Option<CachedValue> {
        if let Some(entry) = self.inner.get(key) {
            // Kiểm tra TTL
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    // Đã hết hạn, xóa và trả về None
                    drop(entry);
                    self.inner.remove(key);
                    return None;
                }
            }
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Xóa một key
    pub fn delete(&self, key: &str) -> bool {
        self.inner.remove(key).is_some()
    }

    /// Kiểm tra key có tồn tại không
    pub fn exists(&self, key: &str) -> bool {
        if let Some(entry) = self.inner.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    drop(entry);
                    self.inner.remove(key);
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Xóa TẤT CẢ (như Redis FLUSHALL)
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Đếm số keys hiện có
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// Kiểm tra cache có rỗng không
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Lấy tất cả keys
    pub fn keys(&self) -> Vec<String> {
        self.inner.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Dọn dẹp các keys đã hết hạn (nên gọi định kỳ)
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .inner
            .iter()
            .filter_map(|entry| {
                if let Some(expires_at) = entry.expires_at {
                    if now > expires_at {
                        return Some(entry.key().clone());
                    }
                }
                None
            })
            .collect();

        for key in expired_keys {
            self.inner.remove(&key);
        }
    }

    /// Increment một số (giống Redis INCR)
    pub fn incr(&self, key: &str, delta: f64) -> LuaResult<f64> {
        let new_value = if let Some(entry) = self.inner.get(key) {
            match &entry.value {
                CachedValue::Number(n) => n + delta,
                CachedValue::Nil => delta,
                _ => {
                    return Err(mlua::Error::RuntimeError(
                        "Value is not a number".to_string(),
                    ))
                }
            }
        } else {
            delta
        };

        self.inner.insert(
            key.to_string(),
            CacheEntry {
                value: CachedValue::Number(new_value),
                expires_at: None,
            },
        );

        Ok(new_value)
    }
}

/// Tạo module `rex.store.cache` cho Lua
pub fn create_cache_module(lua: &Lua, cache: CacheStore) -> LuaResult<Table<'_>> {
    let cache_table = lua.create_table()?;

    // rex.store.cache.set(key, value, ttl_seconds?)
    cache_table.set(
        "set",
        lua.create_function({
            let cache = cache.clone();
            move |_lua, (key, value, ttl): (String, Value, Option<u64>)| {
                let cached_value = CachedValue::from_lua(value)?;
                cache.set(key, cached_value, ttl);
                Ok(true)
            }
        })?,
    )?;

    // rex.store.cache.get(key)
    cache_table.set(
        "get",
        lua.create_function({
            let cache = cache.clone();
            move |lua, key: String| match cache.get(&key) {
                Some(cached_value) => cached_value.to_lua(lua),
                None => Ok(Value::Nil),
            }
        })?,
    )?;

    // rex.store.cache.delete(key)
    cache_table.set(
        "delete",
        lua.create_function({
            let cache = cache.clone();
            move |_, key: String| Ok(cache.delete(&key))
        })?,
    )?;

    // rex.store.cache.exists(key)
    cache_table.set(
        "exists",
        lua.create_function({
            let cache = cache.clone();
            move |_, key: String| Ok(cache.exists(&key))
        })?,
    )?;

    // rex.store.cache.clear()
    cache_table.set(
        "clear",
        lua.create_function({
            let cache = cache.clone();
            move |_, ()| {
                cache.clear();
                Ok(true)
            }
        })?,
    )?;

    // rex.store.cache.len()
    cache_table.set(
        "len",
        lua.create_function({
            let cache = cache.clone();
            move |_, ()| Ok(cache.len())
        })?,
    )?;

    // rex.store.cache.keys()
    cache_table.set(
        "keys",
        lua.create_function({
            let cache = cache.clone();
            move |lua, ()| {
                let keys = cache.keys();
                let table = lua.create_table()?;
                for (i, key) in keys.into_iter().enumerate() {
                    table.set(i + 1, key)?;
                }
                Ok(table)
            }
        })?,
    )?;

    // rex.store.cache.incr(key, delta?)
    cache_table.set(
        "incr",
        lua.create_function({
            let cache = cache.clone();
            move |_, (key, delta): (String, Option<f64>)| cache.incr(&key, delta.unwrap_or(1.0))
        })?,
    )?;

    // rex.store.cache.cleanup() - Dọn keys hết hạn
    cache_table.set(
        "cleanup",
        lua.create_function({
            let cache = cache.clone();
            move |_, ()| {
                cache.cleanup_expired();
                Ok(true)
            }
        })?,
    )?;

    Ok(cache_table)
}
