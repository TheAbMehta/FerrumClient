use ferrum_physics::{collision::Aabb, movement::MovementInput, player::Player, GRAVITY};
use glam::Vec3;

#[test]
fn test_player_creation() {
    let player = Player::new(Vec3::new(0.0, 64.0, 0.0));
    assert_eq!(player.position(), Vec3::new(0.0, 64.0, 0.0));
    assert_eq!(player.velocity(), Vec3::ZERO);
    assert!(!player.on_ground());
}

#[test]
fn test_player_movement_wasd() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    let input = MovementInput {
        forward: true,
        backward: false,
        left: false,
        right: false,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input, 0.05);

    assert!(player.velocity().z < 0.0);
}

#[test]
fn test_player_movement_backward() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    let input = MovementInput {
        forward: false,
        backward: true,
        left: false,
        right: false,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input, 0.05);

    assert!(player.velocity().z > 0.0);
}

#[test]
fn test_player_movement_strafe() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    let input = MovementInput {
        forward: false,
        backward: false,
        left: true,
        right: false,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input, 0.05);
    assert!(player.velocity().x < 0.0);

    player.set_velocity(Vec3::ZERO);

    let input = MovementInput {
        forward: false,
        backward: false,
        left: false,
        right: true,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input, 0.05);
    assert!(player.velocity().x > 0.0);
}

#[test]
fn test_jumping() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    let input = MovementInput {
        forward: false,
        backward: false,
        left: false,
        right: false,
        jump: true,
        sprint: false,
    };
    player.apply_movement(input, 0.05);

    assert!(player.velocity().y > 0.0);
    assert!(!player.on_ground());
}

#[test]
fn test_no_jump_in_air() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(false);

    let input = MovementInput {
        forward: false,
        backward: false,
        left: false,
        right: false,
        jump: true,
        sprint: false,
    };
    player.apply_movement(input, 0.05);

    assert_eq!(player.velocity().y, 0.0);
}

#[test]
fn test_gravity_application() {
    let mut player = Player::new(Vec3::new(0.0, 100.0, 0.0));
    player.set_on_ground(false);

    for _ in 0..20 {
        player.apply_gravity(0.05);
    }

    assert!(player.velocity().y < 0.0);
    assert!(player.velocity().y >= -78.4);
}

#[test]
fn test_gravity_not_applied_on_ground() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);
    player.set_velocity(Vec3::ZERO);

    player.apply_gravity(0.05);

    assert_eq!(player.velocity().y, 0.0);
}

#[test]
fn test_position_update() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_velocity(Vec3::new(4.317, 0.0, 0.0));

    player.update_position(1.0);

    assert!((player.position().x - 4.317).abs() < 0.01);
}

#[test]
fn test_aabb_creation() {
    let aabb = Aabb::new(Vec3::ZERO, Vec3::new(0.6, 1.8, 0.6));
    assert_eq!(aabb.min(), Vec3::ZERO);
    assert_eq!(aabb.max(), Vec3::new(0.6, 1.8, 0.6));
}

#[test]
fn test_aabb_contains_point() {
    let aabb = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));

    assert!(aabb.contains(Vec3::new(0.5, 0.5, 0.5)));
    assert!(!aabb.contains(Vec3::new(1.5, 0.5, 0.5)));
}

#[test]
fn test_aabb_intersects() {
    let aabb1 = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
    let aabb2 = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let aabb3 = Aabb::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));

    assert!(aabb1.intersects(&aabb2));
    assert!(!aabb1.intersects(&aabb3));
}

#[test]
fn test_player_aabb() {
    let player = Player::new(Vec3::new(0.0, 64.0, 0.0));
    let aabb = player.aabb();

    assert_eq!(aabb.min(), Vec3::new(-0.3, 64.0, -0.3));
    assert_eq!(aabb.max(), Vec3::new(0.3, 65.8, 0.3));
}

#[test]
fn test_ground_collision_detection() {
    let mut player = Player::new(Vec3::new(0.0, 1.0, 0.0));
    player.set_velocity(Vec3::new(0.0, -1.0, 0.0));

    let ground = Aabb::new(Vec3::new(-5.0, -1.0, -5.0), Vec3::new(5.0, 0.0, 5.0));

    let collision = player.check_collision(&ground);
    assert!(collision);
}

#[test]
fn test_block_collision_resolution() {
    let mut player = Player::new(Vec3::new(0.0, 1.0, 0.0));
    player.set_velocity(Vec3::new(1.0, 0.0, 0.0));

    let block = Aabb::new(Vec3::new(0.5, 0.0, -0.5), Vec3::new(1.5, 1.0, 0.5));

    assert!(player.velocity().x > 0.0);

    if player.check_collision(&block) {
        player.resolve_collision(&block);
    }

    assert_eq!(player.velocity().x, 0.0);
}

#[test]
fn test_sprint_speed_multiplier() {
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    let input_walk = MovementInput {
        forward: true,
        backward: false,
        left: false,
        right: false,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input_walk, 0.05);
    let walk_speed = player.velocity().length();

    player.set_velocity(Vec3::ZERO);

    let input_sprint = MovementInput {
        forward: true,
        backward: false,
        left: false,
        right: false,
        jump: false,
        sprint: true,
    };
    player.apply_movement(input_sprint, 0.05);
    let sprint_speed = player.velocity().length();

    assert!(sprint_speed > walk_speed * 1.25);
}

#[test]
fn test_gravity_constant() {
    // Minecraft gravity is -32 m/s² (or blocks/s²)
    assert_eq!(GRAVITY, -32.0);
}
