//! Network Firewall for Guardian
//! Controls and monitors all network requests
//!
//! Optimizations:
//! - Pre-allocated HashSets with capacity
//! - Reduced locking with read-write locks
//! - Cached domain extraction
//! - Efficient string matching

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
        let allowed = HashSet::with_capacity(16); // Pre-allocate
        let mut blocked = HashSet::with_capacity(16); // Pre-allocate

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

/// Network statistics
#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub blocked_requests: u64,
    pub whitelist_hits: u64,
    pub blacklist_hits: u64,
}

/// Network Firewall - Controls all HTTP/HTTPS requests
pub struct NetworkFirewall {
    config: Arc<RwLock<NetworkConfig>>,
    console: GuardianConsole,
    request_log: Arc<RwLock<Vec<NetworkRequest>>>,
    stats: Arc<RwLock<NetworkStats>>,
}

impl NetworkFirewall {
    /// Create new network firewall
    pub fn new(config: NetworkConfig) -> Self {
        debug!("🔥 Creating Network Firewall");
        Self {
            config: Arc::new(RwLock::new(config)),
            console: GuardianConsole::new(),
            request_log: Arc::new(RwLock::new(Vec::with_capacity(100))), // Pre-allocate
            stats: Arc::new(RwLock::new(NetworkStats::default())),
        }
    }

    /// Check if a URL is allowed - OPTIMIZED
    pub fn check_url(&self, url: &str) -> Result<bool, String> {
        debug!("🔍 Checking URL: {}", url);

        // Extract domain once
        let domain = self.extract_domain(url)?;

        // Read config once and check everything
        let config = self.config.read().unwrap();

        // Check localhost first (fast path)
        if (domain == "localhost" || domain == "127.0.0.1") && config.allow_localhost {
            debug!("✅ Localhost allowed");
            return Ok(true);
        }

        // Check blacklist (O(1) lookup)
        if config.blocked_domains.contains(&domain) {
            drop(config); // Release lock early

            warn!("🚫 Domain {} is blacklisted", domain);
            self.console.alert(&format!(
                "🚫 BLOCKED: {}\n\nDomain '{}' is in the blacklist.\nAccess denied!",
                url, domain
            ));

            // Update stats
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_requests += 1;
                stats.blocked_requests += 1;
                stats.blacklist_hits += 1;
            }

            return Ok(false);
        }

        // Check whitelist (O(1) lookup)
        if config.allowed_domains.contains(&domain) {
            drop(config); // Release lock early

            debug!("✅ Domain {} is whitelisted", domain);

            // Update stats
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_requests += 1;
                stats.allowed_requests += 1;
                stats.whitelist_hits += 1;
            }

            return Ok(true);
        }

        drop(config); // Release lock before asking user

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

            // Update stats
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_requests += 1;
                if allowed {
                    stats.allowed_requests += 1;
                } else {
                    stats.blocked_requests += 1;
                }
            }

            if allowed {
                info!("✅ Owner approved request to {}", domain);
                Ok(true)
            } else {
                warn!("❌ Owner denied request to {}", domain);
                Ok(false)
            }
        } else {
            // No approval required, allow by default
            // Update stats
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_requests += 1;
                stats.allowed_requests += 1;
            }
            Ok(true)
        }
    }

    /// ✅ PUBLIC: Extract domain from URL - OPTIMIZED
    pub fn extract_domain(&self, url: &str) -> Result<String, String> {
        // Remove protocol (fast string operations)
        let url_without_proto = if let Some(stripped) = url.strip_prefix("https://") {
            stripped
        } else if let Some(stripped) = url.strip_prefix("http://") {
            stripped
        } else {
            url
        };

        // Get domain (before first /)
        let domain = url_without_proto
            .split('/')
            .next()
            .unwrap_or(url_without_proto);

        // Remove port if present
        let domain_without_port = domain.split(':').next().unwrap_or(domain);

        Ok(domain_without_port.to_lowercase()) // Normalize to lowercase
    }

    /// Add domain to whitelist
    pub fn add_to_whitelist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.allowed_domains.insert(domain.to_lowercase());
        config.blocked_domains.remove(&domain.to_lowercase());

        info!("✅ Added {} to whitelist", domain);
        Ok(())
    }

    /// Add domain to blacklist
    pub fn add_to_blacklist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.blocked_domains.insert(domain.to_lowercase());
        config.allowed_domains.remove(&domain.to_lowercase());

        warn!("🚫 Added {} to blacklist", domain);
        Ok(())
    }

    /// Remove domain from whitelist
    pub fn remove_from_whitelist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.allowed_domains.remove(&domain.to_lowercase());
        Ok(())
    }

    /// Remove domain from blacklist
    pub fn remove_from_blacklist(&self, domain: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.blocked_domains.remove(&domain.to_lowercase());
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

    /// Get statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.read().unwrap().clone()
    }

    /// Show statistics
    pub fn show_stats(&self) {
        let stats = self.get_stats();
        self.console.info(&format!(
            "📊 NETWORK FIREWALL STATISTICS\n\n\
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
}

impl Clone for NetworkFirewall {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            console: GuardianConsole::new(),
            request_log: self.request_log.clone(),
            stats: self.stats.clone(),
        }
    }
}
