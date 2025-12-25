use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::GenericImageView;

/// Resource holding the procedurally generated block texture atlas
#[derive(Resource)]
pub struct BlockTextureAtlas {
    pub atlas_handle: Handle<Image>,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub tile_size: u32,
    pub columns: u32,
    pub rows: u32,
}

impl BlockTextureAtlas {
    /// Returns UV coordinates for a block type's tile in the atlas
    /// Returns [[f32; 2]; 4] for the 4 corners of the quad (bottom-left, bottom-right, top-right, top-left)
    pub fn get_uvs(&self, block_type: u32) -> [[f32; 2]; 4] {
        let tile_x = block_type % self.columns;
        let tile_y = block_type / self.columns;

        let u_min = tile_x as f32 / self.columns as f32;
        let v_min = tile_y as f32 / self.rows as f32;
        let u_max = (tile_x + 1) as f32 / self.columns as f32;
        let v_max = (tile_y + 1) as f32 / self.rows as f32;

        [
            [u_min, v_max], // bottom-left
            [u_max, v_max], // bottom-right
            [u_max, v_min], // top-right
            [u_min, v_min], // top-left
        ]
    }
}

pub struct TextureGenPlugin;

impl Plugin for TextureGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_block_textures);
    }
}

/// Load real Minecraft block textures from the internet
fn load_minecraft_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    info!("Downloading Minecraft block atlas...");

    // Download from multiple sources
    let atlas_urls = vec![
        "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/textures/block/stone.png",
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/pc/1.20/atlas/blocks.png",
    ];

    let atlas_url = "https://github.com/InventivetalentDev/minecraft-assets/raw/1.20.1/assets/minecraft/textures/block/stone.png";

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let atlas_image = runtime.block_on(async {
        match download_texture_atlas(atlas_url).await {
            Ok(img) => {
                info!("Successfully downloaded Minecraft block atlas");
                img
            }
            Err(e) => {
                warn!("Failed to download textures: {}. Using fallback.", e);
                generate_fallback_atlas()
            }
        }
    });

    let atlas_handle = images.add(atlas_image);

    commands.insert_resource(BlockTextureAtlas {
        atlas_handle,
        atlas_width: 256,
        atlas_height: 256,
        tile_size: 16,
        columns: 16,
        rows: 16,
    });
}

async fn download_texture_atlas(url: &str) -> Result<Image, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

fn generate_fallback_atlas() -> Image {
    warn!("Generating simple fallback atlas");
    const SIZE: u32 = 256;
    let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];

    // Simple colored grid pattern
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let tile_x = (x / 16) % 16;
            let tile_y = (y / 16) % 16;

            // Different color per tile
            pixels[idx] = ((tile_x * 16) as u8);     // R
            pixels[idx + 1] = ((tile_y * 16) as u8); // G
            pixels[idx + 2] = 128;                    // B
            pixels[idx + 3] = 255;                    // A
        }
    }

    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Simple deterministic pseudo-random generator for reproducible textures
fn simple_hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = seed;
    h = h.wrapping_mul(1664525).wrapping_add(1013904223);
    h ^= x.wrapping_mul(374761393);
    h = h.wrapping_mul(1103515245).wrapping_add(12345);
    h ^= y.wrapping_mul(668265263);
    h = h.wrapping_mul(1103515245).wrapping_add(12345);
    h
}

/// Generate a random value in range [0, 1) from hash
fn hash_to_float(hash: u32) -> f32 {
    (hash as f32) / (u32::MAX as f32)
}

