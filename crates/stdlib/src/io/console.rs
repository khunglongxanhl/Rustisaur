//! Console I/O (stdin/stdout/stderr).

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Read a line from stdin asynchronously.
pub async fn read_line() -> io::Result<String> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await?;
    Ok(buffer.trim().to_string())
}

/// Write a line to stdout asynchronously.
pub async fn write_line(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(text.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

/// Read password input (basic implementation).
pub async fn read_password() -> io::Result<String> {
    read_line().await
}
