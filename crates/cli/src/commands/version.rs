//! Version command.

use rustisaur_core::VERSION_INFO;

pub fn print_version() {
    println!("{VERSION_INFO}");
    println!("  Lua: 5.4");
    println!("  Runtime: Tokio (async I/O enabled)");
    println!("  License: MIT");
}
