//! Redis-like in-memory cache with TTL support
//!
//! Features:
//! - ⚡ Tốc độ phản lực: Lưu trực tiếp trên RAM (DashMap)
//! - ⏰ TTL (Time To Live): Tự động xóa sau X giây
//! - 🧵 Thread-safe: Hoạt động tốt với async/multi-thread
//! - 📦 Hỗ trợ mọi kiểu dữ liệu Lua (string, number, table, boolean)
//! - 📊 Debugging: Thống kê hits/misses, operations count

use dashmap::DashMap;
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::debug;

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

/// Thống kê cache (cho debugging)
#[derive(Default)]
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub sets: AtomicU64,
    pub deletes: AtomicU64,
    pub evictions: AtomicU64,
}

impl CacheStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_set(&self) {
        self.sets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
        if total == 0.0 {
            0.0
        } else {
            hits / total * 100.0
        }
    }
}

/// Cache store - Trái tim của "tốc độ phản lực"
#[derive(Clone)]
pub struct CacheStore {
    inner: Arc<DashMap<String, CacheEntry>>,
    stats: Arc<CacheStats>,
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheStore {
    pub fn new() -> Self {
        debug!("🔥 Initializing CacheStore (Redis-like)");
        Self {
            inner: Arc::new(DashMap::new()),
            stats: Arc::new(CacheStats::default()),
        }
    }

    /// Tạo cache với pre-allocated capacity (tối ưu cho batch operations)
    pub fn with_capacity(capacity: usize) -> Self {
        debug!("🔥 Initializing CacheStore with capacity: {}", capacity);
        Self {
            inner: Arc::new(DashMap::with_capacity(capacity)),
            stats: Arc::new(CacheStats::default()),
        }
    }

    /// Set value với TTL (time to live) tính bằng giây
    pub fn set(&self, key: String, value: CachedValue, ttl_seconds: Option<u64>) {
        debug!(key = %key, ttl = ?ttl_seconds, "📝 Cache SET");
        let expires_at = ttl_seconds.map(|secs| Instant::now() + Duration::from_secs(secs));
        self.inner.insert(key, CacheEntry { value, expires_at });
        self.stats.record_set();
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
                    self.stats.record_eviction();
                    debug!(key = %key, "⏰ Cache TTL expired");
                    self.stats.record_miss();
                    return None;
                }
            }
            debug!(key = %key, "✅ Cache HIT");
            self.stats.record_hit();
            Some(entry.value.clone())
        } else {
            debug!(key = %key, "❌ Cache MISS");
            self.stats.record_miss();
            None
        }
    }

    /// Xóa một key
    pub fn delete(&self, key: &str) -> bool {
        debug!(key = %key, "🗑️  Cache DELETE");
        let removed = self.inner.remove(key).is_some();
        if removed {
            self.stats.record_delete();
        }
        removed
    }

    /// Kiểm tra key có tồn tại không
    pub fn exists(&self, key: &str) -> bool {
        if let Some(entry) = self.inner.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    drop(entry);
                    self.inner.remove(key);
                    self.stats.record_eviction();
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
        debug!("🧹 Cache CLEAR - removing all entries");
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
    pub fn cleanup_expired(&self) -> usize {
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

        let count = expired_keys.len();
        for key in expired_keys {
            self.inner.remove(&key);
            self.stats.record_eviction();
        }

        if count > 0 {
            debug!("🧹 Cleaned up {} expired cache entries", count);
        }
        count
    }

    /// Increment một số (giống Redis INCR)
    pub fn incr(&self, key: &str, delta: f64) -> LuaResult<f64> {
        // TỐI ƯU: Dùng entry() API để tránh double lookup
        use dashmap::mapref::entry::Entry;

        let new_value = match self.inner.entry(key.to_string()) {
            Entry::Occupied(mut entry) => match &entry.get().value {
                CachedValue::Number(n) => {
                    let new_val = n + delta;
                    entry.get_mut().value = CachedValue::Number(new_val);
                    new_val
                }
                CachedValue::Nil => {
                    entry.get_mut().value = CachedValue::Number(delta);
                    delta
                }
                _ => {
                    return Err(mlua::Error::RuntimeError(
                        "Value is not a number".to_string(),
                    ));
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(CacheEntry {
                    value: CachedValue::Number(delta),
                    expires_at: None,
                });
                delta
            }
        };

        debug!(key = %key, delta = delta, new_value = new_value, "🔢 Cache INCR");
        Ok(new_value)
    }

    // ==========================================
    // BATCH OPERATIONS (Tối ưu FFI overhead)
    // ==========================================

    /// Set nhiều keys cùng lúc (Nhanh hơn gọi set() nhiều lần)
    pub fn batch_set(&self, entries: Vec<(String, CachedValue, Option<u64>)>) {
        let count = entries.len(); // ✅ Lưu length TRƯỚC
        debug!("📦 Cache BATCH SET: {} entries", count);

        for (key, value, ttl) in entries {
            let expires_at = ttl.map(|secs| Instant::now() + Duration::from_secs(secs));
            self.inner.insert(key, CacheEntry { value, expires_at });
        }

        for _ in 0..count {
            // ✅ Dùng biến count đã lưu
            self.stats.record_set();
        }
    }

    /// Get nhiều keys cùng lúc
    pub fn batch_get(&self, keys: &[String]) -> Vec<Option<CachedValue>> {
        debug!("📦 Cache BATCH GET: {} keys", keys.len());
        keys.iter().map(|key| self.get(key)).collect()
    }

    /// Xóa nhiều keys cùng lúc
    pub fn batch_delete(&self, keys: &[String]) -> usize {
        debug!("📦 Cache BATCH DELETE: {} keys", keys.len());
        let mut count = 0;
        for key in keys {
            if self.inner.remove(key).is_some() {
                count += 1;
                self.stats.record_delete();
            }
        }
        count
    }

    /// Lấy thống kê cache (cho debugging)
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64, f64) {
        (
            self.stats.hits.load(Ordering::Relaxed),
            self.stats.misses.load(Ordering::Relaxed),
            self.stats.sets.load(Ordering::Relaxed),
            self.stats.deletes.load(Ordering::Relaxed),
            self.stats.evictions.load(Ordering::Relaxed),
            self.stats.hit_rate(),
        )
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

    // rex.store.cache.is_empty()
    cache_table.set(
        "is_empty",
        lua.create_function({
            let cache = cache.clone();
            move |_, ()| Ok(cache.is_empty())
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
            move |_, ()| Ok(cache.cleanup_expired())
        })?,
    )?;

    // ==========================================
    // BATCH OPERATIONS (Tối ưu FFI overhead)
    // ==========================================

    // rex.store.cache.batch_set(entries) - entries là array of {key, value, ttl?}
    cache_table.set(
        "batch_set",
        lua.create_function({
            let cache = cache.clone();
            move |_, entries: Table| {
                let mut batch = Vec::with_capacity(entries.len().unwrap_or(0) as usize);
                for pair in entries.pairs::<i64, Table>() {
                    let (_, item) = pair?;
                    let key: String = item.get("key")?;
                    let value: Value = item.get("value")?;
                    let ttl: Option<u64> = item.get("ttl").ok();

                    let cached_value = CachedValue::from_lua(value)?;
                    batch.push((key, cached_value, ttl));
                }
                cache.batch_set(batch);
                Ok(true)
            }
        })?,
    )?;

    // rex.store.cache.batch_get(keys) - keys là array of string
    cache_table.set(
        "batch_get",
        lua.create_function({
            let cache = cache.clone();
            move |lua, keys: Table| {
                let mut key_list = Vec::with_capacity(keys.len().unwrap_or(0) as usize);
                for pair in keys.pairs::<i64, String>() {
                    let (_, key) = pair?;
                    key_list.push(key);
                }

                let results = cache.batch_get(&key_list);
                let result_table = lua.create_table()?;
                for (i, val) in results.into_iter().enumerate() {
                    match val {
                        Some(v) => result_table.set(i + 1, v.to_lua(lua)?)?,
                        None => result_table.set(i + 1, Value::Nil)?,
                    }
                }
                Ok(result_table)
            }
        })?,
    )?;

    // rex.store.cache.batch_delete(keys) - keys là array of string
    cache_table.set(
        "batch_delete",
        lua.create_function({
            let cache = cache.clone();
            move |_, keys: Table| {
                let mut key_list = Vec::with_capacity(keys.len().unwrap_or(0) as usize);
                for pair in keys.pairs::<i64, String>() {
                    let (_, key) = pair?;
                    key_list.push(key);
                }
                Ok(cache.batch_delete(&key_list))
            }
        })?,
    )?;

    // rex.store.cache.stats() - Lấy thống kê cache (cho debugging)
    cache_table.set(
        "stats",
        lua.create_function({
            let cache = cache.clone();
            move |lua, ()| {
                let (hits, misses, sets, deletes, evictions, hit_rate) = cache.get_stats();
                let stats = lua.create_table()?;
                stats.set("hits", hits)?;
                stats.set("misses", misses)?;
                stats.set("sets", sets)?;
                stats.set("deletes", deletes)?;
                stats.set("evictions", evictions)?;
                stats.set("hit_rate", hit_rate)?;
                stats.set("size", cache.len())?;
                Ok(stats)
            }
        })?,
    )?;

    Ok(cache_table)
}
