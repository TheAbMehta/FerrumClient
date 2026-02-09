use azalea_core::position::Vec3 as AzaleaVec3;
use azalea_protocol::common::movements::MoveFlags;
use azalea_protocol::packets::game::{
    s_move_player_pos::ServerboundMovePlayerPos,
    s_move_player_pos_rot::ServerboundMovePlayerPosRot,
    s_move_player_status_only::ServerboundMovePlayerStatusOnly,
};
use bevy::prelude::*;
use glam::Vec3;
use std::time::{Duration, Instant};

const TICK_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Resource)]
pub struct PlayerPositionTracker {
    last_update: Instant,
    last_position: Vec3,
    last_on_ground: bool,
}

impl PlayerPositionTracker {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            last_position: Vec3::ZERO,
            last_on_ground: false,
        }
    }

    pub fn should_send_update(&self) -> bool {
        self.last_update.elapsed() >= TICK_INTERVAL
    }

    pub fn has_position_changed(&self, current_position: Vec3) -> bool {
        (current_position - self.last_position).length() > 0.001
    }

    pub fn update_state(&mut self, position: Vec3, on_ground: bool) {
        self.last_update = Instant::now();
        self.last_position = position;
        self.last_on_ground = on_ground;
    }
}

impl Default for PlayerPositionTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn glam_to_azalea_vec3(v: Vec3) -> AzaleaVec3 {
    AzaleaVec3 {
        x: v.x as f64,
        y: v.y as f64,
        z: v.z as f64,
    }
}

pub fn create_position_packet(position: Vec3, on_ground: bool) -> ServerboundMovePlayerPos {
    ServerboundMovePlayerPos {
        pos: glam_to_azalea_vec3(position),
        flags: MoveFlags {
            on_ground,
            horizontal_collision: false,
        },
    }
}

pub fn create_position_rotation_packet(
    position: Vec3,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
) -> ServerboundMovePlayerPosRot {
    ServerboundMovePlayerPosRot {
        pos: glam_to_azalea_vec3(position),
        look_direction: azalea_entity::LookDirection::new(yaw, pitch),
        flags: MoveFlags {
            on_ground,
            horizontal_collision: false,
        },
    }
}

pub fn create_status_only_packet(on_ground: bool) -> ServerboundMovePlayerStatusOnly {
    ServerboundMovePlayerStatusOnly {
        flags: MoveFlags {
            on_ground,
            horizontal_collision: false,
        },
    }
}

/// Bevy system that sends player position updates to the server
/// This should run every frame and will throttle updates internally
pub fn send_player_position_updates(
    mut tracker: ResMut<PlayerPositionTracker>,
    player_query: Query<&Transform, With<Camera3d>>,
) {
    // Get player transform (using camera as player proxy)
    let Ok(transform) = player_query.single() else {
        return;
    };

    let current_position = transform.translation;

    // Check if we should send an update
    if !tracker.should_send_update() {
        return;
    }

    // Check if position has actually changed
    if !tracker.has_position_changed(current_position) {
        return;
    }

    // TODO: Actually send the packet via ServerConnection
    // For now, just log and update tracker
    debug!(
        "Would send position update: {:?}",
        current_position
    );

    // Assume player is on ground for now (TODO: proper ground detection)
    let on_ground = true;
    tracker.update_state(current_position, on_ground);
}

/// Plugin for player position synchronization
pub struct PlayerPositionPlugin;

impl Plugin for PlayerPositionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPositionTracker>()
            .add_systems(Update, send_player_position_updates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_tracker_throttling() {
        let tracker = PlayerPositionTracker::new();

        assert!(!tracker.should_send_update());

        std::thread::sleep(Duration::from_millis(51));
        assert!(tracker.should_send_update());
    }

    #[test]
    fn test_position_change_detection() {
        let tracker = PlayerPositionTracker::new();

        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(0.1, 0.0, 0.0);

        assert!(tracker.has_position_changed(pos2));

        let pos3 = Vec3::new(0.0001, 0.0, 0.0);
        assert!(!tracker.has_position_changed(pos3));
    }

    #[test]
    fn test_create_position_packet() {
        let position = Vec3::new(100.0, 64.0, 200.0);
        let packet = create_position_packet(position, true);

        assert_eq!(packet.pos.x, 100.0);
        assert_eq!(packet.pos.y, 64.0);
        assert_eq!(packet.pos.z, 200.0);
        assert_eq!(packet.flags.on_ground, true);
    }

    #[test]
    fn test_create_position_rotation_packet() {
        let position = Vec3::new(100.0, 64.0, 200.0);
        let yaw = 90.0;
        let pitch = 45.0;
        let packet = create_position_rotation_packet(position, yaw, pitch, false);

        assert_eq!(packet.pos.x, 100.0);
        assert_eq!(packet.pos.y, 64.0);
        assert_eq!(packet.pos.z, 200.0);
        assert_eq!(packet.look_direction.y_rot(), yaw);
        assert_eq!(packet.look_direction.x_rot(), pitch);
        assert_eq!(packet.flags.on_ground, false);
    }

    #[test]
    fn test_create_status_only_packet() {
        let packet = create_status_only_packet(true);
        assert_eq!(packet.flags.on_ground, true);

        let packet = create_status_only_packet(false);
        assert_eq!(packet.flags.on_ground, false);
    }

    #[test]
    fn test_tracker_update_state() {
        let mut tracker = PlayerPositionTracker::new();
        let position = Vec3::new(10.0, 20.0, 30.0);

        tracker.update_state(position, true);

        assert_eq!(tracker.last_position, position);
        assert_eq!(tracker.last_on_ground, true);
        assert!(!tracker.should_send_update());
    }
}
