use azalea_protocol::packets::game::{
    s_chunk_batch_received::ServerboundChunkBatchReceived,
    s_keep_alive::ServerboundKeepAlive as GameServerboundKeepAlive,
    ClientboundGamePacket,
};
use bevy::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::Instant;

use super::connection::ReceivedChunks;

/// Resource holding channels for the persistent server connection
#[derive(Resource)]
pub struct ServerConnection {
    pub player_id: i32,
    pub packet_receiver: UnboundedReceiver<ClientboundGamePacket>,
    pub last_keepalive: Instant,
}

impl ServerConnection {
    pub fn new(player_id: i32, packet_receiver: UnboundedReceiver<ClientboundGamePacket>) -> Self {
        Self {
            player_id,
            packet_receiver,
            last_keepalive: Instant::now(),
        }
    }
}

/// Channel to send outgoing packets to server
#[derive(Resource, Clone)]
pub struct ServerPacketSender {
    pub sender: UnboundedSender<ClientboundGamePacket>,
}

/// Bevy system that continuously reads packets from the server
pub fn handle_incoming_packets(
    mut server_conn: Option<ResMut<ServerConnection>>,
    mut received_chunks: ResMut<ReceivedChunks>,
) {
    let Some(ref mut server_conn) = server_conn else {
        return;
    };

    // Try to receive packets (non-blocking)
    while let Ok(packet) = server_conn.packet_receiver.try_recv() {
        match packet {
            ClientboundGamePacket::KeepAlive(keep_alive) => {
                info!("Received KeepAlive: {}", keep_alive.id);
                server_conn.last_keepalive = Instant::now();
                // Send keepalive response via sender (implementation needed)
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
                // Send acknowledgment (implementation needed)
            }
            ClientboundGamePacket::Disconnect(disconnect) => {
                warn!("Server disconnected: {:?}", disconnect.reason);
                // TODO: Handle disconnection properly
            }
            _ => {
                trace!("Unhandled packet: {:?}", std::any::type_name_of_val(&packet));
            }
        }
    }
}

/// Plugin for persistent server connection management
pub struct PersistentConnectionPlugin;

impl Plugin for PersistentConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_incoming_packets);
    }
}
