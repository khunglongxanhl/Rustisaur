//! Async file I/O.

use std::path::{Path, PathBuf};

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// Async file handle for reading.
pub struct RexFile {
    inner: BufReader<File>,
    path: PathBuf,
}

impl RexFile {
    /// Open a file for reading.
    pub async fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path).await?;
        Ok(Self {
            inner: BufReader::new(file),
            path: path.to_path_buf(),
        })
    }

    /// Read entire file contents as string.
    pub async fn read_to_string(&mut self) -> std::io::Result<String> {
        let mut contents = String::new();
        self.inner.read_to_string(&mut contents).await?;
        Ok(contents)
    }

    /// Read a single line.
    pub async fn read_line(&mut self) -> std::io::Result<Option<String>> {
        let mut buffer = String::new();
        let bytes = self.inner.read_line(&mut buffer).await?;
        if bytes == 0 {
            Ok(None)
        } else {
            Ok(Some(buffer.trim().to_string()))
        }
    }

    /// Get the file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Write contents to a file.
pub async fn write_file(path: &Path, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path).await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}

/// Read file contents as string.
pub async fn read_file(path: &Path) -> std::io::Result<String> {
    tokio::fs::read_to_string(path).await
}

/// Append contents to a file.
pub async fn append_file(path: &Path, contents: &str) -> std::io::Result<()> {
    use tokio::io::AsyncSeekExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}
