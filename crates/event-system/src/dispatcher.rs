//! Event dispatcher for routing events.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::emitter::EventEmitter;
use crate::types::{EventData, EventError, Result};

/// Routes events between multiple emitters.
pub struct EventDispatcher {
    emitter: Arc<Mutex<EventEmitter>>,
}

impl EventDispatcher {
    /// Create a new dispatcher.
    pub fn new() -> Self {
        Self {
            emitter: Arc::new(Mutex::new(EventEmitter::new())),
        }
    }

    /// Subscribe to events through the dispatcher.
    pub async fn subscribe(&self, event_name: &str) -> tokio::sync::broadcast::Receiver<EventData> {
        let mut emitter = self.emitter.lock().await;
        emitter.on(event_name)
    }

    /// Dispatch an event.
    pub async fn dispatch(&self, event_name: &str, payload: serde_json::Value) -> Result<()> {
        let emitter = self.emitter.lock().await;
        emitter.emit(event_name, payload).await
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
