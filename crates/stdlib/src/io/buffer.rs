//! Buffered I/O utilities.

use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};

/// Wrap a reader in a buffer.
pub fn buffered_reader<R: AsyncRead + Unpin>(reader: R) -> BufReader<R> {
    BufReader::new(reader)
}

/// Wrap a writer in a buffer.
pub fn buffered_writer<W: AsyncWrite + Unpin>(writer: W) -> BufWriter<W> {
    BufWriter::new(writer)
}
