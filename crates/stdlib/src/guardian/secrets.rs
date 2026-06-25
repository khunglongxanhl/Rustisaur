//! Secret Protection for Guardian
//! Protects API keys, passwords, and sensitive data from exposure

use super::console::GuardianConsole;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Secret protection configuration
#[derive(Clone, Debug)]
pub struct SecretConfig {
    pub protected_patterns: HashSet<String>,
    pub log_all_access: bool,
}

impl Default for SecretConfig {
    fn default() -> Self {
        let mut protected_patterns = HashSet::new();

        // Default sensitive patterns
        protected_patterns.insert("PASSWORD".to_string());
        protected_patterns.insert("PASSWD".to_string());
        protected_patterns.insert("SECRET".to_string());
        protected_patterns.insert("API_KEY".to_string());
        protected_patterns.insert("TOKEN".to_string());
        protected_patterns.insert("PRIVATE_KEY".to_string());
        protected_patterns.insert("ACCESS_KEY".to_string());
        protected_patterns.insert("CREDENTIAL".to_string());

        Self {
            protected_patterns,
            log_all_access: true,
        }
    }
}

/// Secret access info
#[derive(Clone, Debug)]
pub struct SecretAccess {
    pub variable_name: String,
    pub access_type: SecretAccessType,
    pub timestamp: u64,
    pub blocked: bool,
}

/// Secret access type
#[derive(Clone, Debug, PartialEq)]
pub enum SecretAccessType {
    Read,
    Write,
    Print,
    Export,
}

impl SecretAccessType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(SecretAccessType::Read),
            "write" => Some(SecretAccessType::Write),
            "print" => Some(SecretAccessType::Print),
            "export" => Some(SecretAccessType::Export),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SecretAccessType::Read => "READ",
            SecretAccessType::Write => "WRITE",
            SecretAccessType::Print => "PRINT",
            SecretAccessType::Export => "EXPORT",
        }
    }

    /// Check if this access type could expose the secret
    pub fn is_exposure(&self) -> bool {
        matches!(self, SecretAccessType::Print | SecretAccessType::Export)
    }
}

/// Secret statistics
#[derive(Clone, Debug, Default)]
pub struct SecretStats {
    pub total_access: u64,
    pub allowed_access: u64,
    pub blocked_access: u64,
    pub exposure_attempts: u64,
}

/// Secret Protection Firewall
pub struct SecretProtection {
    config: Arc<RwLock<SecretConfig>>,
    console: GuardianConsole,
    access_log: Arc<RwLock<Vec<SecretAccess>>>,
    stats: Arc<RwLock<SecretStats>>,
}

impl SecretProtection {
    /// Create new secret protection
    pub fn new(config: SecretConfig) -> Self {
        debug!("🔐 Creating Secret Protection");
        Self {
            config: Arc::new(RwLock::new(config)),
            console: GuardianConsole::new(),
            access_log: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SecretStats::default())),
        }
    }

    /// Check if accessing a secret variable is allowed
    pub fn check_access(
        &self,
        variable_name: &str,
        access_type: SecretAccessType,
    ) -> Result<bool, String> {
        debug!(
            " Checking secret access: {} ({:?})",
            variable_name, access_type
        );

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_access += 1;
        }

        // Check if variable name matches protected patterns
        let var_upper = variable_name.to_uppercase();
        let mut is_protected = false;

        for pattern in &self.config.read().unwrap().protected_patterns {
            if var_upper.contains(pattern) {
                is_protected = true;
                break;
            }
        }

        // If it's a protected variable and the access type is exposure (Print/Export)
        if is_protected && access_type.is_exposure() {
            {
                let mut stats = self.stats.write().unwrap();
                stats.exposure_attempts += 1;
                stats.blocked_access += 1;
            }

            self.console.alert(&format!(
                "🚫 BLOCKED: Secret exposure attempt!\n\n\
                 Variable: {}\n\
                 Access Type: {:?}\n\n\
                 This variable contains sensitive data and cannot be exposed.\n\
                 Use rex.guardian.mask_secret() to safely view it.",
                variable_name, access_type
            ));

            // Log the blocked attempt
            if self.config.read().unwrap().log_all_access {
                self.log_access(SecretAccess {
                    variable_name: variable_name.to_string(),
                    access_type,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    blocked: true,
                });
            }

            return Ok(false);
        }

        // Log access if enabled
        if self.config.read().unwrap().log_all_access {
            self.log_access(SecretAccess {
                variable_name: variable_name.to_string(),
                access_type,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                blocked: false,
            });
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.allowed_access += 1;
        }

        Ok(true)
    }

    /// Mask a secret value (show only first/last few characters)
    pub fn mask_value(&self, value: &str) -> String {
        if value.is_empty() {
            return "****".to_string();
        }
        if value.len() <= 4 {
            return "****".to_string();
        }

        let visible = 2;
        let first = &value[..visible];
        let last = &value[value.len() - visible..];
        let masked_len = value.len() - (visible * 2);
        let mask = "*".repeat(masked_len);

        format!("{}{}{}", first, mask, last)
    }

    /// Log a secret access
    pub fn log_access(&self, access: SecretAccess) {
        let mut log = self.access_log.write().unwrap();
        log.push(access);
    }

    /// Get access log
    pub fn get_access_log(&self) -> Vec<SecretAccess> {
        self.access_log.read().unwrap().clone()
    }

    /// Get statistics
    pub fn get_stats(&self) -> SecretStats {
        self.stats.read().unwrap().clone()
    }

    /// Show statistics
    pub fn show_stats(&self) {
        let stats = self.get_stats();
        self.console.info(&format!(
            "📊 SECRET PROTECTION STATISTICS\n\n\
             Total Access: {}\n\
             Allowed: {}\n\
             Blocked: {}\n\
             Exposure Attempts: {}\n\
             \n\
             Block Rate: {:.2}%",
            stats.total_access,
            stats.allowed_access,
            stats.blocked_access,
            stats.exposure_attempts,
            if stats.total_access > 0 {
                (stats.blocked_access as f64 / stats.total_access as f64) * 100.0
            } else {
                0.0
            }
        ));
    }

    /// Add protected pattern
    pub fn add_protected_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.protected_patterns.insert(pattern.to_uppercase());
        info!("✅ Added protected pattern: {}", pattern);
    }

    /// Remove protected pattern
    pub fn remove_protected_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.protected_patterns.remove(&pattern.to_uppercase());
        info!("🗑️  Removed protected pattern: {}", pattern);
    }
}

impl Clone for SecretProtection {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            console: GuardianConsole::new(),
            access_log: self.access_log.clone(),
            stats: self.stats.clone(),
        }
    }
}
