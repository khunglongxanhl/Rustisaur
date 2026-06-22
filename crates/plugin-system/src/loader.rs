//! Plugin loader.

use std::path::Path;

use crate::traits::{Plugin, PluginError, Result, SimplePlugin};

/// Loads plugins from files or in-process definitions.
pub struct PluginLoader;

impl PluginLoader {
    /// Load a simple in-process plugin by name.
    pub fn load_simple(name: &str, version: &str) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(SimplePlugin::new(name, version)))
    }

    /// Load a dynamic library plugin (placeholder for future native plugin support).
    pub fn load_dynamic(path: &Path) -> Result<Box<dyn Plugin>> {
        if !path.exists() {
            return Err(PluginError::NotFound(path.display().to_string()));
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Ok(Box::new(SimplePlugin::new(name, "0.1.0")))
    }
}
