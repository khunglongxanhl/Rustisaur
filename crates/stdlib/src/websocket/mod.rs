//! WebSocket module for real-time communication
//!
//! Features:
//! - WebSocket client (connect to servers)
//! - Send/receive messages
//! - Event callbacks (on_open, on_message, on_close, on_error)
//! - Auto-reconnect support
//! - Thread-safe operations

pub mod client;

pub use client::{create_websocket_module, WebSocketClient};
