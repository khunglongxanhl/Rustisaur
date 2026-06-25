//! Guardian - Multi-layer Security Firewall for Rustisaur
//!
//! Features:
//! - 🔥 Network Firewall (Phase 1)
//! - 🗄️  Database Firewall (Phase 2)
//! - 📁 File System Firewall (Phase 2)
//! - 🔐 Secret Protection (Phase 2)
//! - 🔮 Predictive Threat Detection (Phase 3 - Coming soon)

pub mod console;
pub mod core;
pub mod database;
pub mod filesystem;
pub mod network;
pub mod secrets;

// Re-export main types
pub use console::GuardianConsole;
pub use core::{Guardian, GuardianConfig, GuardianStats};

// Network firewall
pub use network::{NetworkConfig, NetworkFirewall, NetworkRequest};

// Database firewall
pub use database::{DatabaseConfig, DatabaseFirewall, DatabaseQuery, DatabaseStats, QueryType};

// Filesystem firewall
pub use filesystem::{
    FileOpType, FileOperation, FileSystemConfig, FileSystemFirewall, FileSystemStats,
};

// Secret protection
pub use secrets::{SecretAccess, SecretAccessType, SecretConfig, SecretProtection, SecretStats};
