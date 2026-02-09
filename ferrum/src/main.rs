mod block_interact;
mod chat;
mod death_screen;
mod entity_renderer;
mod hud;
mod inventory_screen;
mod menu;
mod network;
mod particles;
mod player_controller;
mod screenshot;
mod sky;
mod sounds;
mod weather;
mod texture_loader;
mod title_screen;

use azalea_block::BlockState;
use azalea_registry::builtin::BlockKind;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::window::{CursorGrabMode, CursorOptions};
use ferrum_config::{Config, ConfigPlugin};
use ferrum_meshing_cpu::{ChunkMesher, CpuMesher, CHUNK_SIZE, CHUNK_SIZE_CB, CHUNK_SIZE_SQ};
use ferrum_render::{BlockRenderer, TextureAtlas};
use network::ReceivedChunks;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Resource to manage the Pumpkin server process lifecycle
#[derive(Resource)]
struct PumpkinServerHandle {
    _child: Option<Child>,
}

/// Marker resource to track if scene has been set up
#[derive(Resource)]
struct SceneSetup {
    done: bool,
}

impl Drop for PumpkinServerHandle {
    fn drop(&mut self) {
        if let Some(mut child) = self._child.take() {
            info!("Shutting down Pumpkin server...");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ferrum Client".to_string(),
                resolution: (1920, 1080).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ConfigPlugin {
            config_path: "config.toml".into(),
        })
        .add_plugins(texture_loader::TextureLoaderPlugin)
        .add_plugins(title_screen::TitleScreenPlugin)
        .add_plugins(death_screen::DeathScreenPlugin)
        .add_plugins(player_controller::PlayerControllerPlugin)
        .add_plugins(hud::HudPlugin)
        .add_plugins(chat::ChatPlugin)
        .add_plugins(menu::MenuPlugin)
        .add_plugins(sky::SkyPlugin)
        .add_plugins(block_interact::BlockInteractPlugin)
        .add_plugins(inventory_screen::InventoryPlugin)
        .add_plugins(entity_renderer::EntityRenderPlugin)
        .add_plugins(sounds::SoundPlugin)
        // Network/Multiplayer plugins
        .add_plugins(network::PersistentConnectionPlugin)
        .add_plugins(network::EntitySyncPlugin)
        .add_plugins(network::PlayerPositionPlugin)
        // Utility plugins
        .add_plugins(screenshot::ScreenshotPlugin)
        .add_plugins(particles::ParticlePlugin)
        .add_plugins(weather::WeatherPlugin)
        .insert_resource(SceneSetup { done: false })
        .add_systems(
            OnEnter(title_screen::GameState::InGame),
            (
                auto_start_pumpkin,
                connect_to_server,
                grab_cursor,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                setup_scene.run_if(scene_not_setup),
                toggle_cursor.run_if(in_state(title_screen::GameState::InGame)),
            ),
        )
        .run();
}

fn scene_not_setup(scene_setup: Res<SceneSetup>) -> bool {
    !scene_setup.done
}

fn auto_start_pumpkin(mut commands: Commands, config: Res<Config>) {
    if config.server.auto_start {
        info!("Auto-starting Pumpkin server...");

        let pumpkin_path = PathBuf::from("./pumpkin-server/target/release/pumpkin");

        if !pumpkin_path.exists() {
            warn!(
                "Pumpkin server binary not found at {:?}. Continuing without local server.",
                pumpkin_path
            );
            commands.insert_resource(PumpkinServerHandle { _child: None });
            return;
        }

        match Command::new(&pumpkin_path)
            .current_dir("./pumpkin-server")
            .spawn()
        {
            Ok(child) => {
                info!("Pumpkin server process started, waiting 3 seconds for startup...");
                thread::sleep(Duration::from_secs(3));
                commands.insert_resource(PumpkinServerHandle {
                    _child: Some(child),
                });
                info!("Pumpkin server should be ready");
            }
            Err(e) => {
                warn!(
                    "Failed to start Pumpkin server: {}. Continuing without local server.",
                    e
                );
                commands.insert_resource(PumpkinServerHandle { _child: None });
            }
        }
    } else {
        commands.insert_resource(PumpkinServerHandle { _child: None });
    }
}

fn connect_to_server(mut commands: Commands, config: Res<Config>) {
    info!(
        "Connecting to Minecraft server at {}...",
        config.server.address
    );

    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            // Add a timeout to prevent hanging
            let connection_future = network::connect_and_play(config.server.address.clone());
            match tokio::time::timeout(Duration::from_secs(3), connection_future).await {
                Ok(Ok(received_chunks)) => {
                    info!(
                        "Successfully connected and received {} chunks!",
                        received_chunks.chunks.len()
                    );
                    commands.insert_resource(received_chunks);
                }
                Ok(Err(e)) => {
                    warn!(
                        "Connection failed: {}. Continuing without server connection.",
                        e
                    );
                }
                Err(_) => {
                    warn!("Connection timed out after 3 seconds. Continuing without server connection.");
                }
            }
        });
}

