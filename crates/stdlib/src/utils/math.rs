//! Math utilities.

use rand::Rng;

/// Random float in [0, 1).
pub fn random() -> f64 {
    rand::thread_rng().gen()
}

/// Random integer in range [min, max].
pub fn random_int(min: i64, max: i64) -> i64 {
    rand::thread_rng().gen_range(min..=max)
}

/// Minimum of two floats.
pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two floats.
pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Clamp value between min and max.
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}
