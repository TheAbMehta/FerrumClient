use bevy::prelude::*;

pub struct BlockInteractPlugin;

impl Plugin for BlockInteractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockTarget>().add_systems(
            Update,
            (
                raycast_block,
                handle_block_break,
                handle_block_place,
                update_block_highlight,
            ),
        );
    }
}

#[derive(Resource, Default)]
pub struct BlockTarget {
    pub targeted_block: Option<IVec3>,
    pub targeted_face: Option<Face>,
    pub break_progress: f32,
    pub is_breaking: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl Face {
    /// Get the normal vector for this face
    fn normal(&self) -> Vec3 {
        match self {
            Face::Top => Vec3::Y,
            Face::Bottom => Vec3::NEG_Y,
            Face::North => Vec3::NEG_Z,
            Face::South => Vec3::Z,
            Face::East => Vec3::X,
            Face::West => Vec3::NEG_X,
        }
    }

    /// Get the offset to place a block adjacent to this face
    fn offset(&self) -> IVec3 {
        match self {
            Face::Top => IVec3::Y,
            Face::Bottom => IVec3::NEG_Y,
            Face::North => IVec3::new(0, 0, -1),
            Face::South => IVec3::new(0, 0, 1),
            Face::East => IVec3::X,
            Face::West => IVec3::NEG_X,
        }
    }
}

#[derive(Component)]
struct BlockHighlight;

/// Raycast from camera to find targeted block
fn raycast_block(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut block_target: ResMut<BlockTarget>,
) {
    let Some(camera_transform) = camera_query.iter().next() else {
        return;
    };

    let ray_origin = camera_transform.translation;
    let ray_direction = *camera_transform.forward();

    const MAX_DISTANCE: f32 = 5.0;
    const STEP_SIZE: f32 = 0.1;

    let mut current_pos = ray_origin;
    let mut found_block = None;
    let mut found_face = None;

    for _ in 0..(MAX_DISTANCE / STEP_SIZE) as i32 {
        current_pos += ray_direction * STEP_SIZE;

        // Simplified ground plane check until proper voxel lookup is available
        if current_pos.y <= 64.0 && current_pos.y > 63.0 {
            let block_pos = IVec3::new(
                current_pos.x.floor() as i32,
                64,
                current_pos.z.floor() as i32,
            );

            found_face = Some(Face::Top);
            found_block = Some(block_pos);
            break;
        }
    }

    block_target.targeted_block = found_block;
    block_target.targeted_face = found_face;
}

/// Handle block breaking with left mouse button
fn handle_block_break(
    mouse_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut block_target: ResMut<BlockTarget>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(block_pos) = block_target.targeted_block {
            block_target.is_breaking = true;

            const BREAK_SPEED: f32 = 1.0;
            block_target.break_progress += time.delta_secs() * BREAK_SPEED;

            if block_target.break_progress >= 1.0 {
                info!("Broke block at {:?}", block_pos);
                // TODO: Send block break packet to server
                // TODO: Update local world state

                block_target.break_progress = 0.0;
                block_target.is_breaking = false;
            }
        }
    } else if block_target.is_breaking {
        block_target.break_progress = 0.0;
        block_target.is_breaking = false;
    }
}

/// Handle block placing with right mouse button
fn handle_block_place(mouse_input: Res<ButtonInput<MouseButton>>, block_target: Res<BlockTarget>) {
    if mouse_input.just_pressed(MouseButton::Right) {
        if let (Some(block_pos), Some(face)) =
            (block_target.targeted_block, block_target.targeted_face)
        {
            let place_pos = block_pos + face.offset();

            info!("Placed block at {:?}", place_pos);
            // TODO: Send block place packet to server
            // TODO: Update local world state
        }
    }
}

/// Update block highlight outline
fn update_block_highlight(
    mut commands: Commands,
    block_target: Res<BlockTarget>,
    highlight_query: Query<Entity, With<BlockHighlight>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if block_target.is_changed() {
        for entity in &highlight_query {
            commands.entity(entity).despawn();
        }
    }

    if let Some(block_pos) = block_target.targeted_block {
        if highlight_query.is_empty() {
            let cube_mesh = meshes.add(Cuboid::new(1.002, 1.002, 1.002));

            let outline_material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.3),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            commands.spawn((
                Mesh3d(cube_mesh),
                MeshMaterial3d(outline_material),
                Transform::from_translation(Vec3::new(
                    block_pos.x as f32 + 0.5,
                    block_pos.y as f32 + 0.5,
                    block_pos.z as f32 + 0.5,
                )),
                BlockHighlight,
            ));
        }
    }
}
