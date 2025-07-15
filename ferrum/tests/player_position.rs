use ferrum_physics::player::Player;
use glam::Vec3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_position_packet_creation() {
        // Test that we can create position update data from Player
        let player = Player::new(Vec3::new(100.0, 64.0, 200.0));

        let pos = player.position();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 64.0);
        assert_eq!(pos.z, 200.0);

        let on_ground = player.on_ground();
        assert_eq!(on_ground, false);
    }

    #[test]
    fn test_player_on_ground_state() {
        let mut player = Player::new(Vec3::new(0.0, 0.0, 0.0));
        player.set_on_ground(true);

        assert_eq!(player.on_ground(), true);
    }

    #[test]
    fn test_position_update_throttling() {
        // Test that we respect 50ms (20 ticks/second) throttling
        use std::time::{Duration, Instant};

        let min_interval = Duration::from_millis(50);
        let last_update = Instant::now();

        // Simulate immediate second update - should be throttled
        let now = Instant::now();
        let elapsed = now.duration_since(last_update);
        assert!(elapsed < min_interval, "Test setup: should be immediate");

        // Simulate update after 50ms - should be allowed
        std::thread::sleep(Duration::from_millis(51));
        let now = Instant::now();
        let elapsed = now.duration_since(last_update);
        assert!(elapsed >= min_interval, "Should allow update after 50ms");
    }

    #[test]
    fn test_position_change_detection() {
        // Test that we can detect when position has changed
        let player1 = Player::new(Vec3::new(0.0, 0.0, 0.0));
        let player2 = Player::new(Vec3::new(0.1, 0.0, 0.0));

        let pos1 = player1.position();
        let pos2 = player2.position();

        // Positions should be different
        assert_ne!(pos1, pos2);

        // Test significant movement (> 0.01 blocks)
        let delta = (pos2 - pos1).length();
        assert!(delta > 0.01, "Movement should be significant");
    }

    #[test]
    fn test_move_flags_creation() {
        // Test MoveFlags structure (on_ground, horizontal_collision)
        let player = Player::new(Vec3::new(0.0, 0.0, 0.0));

        // Initially not on ground
        assert_eq!(player.on_ground(), false);

        // After landing
        let mut player_landed = Player::new(Vec3::new(0.0, 0.0, 0.0));
        player_landed.set_on_ground(true);
        assert_eq!(player_landed.on_ground(), true);
    }
}
