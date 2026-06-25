//! Network Firewall for Guardian
//! Controls and monitors all network requests

use super::console::GuardianConsole;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Network firewall configuration
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub allowed_domains: HashSet<String>,
    pub blocked_domains: HashSet<String>,
    pub allow_localhost: bool,
    pub require_approval: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let allowed = HashSet::new();
        let mut blocked = HashSet::new();

        // Default blocked domains (common malicious patterns)
        blocked.insert("malware.com".to_string());
        blocked.insert("phishing.net".to_string());
        blocked.insert("evil.com".to_string());

        Self {
            allowed_domains: allowed,
            blocked_domains: blocked,
            allow_localhost: true,
            require_approval: true,
        }
    }
}

/// Network request info
#[derive(Clone, Debug)]
pub struct NetworkRequest {
    pub url: String,
    pub domain: String,
    pub method: String,
    pub timestamp: u64,
}

/// Network Firewall - Controls all HTTP/HTTPS requests
pub struct NetworkFirewall {
    config: Arc<RwLock<NetworkConfig>>,
    console: GuardianConsole,
    request_log: Arc<RwLock<Vec<NetworkRequest>>>,
}

impl NetworkFirewall {
    /// Create new network firewall
    pub fn new(config: NetworkConfig) -> Self {
        debug!("🔥 Creating Network Firewall");
        Self {
            config: Arc::new(RwLock::new(config)),
            console: GuardianConsole::new(),
            request_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if a URL is allowed
    pub fn check_url(&self, url: &str) -> Result<bool, String> {
        let domain = self.extract_domain(url)?;

        debug!("🔍 Checking URL: {} (domain: {})", url, domain);

        // Check if localhost is allowed
        if domain == "localhost" || domain == "127.0.0.1" {
            if self.config.read().unwrap().allow_localhost {
                debug!("✅ Localhost allowed");
                return Ok(true);
            }
        }

        // Check blacklist
        {
            let config = self.config.read().unwrap();
            if config.blocked_domains.contains(&domain) {
                warn!("🚫 Domain {} is blacklisted", domain);
                self.console.alert(&format!(
                    "🚫 BLOCKED: {}\n\nDomain '{}' is in the blacklist.\nAccess denied!",
                    url, domain
                ));
                return Ok(false);
            }

            // Check whitelist
            if config.allowed_domains.contains(&domain) {
                debug!("✅ Domain {} is whitelisted", domain);
                return Ok(true);
            }
        }

        // If not in whitelist and approval required, ask owner
        if self.config.read().unwrap().require_approval {
            self.console.warn(&format!(
                "⚠️  Domain '{}' is not in whitelist\nRequesting owner approval...",
                domain
            ));

            let allowed = self
                .console
                .ask_network_permission(
                    url,
                    &domain,
                    Some("Script wants to make a network request"),
                )
                .map_err(|e| format!("Failed to get owner approval: {}", e))?;

            if allowed {
                info!("✅ Owner approved request to {}", domain);
                Ok(true)
            } else {
                warn!("❌ Owner denied request to {}", domain);
                Ok(false)
            }
        } else {
            // No approval required, allow by default
            Ok(true)
        }
    }

    /// ✅ PUBLIC: Extract domain from URL
    pub fn extract_domain(&self, url: &str) -> Result<String, String> {
        // Remove protocol
        let url_without_proto = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        // Get domain (before first /)
        let domain = url_without_proto
            .split('/')
            .next()
            .unwrap_or(url_without_proto);

        // Remove port if present
        let domain_without_port = domain.split(':').next().unwrap_or(domain);

        Ok(domain_without_port.to_string())
    }

    /// Add domain to whitelist
    pub fn add_to_whitelist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.allowed_domains.insert(domain.to_string());
        config.blocked_domains.remove(domain);

        info!("✅ Added {} to whitelist", domain);
        Ok(())
    }

    /// Add domain to blacklist
    pub fn add_to_blacklist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.blocked_domains.insert(domain.to_string());
        config.allowed_domains.remove(domain);

        warn!("🚫 Added {} to blacklist", domain);
        Ok(())
    }

    /// Remove domain from whitelist
    pub fn remove_from_whitelist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.allowed_domains.remove(domain);
        Ok(())
    }

    /// Remove domain from blacklist
    pub fn remove_from_blacklist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.blocked_domains.remove(domain);
        Ok(())
    }

    /// Log a network request
    pub fn log_request(&self, request: NetworkRequest) {
        let mut log = self.request_log.write().unwrap();
        log.push(request);
    }

    /// Get request log
    pub fn get_request_log(&self) -> Vec<NetworkRequest> {
        self.request_log.read().unwrap().clone()
    }

    /// Get configuration
    pub fn get_config(&self) -> NetworkConfig {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: NetworkConfig) {
        let mut current = self.config.write().unwrap();
        *current = config;
    }
}

impl Clone for NetworkFirewall {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            console: GuardianConsole::new(),
            request_log: self.request_log.clone(),
        }
    }
}
