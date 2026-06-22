//! YAML serialization.

use crate::error::{Result, StdlibError};

/// Parse YAML string.
pub fn parse(yaml_str: &str) -> Result<serde_json::Value> {
    serde_yaml::from_str(yaml_str).map_err(|e| StdlibError::Yaml(e.to_string()))
}

/// Serialize to YAML string.
pub fn stringify(value: &serde_json::Value) -> Result<String> {
    serde_yaml::to_string(value).map_err(|e| StdlibError::Yaml(e.to_string()))
}
