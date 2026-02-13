use crate::title_screen::GameState;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use ferrum_physics::movement::MovementInput;
use ferrum_physics::Player;

const EYE_HEIGHT: f32 = 1.62;
const FEET_TO_GROUND_OFFSET: f32 = 0.5;
const DEFAULT_GROUND_LEVEL: f32 = 17.0; // TODO: Replace with proper chunk-based collision detection

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

#[derive(Resource)]
pub struct PlayerState {
    player: Player,
    game_mode: GameMode,
    is_flying: bool,
    fly_speed: f32,
    pub ground_level: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            player: Player::new(Vec3::new(0.0, 80.0, 0.0)),
            game_mode: GameMode::Survival,
            is_flying: false,
            fly_speed: 20.0,
            ground_level: DEFAULT_GROUND_LEVEL,
        }
    }
}

impl PlayerState {
    pub fn set_spawn_position(&mut self, position: Vec3) {
        self.player.set_position(position);
        self.ground_level = position.y - FEET_TO_GROUND_OFFSET;
    }
}

#[derive(Component)]
pub struct PlayerCamera {
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>().add_systems(
            Update,
            (
                toggle_game_mode,
                camera_look,
                player_movement,
                player_jump,
                player_sprint,
                player_collision,
                update_camera_position,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

fn toggle_game_mode(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<PlayerState>) {
    if keys.just_pressed(KeyCode::F5) {
        state.game_mode = match state.game_mode {
            GameMode::Survival => {
                info!("Switched to Creative mode");
                GameMode::Creative
            }
            GameMode::Creative => {
                info!("Switched to Survival mode");
                state.is_flying = false;
                GameMode::Survival
            }
        };
    }

    if state.game_mode == GameMode::Creative && keys.just_pressed(KeyCode::Space) {
        state.is_flying = true;
    }
}

fn camera_look(
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
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

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut state: ResMut<PlayerState>,
    query: Query<&Transform, With<PlayerCamera>>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    let dt = time.delta_secs();

    match state.game_mode {
        GameMode::Survival => {
            let forward: Vec3 = *transform.forward();
            let right: Vec3 = *transform.right();

            let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
            let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

            let mut input = MovementInput::default();

            if keys.pressed(KeyCode::KeyW) {
                input.forward = true;
            }
            if keys.pressed(KeyCode::KeyS) {
                input.backward = true;
            }
            if keys.pressed(KeyCode::KeyA) {
                input.left = true;
            }
            if keys.pressed(KeyCode::KeyD) {
                input.right = true;
            }

            let mut movement_direction = Vec3::ZERO;
            if input.forward {
                movement_direction += forward_xz;
            }
            if input.backward {
                movement_direction -= forward_xz;
            }
            if input.left {
                movement_direction -= right_xz;
            }
            if input.right {
                movement_direction += right_xz;
            }

            if movement_direction.length_squared() > 0.0 {
                movement_direction = movement_direction.normalize();
            }

            let current_velocity = state.player.velocity();
            let speed = if input.sprint { 4.317 * 1.3 } else { 4.317 };

            if state.player.on_ground() {
                let target_velocity = movement_direction * speed;
                let acceleration = 0.6;
                let friction = 0.91;

                let mut new_velocity = current_velocity;
                new_velocity.x += (target_velocity.x - current_velocity.x) * acceleration;
                new_velocity.z += (target_velocity.z - current_velocity.z) * acceleration;
                new_velocity.x *= friction;
                new_velocity.z *= friction;

                state.player.set_velocity(new_velocity);
            }

            state.player.apply_gravity(dt);
            state.player.update_position(dt);
        }
        GameMode::Creative => {
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
                velocity = velocity.normalize() * state.fly_speed * dt;
                let new_pos = state.player.position() + velocity;
                state.player.set_position(new_pos);
            }
        }
    }
}

fn player_jump(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<PlayerState>) {
    if state.game_mode != GameMode::Survival {
        return;
    }

    if keys.just_pressed(KeyCode::Space) && state.player.on_ground() {
        let velocity = state.player.velocity();
        let new_velocity = ferrum_physics::gravity::apply_jump(velocity, state.player.on_ground());
        state.player.set_velocity(new_velocity);
        state.player.set_on_ground(false);
    }
}

fn player_sprint(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<PlayerState>) {
    if state.game_mode != GameMode::Survival {
        return;
    }
}

fn player_collision(mut state: ResMut<PlayerState>) {
    if state.game_mode != GameMode::Survival {
        return;
    }

    // Simple ground plane collision at GROUND_LEVEL
    let player_pos = state.player.position();

    if player_pos.y <= state.ground_level {
        // Player is at or below ground — snap to ground, zero vertical velocity, mark
        // grounded
        let mut pos = player_pos;
        pos.y = state.ground_level;
        state.player.set_position(pos);

        let mut vel = state.player.velocity();
        if vel.y < 0.0 {
            vel.y = 0.0;
        }
        state.player.set_velocity(vel);

        state.player.set_on_ground(true);
    } else if player_pos.y <= state.ground_level + 0.05 {
        // Very close to ground — still grounded (prevents jitter)
        state.player.set_on_ground(true);
    } else {
        state.player.set_on_ground(false);
    }
}

fn update_camera_position(
    state: Res<PlayerState>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let player_pos = state.player.position();
    let camera_pos = Vec3::new(player_pos.x, player_pos.y + EYE_HEIGHT, player_pos.z);

    transform.translation = camera_pos;
}
