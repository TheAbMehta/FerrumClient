use super::ConnectionError;
use std::net::{SocketAddr, ToSocketAddrs};

pub async fn perform_handshake(
    server_address: String,
    _port: u16,
) -> Result<SocketAddr, ConnectionError> {
    // server_address already contains the port (e.g., "127.0.0.1:25565")
    let addr = server_address
        .to_socket_addrs()
        .map_err(|e| ConnectionError::ConnectionFailed(e))?
        .next()
        .ok_or_else(|| ConnectionError::HandshakeFailed("Failed to resolve address".to_string()))?;

    Ok(addr)
}
