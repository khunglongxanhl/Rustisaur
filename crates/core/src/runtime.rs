//! Tokio runtime management.

use std::sync::Arc;

use tokio::runtime::{Handle, Runtime};

use crate::error::EngineError;

/// Manages the Tokio async runtime for Rustisaur.
pub struct RexRuntime {
    runtime: Arc<Runtime>,
}

impl RexRuntime {
    /// Create and start a new multi-threaded Tokio runtime.
    pub fn new() -> Result<Self, EngineError> {
        let runtime = Runtime::new()
            .map_err(|e| EngineError::RuntimeInit(e.to_string()))?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    /// Get a handle to the runtime.
    pub fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }

    /// Run a future to completion on this runtime.
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Access the inner runtime.
    pub fn inner(&self) -> &Runtime {
        &self.runtime
    }
}

impl Default for RexRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create Tokio runtime")
    }
}
