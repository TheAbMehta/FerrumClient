use bevy::prelude::*;
use bevy::hierarchy::ChildBuilder;
use std::collections::HashMap;

/// Plugin that handles rendering of game entities (players, mobs, items)
pub struct EntityRenderPlugin;

impl Plugin for EntityRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerEntities>()
            .add_systems(Startup, spawn_test_entities)
            .add_systems(
                Update,
                (
                    spawn_entity_meshes,
                    update_entity_positions,
                    despawn_removed_entities,
                    animate_entities,
                    update_health_bars,
                ),
            );
    }
}

/// Types of entities that can be rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Player,
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Pig,
    Cow,
    Sheep,
    Chicken,
    DroppedItem,
}

/// Component attached to rendered entities
#[derive(Component)]
pub struct GameEntity {
    pub entity_type: EntityType,
    pub entity_id: i32,
    pub position: Vec3,
    pub rotation: f32, // yaw in radians
    pub health: f32,
}

/// Marker component for entity root (parent of all body parts)
#[derive(Component)]
struct EntityRoot;

/// Marker component for health bar entities
#[derive(Component)]
struct HealthBar {
    entity_id: i32,
}

/// Resource storing entity data received from server
#[derive(Resource, Default)]
pub struct ServerEntities {
    pub entities: HashMap<i32, EntityData>,
}

/// Data for a single entity from the server
pub struct EntityData {
    pub entity_type: EntityType,
    pub position: Vec3,
    pub rotation: f32,
    pub health: f32,
}

/// System that spawns mesh hierarchies for new entities
fn spawn_entity_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server_entities: Res<ServerEntities>,
    existing_entities: Query<&GameEntity>,
) {
    // Build set of existing entity IDs
    let existing_ids: std::collections::HashSet<i32> =
        existing_entities.iter().map(|e| e.entity_id).collect();

    // Spawn meshes for new entities
    for (&entity_id, entity_data) in &server_entities.entities {
        if existing_ids.contains(&entity_id) {
            continue;
        }

        spawn_entity(
            &mut commands,
            &mut meshes,
            &mut materials,
            entity_id,
            entity_data,
        );
    }
}

/// Spawns a single entity with all its mesh parts
fn spawn_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_id: i32,
    entity_data: &EntityData,
) {
    let entity_type = entity_data.entity_type;

    commands
        .spawn((
            Transform::from_translation(entity_data.position)
                .with_rotation(Quat::from_rotation_y(entity_data.rotation)),
            Visibility::default(),
            GameEntity {
                entity_type,
                entity_id,
                position: entity_data.position,
                rotation: entity_data.rotation,
                health: entity_data.health,
            },
            EntityRoot,
        ))
        .with_children(|parent| {
            match entity_type {
                EntityType::Player => spawn_player_mesh(parent, meshes, materials),
                EntityType::Zombie => spawn_zombie_mesh(parent, meshes, materials),
                EntityType::Skeleton => spawn_skeleton_mesh(parent, meshes, materials),
                EntityType::Creeper => spawn_creeper_mesh(parent, meshes, materials),
                EntityType::Spider => spawn_spider_mesh(parent, meshes, materials),
                EntityType::Pig => spawn_pig_mesh(parent, meshes, materials),
                EntityType::Cow => spawn_cow_mesh(parent, meshes, materials),
                EntityType::Sheep => spawn_sheep_mesh(parent, meshes, materials),
                EntityType::Chicken => spawn_chicken_mesh(parent, meshes, materials),
                EntityType::DroppedItem => spawn_dropped_item_mesh(parent, meshes, materials),
            }

            // Spawn health bar
            spawn_health_bar(parent, meshes, materials, entity_id, entity_data.health);
        });
}

/// Spawns a player entity mesh (humanoid with colored parts)
fn spawn_player_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Head (skin color)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.375, 0.0),
    ));

    // Body (blue shirt)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.625, 0.0),
    ));

    // Left arm
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.5),
            ..default()
        })),
        Transform::from_xyz(-0.375, 0.625, 0.0),
    ));

    // Right arm
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.375, 0.625, 0.0),
    ));

    // Left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.3),
            ..default()
        })),
        Transform::from_xyz(-0.125, -0.125, 0.0),
    ));

    // Right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.125, -0.125, 0.0),
    ));
}

