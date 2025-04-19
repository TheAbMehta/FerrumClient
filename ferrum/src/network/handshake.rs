use super::ConnectionError;
use std::net::{SocketAddr, ToSocketAddrs};

pub async fn perform_handshake(
    server_address: String,
    port: u16,
) -> Result<SocketAddr, ConnectionError> {
    let addr_str = format!("{}:{}", server_address, port);
    let addr = addr_str
        .to_socket_addrs()
        .map_err(|e| ConnectionError::ConnectionFailed(e))?
        .next()
        .ok_or_else(|| ConnectionError::HandshakeFailed("Failed to resolve address".to_string()))?;
    
    Ok(addr)
}
