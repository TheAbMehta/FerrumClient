use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::path::PathBuf;

#[derive(Resource)]
pub struct BlockTextureAtlas {
    pub atlas_handle: Handle<Image>,
    pub tile_size: u32,
}

pub struct TextureLoaderPlugin;

impl Plugin for TextureLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_real_textures);
    }
}

fn load_real_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    info!("Loading real Minecraft textures...");

    let texture_dir = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".ferrum/textures");

    // Load individual textures and create atlas
    let texture_files = vec![
        ("stone.png", 0),
        ("dirt.png", 1),
        ("grass_block_top.png", 2),
        ("grass_block_side.png", 3),
        ("cobblestone.png", 4),
        ("oak_planks.png", 5),
        ("oak_log.png", 6),
        ("sand.png", 7),
        ("gravel.png", 8),
        ("iron_ore.png", 9),
        ("gold_ore.png", 10),
        ("diamond_ore.png", 11),
        ("coal_ore.png", 12),
        ("water_still.png", 13),
        ("lava_still.png", 14),
        ("bedrock.png", 15),
        ("glass.png", 16),
        ("oak_leaves.png", 17),
        ("spruce_log.png", 18),
        ("birch_log.png", 19),
        ("netherrack.png", 20),
        ("soul_sand.png", 21),
        ("obsidian.png", 22),
        ("glowstone.png", 23),
        ("snow.png", 24),
        ("ice.png", 25),
        ("clay.png", 26),
        ("terracotta.png", 27),
        ("deepslate.png", 28),
    ];

    const TILE_SIZE: u32 = 16;
    const ATLAS_SIZE: u32 = 256; // 16x16 tiles
    let mut atlas_data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];

    // Fill with default grey
    for pixel in atlas_data.chunks_exact_mut(4) {
        pixel[0] = 128; // R
        pixel[1] = 128; // G
        pixel[2] = 128; // B
        pixel[3] = 255; // A
    }

    let mut loaded_count = 0;
    for (filename, tile_index) in texture_files {
        let path = texture_dir.join(filename);

        if let Ok(img_bytes) = std::fs::read(&path) {
            if let Ok(img) = image::load_from_memory(&img_bytes) {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();

                if width == TILE_SIZE && height == TILE_SIZE {
                    // Calculate position in atlas
                    let tile_x = (tile_index % 16) * TILE_SIZE;
                    let tile_y = (tile_index / 16) * TILE_SIZE;

                    // Get raw byte data from image
                    let raw_data = rgba.as_raw();

                    // Copy texture into atlas
                    for y in 0..TILE_SIZE {
                        for x in 0..TILE_SIZE {
                            let src_idx = ((y * TILE_SIZE + x) * 4) as usize;
                            let dst_x = tile_x + x;
                            let dst_y = tile_y + y;
                            let dst_idx = ((dst_y * ATLAS_SIZE + dst_x) * 4) as usize;

                            atlas_data[dst_idx..dst_idx + 4]
                                .copy_from_slice(&raw_data[src_idx..src_idx + 4]);
                        }
                    }
                    loaded_count += 1;
                }
            }
        }
    }

    info!("Loaded {} real Minecraft textures into atlas", loaded_count);

    let atlas_image = Image::new(
        Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
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
        tile_size: TILE_SIZE,
    });
}