fn mc_block_state_to_type(state_id: u16) -> u32 {
    let Ok(block_state) = BlockState::try_from(state_id) else {
        return 0;
    };
    let kind: BlockKind = block_state.into();

    match kind {
        BlockKind::Air | BlockKind::CaveAir | BlockKind::VoidAir => 0,
        BlockKind::Stone
        | BlockKind::Granite
        | BlockKind::Diorite
        | BlockKind::Andesite
        | BlockKind::PolishedGranite
        | BlockKind::PolishedDiorite
        | BlockKind::PolishedAndesite => 1,
        BlockKind::Dirt | BlockKind::CoarseDirt | BlockKind::RootedDirt => 2,
        BlockKind::GrassBlock | BlockKind::Podzol => 3,
        BlockKind::Bedrock => 4,
        BlockKind::Water => 5,
        BlockKind::Lava => 6,
        BlockKind::Sand | BlockKind::RedSand => 7,
        BlockKind::Gravel => 8,
        BlockKind::GoldOre | BlockKind::DeepslateGoldOre => 9,
        BlockKind::IronOre | BlockKind::DeepslateIronOre => 10,
        BlockKind::CoalOre | BlockKind::DeepslateCoalOre => 11,
        BlockKind::OakLog
        | BlockKind::SpruceLog
        | BlockKind::BirchLog
        | BlockKind::JungleLog
        | BlockKind::AcaciaLog
        | BlockKind::DarkOakLog
        | BlockKind::CherryLog => 12,
        BlockKind::OakLeaves
        | BlockKind::SpruceLeaves
        | BlockKind::BirchLeaves
        | BlockKind::JungleLeaves
        | BlockKind::AcaciaLeaves
        | BlockKind::DarkOakLeaves
        | BlockKind::CherryLeaves
        | BlockKind::AzaleaLeaves
        | BlockKind::FloweringAzaleaLeaves => 13,
        BlockKind::OakPlanks
        | BlockKind::SprucePlanks
        | BlockKind::BirchPlanks
        | BlockKind::JunglePlanks
        | BlockKind::AcaciaPlanks
        | BlockKind::DarkOakPlanks
        | BlockKind::CherryPlanks => 14,
        BlockKind::Cobblestone | BlockKind::MossyCobblestone => 15,
        BlockKind::DiamondOre | BlockKind::DeepslateDiamondOre => 16,
        BlockKind::Deepslate => 17,
        BlockKind::Snow | BlockKind::SnowBlock => 18,
        BlockKind::Ice | BlockKind::PackedIce | BlockKind::BlueIce => 19,
        BlockKind::Clay => 20,
        BlockKind::Obsidian => 21,
        BlockKind::Netherrack => 22,
        BlockKind::Glowstone => 23,
        BlockKind::SoulSand => 24,
        BlockKind::Terracotta => 25,
        BlockKind::ShortGrass | BlockKind::TallGrass | BlockKind::Fern | BlockKind::LargeFern => 0,
        _ => {
            if state_id == 0 {
                0
            } else {
                1
            }
        }
    }
}

fn block_type_color(block_type: u32) -> [f32; 4] {
    match block_type {
        0 => [0.0, 0.0, 0.0, 0.0],
        1 => [0.5, 0.5, 0.5, 1.0],
        2 => [0.45, 0.3, 0.15, 1.0],
        3 => [0.3, 0.7, 0.2, 1.0],
        4 => [0.2, 0.2, 0.2, 1.0],
        5 => [0.2, 0.3, 0.8, 0.7],
        6 => [1.0, 0.4, 0.0, 1.0],
        7 => [0.85, 0.8, 0.55, 1.0],
        8 => [0.55, 0.5, 0.45, 1.0],
        9 => [0.9, 0.8, 0.2, 1.0],
        10 => [0.7, 0.55, 0.45, 1.0],
        11 => [0.3, 0.3, 0.3, 1.0],
        12 => [0.4, 0.3, 0.15, 1.0],
        13 => [0.15, 0.5, 0.1, 0.9],
        14 => [0.6, 0.45, 0.25, 1.0],
        15 => [0.45, 0.45, 0.45, 1.0],
        16 => [0.4, 0.8, 0.9, 1.0],
        17 => [0.3, 0.3, 0.35, 1.0],
        18 => [0.95, 0.95, 0.95, 1.0],
        19 => [0.6, 0.8, 1.0, 0.8],
        20 => [0.6, 0.6, 0.7, 1.0],
        21 => [0.15, 0.05, 0.2, 1.0],
        22 => [0.5, 0.2, 0.2, 1.0],
        23 => [0.9, 0.8, 0.4, 1.0],
        24 => [0.4, 0.35, 0.25, 1.0],
        25 => [0.6, 0.4, 0.3, 1.0],
        _ => [0.6, 0.6, 0.6, 1.0],
    }
}

