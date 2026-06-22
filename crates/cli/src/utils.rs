//! CLI utility functions.

use std::path::Path;

/// Check if a file has a Rustisaur extension.
#[allow(dead_code)]
pub fn is_rustisaur_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e, "rex" | "lua"))
        .unwrap_or(false)
}
