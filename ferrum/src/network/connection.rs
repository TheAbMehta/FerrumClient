use azalea_protocol::connect::{Connection, ConnectionError as AzaleaConnectionError};
use azalea_protocol::packets::handshake::{
    s_intention::ServerboundIntention, ClientboundHandshakePacket, ServerboundHandshakePacket,
};
use azalea_protocol::packets::login::{s_hello::ServerboundHello, ClientboundLoginPacket};
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use std::net::SocketAddr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(#[from] std::io::Error),

    #[error("Azalea connection error: {0}")]
    AzaleaError(#[from] AzaleaConnectionError),

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("Packet write failed")]
    PacketWriteFailed,

    #[error("Packet read failed")]
    PacketReadFailed,
}

pub struct MinecraftConnection {
    address: SocketAddr,
}

impl MinecraftConnection {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub async fn connect(&self) -> Result<(), ConnectionError> {
        let mut conn = Connection::<ClientboundHandshakePacket, ServerboundHandshakePacket>::new(
            &self.address,
        )
        .await?;

        conn.write(ServerboundIntention {
            protocol_version: PROTOCOL_VERSION,
            hostname: self.address.ip().to_string(),
            port: self.address.port(),
            intention: ClientIntention::Login,
        })
        .await
        .map_err(|_| ConnectionError::PacketWriteFailed)?;

        let mut conn = conn.login();

        conn.write(ServerboundHello {
            name: "FerrumBot".to_string(),
            profile_id: Uuid::nil(),
        })
        .await
        .map_err(|_| ConnectionError::PacketWriteFailed)?;

        loop {
            match conn.read().await {
                Ok(packet) => match packet {
                    ClientboundLoginPacket::LoginFinished(_profile) => {
                        return Ok(());
                    }
                    ClientboundLoginPacket::LoginCompression(compression) => {
                        conn.set_compression_threshold(compression.compression_threshold);
                    }
                    ClientboundLoginPacket::LoginDisconnect(disconnect) => {
                        return Err(ConnectionError::LoginFailed(format!(
                            "Server disconnected: {:?}",
                            disconnect.reason
                        )));
                    }
                    _ => {}
                },
                Err(_) => {
                    return Err(ConnectionError::PacketReadFailed);
                }
            }
        }
    }
}
