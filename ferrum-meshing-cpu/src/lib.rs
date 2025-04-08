pub mod binary_greedy;

pub use ferrum_meshing_gpu::{CHUNK_SIZE, CHUNK_SIZE_CB, CHUNK_SIZE_SQ};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Right, // +X
    Left,  // -X
    Up,    // +Y
    Down,  // -Y
    Front, // +Z
    Back,  // -Z
}

impl Face {
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug)]
pub struct MeshQuad {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub width: u8,
    pub height: u8,
    pub face: Face,
    pub block_type: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ChunkMesh {
    pub quads: Vec<MeshQuad>,
}

impl ChunkMesh {
    pub fn new() -> Self {
        Self { quads: Vec::new() }
    }

    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }
}

pub trait ChunkMesher: Send + Sync {
    fn mesh_chunk(&self, voxels: &[u32; CHUNK_SIZE_CB]) -> ChunkMesh;
}

pub struct CpuMesher;

impl CpuMesher {
    pub fn new() -> Self {
        Self
    }
}

impl ChunkMesher for CpuMesher {
    fn mesh_chunk(&self, voxels: &[u32; CHUNK_SIZE_CB]) -> ChunkMesh {
        binary_greedy::mesh(voxels)
    }
}

pub struct GpuMesher {
    inner: ferrum_meshing_gpu::GpuChunkMesher,
}

impl GpuMesher {
    pub fn new() -> Option<Self> {
        ferrum_meshing_gpu::GpuChunkMesher::new().map(|inner| Self { inner })
    }
}

impl ChunkMesher for GpuMesher {
    fn mesh_chunk(&self, voxels: &[u32; CHUNK_SIZE_CB]) -> ChunkMesh {
        let gpu_quads = self.inner.mesh_chunk(voxels);
        let mut mesh = ChunkMesh::new();
        for q in &gpu_quads {
            let face = match q.face() {
                0 => Face::Right,
                1 => Face::Left,
                2 => Face::Up,
                3 => Face::Down,
                4 => Face::Front,
                5 => Face::Back,
                _ => continue,
            };
            mesh.quads.push(MeshQuad {
                x: q.x() as u8,
                y: q.y() as u8,
                z: q.z() as u8,
                width: q.width() as u8,
                height: q.height() as u8,
                face,
                block_type: q.block_type,
            });
        }
        mesh
    }
}

pub fn create_mesher() -> Box<dyn ChunkMesher> {
    if let Some(gpu) = GpuMesher::new() {
        Box::new(gpu)
    } else {
        Box::new(CpuMesher::new())
    }
}

pub fn uniform_chunk(block_id: u32) -> [u32; CHUNK_SIZE_CB] {
    [block_id; CHUNK_SIZE_CB]
}

pub fn checkerboard_chunk(block_id: u32) -> [u32; CHUNK_SIZE_CB] {
    let mut voxels = [0u32; CHUNK_SIZE_CB];
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if (x + y + z) % 2 == 0 {
                    voxels[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x] = block_id;
                }
            }
        }
    }
    voxels
}

pub fn terrain_chunk() -> [u32; CHUNK_SIZE_CB] {
    let mut voxels = [0u32; CHUNK_SIZE_CB];
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = 16 + ((x * 3 + z * 7) % 5) as i32 - 2;
                if (y as i32) < height {
                    let block = if (y as i32) < height - 3 {
                        1
                    } else if (y as i32) < height - 1 {
                        2
                    } else {
                        3
                    };
                    voxels[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x] = block;
                }
            }
        }
    }
    voxels
}
