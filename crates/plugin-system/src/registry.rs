//! Plugin registry.

use std::collections::HashMap;

use crate::traits::{Plugin, PluginError, PluginMetadata, Result};

/// Registry of loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.metadata().name.clone();
        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyRegistered(name));
        }
        self.plugins.insert(name, plugin);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Plugin>> {
        self.plugins.get_mut(name)
    }

    pub fn list(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Plugin>> {
        self.plugins.remove(name)
    }
}
