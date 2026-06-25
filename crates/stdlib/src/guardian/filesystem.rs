//! File System Firewall for Guardian
//! Protects sensitive files and directories

use super::console::GuardianConsole;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// File system firewall configuration
#[derive(Clone, Debug)]
pub struct FileSystemConfig {
    pub allowed_paths: HashSet<String>,
    pub blocked_patterns: HashSet<String>,
    pub max_file_size_mb: u64,
    pub require_approval_for_sensitive: bool,
    pub log_all_operations: bool,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        let mut allowed_paths = HashSet::new();

        // Default allowed paths (relative to current directory)
        allowed_paths.insert("./data".to_string());
        allowed_paths.insert("./temp".to_string());
        allowed_paths.insert("./logs".to_string());
        allowed_paths.insert("./config".to_string());

        let mut blocked_patterns = HashSet::new();

        // Dangerous file patterns
        blocked_patterns.insert("*.exe".to_string());
        blocked_patterns.insert("*.dll".to_string());
        blocked_patterns.insert("*.sys".to_string());
        blocked_patterns.insert("*.bat".to_string());
        blocked_patterns.insert("*.cmd".to_string());
        blocked_patterns.insert("*.sh".to_string());

        // Sensitive system paths
        blocked_patterns.insert("/etc/passwd".to_string());
        blocked_patterns.insert("/etc/shadow".to_string());
        blocked_patterns.insert("/etc/sudoers".to_string());
        blocked_patterns.insert("/root/*".to_string());
        blocked_patterns.insert("/windows/system32/*".to_string());
        blocked_patterns.insert("C:\\Windows\\System32\\*".to_string());
        blocked_patterns.insert("C:\\Users\\*\\AppData\\*".to_string());

        Self {
            allowed_paths,
            blocked_patterns,
            max_file_size_mb: 100,
            require_approval_for_sensitive: true,
            log_all_operations: true,
        }
    }
}

/// File operation info
#[derive(Clone, Debug)]
pub struct FileOperation {
    pub path: String,
    pub operation: FileOpType,
    pub timestamp: u64,
    pub success: bool,
    pub blocked: bool,
}

/// File operation type
#[derive(Clone, Debug, PartialEq)]
pub enum FileOpType {
    Read,
    Write,
    Delete,
    Create,
    Rename,
    Copy,
    Execute,
}

impl FileOpType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(FileOpType::Read),
            "write" => Some(FileOpType::Write),
            "delete" => Some(FileOpType::Delete),
            "create" => Some(FileOpType::Create),
            "rename" => Some(FileOpType::Rename),
            "copy" => Some(FileOpType::Copy),
            "execute" => Some(FileOpType::Execute),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileOpType::Read => "READ",
            FileOpType::Write => "WRITE",
            FileOpType::Delete => "DELETE",
            FileOpType::Create => "CREATE",
            FileOpType::Rename => "RENAME",
            FileOpType::Copy => "COPY",
            FileOpType::Execute => "EXECUTE",
        }
    }

    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            FileOpType::Delete | FileOpType::Write | FileOpType::Execute
        )
    }
}

/// File system statistics
#[derive(Clone, Debug, Default)]
pub struct FileSystemStats {
    pub total_operations: u64,
    pub allowed_operations: u64,
    pub blocked_operations: u64,
    pub sensitive_access: u64,
    pub blocked_patterns_hits: u64,
}

/// File System Firewall
pub struct FileSystemFirewall {
    config: Arc<RwLock<FileSystemConfig>>,
    console: GuardianConsole,
    operation_log: Arc<RwLock<Vec<FileOperation>>>,
    stats: Arc<RwLock<FileSystemStats>>,
}

