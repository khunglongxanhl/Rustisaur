//! HTTP module for web and API functionality
//!
//! Features:
//! - HTTP Client (GET, POST, PUT, DELETE) - sync version
//! - HTTP Server (REST API) - coming soon
//! - WebSocket (real-time communication) - coming soon

pub mod client;

// Re-export
pub use client::{create_http_module, HttpClient};
