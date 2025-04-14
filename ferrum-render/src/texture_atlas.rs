use ferrum_meshing_cpu::Face;
use std::collections::HashMap;

pub struct TextureAtlas {
    tile_size: u32,
    block_textures: HashMap<(u32, Face), (u32, u32)>,
}

impl TextureAtlas {
    pub fn new(tile_size: u32) -> Self {
        let mut block_textures = HashMap::new();

        block_textures.insert((0, Face::Up), (0, 0));
        block_textures.insert((0, Face::Down), (0, 0));
        block_textures.insert((0, Face::Right), (0, 0));
        block_textures.insert((0, Face::Left), (0, 0));
        block_textures.insert((0, Face::Front), (0, 0));
        block_textures.insert((0, Face::Back), (0, 0));

        block_textures.insert((1, Face::Up), (1, 0));
        block_textures.insert((1, Face::Down), (1, 0));
        block_textures.insert((1, Face::Right), (1, 0));
        block_textures.insert((1, Face::Left), (1, 0));
        block_textures.insert((1, Face::Front), (1, 0));
        block_textures.insert((1, Face::Back), (1, 0));

        block_textures.insert((2, Face::Up), (0, 1));
        block_textures.insert((2, Face::Down), (2, 0));
        block_textures.insert((2, Face::Right), (3, 0));
        block_textures.insert((2, Face::Left), (3, 0));
        block_textures.insert((2, Face::Front), (3, 0));
        block_textures.insert((2, Face::Back), (3, 0));

        block_textures.insert((3, Face::Up), (2, 1));
        block_textures.insert((3, Face::Down), (2, 1));
        block_textures.insert((3, Face::Right), (2, 1));
        block_textures.insert((3, Face::Left), (2, 1));
        block_textures.insert((3, Face::Front), (2, 1));
        block_textures.insert((3, Face::Back), (2, 1));

        Self {
            tile_size,
            block_textures,
        }
    }

    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    pub fn get_uvs(&self, block_type: u32, face: Face) -> [[f32; 2]; 4] {
        let (tile_x, tile_y) = self
            .block_textures
            .get(&(block_type, face))
            .copied()
            .unwrap_or((0, 0));

        let atlas_width = 16.0;
        let atlas_height = 16.0;

        let u_min = tile_x as f32 / atlas_width;
        let v_min = tile_y as f32 / atlas_height;
        let u_max = (tile_x + 1) as f32 / atlas_width;
        let v_max = (tile_y + 1) as f32 / atlas_height;

        [
            [u_min, v_max],
            [u_max, v_max],
            [u_max, v_min],
            [u_min, v_min],
        ]
    }
}
