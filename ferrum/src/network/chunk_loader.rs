use ferrum_protocol::ChunkDataPacket;
use ferrum_world::{Chunk, ChunkPos, World};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChunkLoaderError {
    #[error("Failed to parse chunk data")]
    ParseError,

    #[error("Invalid chunk position: ({x}, {z})")]
    InvalidPosition { x: i32, z: i32 },
}

pub struct ChunkLoader {
    world: World,
}

impl ChunkLoader {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn load_chunk(&mut self, packet: &ChunkDataPacket) -> Result<(), ChunkLoaderError> {
        let pos = ChunkPos {
            x: packet.x,
            z: packet.z,
        };

        let chunk = Chunk::new();

        self.world.set_chunk(pos, chunk);

        Ok(())
    }

    pub fn unload_chunk(&mut self, x: i32, z: i32) -> Option<Chunk> {
        let pos = ChunkPos { x, z };
        self.world.remove_chunk(pos)
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_loader_creation() {
        let loader = ChunkLoader::new();
        assert!(loader.world().is_empty());
    }

    #[test]
    fn test_unload_chunk_directly() {
        let mut loader = ChunkLoader::new();

        let pos = ChunkPos { x: 5, z: 10 };
        loader.world_mut().set_chunk(pos, Chunk::new());
        assert!(loader.world().has_chunk(pos));

        let unloaded = loader.unload_chunk(5, 10);
        assert!(unloaded.is_some());
        assert!(!loader.world().has_chunk(pos));
    }

    #[test]
    fn test_unload_nonexistent_chunk() {
        let mut loader = ChunkLoader::new();
        let unloaded = loader.unload_chunk(99, 99);
        assert!(unloaded.is_none());
    }

    #[test]
    fn test_world_access() {
        let mut loader = ChunkLoader::new();

        loader
            .world_mut()
            .set_chunk(ChunkPos { x: 0, z: 0 }, Chunk::new());
        loader
            .world_mut()
            .set_chunk(ChunkPos { x: 1, z: 1 }, Chunk::new());

        assert_eq!(loader.world().chunk_count(), 2);
    }
}
