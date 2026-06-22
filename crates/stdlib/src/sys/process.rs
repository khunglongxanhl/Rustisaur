//! Process management utilities.

use std::path::PathBuf;

/// Get current working directory.
pub fn cwd() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

/// Get current process ID.
pub fn pid() -> u32 {
    std::process::id()
}

/// Get command-line arguments.
pub fn args() -> Vec<String> {
    std::env::args().collect()
}
