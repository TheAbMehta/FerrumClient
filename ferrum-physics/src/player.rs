use crate::collision::Aabb;
use crate::gravity;
use crate::movement::MovementInput;
use glam::Vec3;

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;

pub struct Player {
    position: Vec3,
    velocity: Vec3,
    on_ground: bool,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            on_ground: false,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    pub fn set_on_ground(&mut self, on_ground: bool) {
        self.on_ground = on_ground;
    }

    pub fn aabb(&self) -> Aabb {
        let half_width = PLAYER_WIDTH / 2.0;
        let min = Vec3::new(
            self.position.x - half_width,
            self.position.y,
            self.position.z - half_width,
        );
        let max = Vec3::new(
            self.position.x + half_width,
            self.position.y + PLAYER_HEIGHT,
            self.position.z + half_width,
        );
        Aabb::new(min, max)
    }

    pub fn apply_movement(&mut self, input: MovementInput, dt: f32) {
        self.velocity = input.calculate_velocity(self.velocity, self.on_ground, dt);

        if input.jump && self.on_ground {
            self.velocity = gravity::apply_jump(self.velocity, self.on_ground);
            self.on_ground = false;
        }
    }

    pub fn apply_gravity(&mut self, dt: f32) {
        self.velocity = gravity::apply_gravity(self.velocity, self.on_ground, dt);
    }

    pub fn update_position(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }

    pub fn check_collision(&self, other: &Aabb) -> bool {
        self.aabb().intersects(other)
    }

    pub fn resolve_collision(&mut self, other: &Aabb) {
        if let Some(penetration) = self.aabb().penetration(other) {
            self.position -= penetration;

            if penetration.x.abs() > 0.0 {
                self.velocity.x = 0.0;
            }
            if penetration.y.abs() > 0.0 {
                self.velocity.y = 0.0;
                if penetration.y < 0.0 {
                    self.on_ground = true;
                }
            }
            if penetration.z.abs() > 0.0 {
                self.velocity.z = 0.0;
            }
        }
    }
}
