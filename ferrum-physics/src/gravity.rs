use glam::Vec3;

pub const GRAVITY: f32 = -32.0;
pub const TERMINAL_VELOCITY: f32 = -78.4;
pub const JUMP_VELOCITY: f32 = 10.0;

pub fn apply_gravity(velocity: Vec3, on_ground: bool, dt: f32) -> Vec3 {
    if on_ground {
        return velocity;
    }

    let mut new_velocity = velocity;
    new_velocity.y += GRAVITY * dt;

    if new_velocity.y < TERMINAL_VELOCITY {
        new_velocity.y = TERMINAL_VELOCITY;
    }

    new_velocity
}

pub fn apply_jump(velocity: Vec3, on_ground: bool) -> Vec3 {
    if !on_ground {
        return velocity;
    }

    let mut new_velocity = velocity;
    new_velocity.y = JUMP_VELOCITY;
    new_velocity
}
