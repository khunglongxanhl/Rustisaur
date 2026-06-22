//! TOML serialization.

use crate::error::{Result, StdlibError};

/// Parse TOML string.
pub fn parse(toml_str: &str) -> Result<serde_json::Value> {
    let val: toml::Value = toml_str
        .parse()
        .map_err(|e: toml::de::Error| StdlibError::Toml(e.to_string()))?;
    serde_json::to_value(val).map_err(StdlibError::from)
}

/// Serialize to TOML string.
pub fn stringify(value: &serde_json::Value) -> Result<String> {
    let toml_val: toml::Value =
        serde_json::from_value(value.clone()).map_err(StdlibError::from)?;
    toml::to_string(&toml_val).map_err(|e| StdlibError::Toml(e.to_string()))
}
