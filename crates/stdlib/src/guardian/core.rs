//! Guardian Core - Central security manager

use super::console::GuardianConsole;
use super::database::{DatabaseConfig, DatabaseFirewall};
use super::filesystem::{FileOpType, FileSystemConfig, FileSystemFirewall};
use super::network::{NetworkConfig, NetworkFirewall};
use super::secrets::{SecretAccessType, SecretConfig, SecretProtection};
use tracing::info;

/// Guardian configuration
#[derive(Clone, Debug)]
pub struct GuardianConfig {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub filesystem: FileSystemConfig,
    pub secrets: SecretConfig,
    pub interactive_mode: bool,
    pub log_all_requests: bool,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            filesystem: FileSystemConfig::default(),
            secrets: SecretConfig::default(),
            interactive_mode: true,
            log_all_requests: true,
        }
    }
}

/// Combined statistics from all firewalls
#[derive(Clone, Debug, Default)]
pub struct GuardianStats {
    // Network stats
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub blocked_requests: u64,
    // Database stats
    pub total_queries: u64,
    pub blocked_queries: u64,
    // Filesystem stats
    pub total_file_ops: u64,
    pub blocked_file_ops: u64,
    // Secret stats
    pub total_secret_access: u64,
    pub blocked_secret_access: u64,
}

/// Guardian Core - Main security manager
pub struct Guardian {
    config: GuardianConfig,
    network_firewall: NetworkFirewall,
    database_firewall: DatabaseFirewall,
    filesystem_firewall: FileSystemFirewall,
    secret_protection: SecretProtection,
    console: GuardianConsole,
}

impl Guardian {
    /// Create new Guardian with config
    pub fn new(config: GuardianConfig) -> Self {
        info!("🛡️  Initializing Guardian Security System (Phase 1 + Phase 2)");

        let network_firewall = NetworkFirewall::new(config.network.clone());
        let database_firewall = DatabaseFirewall::new(config.database.clone());
        let filesystem_firewall = FileSystemFirewall::new(config.filesystem.clone());
        let secret_protection = SecretProtection::new(config.secrets.clone());

        Self {
            config,
            network_firewall,
            database_firewall,
            filesystem_firewall,
            secret_protection,
            console: GuardianConsole::new(),
        }
    }

    /// Create Guardian with default config
    pub fn new_default() -> Self {
        Self::new(GuardianConfig::default())
    }

    // ========================================
    // NETWORK FIREWALL METHODS (Phase 1)
    // ========================================

    /// Check if network request is allowed
    pub fn check_network(&self, url: &str) -> Result<bool, String> {
        self.network_firewall.check_url(url)
    }

    /// Get network firewall
    pub fn network(&self) -> &NetworkFirewall {
        &self.network_firewall
    }

    // ========================================
    // DATABASE FIREWALL METHODS (Phase 2)
    // ========================================

    /// Check if database query is allowed
    pub fn check_database(&self, sql: &str) -> Result<bool, String> {
        self.database_firewall.check_query(sql)
    }

    /// Get database firewall
    pub fn database(&self) -> &DatabaseFirewall {
        &self.database_firewall
    }

    // ========================================
    // FILESYSTEM FIREWALL METHODS (Phase 2)
    // ========================================

    /// Check if file operation is allowed
    pub fn check_filesystem(&self, path: &str, operation: FileOpType) -> Result<bool, String> {
        self.filesystem_firewall.check_operation(path, operation)
    }

    /// Get filesystem firewall
    pub fn filesystem(&self) -> &FileSystemFirewall {
        &self.filesystem_firewall
    }

    // ========================================
    // SECRET PROTECTION METHODS (Phase 2)
    // ========================================

    /// Check if secret access is allowed
    pub fn check_secret(
        &self,
        variable_name: &str,
        access_type: SecretAccessType,
    ) -> Result<bool, String> {
        self.secret_protection
            .check_access(variable_name, access_type)
    }

    /// Get secret protection
    pub fn secrets(&self) -> &SecretProtection {
        &self.secret_protection
    }

    // ========================================
    // COMBINED STATISTICS
    // ========================================

    /// Get combined statistics
    pub fn get_stats(&self) -> GuardianStats {
        let net_stats = self.network_firewall.get_stats();
        let db_stats = self.database_firewall.get_stats();
        let fs_stats = self.filesystem_firewall.get_stats();
        let sec_stats = self.secret_protection.get_stats();

        GuardianStats {
            total_requests: net_stats.total_requests,
            allowed_requests: net_stats.allowed_requests,
            blocked_requests: net_stats.blocked_requests,
            total_queries: db_stats.total_queries,
            blocked_queries: db_stats.blocked_queries,
            total_file_ops: fs_stats.total_operations,
            blocked_file_ops: fs_stats.blocked_operations,
            total_secret_access: sec_stats.total_access,
            blocked_secret_access: sec_stats.blocked_access,
        }
    }

    /// Display all statistics
    pub fn show_stats(&self) {
        self.console.info(&format!(
            "🛡️  GUARDIAN SECURITY DASHBOARD\n\n{}",
            "═".repeat(60)
        ));

        // Network stats
        self.network_firewall.show_stats();

        // Database stats
        self.database_firewall.show_stats();

        // Filesystem stats
        self.filesystem_firewall.show_stats();

        // Secret stats
        self.secret_protection.show_stats();
    }

    /// Enable interactive mode
    pub fn enable_interactive(&mut self) {
        self.config.interactive_mode = true;
        info!("🔔 Interactive mode enabled");
    }

    /// Disable interactive mode (auto-decide)
    pub fn disable_interactive(&mut self) {
        self.config.interactive_mode = false;
        info!("🔇 Interactive mode disabled (auto-decide)");
    }
}
