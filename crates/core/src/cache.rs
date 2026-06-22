//! Script caching for improved performance.

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use mlua::Lua;
use tracing::debug;

/// Cached compiled script.
#[derive(Clone)]
pub struct CachedScript {
    /// Original source code (for cache key validation)
    pub source: String,
    /// Compiled bytecode
    pub bytecode: Vec<u8>,
}

/// Script cache manager.
pub struct ScriptCache {
    cache: Arc<Mutex<LruCache<String, CachedScript>>>,
}

impl ScriptCache {
    /// Create a new script cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let cache_size = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }

    /// Get or compile a script.
    pub fn get_or_compile(&self, lua: &Lua, source: &str) -> mlua::Result<CachedScript> {
        // Check cache first
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|e| mlua::Error::RuntimeError(format!("Cache lock error: {}", e)))?;

            if let Some(cached) = cache.get(source) {
                debug!("Script cache hit: {} bytes", cached.bytecode.len());
                return Ok(cached.clone());
            }
        }

        // Compile the script
        debug!("Script cache miss, compiling: {} bytes", source.len());
        let chunk = lua.load(source);
        let bytecode = chunk.into_function()?.dump(true); // strip = true for smaller bytecode

        let cached = CachedScript {
            source: source.to_string(),
            bytecode,
        };

        // Store in cache
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|e| mlua::Error::RuntimeError(format!("Cache lock error: {}", e)))?;
            cache.put(source.to_string(), cached.clone());
        }

        debug!("Script cached successfully");
        Ok(cached)
    }

    /// Execute a cached script.
    pub fn execute_cached<'a>(
        &self,
        lua: &'a Lua,
        cached: &CachedScript,
    ) -> mlua::Result<mlua::MultiValue<'a>> {
        let chunk = lua.load(&cached.bytecode);
        chunk.eval()
    }

    /// Clear the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
            debug!("Script cache cleared");
        }
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.lock() {
            CacheStats {
                size: cache.len(),
                capacity: cache.cap().get(), // Convert NonZero<usize> to usize
            }
        } else {
            CacheStats {
                size: 0,
                capacity: 0,
            }
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cache: {}/{} entries", self.size, self.capacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_cache() {
        let lua = Lua::new();
        let cache = ScriptCache::new(10);

        // First call: compile
        let cached1 = cache.get_or_compile(&lua, "return 42").unwrap();
        assert_eq!(cached1.source, "return 42");

        // Second call: cache hit
        let cached2 = cache.get_or_compile(&lua, "return 42").unwrap();
        assert_eq!(cached2.bytecode, cached1.bytecode);

        // Execute cached script
        let result = cache.execute_cached(&lua, &cached1).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_cache_stats() {
        let lua = Lua::new();
        let cache = ScriptCache::new(10);

        cache.get_or_compile(&lua, "return 1").unwrap();
        cache.get_or_compile(&lua, "return 2").unwrap();

        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.capacity, 10);
    }
}
