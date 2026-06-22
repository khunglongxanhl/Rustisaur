//! File writing utilities.

use std::path::Path;

/// Write string contents to a file.
pub async fn write_file(path: &Path, contents: &str) -> std::io::Result<()> {
    tokio::fs::write(path, contents).await
}

/// Append string contents to a file.
pub async fn append_file(path: &Path, contents: &str) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}
