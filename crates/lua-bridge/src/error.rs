//! Lua bridge error types.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LuaBridgeError>;

#[derive(Error, Debug)]
pub enum LuaBridgeError {
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Async runtime not available")]
    AsyncRuntimeUnavailable,

    #[error("Conversion error: {0}")]
    Conversion(String),
}
