use azalea_core::position::ChunkSectionBlockPos;
use azalea_protocol::connect::{Connection, ConnectionError as AzaleaConnectionError};
use azalea_protocol::packets::config::{
    s_cookie_response::ServerboundCookieResponse as ConfigServerboundCookieResponse,
    s_finish_configuration::ServerboundFinishConfiguration,
    s_keep_alive::ServerboundKeepAlive as ConfigServerboundKeepAlive,
    s_pong::ServerboundPong as ConfigServerboundPong,
    s_select_known_packs::ServerboundSelectKnownPacks, ClientboundConfigPacket,
};
use azalea_protocol::packets::game::{
    s_accept_teleportation::ServerboundAcceptTeleportation,
    s_chunk_batch_received::ServerboundChunkBatchReceived,
    s_keep_alive::ServerboundKeepAlive as GameServerboundKeepAlive,
    s_player_loaded::ServerboundPlayerLoaded, ClientboundGamePacket,
};
use azalea_protocol::packets::handshake::{
    s_intention::ServerboundIntention, ClientboundHandshakePacket, ServerboundHandshakePacket,
};
use azalea_protocol::packets::login::{
    s_cookie_response::ServerboundCookieResponse as LoginServerboundCookieResponse,
    s_hello::ServerboundHello, ClientboundLoginPacket,
};
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use azalea_world::chunk_storage::Chunk;
use bevy::prelude::*;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
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

    #[error("Config failed: {0}")]
    ConfigFailed(String),

    #[error("Game failed: {0}")]
    GameFailed(String),

    #[error("Packet write failed")]
    PacketWriteFailed,

    #[error("Packet read failed")]
    PacketReadFailed,

    #[error("Chunk parse failed: {0}")]
    ChunkParseFailed(String),
}

/// Storage for received chunk data from the server
#[derive(Resource, Clone, Debug)]
pub struct ReceivedChunks {
    /// Map from (chunk_x, chunk_z) to 3D block state array
    /// The array is [y][z][x] where each value is a BlockState ID
    pub chunks: HashMap<(i32, i32), Vec<Vec<Vec<u16>>>>,
    pub dimension_height: u32,
    pub min_y: i32,
}

impl ReceivedChunks {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dimension_height: 384,
            min_y: -64,
        }
    }

    /// Parse and store a chunk from raw packet data
    pub fn add_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_data: &azalea_protocol::packets::game::c_level_chunk_with_light::ClientboundLevelChunkPacketData,
    ) -> Result<(), ConnectionError> {
        // Parse the chunk using azalea-world
        let mut cursor = Cursor::new(&**chunk_data.data);
        let chunk = Chunk::read_with_dimension_height(
            &mut cursor,
            self.dimension_height,
            self.min_y,
            &chunk_data.heightmaps,
        )
        .map_err(|e| {
            ConnectionError::ChunkParseFailed(format!("Failed to parse chunk: {:?}", e))
        })?;

        // Convert to our format: Vec<Vec<Vec<u16>>> [y][z][x]
        let sections_count = (self.dimension_height / 16) as usize;
        let mut block_data = vec![vec![vec![0u16; 16]; 16]; self.dimension_height as usize];

        for section_idx in 0..sections_count {
            if section_idx >= chunk.sections.len() {
                break;
            }
            let section = &chunk.sections[section_idx];

            for local_y in 0..16 {
                for local_z in 0..16 {
                    for local_x in 0..16 {
                        let pos = ChunkSectionBlockPos::new(local_x, local_y, local_z);
                        let block_state = section.states.get(pos);
                        let block_id = block_state.id();

                        let world_y = (section_idx * 16 + local_y as usize) as usize;
                        if world_y < block_data.len() {
                            block_data[world_y][local_z as usize][local_x as usize] = block_id;
                        }
                    }
                }
            }
        }

        self.chunks.insert((chunk_x, chunk_z), block_data);
        info!(
            "Stored chunk ({}, {}) with {} sections",
            chunk_x, chunk_z, sections_count
        );
        Ok(())
    }
}

impl Default for ReceivedChunks {
    fn default() -> Self {
        Self::new()
    }
}

