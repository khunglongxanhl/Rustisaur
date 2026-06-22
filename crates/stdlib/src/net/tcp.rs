//! TCP socket utilities (placeholder for future expansion).

use std::net::SocketAddr;

/// TCP connection info.
#[derive(Debug, Clone)]
pub struct TcpEndpoint {
    pub addr: SocketAddr,
}

impl TcpEndpoint {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
