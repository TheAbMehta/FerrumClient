pub use azalea_protocol::packets::config::ClientboundConfigPacket as ConfigPacket;
pub use azalea_protocol::packets::game::c_level_chunk_with_light::ClientboundLevelChunkWithLight as ChunkDataPacket;
pub use azalea_protocol::packets::game::ClientboundGamePacket as GamePacket;
pub use azalea_protocol::packets::login::ClientboundLoginPacket as LoginPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Config,
    Play,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionStateError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ProtocolState,
        to: ProtocolState,
    },
}

pub struct ConnectionState {
    current: ProtocolState,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self {
            current: ProtocolState::Handshake,
        }
    }

    pub fn current(&self) -> ProtocolState {
        self.current
    }

    pub fn transition_to_login(&mut self) -> Result<(), ConnectionStateError> {
        match self.current {
            ProtocolState::Handshake => {
                self.current = ProtocolState::Login;
                Ok(())
            }
            _ => Err(ConnectionStateError::InvalidTransition {
                from: self.current,
                to: ProtocolState::Login,
            }),
        }
    }

    pub fn transition_to_status(&mut self) -> Result<(), ConnectionStateError> {
        match self.current {
            ProtocolState::Handshake => {
                self.current = ProtocolState::Status;
                Ok(())
            }
            _ => Err(ConnectionStateError::InvalidTransition {
                from: self.current,
                to: ProtocolState::Status,
            }),
        }
    }

    pub fn transition_to_config(&mut self) -> Result<(), ConnectionStateError> {
        match self.current {
            ProtocolState::Login => {
                self.current = ProtocolState::Config;
                Ok(())
            }
            _ => Err(ConnectionStateError::InvalidTransition {
                from: self.current,
                to: ProtocolState::Config,
            }),
        }
    }

    pub fn transition_to_play(&mut self) -> Result<(), ConnectionStateError> {
        match self.current {
            ProtocolState::Config => {
                self.current = ProtocolState::Play;
                Ok(())
            }
            _ => Err(ConnectionStateError::InvalidTransition {
                from: self.current,
                to: ProtocolState::Play,
            }),
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::new()
    }
}
