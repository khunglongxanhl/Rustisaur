//! Path utilities.

use std::path::{Path, PathBuf};

/// Join path segments.
pub fn join(base: &Path, segment: &str) -> PathBuf {
    base.join(segment)
}

/// Get file extension.
pub fn extension(path: &Path) -> Option<String> {
    path.extension().map(|e| e.to_string_lossy().to_string())
}

/// Get file name without directory.
pub fn file_name(path: &Path) -> Option<String> {
    path.file_name().map(|n| n.to_string_lossy().to_string())
}

/// Check if path exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}
