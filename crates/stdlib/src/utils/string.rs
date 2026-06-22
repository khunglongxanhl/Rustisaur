//! String utilities.

/// Convert to uppercase.
pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

/// Convert to lowercase.
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// Split string by delimiter.
pub fn split(s: &str, delim: &str) -> Vec<String> {
    s.split(delim).map(String::from).collect()
}

/// Trim whitespace.
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Check if string starts with prefix.
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix.
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}
