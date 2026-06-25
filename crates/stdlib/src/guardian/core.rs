//! Guardian Core - Central security manager

use super::console::GuardianConsole;
use super::network::{NetworkConfig, NetworkFirewall, NetworkRequest};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Guardian configuration
#[derive(Clone, Debug)]
pub struct GuardianConfig {
    pub network: NetworkConfig,
    pub interactive_mode: bool,
    pub log_all_requests: bool,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            interactive_mode: true,
            log_all_requests: true,
        }
    }
}

/// Guardian statistics
#[derive(Clone, Debug, Default)]
pub struct GuardianStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub blocked_requests: u64,
    pub whitelist_hits: u64,
    pub blacklist_hits: u64,
}

/// Guardian Core - Main security manager
pub struct Guardian {
    config: GuardianConfig,
    network_firewall: NetworkFirewall,
    console: GuardianConsole,
    stats: Arc<std::sync::RwLock<GuardianStats>>,
}

impl Guardian {
    /// Create new Guardian with config
    pub fn new(config: GuardianConfig) -> Self {
        info!("🛡️  Initializing Guardian Security System");

        let network_firewall = NetworkFirewall::new(config.network.clone());

        Self {
            config,
            network_firewall,
            console: GuardianConsole::new(),
            stats: Arc::new(std::sync::RwLock::new(GuardianStats::default())),
        }
    }

    /// Create Guardian with default config
    pub fn default() -> Self {
        Self::new(GuardianConfig::default())
    }

    /// Check if network request is allowed
    pub fn check_network(&self, url: &str) -> Result<bool, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_requests += 1;
        }

        // Check with network firewall
        match self.network_firewall.check_url(url) {
            Ok(true) => {
                // Log allowed request
                if self.config.log_all_requests {
                    self.network_firewall.log_request(NetworkRequest {
                        url: url.to_string(),
                        domain: self
                            .network_firewall
                            .extract_domain(url)
                            .unwrap_or_default(),
                        method: "GET".to_string(),
                        timestamp,
                    });
                }

                // Update stats
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.allowed_requests += 1;
                }

                Ok(true)
            }
            Ok(false) => {
                // Update stats
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.blocked_requests += 1;
                }

                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Get network firewall
    pub fn network(&self) -> &NetworkFirewall {
        &self.network_firewall
    }

    /// Get statistics
    pub fn get_stats(&self) -> GuardianStats {
        self.stats.read().unwrap().clone()
    }

    /// Display statistics
    pub fn show_stats(&self) {
        let stats = self.get_stats();

        self.console.info(&format!(
            "📊 GUARDIAN STATISTICS\n\n\
             Total Requests: {}\n\
             Allowed: {}\n\
             Blocked: {}\n\
             Whitelist Hits: {}\n\
             Blacklist Hits: {}\n\
             \n\
             Block Rate: {:.2}%",
            stats.total_requests,
            stats.allowed_requests,
            stats.blocked_requests,
            stats.whitelist_hits,
            stats.blacklist_hits,
            if stats.total_requests > 0 {
                (stats.blocked_requests as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        ));
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

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = GuardianStats::default();
        info!("📊 Statistics reset");
    }
}
