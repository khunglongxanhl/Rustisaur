//! Guardian - Multi-layer Security Firewall for Rustisaur
//!
//! Features:
//! - 🔥 Network Firewall (Phase 1)
//! - 🗄️  Database Firewall (Phase 2)
//! - 📁 File System Firewall (Phase 2)
//! - 🔐 Secret Protection (Phase 2)
//! - 🔮 Predictive Threat Detection (Phase 3)

pub mod console;
pub mod core;
pub mod network;

pub use console::GuardianConsole;
pub use core::{Guardian, GuardianConfig, GuardianStats};
pub use network::{NetworkConfig, NetworkFirewall, NetworkRequest};
