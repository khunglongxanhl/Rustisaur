//! Event emitter implementation.

use std::collections::HashMap;

use serde_json::Value;
use tokio::sync::broadcast;

use crate::types::{EventData, EventError, Result};

/// Broadcast-based event emitter.
pub struct EventEmitter {
    channels: HashMap<String, broadcast::Sender<EventData>>,
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter {
    /// Create a new event emitter.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// Subscribe to an event, creating the channel if needed.
    pub fn on(&mut self, event_name: &str) -> broadcast::Receiver<EventData> {
        let tx = self
            .channels
            .entry(event_name.to_string())
            .or_insert_with(|| broadcast::channel(100).0);
        tx.subscribe()
    }

    /// Emit an event to all subscribers (synchronous).
    pub fn emit_sync(&self, event_name: &str, payload: Value) -> Result<()> {
        if let Some(tx) = self.channels.get(event_name) {
            let data = EventData::new(event_name, payload);
            tx.send(data)
                .map_err(|e| EventError::SendError(e.to_string()))?;
            Ok(())
        } else {
            Err(EventError::NoListeners(event_name.to_string()))
        }
    }

    /// Emit an event to all subscribers.
    pub async fn emit(&self, event_name: &str, payload: Value) -> Result<()> {
        self.emit_sync(event_name, payload)
    }

    /// Check if an event has subscribers.
    pub fn has_listeners(&self, event_name: &str) -> bool {
        self.channels.contains_key(event_name)
    }

    /// Remove an event channel.
    pub fn off(&mut self, event_name: &str) {
        self.channels.remove(event_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn emit_and_receive() {
        let mut emitter = EventEmitter::new();
        let mut rx = emitter.on("test");

        emitter
            .emit("test", json!({"msg": "hello"}))
            .await
            .unwrap();

        let data = rx.recv().await.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.payload["msg"], "hello");
    }
}
