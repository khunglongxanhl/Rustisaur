//! Plugin manager.

use crate::loader::PluginLoader;
use crate::registry::PluginRegistry;
use crate::traits::{PluginMetadata, Result};

/// Manages plugin lifecycle.
pub struct PluginManager {
    registry: PluginRegistry,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }

    pub fn load_and_register(&mut self, name: &str, version: &str) -> Result<()> {
        let mut plugin = PluginLoader::load_simple(name, version)?;
        plugin.initialize()?;
        self.registry.register(plugin)
    }

    pub fn shutdown_all(&mut self) -> Result<()> {
        for metadata in self.registry.list() {
            if let Some(plugin) = self.registry.get_mut(&metadata.name) {
                plugin.shutdown()?;
            }
        }
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.registry.list()
    }

    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }
}
