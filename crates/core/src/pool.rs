//! Engine pooling for high-performance concurrent execution.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use crate::config::EngineConfig;
use crate::engine::RustisaurEngine;
use crate::error::{EngineError, Result, RexError};

/// Thread-safe pool of Rustisaur engines.
pub struct EnginePool {
    /// Available engines in the pool
    engines: Arc<Mutex<Vec<RustisaurEngine>>>,
    /// Configuration for creating new engines
    config: EngineConfig,
    /// Maximum number of engines in the pool
    max_size: usize,
    /// Current total number of engines (available + in use)
    total_count: Arc<Mutex<usize>>,
    /// Statistics
    stats: Arc<Mutex<PoolStats>>,
}

/// Statistics about the pool.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_acquired: u64,
    pub total_released: u64,
    pub total_created: u64,
    pub total_errors: u64,
    pub peak_usage: usize,
    pub current_available: usize,
    pub current_in_use: usize,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool: {}/{} available, acquired: {}, created: {}, peak: {}",
            self.current_available,
            self.current_available + self.current_in_use,
            self.total_acquired,
            self.total_created,
            self.peak_usage
        )
    }
}

/// A pooled engine that automatically returns to the pool when dropped.
pub struct PooledEngine {
    engine: Option<RustisaurEngine>,
    pool: Arc<EnginePool>,
    acquired_at: Instant,
}

impl EnginePool {
    /// Create a new engine pool with pre-allocated engines.
    pub fn new(config: EngineConfig, initial_size: usize, max_size: usize) -> Result<Self> {
        if initial_size > max_size {
            return Err(RexError::Engine(EngineError::Config(
                "initial_size cannot exceed max_size".to_string(),
            )));
        }

        let mut engines = Vec::with_capacity(initial_size);

        info!(
            "Creating engine pool: initial={}, max={}",
            initial_size, max_size
        );

        // Pre-allocate engines
        for i in 0..initial_size {
            match RustisaurEngine::new(config.clone()) {
                Ok(engine) => {
                    engines.push(engine);
                    debug!("Pre-allocated engine {}/{}", i + 1, initial_size);
                }
                Err(e) => {
                    warn!("Failed to create engine {}/{}: {}", i + 1, initial_size, e);
                    return Err(e);
                }
            }
        }

        let pool = Self {
            engines: Arc::new(Mutex::new(engines)),
            config,
            max_size,
            total_count: Arc::new(Mutex::new(initial_size)),
            stats: Arc::new(Mutex::new(PoolStats {
                current_available: initial_size,
                ..Default::default()
            })),
        };

        info!("Engine pool created successfully");
        Ok(pool)
    }

    /// Acquire an engine from the pool.
    /// If no engines are available and we haven't reached max_size, creates a new one.
    pub fn acquire(self: &Arc<Self>) -> Result<PooledEngine> {
        let start = Instant::now();

        // Try to get an available engine
        let engine = {
            let mut engines = self.engines.lock().map_err(|e| {
                RexError::Engine(EngineError::Runtime(format!("Pool lock error: {}", e)))
            })?;

            engines.pop()
        };

        let engine = if let Some(engine) = engine {
            // Got one from pool
            debug!("Acquired engine from pool");
            engine
        } else {
            // Pool is empty, try to create a new one if under limit
            let mut total = self.total_count.lock().map_err(|e| {
                RexError::Engine(EngineError::Runtime(format!("Count lock error: {}", e)))
            })?;

            if *total < self.max_size {
                *total += 1;
                drop(total); // Release lock before creating engine

                debug!("Creating new engine (pool exhausted)");
                match RustisaurEngine::new(self.config.clone()) {
                    Ok(engine) => {
                        let mut stats = self.stats.lock().unwrap();
                        stats.total_created += 1;
                        engine
                    }
                    Err(e) => {
                        // Rollback count
                        let mut total = self.total_count.lock().unwrap();
                        *total -= 1;

                        let mut stats = self.stats.lock().unwrap();
                        stats.total_errors += 1;

                        return Err(e);
                    }
                }
            } else {
                let mut stats = self.stats.lock().unwrap();
                stats.total_errors += 1;

                return Err(RexError::Engine(EngineError::Runtime(
                    "Pool exhausted: all engines in use".to_string(),
                )));
            }
        };

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_acquired += 1;
            stats.current_available = stats.current_available.saturating_sub(1);
            stats.current_in_use += 1;

            if stats.current_in_use > stats.peak_usage {
                stats.peak_usage = stats.current_in_use;
            }
        }

        let elapsed = start.elapsed();
        debug!("Engine acquired in {:?}", elapsed);

