//! Environment variable utilities.

/// Get an environment variable.
pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Set an environment variable.
pub fn set(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// Get all environment variables.
pub fn all() -> Vec<(String, String)> {
    std::env::vars().collect()
}
