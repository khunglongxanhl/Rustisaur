//! Standard library error types.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, StdlibError>;

#[derive(Error, Debug)]
pub enum StdlibError {
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("TOML error: {0}")]
    Toml(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}
