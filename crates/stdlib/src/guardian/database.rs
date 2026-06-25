//! Database Firewall for Guardian
//! Protects against SQL injection and dangerous queries

use super::console::GuardianConsole;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Database firewall configuration
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub blocked_keywords: HashSet<String>,
    pub blocked_patterns: HashSet<String>,
    pub max_rows_affected: usize,
    pub require_approval_for_dangerous: bool,
    pub log_all_queries: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let mut blocked_keywords = HashSet::with_capacity(10);

        // Dangerous SQL keywords
        blocked_keywords.insert("DROP TABLE".to_string());
        blocked_keywords.insert("DROP DATABASE".to_string());
        blocked_keywords.insert("TRUNCATE".to_string());
        blocked_keywords.insert("DELETE FROM".to_string());
        blocked_keywords.insert("ALTER TABLE".to_string());
        blocked_keywords.insert("GRANT".to_string());
        blocked_keywords.insert("REVOKE".to_string());

        let mut blocked_patterns = HashSet::with_capacity(10);

        // SQL injection patterns
        blocked_patterns.insert("' OR '1'='1".to_string());
        blocked_patterns.insert("' OR 1=1".to_string());
        blocked_patterns.insert("' OR ''='".to_string());
        blocked_patterns.insert("UNION SELECT".to_string());
        blocked_patterns.insert("UNION ALL SELECT".to_string());
        blocked_patterns.insert("'; DROP TABLE".to_string());
        blocked_patterns.insert("--".to_string());
        blocked_patterns.insert("/*".to_string());

        Self {
            blocked_keywords,
            blocked_patterns,
            max_rows_affected: 1000,
            require_approval_for_dangerous: true,
            log_all_queries: true,
        }
    }
}

/// Database query info
#[derive(Clone, Debug)]
pub struct DatabaseQuery {
    pub sql: String,
    pub query_type: QueryType,
    pub timestamp: u64,
    pub rows_affected: Option<usize>,
    pub blocked: bool,
}

/// Query type
#[derive(Clone, Debug, PartialEq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Truncate,
    Unknown,
}

impl QueryType {
    pub fn from_sql(sql: &str) -> Self {
        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") {
            QueryType::Select
        } else if sql_upper.starts_with("INSERT") {
            QueryType::Insert
        } else if sql_upper.starts_with("UPDATE") {
            QueryType::Update
        } else if sql_upper.starts_with("DELETE") {
            QueryType::Delete
        } else if sql_upper.starts_with("CREATE") {
            QueryType::Create
        } else if sql_upper.starts_with("DROP") {
            QueryType::Drop
        } else if sql_upper.starts_with("ALTER") {
            QueryType::Alter
        } else if sql_upper.starts_with("TRUNCATE") {
            QueryType::Truncate
        } else {
            QueryType::Unknown
        }
    }

    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            QueryType::Drop | QueryType::Delete | QueryType::Truncate | QueryType::Alter
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            QueryType::Select => "SELECT",
            QueryType::Insert => "INSERT",
            QueryType::Update => "UPDATE",
            QueryType::Delete => "DELETE",
            QueryType::Create => "CREATE",
            QueryType::Drop => "DROP",
            QueryType::Alter => "ALTER",
            QueryType::Truncate => "TRUNCATE",
            QueryType::Unknown => "UNKNOWN",
        }
    }
}

/// Database statistics
#[derive(Clone, Debug, Default)]
pub struct DatabaseStats {
    pub total_queries: u64,
    pub allowed_queries: u64,
    pub blocked_queries: u64,
    pub dangerous_queries: u64,
    pub injection_attempts: u64,
}

/// Database Firewall - Protects against SQL injection and dangerous queries
pub struct DatabaseFirewall {
    config: Arc<RwLock<DatabaseConfig>>,
    console: GuardianConsole,
    query_log: Arc<RwLock<Vec<DatabaseQuery>>>,
    stats: Arc<RwLock<DatabaseStats>>,
}

