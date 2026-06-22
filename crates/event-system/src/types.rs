//! Event type definitions.

use std::time::Instant;

use serde_json::Value;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventError>;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("No listeners for event: {0}")]
    NoListeners(String),

    #[error("Failed to send event: {0}")]
    SendError(String),

    #[error("Event handler error: {0}")]
    HandlerError(String),
}

/// Event payload with metadata.
#[derive(Debug, Clone)]
pub struct EventData {
    pub name: String,
    pub payload: Value,
    pub timestamp: Instant,
}

impl EventData {
    pub fn new(name: impl Into<String>, payload: Value) -> Self {
        Self {
            name: name.into(),
            payload,
            timestamp: Instant::now(),
        }
    }
}