impl FileSystemFirewall {
    /// Create new file system firewall
    pub fn new(config: FileSystemConfig) -> Self {
        debug!("📁 Creating File System Firewall");
        Self {
            config: Arc::new(RwLock::new(config)),
            console: GuardianConsole::new(),
            operation_log: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(FileSystemStats::default())),
        }
    }

    /// Check if a file operation is allowed
    pub fn check_operation(&self, path: &str, operation: FileOpType) -> Result<bool, String> {
        debug!("🔍 Checking file operation: {:?} on {}", operation, path);

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_operations += 1;
        }

        // Check if path matches blocked patterns
        for pattern in &self.config.read().unwrap().blocked_patterns {
            if self.matches_pattern(path, pattern) {
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.blocked_operations += 1;
                    stats.blocked_patterns_hits += 1;
                }

                self.console.alert(&format!(
                    "🚫 BLOCKED: File operation denied!\n\n\
                     Path: {}\n\
                     Operation: {:?}\n\
                     Reason: Matches blocked pattern '{}'\n\n\
                     This operation has been blocked for security.",
                    path, operation, pattern
                ));

                // Log the blocked operation
                if self.config.read().unwrap().log_all_operations {
                    self.log_operation(FileOperation {
                        path: path.to_string(),
                        operation,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        success: false,
                        blocked: true,
                    });
                }

                return Ok(false);
            }
        }

        // Check if operation is sensitive
        if operation.is_sensitive() && self.config.read().unwrap().require_approval_for_sensitive {
            {
                let mut stats = self.stats.write().unwrap();
                stats.sensitive_access += 1;
            }

            self.console.warn(&format!(
                "⚠️  SENSITIVE FILE OPERATION!\n\n\
                 Path: {}\n\
                 Operation: {:?}\n\n\
                 This operation could modify or delete files.",
                path, operation
            ));

            let allowed = self
                .console
                .ask_yes_no("Do you want to allow this file operation?", false)
                .map_err(|e| format!("Failed to get owner approval: {}", e))?;

            if !allowed {
                warn!("❌ Owner denied file operation");
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.blocked_operations += 1;
                }

                // Log the blocked operation
                if self.config.read().unwrap().log_all_operations {
                    self.log_operation(FileOperation {
                        path: path.to_string(),
                        operation,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        success: false,
                        blocked: true,
                    });
                }

                return Ok(false);
            }
        }

        // Check if path is in allowed paths (if not empty)
        let allowed_paths = &self.config.read().unwrap().allowed_paths;
        if !allowed_paths.is_empty() {
            let is_allowed = allowed_paths.iter().any(|allowed| {
                let abs_path = Path::new(path);
                let abs_allowed = Path::new(allowed);

                // Check if path starts with allowed path
                abs_path.starts_with(abs_allowed)
            });

            if !is_allowed {
                self.console.warn(&format!(
                    "⚠️  PATH NOT IN WHITELIST!\n\n\
                     Path: {}\n\
                     Operation: {:?}\n\n\
                     This path is not in the allowed list.",
                    path, operation
                ));

                let allowed = self
                    .console
                    .ask_yes_no("Do you want to allow this path?", false)
                    .map_err(|e| format!("Failed to get owner approval: {}", e))?;

                if !allowed {
                    warn!("❌ Owner denied access to path");
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.blocked_operations += 1;
                    }

                    // Log the blocked operation
                    if self.config.read().unwrap().log_all_operations {
                        self.log_operation(FileOperation {
                            path: path.to_string(),
                            operation,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            success: false,
                            blocked: true,
                        });
                    }

                    return Ok(false);
                }
            }
        }

        // Log operation if enabled
        if self.config.read().unwrap().log_all_operations {
            self.log_operation(FileOperation {
                path: path.to_string(),
                operation,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                success: true,
                blocked: false,
            });
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.allowed_operations += 1;
        }

        Ok(true)
    }

    /// Check if path matches pattern (simple glob matching)
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Normalize path separators
        let normalized_path = path.replace('\\', "/");
        let normalized_pattern = pattern.replace('\\', "/");

        if normalized_pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = normalized_pattern.split('*').collect();
            if parts.len() == 2 {
                let starts_with = parts[0].is_empty() || normalized_path.starts_with(parts[0]);
                let ends_with = parts[1].is_empty() || normalized_path.ends_with(parts[1]);
                return starts_with && ends_with;
            }
        }

        // Exact match or contains
        normalized_path.contains(&normalized_pattern)
    }

    /// Log a file operation
    pub fn log_operation(&self, operation: FileOperation) {
        let mut log = self.operation_log.write().unwrap();
        log.push(operation);
    }

    /// Get operation log
    pub fn get_operation_log(&self) -> Vec<FileOperation> {
        self.operation_log.read().unwrap().clone()
    }

    /// Get statistics
    pub fn get_stats(&self) -> FileSystemStats {
        self.stats.read().unwrap().clone()
    }

    /// Show statistics
    pub fn show_stats(&self) {
        let stats = self.get_stats();
        self.console.info(&format!(
            "📊 FILE SYSTEM FIREWALL STATISTICS\n\n\
             Total Operations: {}\n\
             Allowed: {}\n\
             Blocked: {}\n\
             Sensitive Access: {}\n\
             Blocked Pattern Hits: {}\n\
             \n\
             Block Rate: {:.2}%",
            stats.total_operations,
            stats.allowed_operations,
            stats.blocked_operations,
            stats.sensitive_access,
            stats.blocked_patterns_hits,
            if stats.total_operations > 0 {
                (stats.blocked_operations as f64 / stats.total_operations as f64) * 100.0
            } else {
                0.0
            }
        ));
    }

    /// Add allowed path
    pub fn add_allowed_path(&self, path: &str) {
        let mut config = self.config.write().unwrap();
        config.allowed_paths.insert(path.to_string());
        info!("✅ Added allowed path: {}", path);
    }

    /// Add blocked pattern
    pub fn add_blocked_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_patterns.insert(pattern.to_string());
        info!("✅ Added blocked pattern: {}", pattern);
    }

    /// Remove allowed path
    pub fn remove_allowed_path(&self, path: &str) {
        let mut config = self.config.write().unwrap();
        config.allowed_paths.remove(path);
        info!("🗑️  Removed allowed path: {}", path);
    }

    /// Remove blocked pattern
    pub fn remove_blocked_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_patterns.remove(pattern);
        info!("🗑️  Removed blocked pattern: {}", pattern);
    }

    /// Set max file size
    pub fn set_max_file_size(&self, size_mb: u64) {
        let mut config = self.config.write().unwrap();
        config.max_file_size_mb = size_mb;
        info!("🔧 Set max file size to {} MB", size_mb);
    }

    /// Enable/disable approval for sensitive operations
    pub fn set_require_approval(&self, require: bool) {
        let mut config = self.config.write().unwrap();
        config.require_approval_for_sensitive = require;
        info!("🔧 Set require_approval_for_sensitive: {}", require);
    }
}

impl Clone for FileSystemFirewall {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            console: GuardianConsole::new(),
            operation_log: self.operation_log.clone(),
            stats: self.stats.clone(),
        }
    }
}