impl DatabaseFirewall {
    /// Create new database firewall
    pub fn new(config: DatabaseConfig) -> Self {
        debug!("🗄️  Creating Database Firewall");
        Self {
            config: Arc::new(RwLock::new(config)),
            console: GuardianConsole::new(),
            query_log: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(DatabaseStats::default())),
        }
    }

    /// Check if a SQL query is allowed
    pub fn check_query(&self, sql: &str) -> Result<bool, String> {
        debug!("🔍 Checking SQL query: {}", sql);

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_queries += 1;
        }

        let query_type = QueryType::from_sql(sql);

        // Check for SQL injection patterns (FAST - using to_uppercase once)
        let sql_upper = sql.to_uppercase();
        {
            let config = self.config.read().unwrap();
            for pattern in &config.blocked_patterns {
                if sql_upper.contains(&pattern.to_uppercase()) {
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.injection_attempts += 1;
                        stats.blocked_queries += 1;
                    }

                    self.console.alert(&format!(
                        "🚫 BLOCKED: SQL Injection attempt detected!\n\n\
                         Pattern: {}\n\
                         SQL: {}\n\n\
                         This query has been blocked for security.",
                        pattern, sql
                    ));

                    // Log the blocked query
                    if config.log_all_queries {
                        self.log_query(DatabaseQuery {
                            sql: sql.to_string(),
                            query_type: query_type.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            rows_affected: None,
                            blocked: true,
                        });
                    }

                    return Ok(false);
                }
            }

            // Check for dangerous keywords
            for keyword in &config.blocked_keywords {
                if sql_upper.contains(keyword) {
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.dangerous_queries += 1;
                        stats.blocked_queries += 1;
                    }

                    self.console.alert(&format!(
                        "🚫 BLOCKED: Dangerous keyword detected!\n\n\
                         Keyword: {}\n\
                         Query Type: {}\n\
                         SQL: {}\n\n\
                         This query has been blocked for security.",
                        keyword,
                        query_type.as_str(),
                        sql
                    ));

                    // Log the blocked query
                    if config.log_all_queries {
                        self.log_query(DatabaseQuery {
                            sql: sql.to_string(),
                            query_type: query_type.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            rows_affected: None,
                            blocked: true,
                        });
                    }

                    return Ok(false);
                }
            }
        }

        // Check for dangerous query types
        if query_type.is_dangerous() {
            {
                let mut stats = self.stats.write().unwrap();
                stats.dangerous_queries += 1;
            }

            let config = self.config.read().unwrap();
            if config.require_approval_for_dangerous {
                drop(config);

                self.console.warn(&format!(
                    "⚠️  DANGEROUS QUERY DETECTED!\n\n\
                     Type: {}\n\
                     SQL: {}\n\n\
                     This query could cause data loss!",
                    query_type.as_str(),
                    sql
                ));

                let allowed = self
                    .console
                    .ask_yes_no("Do you want to allow this dangerous query?", false)
                    .map_err(|e| format!("Failed to get owner approval: {}", e))?;

                if !allowed {
                    warn!("❌ Owner denied dangerous query");
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.blocked_queries += 1;
                    }

                    // Log the blocked query
                    let config = self.config.read().unwrap();
                    if config.log_all_queries {
                        self.log_query(DatabaseQuery {
                            sql: sql.to_string(),
                            query_type: query_type.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            rows_affected: None,
                            blocked: true,
                        });
                    }

                    return Ok(false);
                }
            }
        }

        // Log query if enabled
        let config = self.config.read().unwrap();
        if config.log_all_queries {
            self.log_query(DatabaseQuery {
                sql: sql.to_string(),
                query_type,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                rows_affected: None,
                blocked: false,
            });
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.allowed_queries += 1;
        }

        Ok(true)
    }

    /// Log a database query
    pub fn log_query(&self, query: DatabaseQuery) {
        let mut log = self.query_log.write().unwrap();
        log.push(query);
    }

    /// Get query log
    pub fn get_query_log(&self) -> Vec<DatabaseQuery> {
        self.query_log.read().unwrap().clone()
    }

    /// Get statistics
    pub fn get_stats(&self) -> DatabaseStats {
        self.stats.read().unwrap().clone()
    }

    /// Show statistics
    pub fn show_stats(&self) {
        let stats = self.get_stats();
        self.console.info(&format!(
            "📊 DATABASE FIREWALL STATISTICS\n\n\
             Total Queries: {}\n\
             Allowed: {}\n\
             Blocked: {}\n\
             Dangerous Queries: {}\n\
             Injection Attempts: {}\n\
             \n\
             Block Rate: {:.2}%",
            stats.total_queries,
            stats.allowed_queries,
            stats.blocked_queries,
            stats.dangerous_queries,
            stats.injection_attempts,
            if stats.total_queries > 0 {
                (stats.blocked_queries as f64 / stats.total_queries as f64) * 100.0
            } else {
                0.0
            }
        ));
    }

    /// Add blocked keyword
    pub fn add_blocked_keyword(&self, keyword: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_keywords.insert(keyword.to_uppercase());
        info!("✅ Added blocked keyword: {}", keyword);
    }

    /// Remove blocked keyword
    pub fn remove_blocked_keyword(&self, keyword: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_keywords.remove(&keyword.to_uppercase());
        info!("🗑️  Removed blocked keyword: {}", keyword);
    }

    /// Add blocked pattern
    pub fn add_blocked_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_patterns.insert(pattern.to_string());
        info!("✅ Added blocked pattern: {}", pattern);
    }

    /// Remove blocked pattern
    pub fn remove_blocked_pattern(&self, pattern: &str) {
        let mut config = self.config.write().unwrap();
        config.blocked_patterns.remove(pattern);
        info!("🗑️  Removed blocked pattern: {}", pattern);
    }

    /// Enable/disable approval for dangerous queries
    pub fn set_require_approval(&self, require: bool) {
        let mut config = self.config.write().unwrap();
        config.require_approval_for_dangerous = require;
        info!("🔧 Set require_approval_for_dangerous: {}", require);
    }
}

impl Clone for DatabaseFirewall {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            console: GuardianConsole::new(),
            query_log: self.query_log.clone(),
            stats: self.stats.clone(),
        }
    }
}