/// Generate a single 16x16 block texture
fn generate_block_texture(block_type: u32) -> Vec<u8> {
    const TILE_SIZE: usize = 16;
    let mut pixels = vec![0u8; TILE_SIZE * TILE_SIZE * 4]; // RGBA

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let idx = (y * TILE_SIZE + x) * 4;
            let hash = simple_hash(x as u32, y as u32, block_type);
            let rand = hash_to_float(hash);

            let (r, g, b, a) = match block_type {
                0 => (0, 0, 0, 0), // Air: transparent

                1 => {
                    // Stone: Much brighter grey
                    let base = 180;
                    let variation = ((rand - 0.5) * 30.0) as i32;
                    let val = (base + variation).clamp(0, 255) as u8;
                    (val, val, val, 255)
                }

                2 => {
                    // Dirt: Brighter brown
                    let r = (180.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                    let g = (130.0 + (rand - 0.5) * 25.0).clamp(0.0, 255.0) as u8;
                    let b = (90.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 255)
                }

                3 => {
                    // Grass: Bright vibrant green top, dirt bottom
                    if y > 12 {
                        let r = (100.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                        let g = (220.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        let b = (60.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    } else {
                        let r = (150.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        let g = (110.0 + (rand - 0.5) * 25.0).clamp(0.0, 255.0) as u8;
                        let b = (70.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                4 => {
                    // Bedrock: Very dark grey with chaotic noise
                    let val = (48.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                    (val, val, val, 255)
                }

                5 => {
                    // Water: Blue with lighter wave-like horizontal streaks
                    let wave = if y % 4 < 2 { 20.0f32 } else { 0.0f32 };
                    let r = (30.0f32 + wave).clamp(0.0, 255.0) as u8;
                    let g = (50.0f32 + wave).clamp(0.0, 255.0) as u8;
                    let b = (170.0f32 + wave).clamp(0.0, 255.0) as u8;
                    (r, g, b, 180)
                }

                6 => {
                    // Lava: Orange-red with bright yellow spots
                    if rand > 0.85 {
                        (255, 200, 0, 255)
                    } else {
                        let r = (215.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        let g = (90.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                        (r, g, 0, 255)
                    }
                }

                7 => {
                    // Sand: Light yellow with fine grain noise
                    let r = (219.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                    let g = (211.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                    let b = (160.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 255)
                }

                8 => {
                    // Gravel: Mixed grey-brown with multi-colored small pebble spots
                    if rand > 0.8 {
                        let hash2 = simple_hash(x as u32 + 100, y as u32 + 100, block_type);
                        let rand2 = hash_to_float(hash2);
                        let val = (100.0 + rand2 * 100.0) as u8;
                        (val, val, val, 255)
                    } else {
                        let r = (120.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        let g = (110.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        let b = (100.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                9 => {
                    // Gold Ore: Stone base with bright gold pixel clusters
                    let cluster_hash = simple_hash(x as u32 / 3, y as u32 / 3, block_type + 1000);
                    if hash_to_float(cluster_hash) > 0.7 {
                        (255, 215, 0, 255)
                    } else {
                        let val = (128.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (val, val, val, 255)
                    }
                }

                10 => {
                    // Iron Ore: Stone base with tan/pink pixel clusters
                    let cluster_hash = simple_hash(x as u32 / 3, y as u32 / 3, block_type + 1000);
                    if hash_to_float(cluster_hash) > 0.7 {
                        (200, 170, 150, 255)
                    } else {
                        let val = (128.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (val, val, val, 255)
                    }
                }

                11 => {
                    // Coal Ore: Stone base with dark pixel clusters
                    let cluster_hash = simple_hash(x as u32 / 3, y as u32 / 3, block_type + 1000);
                    if hash_to_float(cluster_hash) > 0.7 {
                        (30, 30, 30, 255)
                    } else {
                        let val = (128.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (val, val, val, 255)
                    }
                }

                12 => {
                    // Log: Brown with vertical dark lines for bark texture
                    if x % 4 == 0 || x % 4 == 3 {
                        (70, 45, 20, 255)
                    } else {
                        let r = (101.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let g = (67.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let b = (33.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                13 => {
                    // Leaves: Semi-transparent green with scattered darker/lighter spots
                    let r = (40.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                    let g = (130.0 + (rand - 0.5) * 40.0).clamp(0.0, 255.0) as u8;
                    let b = (30.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 230)
                }

                14 => {
                    // Planks: Light brown with horizontal wood grain lines every 4px
                    if y % 4 == 0 {
                        (150, 120, 70, 255)
                    } else {
                        let r = (188.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let g = (152.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let b = (98.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                15 => {
                    // Cobblestone: Grey base with irregular block outlines
                    let edge = (x % 5 == 0 || y % 5 == 0) && rand > 0.3;
                    if edge {
                        (80, 80, 80, 255)
                    } else {
                        let val = (120.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (val, val, val, 255)
                    }
                }

                16 => {
                    // Diamond Ore: Stone base with cyan pixel clusters
                    let cluster_hash = simple_hash(x as u32 / 3, y as u32 / 3, block_type + 1000);
                    if hash_to_float(cluster_hash) > 0.7 {
                        (0, 230, 230, 255)
                    } else {
                        let val = (128.0 + (rand - 0.5) * 30.0).clamp(0.0, 255.0) as u8;
                        (val, val, val, 255)
                    }
                }

                17 => {
                    // Deepslate: Dark grey with vertical streaky patterns
                    let streak = if x % 3 == 0 { -10.0 } else { 0.0 };
                    let val = (54.0 + streak + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                    (
                        val,
                        val,
                        (60.0 + streak + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8,
                        255,
                    )
                }

                18 => {
                    // Snow: White with very subtle blue-white specks
                    let r = (250.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let g = (250.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let b = (250.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 255)
                }

                19 => {
                    // Ice: Light blue with white crack lines
                    let crack = (x + y) % 7 == 0 && rand > 0.5;
                    if crack {
                        (255, 255, 255, 255)
                    } else {
                        let r = (170.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let g = (210.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let b = (240.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                20 => {
                    // Clay: Light grey-blue smooth texture with subtle noise
                    let r = (160.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let g = (165.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let b = (175.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 255)
                }

                21 => {
                    // Obsidian: Very dark purple-black with subtle purple highlights
                    if rand > 0.9 {
                        (50, 0, 70, 255)
                    } else {
                        let r = (15.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        let g = (5.0 + (rand - 0.5) * 5.0).clamp(0.0, 255.0) as u8;
                        let b = (25.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                22 => {
                    // Netherrack: Dark red-brown with lighter red random blobs
                    if rand > 0.7 {
                        (150, 60, 55, 255)
                    } else {
                        let r = (110.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                        let g = (40.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let b = (40.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                23 => {
                    // Glowstone: Bright yellow with white-yellow spots and dark cracks
                    if (x + y) % 6 == 0 && rand > 0.6 {
                        (100, 80, 40, 255)
                    } else if rand > 0.8 {
                        (255, 255, 200, 255)
                    } else {
                        let r = (250.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        let g = (220.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        let b = (120.0 + (rand - 0.5) * 20.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                24 => {
                    // Soul Sand: Dark brown with face-like darker depressions
                    let depression = (x / 4 + y / 4) % 3 == 0 && rand > 0.6;
                    if depression {
                        (50, 40, 25, 255)
                    } else {
                        let r = (75.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let g = (60.0 + (rand - 0.5) * 15.0).clamp(0.0, 255.0) as u8;
                        let b = (40.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                        (r, g, b, 255)
                    }
                }

                25 => {
                    // Terracotta: Warm brown-orange with smooth texture
                    let r = (162.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let g = (95.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    let b = (65.0 + (rand - 0.5) * 10.0).clamp(0.0, 255.0) as u8;
                    (r, g, b, 255)
                }

                _ => (128, 128, 128, 255), // Default grey
            };

            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = a;
        }
    }

    pixels
}

/// System that generates the block texture atlas on startup
fn generate_block_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    const TILE_SIZE: u32 = 16;
    const COLUMNS: u32 = 8;
    const ROWS: u32 = 4;
    const NUM_BLOCKS: u32 = 26;

    let atlas_width = COLUMNS * TILE_SIZE;
    let atlas_height = ROWS * TILE_SIZE;

    let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    // Generate textures for all block types
    for block_type in 0..NUM_BLOCKS {
        let tile_x = block_type % COLUMNS;
        let tile_y = block_type / COLUMNS;

        let texture_data = generate_block_texture(block_type);

        // Copy texture into atlas
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let src_idx = ((y * TILE_SIZE + x) * 4) as usize;
                let dst_x = tile_x * TILE_SIZE + x;
                let dst_y = tile_y * TILE_SIZE + y;
                let dst_idx = ((dst_y * atlas_width + dst_x) * 4) as usize;

                atlas_data[dst_idx..dst_idx + 4]
                    .copy_from_slice(&texture_data[src_idx..src_idx + 4]);
            }
        }
    }

    // Create the atlas image
    let atlas_image = Image::new(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let atlas_handle = images.add(atlas_image);

    commands.insert_resource(BlockTextureAtlas {
        atlas_handle,
        atlas_width,
        atlas_height,
        tile_size: TILE_SIZE,
        columns: COLUMNS,
        rows: ROWS,
    });

    info!(
        "Generated procedural block texture atlas ({}x{} with {} blocks)",
        atlas_width, atlas_height, NUM_BLOCKS
    );
}
