//! Event-driven architecture for Rustisaur.

pub mod dispatcher;
pub mod emitter;
pub mod listener;
pub mod types;

pub use dispatcher::EventDispatcher;
pub use emitter::EventEmitter;
pub use listener::EventListener;
pub use types::{EventData, EventError, Result};
