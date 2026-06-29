//! HTTP module for web and API functionality

pub mod client;
pub mod server;

pub use client::{create_http_module, HttpClient};
pub use server::{create_http_server_module, HttpServer, ServerConfig};
