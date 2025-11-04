mod network;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use ferrum_config::{Config, ConfigPlugin};
use ferrum_meshing_cpu::{ChunkMesher, CpuMesher};
use ferrum_render::{BlockRenderer, TextureAtlas};
use ferrum_subprocess::PumpkinServer;
use std::path::PathBuf;

#[derive(Component)]
struct FlyCamera {
    speed: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            speed: 20.0,
            sensitivity: 0.002,
            yaw: 0.0,
            pitch: 0.0,
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
        .add_systems(
            Startup,
            (
                auto_start_pumpkin,
                connect_to_server,
                setup_scene,
                grab_cursor,
            )
                .chain(),
        )
        .add_systems(Update, (camera_look, camera_move, toggle_cursor))
        .run();
}

fn auto_start_pumpkin(config: Res<Config>) {
    if config.server.auto_start {
        info!("Auto-starting Pumpkin server...");

        let pumpkin_path = PathBuf::from("./pumpkin-server/target/release/pumpkin");

        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async {
                let mut server = PumpkinServer::new(pumpkin_path);
                match server.start().await {
                    Ok(_) => info!("Pumpkin server started successfully"),
                    Err(e) => warn!(
                        "Failed to start Pumpkin server: {}. Continuing without local server.",
                        e
                    ),
                }

                std::mem::forget(server);
            });
    }
}

fn connect_to_server(config: Res<Config>) {
    info!(
        "Connecting to Minecraft server at {}...",
        config.server.address
    );

    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            let addr = match network::perform_handshake(config.server.address.clone(), 25565).await
            {
                Ok(addr) => {
                    info!("Handshake successful, resolved address: {}", addr);
                    addr
                }
                Err(e) => {
                    warn!("Handshake failed: {}. Skipping server connection.", e);
                    return;
                }
            };

            match network::perform_login(addr).await {
                Ok(_) => {
                    info!("Successfully connected and logged in to server!");
                }
                Err(e) => {
                    warn!("Login failed: {}. Continuing without server connection.", e);
                }
            }
        });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera with FlyCamera component
    let initial_yaw = -std::f32::consts::FRAC_PI_4;
    let initial_pitch = -0.3;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(16.0, 40.0, 50.0).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            initial_yaw,
            initial_pitch,
            0.0,
        )),
        FlyCamera {
            yaw: initial_yaw,
            pitch: initial_pitch,
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Generate 4x4 grid of terrain chunks
    for cx in -2..2 {
        for cz in -2..2 {
            let voxels = ferrum_meshing_gpu::terrain_chunk();
            let mesher = CpuMesher::new();
            let chunk_mesh = mesher.mesh_chunk(&voxels);
            let atlas = TextureAtlas::new(16);
            let mut mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);

            // Add vertex colors based on block type
            add_vertex_colors(&mut mesh, &chunk_mesh);

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    ..default()
                })),
                Transform::from_xyz(cx as f32 * 32.0, 0.0, cz as f32 * 32.0),
            ));
        }
    }
}

fn add_vertex_colors(mesh: &mut Mesh, chunk_mesh: &ferrum_meshing_cpu::ChunkMesh) {
    let mut colors = Vec::new();

    for quad in &chunk_mesh.quads {
        let color = match quad.block_type {
            1 => [0.5, 0.5, 0.5, 1.0],   // Stone - grey
            2 => [0.45, 0.3, 0.15, 1.0], // Dirt - brown
            3 => [0.3, 0.7, 0.2, 1.0],   // Grass - green
            _ => [1.0, 1.0, 1.0, 1.0],   // Default - white
        };

        // Each quad has 4 vertices
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

fn toggle_cursor(keys: Res<ButtonInput<KeyCode>>, mut cursor_options: Single<&mut CursorOptions>) {
    if keys.just_pressed(KeyCode::Escape) {
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

fn camera_look(
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    for ev in motion.read() {
        for (mut transform, mut cam) in &mut query {
            cam.yaw -= ev.delta.x * cam.sensitivity;
            cam.pitch -= ev.delta.y * cam.sensitivity;
            cam.pitch = cam
                .pitch
                .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
            transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
        }
    }
}

fn camera_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FlyCamera)>,
) {
    for (mut transform, cam) in &mut query {
        let mut velocity = Vec3::ZERO;
        let forward = *transform.forward();
        let right = *transform.right();

        if keys.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            velocity += right;
        }
        if keys.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if keys.pressed(KeyCode::Space) {
            velocity += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            velocity -= Vec3::Y;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * cam.speed * time.delta_secs();
            transform.translation += velocity;
        }
    }
}
