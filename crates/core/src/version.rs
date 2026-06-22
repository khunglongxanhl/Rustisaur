//! Version information.

/// Rustisaur version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Detailed version information.
pub struct VersionInfo {
    pub version: &'static str,
    pub lua_version: &'static str,
    pub rust_edition: &'static str,
}

/// Static version info instance.
pub const VERSION_INFO: VersionInfo = VersionInfo {
    version: VERSION,
    lua_version: "5.4",
    rust_edition: "2021",
};

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rustisaur {} (Lua {}, Rust edition {})",
            self.version, self.lua_version, self.rust_edition
        )
    }
}