/// Connect to a Minecraft server and play through the full protocol flow
pub async fn connect_and_play(address: String) -> Result<ReceivedChunks, ConnectionError> {
    info!("Starting connection to {}", address);

    // Resolve address
    let socket_addr = address
        .to_socket_addrs()
        .map_err(|e| ConnectionError::ConnectionFailed(e))?
        .next()
        .ok_or_else(|| ConnectionError::HandshakeFailed("Failed to resolve address".to_string()))?;

    // Phase 1: Handshake
    info!("Phase 1: Handshake");
    let mut conn =
        Connection::<ClientboundHandshakePacket, ServerboundHandshakePacket>::new(&socket_addr)
            .await?;

    conn.write(ServerboundIntention {
        protocol_version: PROTOCOL_VERSION,
        hostname: socket_addr.ip().to_string(),
        port: socket_addr.port(),
        intention: ClientIntention::Login,
    })
    .await
    .map_err(|_| ConnectionError::PacketWriteFailed)?;

    // Phase 2: Login
    info!("Phase 2: Login");
    let mut conn = conn.login();

    conn.write(ServerboundHello {
        name: "FerrumBot".to_string(),
        profile_id: Uuid::nil(),
    })
    .await
    .map_err(|_| ConnectionError::PacketWriteFailed)?;

    // Handle login packets
    loop {
        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundLoginPacket::LoginCompression(compression) => {
                    info!(
                        "Setting compression threshold: {}",
                        compression.compression_threshold
                    );
                    conn.set_compression_threshold(compression.compression_threshold);
                }
                ClientboundLoginPacket::CookieRequest(cookie_req) => {
                    info!("Received cookie request: {:?}", cookie_req.key);
                    conn.write(LoginServerboundCookieResponse {
                        key: cookie_req.key.clone(),
                        payload: None,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundLoginPacket::LoginFinished(profile) => {
                    info!(
                        "Login finished for profile: {:?}",
                        profile.game_profile.name
                    );
                    break;
                }
                ClientboundLoginPacket::LoginDisconnect(disconnect) => {
                    return Err(ConnectionError::LoginFailed(format!(
                        "Server disconnected: {:?}",
                        disconnect.reason
                    )));
                }
                _ => {
                    debug!("Unhandled login packet: {:?}", packet);
                }
            },
            Err(_) => {
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    // Phase 3: Config
    info!("Phase 3: Config");
    let mut conn = conn.config();

    loop {
        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundConfigPacket::SelectKnownPacks(packs) => {
                    info!(
                        "Received SelectKnownPacks with {} packs",
                        packs.known_packs.len()
                    );
                    conn.write(ServerboundSelectKnownPacks {
                        known_packs: vec![],
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::RegistryData(registry) => {
                    info!("Received RegistryData: {:?}", registry.registry_id);
                }
                ClientboundConfigPacket::KeepAlive(keep_alive) => {
                    debug!("Config KeepAlive: {}", keep_alive.id);
                    conn.write(ConfigServerboundKeepAlive { id: keep_alive.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::Ping(ping) => {
                    debug!("Config Ping: {}", ping.id);
                    conn.write(ConfigServerboundPong { id: ping.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::FinishConfiguration(_) => {
                    info!("Received FinishConfiguration, transitioning to game");
                    conn.write(ServerboundFinishConfiguration {})
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                    break;
                }
                ClientboundConfigPacket::CookieRequest(cookie_req) => {
                    info!("Config cookie request: {:?}", cookie_req.key);
                    conn.write(ConfigServerboundCookieResponse {
                        key: cookie_req.key.clone(),
                        payload: None,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                _ => {
                    debug!(
                        "Unhandled config packet: {:?}",
                        std::any::type_name_of_val(&packet)
                    );
                }
            },
            Err(_) => {
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    // Phase 4: Game
    info!("Phase 4: Game");
    let mut conn = conn.game();
    let mut received_chunks = ReceivedChunks::new();
    let start_time = Instant::now();
    let collection_duration = Duration::from_secs(5);

    loop {
        // Stop after collecting chunks for 5 seconds
        if start_time.elapsed() > collection_duration {
            info!(
                "Collected {} chunks in 5 seconds, stopping",
                received_chunks.chunks.len()
            );
            break;
        }

        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundGamePacket::Login(login) => {
                    info!(
                        "Game Login: entity_id={}, chunk_radius={}",
                        login.player_id, login.chunk_radius
                    );
                    conn.write(ServerboundPlayerLoaded {})
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::PlayerPosition(pos) => {
                    info!("PlayerPosition: teleport_id={}", pos.id);
                    conn.write(ServerboundAcceptTeleportation { id: pos.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::KeepAlive(keep_alive) => {
                    debug!("Game KeepAlive: {}", keep_alive.id);
                    conn.write(GameServerboundKeepAlive { id: keep_alive.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::SetChunkCacheCenter(center) => {
                    info!("SetChunkCacheCenter: ({}, {})", center.x, center.z);
                }
                ClientboundGamePacket::ChunkBatchStart(_) => {
                    info!("ChunkBatchStart");
                }
                ClientboundGamePacket::LevelChunkWithLight(chunk_packet) => {
                    info!("Received chunk at ({}, {})", chunk_packet.x, chunk_packet.z);
                    if let Err(e) = received_chunks.add_chunk(
                        chunk_packet.x,
                        chunk_packet.z,
                        &chunk_packet.chunk_data,
                    ) {
                        warn!("Failed to parse chunk: {}", e);
                    }
                }
                ClientboundGamePacket::ChunkBatchFinished(batch) => {
                    info!("ChunkBatchFinished: batch_size={}", batch.batch_size);
                    conn.write(ServerboundChunkBatchReceived {
                        desired_chunks_per_tick: 5.0,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                _ => {
                    // Log other packets but don't crash
                    trace!(
                        "Unhandled game packet: {:?}",
                        std::any::type_name_of_val(&packet)
                    );
                }
            },
            Err(e) => {
                warn!("Error reading game packet: {:?}", e);
                // Don't immediately fail, server might have closed connection after sending chunks
                if start_time.elapsed() > Duration::from_secs(1) {
                    break;
                }
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    info!(
        "Connection complete, received {} chunks",
        received_chunks.chunks.len()
    );
    Ok(received_chunks)
}

/// Connect to server and return a persistent connection along with initial chunks
/// This function goes through the full handshake/login/config flow, waits for initial
/// chunks, then returns the active connection for continuous use.
pub async fn connect_persistent(
    address: String,
) -> Result<
    (
        Box<dyn std::any::Any + Send>,
        ReceivedChunks,
        i32,
    ),
    ConnectionError,
> {
    info!("Starting persistent connection to {}", address);

    // Resolve address
    let socket_addr = address
        .to_socket_addrs()
        .map_err(|e| ConnectionError::ConnectionFailed(e))?
        .next()
        .ok_or_else(|| ConnectionError::HandshakeFailed("Failed to resolve address".to_string()))?;

    // Phase 1: Handshake
    info!("Phase 1: Handshake");
    let mut conn =
        Connection::<ClientboundHandshakePacket, ServerboundHandshakePacket>::new(&socket_addr)
            .await?;

    conn.write(ServerboundIntention {
        protocol_version: PROTOCOL_VERSION,
        hostname: socket_addr.ip().to_string(),
        port: socket_addr.port(),
        intention: ClientIntention::Login,
    })
    .await
    .map_err(|_| ConnectionError::PacketWriteFailed)?;

    // Phase 2: Login
    info!("Phase 2: Login");
    let mut conn = conn.login();

    conn.write(ServerboundHello {
        name: "FerrumBot".to_string(),
        profile_id: Uuid::nil(),
    })
    .await
    .map_err(|_| ConnectionError::PacketWriteFailed)?;

    // Handle login packets
    loop {
        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundLoginPacket::LoginCompression(compression) => {
                    info!(
                        "Setting compression threshold: {}",
                        compression.compression_threshold
                    );
                    conn.set_compression_threshold(compression.compression_threshold);
                }
                ClientboundLoginPacket::CookieRequest(cookie_req) => {
                    info!("Received cookie request: {:?}", cookie_req.key);
                    conn.write(LoginServerboundCookieResponse {
                        key: cookie_req.key.clone(),
                        payload: None,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundLoginPacket::LoginFinished(profile) => {
                    info!(
                        "Login finished for profile: {:?}",
                        profile.game_profile.name
                    );
                    break;
                }
                ClientboundLoginPacket::LoginDisconnect(disconnect) => {
                    return Err(ConnectionError::LoginFailed(format!(
                        "Server disconnected: {:?}",
                        disconnect.reason
                    )));
                }
                _ => {
                    debug!("Unhandled login packet: {:?}", packet);
                }
            },
            Err(_) => {
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    // Phase 3: Config
    info!("Phase 3: Config");
    let mut conn = conn.config();

    loop {
        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundConfigPacket::SelectKnownPacks(packs) => {
                    info!(
                        "Received SelectKnownPacks with {} packs",
                        packs.known_packs.len()
                    );
                    conn.write(ServerboundSelectKnownPacks {
                        known_packs: vec![],
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::RegistryData(registry) => {
                    info!("Received RegistryData: {:?}", registry.registry_id);
                }
                ClientboundConfigPacket::KeepAlive(keep_alive) => {
                    debug!("Config KeepAlive: {}", keep_alive.id);
                    conn.write(ConfigServerboundKeepAlive { id: keep_alive.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::Ping(ping) => {
                    debug!("Config Ping: {}", ping.id);
                    conn.write(ConfigServerboundPong { id: ping.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundConfigPacket::FinishConfiguration(_) => {
                    info!("Received FinishConfiguration, transitioning to game");
                    conn.write(ServerboundFinishConfiguration {})
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                    break;
                }
                ClientboundConfigPacket::CookieRequest(cookie_req) => {
                    info!("Config cookie request: {:?}", cookie_req.key);
                    conn.write(ConfigServerboundCookieResponse {
                        key: cookie_req.key.clone(),
                        payload: None,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                _ => {
                    debug!(
                        "Unhandled config packet: {:?}",
                        std::any::type_name_of_val(&packet)
                    );
                }
            },
            Err(_) => {
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    // Phase 4: Game - Collect initial chunks but keep connection alive
    info!("Phase 4: Game");
    let mut conn = conn.game();
    let mut received_chunks = ReceivedChunks::new();
    let start_time = Instant::now();
    let initial_collection_duration = Duration::from_secs(3);
    let mut player_id = 0i32;

    // Collect initial chunks
    loop {
        if start_time.elapsed() > initial_collection_duration {
            info!(
                "Initial chunk collection complete ({} chunks), keeping connection alive",
                received_chunks.chunks.len()
            );
            break;
        }

        match conn.read().await {
            Ok(packet) => match packet {
                ClientboundGamePacket::Login(login) => {
                    player_id = login.player_id.0;
                    info!(
                        "Game Login: entity_id={}, chunk_radius={}",
                        login.player_id, login.chunk_radius
                    );
                    conn.write(ServerboundPlayerLoaded {})
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::PlayerPosition(pos) => {
                    info!("PlayerPosition: teleport_id={}", pos.id);
                    conn.write(ServerboundAcceptTeleportation { id: pos.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::KeepAlive(keep_alive) => {
                    debug!("Game KeepAlive: {}", keep_alive.id);
                    conn.write(GameServerboundKeepAlive { id: keep_alive.id })
                        .await
                        .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                ClientboundGamePacket::LevelChunkWithLight(chunk_packet) => {
                    info!("Received chunk at ({}, {})", chunk_packet.x, chunk_packet.z);
                    if let Err(e) = received_chunks.add_chunk(
                        chunk_packet.x,
                        chunk_packet.z,
                        &chunk_packet.chunk_data,
                    ) {
                        warn!("Failed to parse chunk: {}", e);
                    }
                }
                ClientboundGamePacket::ChunkBatchFinished(batch) => {
                    info!("ChunkBatchFinished: batch_size={}", batch.batch_size);
                    conn.write(ServerboundChunkBatchReceived {
                        desired_chunks_per_tick: 5.0,
                    })
                    .await
                    .map_err(|_| ConnectionError::PacketWriteFailed)?;
                }
                _ => {
                    trace!(
                        "Unhandled game packet: {:?}",
                        std::any::type_name_of_val(&packet)
                    );
                }
            },
            Err(e) => {
                warn!("Error reading game packet: {:?}", e);
                if start_time.elapsed() > Duration::from_secs(1) {
                    break;
                }
                return Err(ConnectionError::PacketReadFailed);
            }
        }
    }

    info!(
        "Returning persistent connection with {} initial chunks",
        received_chunks.chunks.len()
    );
    Ok((Box::new(conn), received_chunks, player_id))
}