/// Spawns a zombie entity mesh (green-tinted humanoid)
fn spawn_zombie_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Head (green skin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.375, 0.0),
    ));

    // Body (torn blue)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.2, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.625, 0.0),
    ));

    // Left arm
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.3),
            ..default()
        })),
        Transform::from_xyz(-0.375, 0.625, 0.0),
    ));

    // Right arm
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.375, 0.625, 0.0),
    ));

    // Left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.2, 0.5),
            ..default()
        })),
        Transform::from_xyz(-0.125, -0.125, 0.0),
    ));

    // Right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.75, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.2, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.125, -0.125, 0.0),
    ));
}

/// Spawns a skeleton entity mesh (thin white humanoid)
fn spawn_skeleton_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let bone_color = Color::srgb(0.9, 0.9, 0.85);

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.375, 0.0),
    ));

    // Body (thin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.4, 0.75, 0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.625, 0.0),
    ));

    // Left arm (thin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.75, 0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(-0.3, 0.625, 0.0),
    ));

    // Right arm (thin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.75, 0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(0.3, 0.625, 0.0),
    ));

    // Left leg (thin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.75, 0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(-0.1, -0.125, 0.0),
    ));

    // Right leg (thin)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.75, 0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bone_color,
            ..default()
        })),
        Transform::from_xyz(0.1, -0.125, 0.0),
    ));
}

/// Spawns a creeper entity mesh (no arms, 4 legs)
fn spawn_creeper_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let creeper_green = Color::srgb(0.2, 0.6, 0.2);

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.25, 0.0),
    ));

    // Body
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 1.0, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.25, 0.0),
    ));

    // Front-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.5, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(-0.125, -0.5, -0.125),
    ));

    // Front-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.5, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(0.125, -0.5, -0.125),
    ));

    // Back-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.5, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(-0.125, -0.5, 0.125),
    ));

    // Back-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.5, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: creeper_green,
            ..default()
        })),
        Transform::from_xyz(0.125, -0.5, 0.125),
    ));
}

/// Spawns a spider entity mesh (flat body with head)
fn spawn_spider_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let spider_brown = Color::srgb(0.2, 0.1, 0.05);

    // Body (flat and wide)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.4, 0.7))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: spider_brown,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.2, 0.0),
    ));

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.3, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: spider_brown,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.15, -0.55),
    ));
}

/// Spawns a pig entity mesh (pink quadruped)
fn spawn_pig_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let pig_pink = Color::srgb(0.95, 0.7, 0.7);

    // Body
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.5, 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.4, 0.0),
    ));

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.4, -0.6),
    ));

    // Front-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.3, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(-0.2, 0.0, -0.3),
    ));

    // Front-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.3, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(0.2, 0.0, -0.3),
    ));

    // Back-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.3, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(-0.2, 0.0, 0.3),
    ));

    // Back-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.3, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: pig_pink,
            ..default()
        })),
        Transform::from_xyz(0.2, 0.0, 0.3),
    ));
}

/// Spawns a cow entity mesh (brown/white quadruped)
fn spawn_cow_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let cow_brown = Color::srgb(0.4, 0.3, 0.2);

    // Body
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 1.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, -0.7),
    ));

    // Front-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.6, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(-0.25, -0.15, -0.4),
    ));

    // Front-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.6, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(0.25, -0.15, -0.4),
    ));

    // Back-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.6, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(-0.25, -0.15, 0.4),
    ));

    // Back-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.6, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: cow_brown,
            ..default()
        })),
        Transform::from_xyz(0.25, -0.15, 0.4),
    ));
}