        Ok(PooledEngine {
            engine: Some(engine),
            pool: Arc::clone(self),
            acquired_at: Instant::now(),
        })
    }

    /// Return an engine to the pool.
    fn release(&self, engine: RustisaurEngine) {
        // Reset engine state if needed
        engine.clear_cache();

        // Return to pool
        if let Ok(mut engines) = self.engines.lock() {
            engines.push(engine);

            if let Ok(mut stats) = self.stats.lock() {
                stats.total_released += 1;
                stats.current_in_use = stats.current_in_use.saturating_sub(1);
                stats.current_available += 1;
            }

            debug!("Engine returned to pool");
        } else {
            warn!("Failed to return engine to pool (lock error)");
        }
    }

    /// Get current pool statistics.
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get number of available engines.
    pub fn available(&self) -> usize {
        self.engines.lock().map(|e| e.len()).unwrap_or(0)
    }

    /// Get total number of engines (available + in use).
    pub fn total(&self) -> usize {
        *self.total_count.lock().unwrap()
    }

    /// Clear all engines from the pool (for shutdown or reset).
    pub fn clear(&self) {
        if let Ok(mut engines) = self.engines.lock() {
            let count = engines.len();
            engines.clear();

            if let Ok(mut total) = self.total_count.lock() {
                *total = 0;
            }

            info!("Pool cleared: {} engines removed", count);
        }
    }

    /// Warm up the pool by pre-creating additional engines.
    pub fn warmup(&self, count: usize) -> Result<()> {
        info!("Warming up pool with {} additional engines", count);

        let mut engines = self.engines.lock().map_err(|e| {
            RexError::Engine(EngineError::Runtime(format!("Pool lock error: {}", e)))
        })?;

        let mut total = self.total_count.lock().map_err(|e| {
            RexError::Engine(EngineError::Runtime(format!("Count lock error: {}", e)))
        })?;

        let to_create = std::cmp::min(count, self.max_size - *total);

        for i in 0..to_create {
            match RustisaurEngine::new(self.config.clone()) {
                Ok(engine) => {
                    engines.push(engine);
                    *total += 1;
                    debug!("Warmed up engine {}/{}", i + 1, to_create);
                }
                Err(e) => {
                    warn!("Failed to warm up engine {}/{}: {}", i + 1, to_create, e);
                    return Err(e);
                }
            }
        }

        info!("Pool warmed up: {} engines added", to_create);
        Ok(())
    }
}

impl PooledEngine {
    /// Access the underlying engine.
    pub fn engine(&self) -> &RustisaurEngine {
        self.engine.as_ref().unwrap()
    }

    /// Access the underlying engine mutably.
    pub fn engine_mut(&mut self) -> &mut RustisaurEngine {
        self.engine.as_mut().unwrap()
    }

    /// Get how long this engine has been acquired.
    pub fn elapsed(&self) -> Duration {
        self.acquired_at.elapsed()
    }

    /// Manually release the engine back to the pool.
    /// If not called, it will be released when dropped.
    pub fn release(mut self) {
        if let Some(engine) = self.engine.take() {
            self.pool.release(engine);
        }
    }
}

impl std::ops::Deref for PooledEngine {
    type Target = RustisaurEngine;

    fn deref(&self) -> &Self::Target {
        self.engine()
    }
}

impl std::ops::DerefMut for PooledEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.engine_mut()
    }
}

impl Drop for PooledEngine {
    fn drop(&mut self) {
        if let Some(engine) = self.engine.take() {
            self.pool.release(engine);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let config = EngineConfig::default();
        let pool = EnginePool::new(config, 5, 10).unwrap();

        assert_eq!(pool.available(), 5);
        assert_eq!(pool.total(), 5);
    }

    #[test]
    fn test_pool_acquire_release() {
        let config = EngineConfig::default();
        let pool = Arc::new(EnginePool::new(config, 2, 5).unwrap());

        assert_eq!(pool.available(), 2);

        // Acquire an engine
        let engine = pool.acquire().unwrap();
        assert_eq!(pool.available(), 1);

        // Release it (automatic on drop)
        drop(engine);
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_pool_grows_on_demand() {
        let config = EngineConfig::default();
        let pool = Arc::new(EnginePool::new(config, 1, 5).unwrap());

        assert_eq!(pool.total(), 1);

        // Acquire all available
        let e1 = pool.acquire().unwrap();
        let e2 = pool.acquire().unwrap(); // Should create new

        assert_eq!(pool.total(), 2);

        drop(e1);
        drop(e2);
    }

    #[test]
    fn test_pool_respects_max_size() {
        let config = EngineConfig::default();
        let pool = Arc::new(EnginePool::new(config, 2, 2).unwrap());

        let e1 = pool.acquire().unwrap();
        let e2 = pool.acquire().unwrap();

        // Third acquire should fail
        let result = pool.acquire();
        assert!(result.is_err());

        drop(e1);
        drop(e2);
    }

    #[test]
    fn test_pool_stats() {
        let config = EngineConfig::default();
        let pool = Arc::new(EnginePool::new(config, 3, 5).unwrap());

        let e1 = pool.acquire().unwrap();
        let e2 = pool.acquire().unwrap();

        let stats = pool.stats();
        assert_eq!(stats.total_acquired, 2);
        assert_eq!(stats.current_in_use, 2);
        assert_eq!(stats.current_available, 1);

        drop(e1);
        drop(e2);

        let stats = pool.stats();
        assert_eq!(stats.total_released, 2);
    }

    #[test]
    fn test_pool_warmup() {
        let config = EngineConfig::default();
        let pool = EnginePool::new(config, 2, 10).unwrap();

        assert_eq!(pool.total(), 2);

        pool.warmup(3).unwrap();

        assert_eq!(pool.total(), 5);
    }
}
