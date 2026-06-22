//! Time and date utilities.

use chrono::{DateTime, Local, Utc};

/// Current local time.
pub fn now_local() -> DateTime<Local> {
    Local::now()
}

/// Current UTC time.
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Unix timestamp (seconds).
pub fn unix_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Format current time with strftime-style format.
pub fn format_now(fmt: &str) -> String {
    Local::now().format(fmt).to_string()
}