/// Spawns a sheep entity mesh (white wool quadruped)
fn spawn_sheep_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let wool_white = Color::srgb(0.95, 0.95, 0.95);
    let head_grey = Color::srgb(0.5, 0.5, 0.5);

    // Body (wool)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: wool_white,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.45, 0.0),
    ));

    // Head (grey)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: head_grey,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.45, -0.6),
    ));

    // Front-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.4, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: head_grey,
            ..default()
        })),
        Transform::from_xyz(-0.2, -0.05, -0.3),
    ));

    // Front-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.4, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: head_grey,
            ..default()
        })),
        Transform::from_xyz(0.2, -0.05, -0.3),
    ));

    // Back-left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.4, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: head_grey,
            ..default()
        })),
        Transform::from_xyz(-0.2, -0.05, 0.3),
    ));

    // Back-right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.4, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: head_grey,
            ..default()
        })),
        Transform::from_xyz(0.2, -0.05, 0.3),
    ));
}

/// Spawns a chicken entity mesh (small white bird)
fn spawn_chicken_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chicken_white = Color::srgb(0.95, 0.95, 0.95);
    let beak_yellow = Color::srgb(0.9, 0.8, 0.2);

    // Body
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: chicken_white,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.25, 0.0),
    ));

    // Head
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: chicken_white,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.45, -0.25),
    ));

    // Left leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.2, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: beak_yellow,
            ..default()
        })),
        Transform::from_xyz(-0.1, 0.0, 0.0),
    ));

    // Right leg
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.2, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: beak_yellow,
            ..default()
        })),
        Transform::from_xyz(0.1, 0.0, 0.0),
    ));
}

/// Spawns a dropped item entity mesh (small spinning box)
fn spawn_dropped_item_mesh(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.25, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.125, 0.0),
    ));
}

/// Spawns a health bar above an entity
fn spawn_health_bar(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_id: i32,
    health: f32,
) {
    let max_health = 20.0; // Minecraft standard
    let health_ratio = (health / max_health).clamp(0.0, 1.0);
    let bar_width = 0.5 * health_ratio;

    // Health bar background (dark red)
    parent.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.05, 0.01))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.0, 0.0),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 2.0, 0.0),
        HealthBar { entity_id },
    ));

    // Health bar foreground (bright red)
    if health_ratio > 0.0 {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(bar_width, 0.05, 0.01))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                unlit: true,
                ..default()
            })),
            Transform::from_xyz((bar_width - 0.5) * 0.5, 2.0, 0.0),
            HealthBar { entity_id },
        ));
    }
}

/// System that updates entity positions with smooth interpolation
fn update_entity_positions(
    mut entities: Query<(&mut Transform, &mut GameEntity), With<EntityRoot>>,
    server_entities: Res<ServerEntities>,
    time: Res<Time>,
) {
    for (mut transform, mut game_entity) in &mut entities {
        if let Some(entity_data) = server_entities.entities.get(&game_entity.entity_id) {
            // Smooth interpolation
            let target_pos = entity_data.position;
            transform.translation = transform
                .translation
                .lerp(target_pos, 10.0 * time.delta_secs());

            // Update rotation
            let target_rotation = Quat::from_rotation_y(entity_data.rotation);
            transform.rotation = transform
                .rotation
                .slerp(target_rotation, 10.0 * time.delta_secs());

            // Update component data
            game_entity.position = entity_data.position;
            game_entity.rotation = entity_data.rotation;
            game_entity.health = entity_data.health;
        }
    }
}

/// System that removes entities that no longer exist on the server
fn despawn_removed_entities(
    mut commands: Commands,
    entities: Query<(Entity, &GameEntity), With<EntityRoot>>,
    server_entities: Res<ServerEntities>,
) {
    for (entity, game_entity) in &entities {
        if !server_entities
            .entities
            .contains_key(&game_entity.entity_id)
        {
            commands.entity(entity).despawn();
        }
    }
}

/// System that animates entities (bobbing, rotation)
fn animate_entities(
    mut entities: Query<(&mut Transform, &GameEntity), With<EntityRoot>>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();

    for (mut transform, game_entity) in &mut entities {
        match game_entity.entity_type {
            EntityType::DroppedItem => {
                // Rotate dropped items
                let rotation_speed = 2.0;
                let rotation = Quat::from_rotation_y(elapsed * rotation_speed);
                transform.rotation = rotation;

                // Bob up and down
                let bob_amount = 0.05;
                let bob_speed = 3.0;
                let bob_offset = (elapsed * bob_speed).sin() * bob_amount;
                transform.translation.y = game_entity.position.y + bob_offset;
            }
            _ => {
                // Gentle bobbing for living entities
                let bob_amount = 0.02;
                let bob_speed = 2.0;
                let bob_offset =
                    (elapsed * bob_speed + game_entity.entity_id as f32).sin() * bob_amount;
                transform.translation.y = game_entity.position.y + bob_offset;
            }
        }
    }
}

