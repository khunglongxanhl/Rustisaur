//! Plugin architecture for Rustisaur.

pub mod loader;
pub mod manager;
pub mod registry;
pub mod traits;

pub use loader::PluginLoader;
pub use manager::PluginManager;
pub use registry::PluginRegistry;
pub use traits::{Plugin, PluginError, PluginMetadata, Result};
