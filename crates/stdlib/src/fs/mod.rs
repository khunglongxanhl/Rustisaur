//! Filesystem operations.

pub mod path;
pub mod read;
pub mod watch;
pub mod write;

pub use read::{read_file, read_file_bytes};
pub use write::{append_file, write_file};
