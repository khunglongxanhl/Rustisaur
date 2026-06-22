//! Core error types.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, RexError>;

#[derive(Error, Debug)]
pub enum RexError {
    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),

    #[error("Lua bridge error: {0}")]
    LuaBridge(#[from] rustisaur_lua_bridge::error::LuaBridgeError),

    #[error("Standard library error: {0}")]
    Stdlib(#[from] rustisaur_stdlib::error::StdlibError),

    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Script timeout")]
    Timeout,

    #[error("Memory limit exceeded")]
    MemoryLimit,

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed to initialize runtime: {0}")]
    RuntimeInit(String),

    #[error("Script file not found: {0}")]
    FileNotFound(String),

    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Engine already shut down")]
    ShutDown,

    #[error("Configuration error: {0}")]
    Config(String),
}