fn convert_server_chunk_to_voxels(
    chunk_data: &[Vec<Vec<u16>>],
    y_offset: usize,
) -> [u32; CHUNK_SIZE_CB] {
    let mut voxels = [0u32; CHUNK_SIZE_CB];

    for local_y in 0..CHUNK_SIZE.min(32) {
        let world_y = y_offset + local_y;
        if world_y >= chunk_data.len() {
            break;
        }

        for local_z in 0..16 {
            for local_x in 0..16 {
                let state_id = chunk_data[world_y][local_z][local_x];
                let block_type = mc_block_state_to_type(state_id);

                let idx = local_z * CHUNK_SIZE_SQ + local_y * CHUNK_SIZE + local_x;
                voxels[idx] = block_type;
            }
        }
    }

    voxels
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    received_chunks: Option<Res<ReceivedChunks>>,
    texture_atlas: Option<Res<texture_loader::BlockTextureAtlas>>,
    mut scene_setup: ResMut<SceneSetup>,
    game_state: Res<State<title_screen::GameState>>,
) {
    // Only run in InGame state
    if *game_state.get() != title_screen::GameState::InGame {
        return;
    }

    let Some(texture_atlas) = texture_atlas else {
        // Texture atlas not loaded yet, will retry next frame
        return;
    };
    let initial_yaw = 0.0;
    let initial_pitch = -std::f32::consts::FRAC_PI_6; // Look down less steeply
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 70.0, 0.0).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            initial_yaw,
            initial_pitch,
            0.0,
        )),
        player_controller::PlayerCamera {
            yaw: initial_yaw,
            pitch: initial_pitch,
            ..default()
        },
        DistanceFog {
            color: Color::srgb(0.53, 0.71, 1.0),
            falloff: FogFalloff::Linear {
                start: 300.0,
                end: 400.0,
            },
            directional_light_color: Color::WHITE,
            directional_light_exponent: 10.0,
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 150000.0, // Very bright
            shadows_enabled: false, // Disable shadows for better visibility
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    let mesher = CpuMesher::new();
    let atlas = TextureAtlas::new(16);

    if let Some(chunks) = received_chunks {
        if !chunks.chunks.is_empty() {
            info!("Rendering {} chunks from server", chunks.chunks.len());

            // Create shared material for all chunks to reduce resource usage
            let chunk_material = materials.add(StandardMaterial {
                base_color_texture: Some(texture_atlas.atlas_handle.clone()),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            for ((chunk_x, chunk_z), chunk_data) in chunks.chunks.iter() {
                let height = chunk_data.len();
                let num_y_slices = (height + 31) / 32;

                for y_slice in 0..num_y_slices {
                    let y_offset = y_slice * 32;
                    if y_offset >= height {
                        break;
                    }

                    let voxels = convert_server_chunk_to_voxels(chunk_data, y_offset);

                    if voxels.iter().all(|&v| v == 0) {
                        continue;
                    }

                    let chunk_mesh = mesher.mesh_chunk(&voxels);
                    if chunk_mesh.quads.is_empty() {
                        continue;
                    }

                    let mut mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);
                    add_vertex_colors(&mut mesh, &chunk_mesh);

                    let world_x = *chunk_x as f32 * 16.0;
                    let world_y = (y_offset as i32 + chunks.min_y) as f32;
                    let world_z = *chunk_z as f32 * 16.0;

                    commands.spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(chunk_material.clone()),
                        Transform::from_xyz(world_x, world_y, world_z),
                    ));
                }
            }

            info!("Finished rendering server chunks");
            scene_setup.done = true;
            return;
        }
    }

    info!("No server chunks available, generating local terrain");

    // Create shared material for all chunks to reduce resource usage
    let chunk_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_atlas.atlas_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for cx in -2..2 {
        for cz in -2..2 {
            let voxels = ferrum_meshing_gpu::terrain_chunk();
            let chunk_mesh = mesher.mesh_chunk(&voxels);

            // Skip empty chunks
            if chunk_mesh.quads.is_empty() {
                continue;
            }

            let mut mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);
            add_vertex_colors(&mut mesh, &chunk_mesh);

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(chunk_material.clone()),
                Transform::from_xyz(cx as f32 * 32.0, 0.0, cz as f32 * 32.0),
            ));
        }
    }
    scene_setup.done = true;
}

fn add_vertex_colors(mesh: &mut Mesh, chunk_mesh: &ferrum_meshing_cpu::ChunkMesh) {
    let mut colors = Vec::new();

    for quad in &chunk_mesh.quads {
        let color = block_type_color(quad.block_type);

        for _ in 0..4 {
            colors.push(color);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn grab_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn toggle_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions>,
    menu_state: Res<menu::MenuState>,
) {
    if keys.just_pressed(KeyCode::Escape) && !menu_state.is_open {
        match cursor_options.grab_mode {
            CursorGrabMode::None => {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
            _ => {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
        }
    }
}
