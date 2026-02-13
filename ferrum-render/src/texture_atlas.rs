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

        // Block type 4: Bedrock (tile_index=15 → (15, 0))
        block_textures.insert((4, Face::Up), (15, 0));
        block_textures.insert((4, Face::Down), (15, 0));
        block_textures.insert((4, Face::Right), (15, 0));
        block_textures.insert((4, Face::Left), (15, 0));
        block_textures.insert((4, Face::Front), (15, 0));
        block_textures.insert((4, Face::Back), (15, 0));

        // Block type 5: Water (tile_index=13 → (13, 0))
        block_textures.insert((5, Face::Up), (13, 0));
        block_textures.insert((5, Face::Down), (13, 0));
        block_textures.insert((5, Face::Right), (13, 0));
        block_textures.insert((5, Face::Left), (13, 0));
        block_textures.insert((5, Face::Front), (13, 0));
        block_textures.insert((5, Face::Back), (13, 0));

        // Block type 6: Lava (tile_index=14 → (14, 0))
        block_textures.insert((6, Face::Up), (14, 0));
        block_textures.insert((6, Face::Down), (14, 0));
        block_textures.insert((6, Face::Right), (14, 0));
        block_textures.insert((6, Face::Left), (14, 0));
        block_textures.insert((6, Face::Front), (14, 0));
        block_textures.insert((6, Face::Back), (14, 0));

        // Block type 7: Sand (tile_index=7 → (7, 0))
        block_textures.insert((7, Face::Up), (7, 0));
        block_textures.insert((7, Face::Down), (7, 0));
        block_textures.insert((7, Face::Right), (7, 0));
        block_textures.insert((7, Face::Left), (7, 0));
        block_textures.insert((7, Face::Front), (7, 0));
        block_textures.insert((7, Face::Back), (7, 0));

        // Block type 8: Gravel (tile_index=8 → (8, 0))
        block_textures.insert((8, Face::Up), (8, 0));
        block_textures.insert((8, Face::Down), (8, 0));
        block_textures.insert((8, Face::Right), (8, 0));
        block_textures.insert((8, Face::Left), (8, 0));
        block_textures.insert((8, Face::Front), (8, 0));
        block_textures.insert((8, Face::Back), (8, 0));

        // Block type 9: Gold Ore (tile_index=10 → (10, 0))
        block_textures.insert((9, Face::Up), (10, 0));
        block_textures.insert((9, Face::Down), (10, 0));
        block_textures.insert((9, Face::Right), (10, 0));
        block_textures.insert((9, Face::Left), (10, 0));
        block_textures.insert((9, Face::Front), (10, 0));
        block_textures.insert((9, Face::Back), (10, 0));

        // Block type 10: Iron Ore (tile_index=9 → (9, 0))
        block_textures.insert((10, Face::Up), (9, 0));
        block_textures.insert((10, Face::Down), (9, 0));
        block_textures.insert((10, Face::Right), (9, 0));
        block_textures.insert((10, Face::Left), (9, 0));
        block_textures.insert((10, Face::Front), (9, 0));
        block_textures.insert((10, Face::Back), (9, 0));

        // Block type 11: Coal Ore (tile_index=12 → (12, 0))
        block_textures.insert((11, Face::Up), (12, 0));
        block_textures.insert((11, Face::Down), (12, 0));
        block_textures.insert((11, Face::Right), (12, 0));
        block_textures.insert((11, Face::Left), (12, 0));
        block_textures.insert((11, Face::Front), (12, 0));
        block_textures.insert((11, Face::Back), (12, 0));

        // Block type 12: Log (tile_index=6 → (6, 0))
        block_textures.insert((12, Face::Up), (6, 0));
        block_textures.insert((12, Face::Down), (6, 0));
        block_textures.insert((12, Face::Right), (6, 0));
        block_textures.insert((12, Face::Left), (6, 0));
        block_textures.insert((12, Face::Front), (6, 0));
        block_textures.insert((12, Face::Back), (6, 0));

        // Block type 13: Leaves (tile_index=17 → (1, 1))
        block_textures.insert((13, Face::Up), (1, 1));
        block_textures.insert((13, Face::Down), (1, 1));
        block_textures.insert((13, Face::Right), (1, 1));
        block_textures.insert((13, Face::Left), (1, 1));
        block_textures.insert((13, Face::Front), (1, 1));
        block_textures.insert((13, Face::Back), (1, 1));

        // Block type 14: Planks (tile_index=5 → (5, 0))
        block_textures.insert((14, Face::Up), (5, 0));
        block_textures.insert((14, Face::Down), (5, 0));
        block_textures.insert((14, Face::Right), (5, 0));
        block_textures.insert((14, Face::Left), (5, 0));
        block_textures.insert((14, Face::Front), (5, 0));
        block_textures.insert((14, Face::Back), (5, 0));

        // Block type 15: Cobblestone (tile_index=4 → (4, 0))
        block_textures.insert((15, Face::Up), (4, 0));
        block_textures.insert((15, Face::Down), (4, 0));
        block_textures.insert((15, Face::Right), (4, 0));
        block_textures.insert((15, Face::Left), (4, 0));
        block_textures.insert((15, Face::Front), (4, 0));
        block_textures.insert((15, Face::Back), (4, 0));

        // Block type 16: Diamond Ore (tile_index=11 → (11, 0))
        block_textures.insert((16, Face::Up), (11, 0));
        block_textures.insert((16, Face::Down), (11, 0));
        block_textures.insert((16, Face::Right), (11, 0));
        block_textures.insert((16, Face::Left), (11, 0));
        block_textures.insert((16, Face::Front), (11, 0));
        block_textures.insert((16, Face::Back), (11, 0));

        // Block type 17: Deepslate (tile_index=28 → (12, 1))
        block_textures.insert((17, Face::Up), (12, 1));
        block_textures.insert((17, Face::Down), (12, 1));
        block_textures.insert((17, Face::Right), (12, 1));
        block_textures.insert((17, Face::Left), (12, 1));
        block_textures.insert((17, Face::Front), (12, 1));
        block_textures.insert((17, Face::Back), (12, 1));

        // Block type 18: Snow (tile_index=24 → (8, 1))
        block_textures.insert((18, Face::Up), (8, 1));
        block_textures.insert((18, Face::Down), (8, 1));
        block_textures.insert((18, Face::Right), (8, 1));
        block_textures.insert((18, Face::Left), (8, 1));
        block_textures.insert((18, Face::Front), (8, 1));
        block_textures.insert((18, Face::Back), (8, 1));

        // Block type 19: Ice (tile_index=25 → (9, 1))
        block_textures.insert((19, Face::Up), (9, 1));
        block_textures.insert((19, Face::Down), (9, 1));
        block_textures.insert((19, Face::Right), (9, 1));
        block_textures.insert((19, Face::Left), (9, 1));
        block_textures.insert((19, Face::Front), (9, 1));
        block_textures.insert((19, Face::Back), (9, 1));

        // Block type 20: Clay (tile_index=26 → (10, 1))
        block_textures.insert((20, Face::Up), (10, 1));
        block_textures.insert((20, Face::Down), (10, 1));
        block_textures.insert((20, Face::Right), (10, 1));
        block_textures.insert((20, Face::Left), (10, 1));
        block_textures.insert((20, Face::Front), (10, 1));
        block_textures.insert((20, Face::Back), (10, 1));

        // Block type 21: Obsidian (tile_index=22 → (6, 1))
        block_textures.insert((21, Face::Up), (6, 1));
        block_textures.insert((21, Face::Down), (6, 1));
        block_textures.insert((21, Face::Right), (6, 1));
        block_textures.insert((21, Face::Left), (6, 1));
        block_textures.insert((21, Face::Front), (6, 1));
        block_textures.insert((21, Face::Back), (6, 1));

        // Block type 22: Netherrack (tile_index=20 → (4, 1))
        block_textures.insert((22, Face::Up), (4, 1));
        block_textures.insert((22, Face::Down), (4, 1));
        block_textures.insert((22, Face::Right), (4, 1));
        block_textures.insert((22, Face::Left), (4, 1));
        block_textures.insert((22, Face::Front), (4, 1));
        block_textures.insert((22, Face::Back), (4, 1));

        // Block type 23: Glowstone (tile_index=23 → (7, 1))
        block_textures.insert((23, Face::Up), (7, 1));
        block_textures.insert((23, Face::Down), (7, 1));
        block_textures.insert((23, Face::Right), (7, 1));
        block_textures.insert((23, Face::Left), (7, 1));
        block_textures.insert((23, Face::Front), (7, 1));
        block_textures.insert((23, Face::Back), (7, 1));

        // Block type 24: Soul Sand (tile_index=21 → (5, 1))
        block_textures.insert((24, Face::Up), (5, 1));
        block_textures.insert((24, Face::Down), (5, 1));
        block_textures.insert((24, Face::Right), (5, 1));
        block_textures.insert((24, Face::Left), (5, 1));
        block_textures.insert((24, Face::Front), (5, 1));
        block_textures.insert((24, Face::Back), (5, 1));

        // Block type 25: Terracotta (tile_index=27 → (11, 1))
        block_textures.insert((25, Face::Up), (11, 1));
        block_textures.insert((25, Face::Down), (11, 1));
        block_textures.insert((25, Face::Right), (11, 1));
        block_textures.insert((25, Face::Left), (11, 1));
        block_textures.insert((25, Face::Front), (11, 1));
        block_textures.insert((25, Face::Back), (11, 1));

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

        let atlas_width = 16.0_f32;
        let atlas_height = 16.0_f32;
        let half_texel = 0.5 / (atlas_width * self.tile_size as f32);

        let u_min = tile_x as f32 / atlas_width + half_texel;
        let v_min = tile_y as f32 / atlas_height + half_texel;
        let u_max = (tile_x + 1) as f32 / atlas_width - half_texel;
        let v_max = (tile_y + 1) as f32 / atlas_height - half_texel;

        [
            [u_min, v_max],
            [u_max, v_max],
            [u_max, v_min],
            [u_min, v_min],
        ]
    }
}
