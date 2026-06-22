//! Table/collection utilities.

/// Get length of a vector.
pub fn length<T>(vec: &[T]) -> usize {
    vec.len()
}

/// Push value to vector.
pub fn push<T>(vec: &mut Vec<T>, value: T) {
    vec.push(value);
}

/// Remove value at index.
pub fn remove<T>(vec: &mut Vec<T>, index: usize) -> Option<T> {
    if index < vec.len() {
        Some(vec.remove(index))
    } else {
        None
    }
}
