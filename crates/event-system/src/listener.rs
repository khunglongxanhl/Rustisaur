//! Event listener utilities.

use tokio::sync::broadcast;

use crate::types::{EventData, EventError, Result};

/// Wrapper around a broadcast receiver for event listening.
pub struct EventListener {
    receiver: broadcast::Receiver<EventData>,
}

impl EventListener {
    /// Create from a broadcast receiver.
    pub fn new(receiver: broadcast::Receiver<EventData>) -> Self {
        Self { receiver }
    }

    /// Wait for the next event.
    pub async fn next(&mut self) -> Result<EventData> {
        self.receiver
            .recv()
            .await
            .map_err(|e| EventError::HandlerError(e.to_string()))
    }

    /// Try to receive without blocking.
    pub fn try_next(&mut self) -> Result<EventData> {
        self.receiver
            .try_recv()
            .map_err(|e| EventError::HandlerError(e.to_string()))
    }
}
