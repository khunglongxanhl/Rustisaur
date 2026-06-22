//! UDP socket utilities (placeholder for future expansion).

use std::net::SocketAddr;

/// UDP endpoint info.
#[derive(Debug, Clone)]
pub struct UdpEndpoint {
    pub addr: SocketAddr,
}

impl UdpEndpoint {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
