use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(#[from] std::io::Error),
    
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    
    #[error("Login failed: {0}")]
    LoginFailed(String),
}

pub struct MinecraftConnection {
    address: SocketAddr,
}

impl MinecraftConnection {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }
    
    pub async fn connect(&self) -> Result<(), ConnectionError> {
        Ok(())
    }
}
