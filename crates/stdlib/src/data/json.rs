//! JSON serialization.

use serde_json::{from_str, json, to_string, to_string_pretty, Value};

use crate::error::{Result, StdlibError};

/// Parse JSON string.
pub fn parse(json_str: &str) -> Result<Value> {
    from_str(json_str).map_err(StdlibError::from)
}

/// Serialize to compact JSON string.
pub fn stringify(value: &Value) -> Result<String> {
    to_string(value).map_err(StdlibError::from)
}

/// Serialize to pretty-printed JSON string.
pub fn stringify_pretty(value: &Value) -> Result<String> {
    to_string_pretty(value).map_err(StdlibError::from)
}

/// Create an empty JSON object.
pub fn object() -> Value {
    json!({})
}

/// Create an empty JSON array.
pub fn array() -> Value {
    json!([])
}
