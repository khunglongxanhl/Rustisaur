//! Safe wrappers around FFI handles.

use rustisaur_core::RustisaurEngine;

/// Safe wrapper for the Rustisaur engine in FFI contexts.
pub struct RustisaurHandle {
    engine: RustisaurEngine,
}

impl RustisaurHandle {
    pub fn new(engine: RustisaurEngine) -> Self {
        Self { engine }
    }

    pub fn engine(&self) -> &RustisaurEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut RustisaurEngine {
        &mut self.engine
    }
}
