use azalea_protocol::connect::Connection;
use azalea_protocol::packets::game::{
    s_accept_teleportation::ServerboundAcceptTeleportation,
    s_chunk_batch_received::ServerboundChunkBatchReceived,
    s_keep_alive::ServerboundKeepAlive as GameServerboundKeepAlive,
    s_player_loaded::ServerboundPlayerLoaded,
    s_set_carried_item::ServerboundSetCarriedItem,
    ClientboundGamePacket, ServerboundGamePacket,
};
use bevy::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use super::connection::{ConnectionError, ReceivedChunks};

/// Resource holding the persistent server connection
#[derive(Resource)]
pub struct ServerConnection {
    pub connection: Arc<Mutex<Connection<ClientboundGamePacket, ServerboundGamePacket>>>,
    pub player_id: i32,
    pub last_keepalive: Instant,
}

impl ServerConnection {
    pub fn new(
        connection: Connection<ClientboundGamePacket, ServerboundGamePacket>,
        player_id: i32,
    ) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
            player_id,
            last_keepalive: Instant::now(),
        }
    }

    /// Send a packet to the server
    pub async fn send_packet(
        &self,
        packet: ServerboundGamePacket,
    ) -> Result<(), ConnectionError> {
        let mut conn = self.connection.lock().await;
        conn.write(packet)
            .await
            .map_err(|_| ConnectionError::PacketWriteFailed)
    }
}

/// Bevy system that continuously reads packets from the server
pub fn handle_incoming_packets(
    server_conn: Option<ResMut<ServerConnection>>,
    mut received_chunks: ResMut<ReceivedChunks>,
    mut commands: Commands,
) {
    let Some(mut server_conn) = server_conn else {
        return;
    };

    // Spawn async task to read packets (non-blocking)
    let conn_clone = server_conn.connection.clone();
    let runtime = tokio::runtime::Handle::current();

    runtime.spawn(async move {
        let mut conn = conn_clone.lock().await;

        // Try to read a packet (with timeout to avoid blocking)
        let result = tokio::time::timeout(Duration::from_millis(10), conn.read()).await;

        match result {
            Ok(Ok(packet)) => {
                // Successfully received a packet
                match packet {
                    ClientboundGamePacket::KeepAlive(keep_alive) => {
                        info!("Received KeepAlive: {}", keep_alive.id);
                        // Send keepalive response
                        if let Err(e) = conn
                            .write(GameServerboundKeepAlive { id: keep_alive.id })
                            .await
                        {
                            error!("Failed to send keepalive: {:?}", e);
                        }
                    }
                    ClientboundGamePacket::LevelChunkWithLight(chunk_packet) => {
                        info!("Received chunk at ({}, {})", chunk_packet.x, chunk_packet.z);
                        // Process chunk packet (you'll need to pass received_chunks in properly)
                    }
                    ClientboundGamePacket::ChunkBatchFinished(batch) => {
                        info!("ChunkBatchFinished: batch_size={}", batch.batch_size);
                        if let Err(e) = conn
                            .write(ServerboundChunkBatchReceived {
                                desired_chunks_per_tick: 5.0,
                            })
                            .await
                        {
                            error!("Failed to acknowledge chunk batch: {:?}", e);
                        }
                    }
                    ClientboundGamePacket::Disconnect(disconnect) => {
                        warn!("Server disconnected: {:?}", disconnect.reason);
                        // TODO: Remove ServerConnection resource
                    }
                    _ => {
                        trace!("Unhandled packet: {:?}", std::any::type_name_of_val(&packet));
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Error reading packet: {:?}", e);
            }
            Err(_) => {
                // Timeout - no packet available, this is normal
            }
        }
    });

    server_conn.last_keepalive = Instant::now();
}

/// Plugin for persistent server connection management
pub struct PersistentConnectionPlugin;

impl Plugin for PersistentConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_incoming_packets);
    }
}
