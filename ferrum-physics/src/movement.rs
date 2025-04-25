use glam::Vec3;

#[derive(Debug, Clone, Copy, Default)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
}

const WALK_SPEED: f32 = 4.317;
const SPRINT_MULTIPLIER: f32 = 1.3;
const FRICTION: f32 = 0.546;

impl MovementInput {
    pub fn calculate_velocity(&self, current_velocity: Vec3, on_ground: bool, _dt: f32) -> Vec3 {
        if !on_ground {
            return current_velocity;
        }

        let mut direction = Vec3::ZERO;

        if self.forward {
            direction.z -= 1.0;
        }
        if self.backward {
            direction.z += 1.0;
        }
        if self.left {
            direction.x -= 1.0;
        }
        if self.right {
            direction.x += 1.0;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        let speed = if self.sprint {
            WALK_SPEED * SPRINT_MULTIPLIER
        } else {
            WALK_SPEED
        };

        let target_velocity = direction * speed;
        let acceleration = 0.098;

        let mut new_velocity = current_velocity;
        new_velocity.x += (target_velocity.x - current_velocity.x) * acceleration;
        new_velocity.z += (target_velocity.z - current_velocity.z) * acceleration;

        new_velocity.x *= FRICTION;
        new_velocity.z *= FRICTION;

        new_velocity
    }
}
