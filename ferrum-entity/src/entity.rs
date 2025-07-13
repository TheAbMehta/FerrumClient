use azalea_core::position::Vec3;
use azalea_registry::builtin::EntityKind;
use uuid::Uuid;

/// Represents an entity in the Minecraft world
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// Entity's unique identifier within the world
    pub id: i32,
    /// Entity's UUID
    pub uuid: Uuid,
    /// Entity type (e.g., zombie, chicken, item)
    pub entity_type: EntityKind,
    /// Current position in the world
    pub position: Vec3,
    /// Yaw rotation (horizontal, in degrees)
    pub yaw: f32,
    /// Pitch rotation (vertical, in degrees)
    pub pitch: f32,
    /// Head yaw rotation (in degrees)
    pub head_yaw: f32,
    /// Current velocity
    pub velocity: Vec3,
    /// Whether the entity is on the ground
    pub on_ground: bool,
    /// Additional data (entity-specific, e.g., item stack for dropped items)
    pub data: i32,
}

impl Entity {
    pub fn new(
        id: i32,
        uuid: Uuid,
        entity_type: EntityKind,
        position: Vec3,
        yaw: f32,
        pitch: f32,
        head_yaw: f32,
        data: i32,
    ) -> Self {
        Self {
            id,
            uuid,
            entity_type,
            position,
            yaw,
            pitch,
            head_yaw,
            velocity: Vec3::default(),
            on_ground: false,
            data,
        }
    }

    pub fn update_position(&mut self, new_position: Vec3) {
        self.position = new_position;
    }

    pub fn update_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn update_position_and_rotation(&mut self, new_position: Vec3, yaw: f32, pitch: f32) {
        self.position = new_position;
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn update_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    pub fn update_head_yaw(&mut self, head_yaw: f32) {
        self.head_yaw = head_yaw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            1,
            Uuid::new_v4(),
            EntityKind::Zombie,
            Vec3::new(10.0, 64.0, 10.0),
            0.0,
            0.0,
            0.0,
            0,
        );

        assert_eq!(entity.id, 1);
        assert_eq!(entity.entity_type, EntityKind::Zombie);
        assert_eq!(entity.position, Vec3::new(10.0, 64.0, 10.0));
    }

    #[test]
    fn test_update_position() {
        let mut entity = Entity::new(
            1,
            Uuid::new_v4(),
            EntityKind::Zombie,
            Vec3::new(10.0, 64.0, 10.0),
            0.0,
            0.0,
            0.0,
            0,
        );

        entity.update_position(Vec3::new(20.0, 64.0, 20.0));
        assert_eq!(entity.position, Vec3::new(20.0, 64.0, 20.0));
    }

    #[test]
    fn test_update_rotation() {
        let mut entity = Entity::new(
            1,
            Uuid::new_v4(),
            EntityKind::Zombie,
            Vec3::new(10.0, 64.0, 10.0),
            0.0,
            0.0,
            0.0,
            0,
        );

        entity.update_rotation(90.0, 45.0);
        assert_eq!(entity.yaw, 90.0);
        assert_eq!(entity.pitch, 45.0);
    }

    #[test]
    fn test_update_velocity() {
        let mut entity = Entity::new(
            1,
            Uuid::new_v4(),
            EntityKind::Zombie,
            Vec3::new(10.0, 64.0, 10.0),
            0.0,
            0.0,
            0.0,
            0,
        );

        entity.update_velocity(Vec3::new(1.0, 0.5, 0.0));
        assert_eq!(entity.velocity, Vec3::new(1.0, 0.5, 0.0));
    }
}
