//! String utility functions.

/// Convert string to uppercase.
pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

/// Convert string to lowercase.
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// Trim whitespace from both ends.
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Trim whitespace from left.
pub fn trim_left(s: &str) -> String {
    s.trim_start().to_string()
}

/// Trim whitespace from right.
pub fn trim_right(s: &str) -> String {
    s.trim_end().to_string()
}

/// Split string by delimiter.
pub fn split(s: &str, delim: &str) -> Vec<String> {
    s.split(delim).map(String::from).collect()
}

/// Join strings with delimiter.
pub fn join(parts: &[String], delim: &str) -> String {
    parts.join(delim)
}

/// Replace first occurrence.
pub fn replace(s: &str, from: &str, to: &str) -> String {
    s.replacen(from, to, 1)
}

/// Replace all occurrences.
pub fn replace_all(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Check if string starts with prefix.
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix.
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Check if string contains substring.
pub fn contains(s: &str, pattern: &str) -> bool {
    s.contains(pattern)
}

/// Capitalize first letter.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let capitalized = c.to_uppercase().to_string();
            let rest: String = chars.map(|c| c.to_lowercase().to_string()).collect();
            capitalized + &rest
        }
    }
}

/// Repeat string n times.
pub fn repeat(s: &str, count: usize) -> String {
    s.repeat(count)
}

/// Get substring from start to end.
pub fn slice(s: &str, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end - start).collect()
}

/// Reverse string.
pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Pad left with character.
pub fn pad_left(s: &str, width: usize, ch: char) -> String {
    format!("{: >width$}", s, width = width).replace(' ', &ch.to_string())
}

/// Pad right with character.
pub fn pad_right(s: &str, width: usize, ch: char) -> String {
    format!("{: <width$}", s, width = width).replace(' ', &ch.to_string())
}

/// Get string length.
pub fn len(s: &str) -> usize {
    s.chars().count()
}

/// Check if string is empty.
pub fn is_empty(s: &str) -> bool {
    s.trim().is_empty()
}
