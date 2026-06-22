//! Plugin trait definitions.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PluginError>;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    #[error("Plugin initialization failed: {0}")]
    InitError(String),

    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),
}

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Plugin trait for extensibility.
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

/// Simple in-process plugin implementation.
pub struct SimplePlugin {
    metadata: PluginMetadata,
    initialized: bool,
}

impl SimplePlugin {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            metadata: PluginMetadata {
                name: name.into(),
                version: version.into(),
                description: None,
                author: None,
            },
            initialized: false,
        }
    }
}

impl Plugin for SimplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.initialized = false;
        Ok(())
    }
}

/// Type alias for dynamic plugin loading path.
pub type PluginPath = Box<Path>;
