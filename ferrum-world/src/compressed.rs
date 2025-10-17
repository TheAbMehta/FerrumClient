use ferrum_core::BlockId;

const CHUNK_SIZE: usize = 32;
const TOTAL_BLOCKS: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Palette-compressed chunk storage.
///
/// Maps unique block IDs to small palette indices, then packs indices using
/// variable-width bit encoding (0/1/2/4/8/16 bits per block based on palette size).
///
/// Memory per chunk (32³ blocks):
/// - 1 unique block: ~26 bytes (single-value optimization, 0 bpb)
/// - 2 blocks: ~4.1 KB (1 bpb)
/// - 3-4 blocks: ~8.2 KB (2 bpb)
/// - 5-16 blocks: ~16.4 KB (4 bpb)
/// - 17-256 blocks: ~33.3 KB (8 bpb)
/// - 257+ blocks: ~65.5 KB (16 bpb, uncompressed fallback)
pub struct CompressedChunk {
    palette: Vec<BlockId>,
    data: Vec<u64>,
    bits_per_block: u8,
}

impl CompressedChunk {
    pub fn new() -> Self {
        Self {
            palette: vec![BlockId::new(0)],
            data: Vec::new(),
            bits_per_block: 0,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return BlockId::new(0);
        }

        let index = block_index(x, y, z);
        let palette_idx = self.get_palette_index(index);
        self.palette[palette_idx]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_id: BlockId) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }

        let palette_idx = match self.palette.iter().position(|&b| b == block_id) {
            Some(idx) => idx,
            None => {
                self.palette.push(block_id);
                let new_idx = self.palette.len() - 1;

                let required_bpb = bits_needed(self.palette.len());
                if required_bpb > self.bits_per_block {
                    self.resize_storage(required_bpb);
                }

                new_idx
            }
        };

        let index = block_index(x, y, z);
        self.set_palette_index(index, palette_idx);
    }

    pub fn memory_usage(&self) -> usize {
        let struct_size = std::mem::size_of::<Self>();
        let palette_heap = self.palette.capacity() * std::mem::size_of::<BlockId>();
        let data_heap = self.data.capacity() * std::mem::size_of::<u64>();
        struct_size + palette_heap + data_heap
    }

    pub fn palette_size(&self) -> usize {
        self.palette.len()
    }

    pub fn bits_per_block(&self) -> u8 {
        self.bits_per_block
    }

    pub fn from_blocks(blocks: &[BlockId; TOTAL_BLOCKS]) -> Self {
        let mut palette: Vec<BlockId> = Vec::new();
        let mut indices = [0u16; TOTAL_BLOCKS];

        for (i, &block) in blocks.iter().enumerate() {
            let idx = match palette.iter().position(|&b| b == block) {
                Some(idx) => idx,
                None => {
                    palette.push(block);
                    palette.len() - 1
                }
            };
            indices[i] = idx as u16;
        }

        let bpb = bits_needed(palette.len());
        let data = if bpb == 0 {
            Vec::new()
        } else {
            pack_indices(&indices, bpb)
        };

        Self {
            palette,
            data,
            bits_per_block: bpb,
        }
    }

    fn get_palette_index(&self, block_idx: usize) -> usize {
        if self.bits_per_block == 0 {
            return 0;
        }

        let bpb = self.bits_per_block as usize;
        let indices_per_u64 = 64 / bpb;
        let word_idx = block_idx / indices_per_u64;
        let bit_offset = (block_idx % indices_per_u64) * bpb;
        let mask = (1u64 << bpb) - 1;

        ((self.data[word_idx] >> bit_offset) & mask) as usize
    }

    fn set_palette_index(&mut self, block_idx: usize, palette_idx: usize) {
        if self.bits_per_block == 0 {
            return;
        }

        let bpb = self.bits_per_block as usize;
        let indices_per_u64 = 64 / bpb;
        let word_idx = block_idx / indices_per_u64;
        let bit_offset = (block_idx % indices_per_u64) * bpb;
        let mask = (1u64 << bpb) - 1;

        self.data[word_idx] &= !(mask << bit_offset);
        self.data[word_idx] |= (palette_idx as u64 & mask) << bit_offset;
    }

    fn resize_storage(&mut self, new_bpb: u8) {
        let old_bpb = self.bits_per_block;

        if old_bpb == new_bpb {
            return;
        }

        let mut indices = [0u16; TOTAL_BLOCKS];
        if old_bpb > 0 {
            let old_bpb_usize = old_bpb as usize;
            let old_per_u64 = 64 / old_bpb_usize;
            let old_mask = (1u64 << old_bpb_usize) - 1;

            for i in 0..TOTAL_BLOCKS {
                let word_idx = i / old_per_u64;
                let bit_offset = (i % old_per_u64) * old_bpb_usize;
                indices[i] = ((self.data[word_idx] >> bit_offset) & old_mask) as u16;
            }
        }

        self.bits_per_block = new_bpb;
        self.data = pack_indices(&indices, new_bpb);
    }
}

