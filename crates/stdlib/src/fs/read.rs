//! File reading utilities.

use std::path::Path;

/// Read a file as UTF-8 string.
pub async fn read_file(path: &Path) -> std::io::Result<String> {
    tokio::fs::read_to_string(path).await
}

/// Read a file as raw bytes.
pub async fn read_file_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    tokio::fs::read(path).await
}
