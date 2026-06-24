//! rex.store - Hybrid storage module
//!
//! Kết hợp:
//! - 🔥 Cache (Redis-like): Tốc độ phản lực, lưu trên RAM
//! - 🐘 Database (SQLite): Lưu trữ bền vững, query phức tạp

pub mod cache;
pub mod db;

pub use cache::{create_cache_module, CacheStore};
pub use db::{create_db_module, Database};