impl Default for CompressedChunk {
    fn default() -> Self {
        Self::new()
    }
}

#[inline(always)]
fn block_index(x: usize, y: usize, z: usize) -> usize {
    x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z
}

fn bits_needed(palette_size: usize) -> u8 {
    match palette_size {
        0 | 1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=16 => 4,
        17..=256 => 8,
        _ => 16,
    }
}

fn pack_indices(indices: &[u16; TOTAL_BLOCKS], bpb: u8) -> Vec<u64> {
    if bpb == 0 {
        return Vec::new();
    }

    let bpb = bpb as usize;
    let indices_per_u64 = 64 / bpb;
    let num_words = (TOTAL_BLOCKS + indices_per_u64 - 1) / indices_per_u64;
    let mut data = vec![0u64; num_words];
    let mask = (1u64 << bpb) - 1;

    for (i, &idx) in indices.iter().enumerate() {
        let word_idx = i / indices_per_u64;
        let bit_offset = (i % indices_per_u64) * bpb;
        data[word_idx] |= (idx as u64 & mask) << bit_offset;
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chunk_is_all_air() {
        let chunk = CompressedChunk::new();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    assert_eq!(chunk.get_block(x, y, z), BlockId::new(0));
                }
            }
        }
    }

    #[test]
    fn test_single_value_memory() {
        let chunk = CompressedChunk::new();
        let usage = chunk.memory_usage();
        assert!(usage < 100, "Single-value chunk used {} bytes", usage);
    }

    #[test]
    fn test_set_and_get_block() {
        let mut chunk = CompressedChunk::new();
        let stone = BlockId::new(1);

        chunk.set_block(5, 10, 15, stone);
        assert_eq!(chunk.get_block(5, 10, 15), stone);

        assert_eq!(chunk.get_block(0, 0, 0), BlockId::new(0));
        assert_eq!(chunk.get_block(31, 31, 31), BlockId::new(0));
    }

    #[test]
    fn test_out_of_bounds_returns_air() {
        let chunk = CompressedChunk::new();
        assert_eq!(chunk.get_block(32, 0, 0), BlockId::new(0));
        assert_eq!(chunk.get_block(0, 32, 0), BlockId::new(0));
        assert_eq!(chunk.get_block(0, 0, 32), BlockId::new(0));
    }

    #[test]
    fn test_out_of_bounds_set_is_noop() {
        let mut chunk = CompressedChunk::new();
        chunk.set_block(32, 0, 0, BlockId::new(1));
        assert_eq!(chunk.palette_size(), 1);
    }

    #[test]
    fn test_palette_grows_correctly() {
        let mut chunk = CompressedChunk::new();

        assert_eq!(chunk.bits_per_block(), 0);
        assert_eq!(chunk.palette_size(), 1);

        chunk.set_block(0, 0, 0, BlockId::new(1));
        assert_eq!(chunk.bits_per_block(), 1);
        assert_eq!(chunk.palette_size(), 2);

        chunk.set_block(1, 0, 0, BlockId::new(3));
        assert_eq!(chunk.bits_per_block(), 2);
        assert_eq!(chunk.palette_size(), 3);

        chunk.set_block(2, 0, 0, BlockId::new(4));
        assert_eq!(chunk.bits_per_block(), 2);
        assert_eq!(chunk.palette_size(), 4);

        chunk.set_block(3, 0, 0, BlockId::new(9));
        assert_eq!(chunk.bits_per_block(), 4);
        assert_eq!(chunk.palette_size(), 5);

        assert_eq!(chunk.get_block(0, 0, 0), BlockId::new(1));
        assert_eq!(chunk.get_block(1, 0, 0), BlockId::new(3));
        assert_eq!(chunk.get_block(2, 0, 0), BlockId::new(4));
        assert_eq!(chunk.get_block(3, 0, 0), BlockId::new(9));
        assert_eq!(chunk.get_block(4, 0, 0), BlockId::new(0));
    }

    #[test]
    fn test_many_block_types() {
        let mut chunk = CompressedChunk::new();

        for i in 1..=16 {
            chunk.set_block(i, 0, 0, BlockId::new(i as u16));
        }
        assert_eq!(chunk.palette_size(), 17);
        assert_eq!(chunk.bits_per_block(), 8);

        for i in 1..=16 {
            assert_eq!(chunk.get_block(i, 0, 0), BlockId::new(i as u16));
        }
    }

    #[test]
    fn test_overwrite_block() {
        let mut chunk = CompressedChunk::new();
        let stone = BlockId::new(1);
        let dirt = BlockId::new(3);

        chunk.set_block(5, 5, 5, stone);
        assert_eq!(chunk.get_block(5, 5, 5), stone);

        chunk.set_block(5, 5, 5, dirt);
        assert_eq!(chunk.get_block(5, 5, 5), dirt);
    }

    #[test]
    fn test_from_blocks() {
        let mut blocks = [BlockId::new(0); TOTAL_BLOCKS];

        blocks[block_index(0, 0, 0)] = BlockId::new(1);
        blocks[block_index(5, 10, 15)] = BlockId::new(3);
        blocks[block_index(31, 31, 31)] = BlockId::new(7);

        let chunk = CompressedChunk::from_blocks(&blocks);

        assert_eq!(chunk.get_block(0, 0, 0), BlockId::new(1));
        assert_eq!(chunk.get_block(5, 10, 15), BlockId::new(3));
        assert_eq!(chunk.get_block(31, 31, 31), BlockId::new(7));
        assert_eq!(chunk.get_block(1, 1, 1), BlockId::new(0));
    }

    #[test]
    fn test_roundtrip_all_blocks() {
        let mut chunk = CompressedChunk::new();
        let block_types = [
            BlockId::new(0),
            BlockId::new(1),
            BlockId::new(3),
            BlockId::new(7),
        ];

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let idx = (x + y + z) % block_types.len();
                    chunk.set_block(x, y, z, block_types[idx]);
                }
            }
        }

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let expected = block_types[(x + y + z) % block_types.len()];
                    assert_eq!(
                        chunk.get_block(x, y, z),
                        expected,
                        "Mismatch at ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_memory_usage_typical_terrain() {
        let mut chunk = CompressedChunk::new();
        let types: Vec<BlockId> = (0..8).map(|i| BlockId::new(i)).collect();

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    chunk.set_block(x, y, z, types[(x * 7 + y * 3 + z) % types.len()]);
                }
            }
        }

        let usage = chunk.memory_usage();
        assert!(
            usage < 20_000,
            "Typical terrain chunk used {} bytes (expected <20KB)",
            usage
        );
    }

    #[test]
    fn test_memory_target_64_chunks() {
        let mut total_memory = 0usize;

        for _ in 0..64 {
            let mut chunk = CompressedChunk::new();
            let types: Vec<BlockId> = (0..8).map(|i| BlockId::new(i)).collect();
            for x in 0..32 {
                for y in 0..32 {
                    for z in 0..32 {
                        chunk.set_block(x, y, z, types[(x * 7 + y * 3 + z) % types.len()]);
                    }
                }
            }
            total_memory += chunk.memory_usage();
        }

        let two_gb = 2 * 1024 * 1024 * 1024;
        assert!(
            total_memory < two_gb,
            "64 chunks used {} bytes ({:.2} MB), exceeds 2GB",
            total_memory,
            total_memory as f64 / (1024.0 * 1024.0)
        );

        assert!(
            total_memory < 2_000_000,
            "64 chunks used {} bytes ({:.2} MB), expected <2MB",
            total_memory,
            total_memory as f64 / (1024.0 * 1024.0)
        );
    }

    #[test]
    fn test_memory_target_262k_chunks() {
        // Realistic world: ~70% air-only, ~20% two-type (air+stone), ~10% terrain (5-8 types)
        let air_chunk = CompressedChunk::new();

        let mut two_type_chunk = CompressedChunk::new();
        for x in 0..32 {
            for y in 0..16 {
                for z in 0..32 {
                    two_type_chunk.set_block(x, y, z, BlockId::new(1));
                }
            }
        }

        let mut terrain_chunk = CompressedChunk::new();
        let types: Vec<BlockId> = (0..8).map(|i| BlockId::new(i)).collect();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    terrain_chunk.set_block(x, y, z, types[(x * 7 + y * 3 + z) % types.len()]);
                }
            }
        }

        let total_chunks: u64 = 262_144;
        let total_estimated = (air_chunk.memory_usage() as u64 * (total_chunks * 70 / 100))
            + (two_type_chunk.memory_usage() as u64 * (total_chunks * 20 / 100))
            + (terrain_chunk.memory_usage() as u64 * (total_chunks * 10 / 100));
        let two_gb: u64 = 2 * 1024 * 1024 * 1024;

        assert!(
            total_estimated < two_gb,
            "262K chunks estimated at {} bytes ({:.2} GB), exceeds 2GB",
            total_estimated,
            total_estimated as f64 / (1024.0 * 1024.0 * 1024.0)
        );
    }

    #[test]
    fn test_bits_needed() {
        assert_eq!(bits_needed(0), 0);
        assert_eq!(bits_needed(1), 0);
        assert_eq!(bits_needed(2), 1);
        assert_eq!(bits_needed(3), 2);
        assert_eq!(bits_needed(4), 2);
        assert_eq!(bits_needed(5), 4);
        assert_eq!(bits_needed(16), 4);
        assert_eq!(bits_needed(17), 8);
        assert_eq!(bits_needed(256), 8);
        assert_eq!(bits_needed(257), 16);
    }

    #[test]
    fn test_block_index_ordering() {
        assert_eq!(block_index(0, 0, 0), 0);
        assert_eq!(block_index(0, 0, 1), 1);
        assert_eq!(block_index(0, 1, 0), 32);
        assert_eq!(block_index(1, 0, 0), 32 * 32);
        assert_eq!(block_index(31, 31, 31), TOTAL_BLOCKS - 1);
    }

    #[test]
    fn test_uniform_chunk_compression() {
        let blocks = [BlockId::new(42); TOTAL_BLOCKS];
        let chunk = CompressedChunk::from_blocks(&blocks);
        assert_eq!(chunk.bits_per_block(), 0);
        assert_eq!(chunk.palette_size(), 1);

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    assert_eq!(chunk.get_block(x, y, z), BlockId::new(42));
                }
            }
        }

        assert!(chunk.memory_usage() < 100);
    }

    #[test]
    fn test_two_block_types_compression() {
        let mut chunk = CompressedChunk::new();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    if y < 16 {
                        chunk.set_block(x, y, z, BlockId::new(1));
                    }
                }
            }
        }

        assert_eq!(chunk.palette_size(), 2);
        assert_eq!(chunk.bits_per_block(), 1);

        let usage = chunk.memory_usage();
        assert!(
            usage < 5_000,
            "Two-type chunk used {} bytes (expected ~4.1KB)",
            usage
        );

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let expected = if y < 16 {
                        BlockId::new(1)
                    } else {
                        BlockId::new(0)
                    };
                    assert_eq!(chunk.get_block(x, y, z), expected);
                }
            }
        }
    }
}