/// System that updates health bar visibility and size based on camera distance
fn update_health_bars(
    mut health_bars: Query<(&mut Visibility, &mut Transform, &HealthBar)>,
    entities: Query<(&Transform, &GameEntity), (With<EntityRoot>, Without<HealthBar>)>,
    camera: Query<&Transform, (With<Camera3d>, Without<EntityRoot>, Without<HealthBar>)>,
) {
    let Ok(camera_transform) = camera.single() else {
        return;
    };

    for (mut visibility, mut bar_transform, health_bar) in &mut health_bars {
        // Find the entity this health bar belongs to
        let entity_pos = entities
            .iter()
            .find(|(_, e)| e.entity_id == health_bar.entity_id)
            .map(|(t, _)| t.translation);

        if let Some(pos) = entity_pos {
            let distance = camera_transform.translation.distance(pos);

            // Only show health bars within 16 blocks
            if distance < 16.0 {
                *visibility = Visibility::Visible;

                // Billboard effect: make health bar face camera
                let direction = (camera_transform.translation - pos).normalize();
                let look_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
                bar_transform.rotation = look_rotation;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// System that spawns test entities for development
fn spawn_test_entities(mut server_entities: ResMut<ServerEntities>) {
    // Spawn test entities at fixed positions near spawn
    server_entities.entities.insert(
        1,
        EntityData {
            entity_type: EntityType::Zombie,
            position: Vec3::new(5.0, 65.0, 5.0),
            rotation: 0.0,
            health: 20.0,
        },
    );

    server_entities.entities.insert(
        2,
        EntityData {
            entity_type: EntityType::Skeleton,
            position: Vec3::new(-5.0, 65.0, 5.0),
            rotation: std::f32::consts::PI / 2.0,
            health: 20.0,
        },
    );

    server_entities.entities.insert(
        3,
        EntityData {
            entity_type: EntityType::Creeper,
            position: Vec3::new(5.0, 65.0, -5.0),
            rotation: std::f32::consts::PI,
            health: 20.0,
        },
    );

    server_entities.entities.insert(
        4,
        EntityData {
            entity_type: EntityType::Pig,
            position: Vec3::new(10.0, 65.0, 0.0),
            rotation: -std::f32::consts::PI / 2.0,
            health: 10.0,
        },
    );

    server_entities.entities.insert(
        5,
        EntityData {
            entity_type: EntityType::Cow,
            position: Vec3::new(-10.0, 65.0, 0.0),
            rotation: 0.0,
            health: 10.0,
        },
    );

    server_entities.entities.insert(
        6,
        EntityData {
            entity_type: EntityType::Sheep,
            position: Vec3::new(0.0, 65.0, 10.0),
            rotation: std::f32::consts::PI / 4.0,
            health: 8.0,
        },
    );

    server_entities.entities.insert(
        7,
        EntityData {
            entity_type: EntityType::Chicken,
            position: Vec3::new(0.0, 65.0, -10.0),
            rotation: -std::f32::consts::PI / 4.0,
            health: 4.0,
        },
    );

    server_entities.entities.insert(
        8,
        EntityData {
            entity_type: EntityType::Spider,
            position: Vec3::new(7.0, 65.0, 7.0),
            rotation: std::f32::consts::PI / 6.0,
            health: 16.0,
        },
    );

    server_entities.entities.insert(
        9,
        EntityData {
            entity_type: EntityType::DroppedItem,
            position: Vec3::new(0.0, 65.5, 0.0),
            rotation: 0.0,
            health: 1.0,
        },
    );

    server_entities.entities.insert(
        10,
        EntityData {
            entity_type: EntityType::Player,
            position: Vec3::new(-7.0, 65.0, -7.0),
            rotation: std::f32::consts::PI * 0.75,
            health: 20.0,
        },
    );
}
