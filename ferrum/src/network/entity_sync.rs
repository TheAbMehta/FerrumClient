use azalea_core::entity_id::MinecraftEntityId;
use azalea_core::position::Vec3;
use azalea_protocol::packets::game::ClientboundGamePacket;
use azalea_registry::builtin::EntityKind;
use bevy::prelude::*;
use ferrum_entity::Entity;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Resource)]
pub struct EntitySync {
    entities: HashMap<MinecraftEntityId, Entity>,
}

impl EntitySync {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn has_entity(&self, entity_id: MinecraftEntityId) -> bool {
        self.entities.contains_key(&entity_id)
    }

    pub fn get_entity(&self, entity_id: MinecraftEntityId) -> Option<&Entity> {
        self.entities.get(&entity_id)
    }

    pub fn spawn_entity(
        &mut self,
        entity_id: MinecraftEntityId,
        uuid: Uuid,
        entity_type: EntityKind,
        position: Vec3,
        yaw: f32,
        pitch: f32,
        head_yaw: f32,
        data: i32,
    ) {
        let entity = Entity::new(
            entity_id.0,
            uuid,
            entity_type,
            position,
            yaw,
            pitch,
            head_yaw,
            data,
        );
        self.entities.insert(entity_id, entity);
    }

    pub fn despawn_entity(&mut self, entity_id: MinecraftEntityId) {
        self.entities.remove(&entity_id);
    }

    pub fn update_entity_position(&mut self, entity_id: MinecraftEntityId, position: Vec3) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.update_position(position);
        }
    }

    pub fn update_entity_rotation(&mut self, entity_id: MinecraftEntityId, yaw: f32, pitch: f32) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.update_rotation(yaw, pitch);
        }
    }

    pub fn update_entity_position_and_rotation(
        &mut self,
        entity_id: MinecraftEntityId,
        position: Vec3,
        yaw: f32,
        pitch: f32,
    ) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.update_position_and_rotation(position, yaw, pitch);
        }
    }

    pub fn update_entity_velocity(&mut self, entity_id: MinecraftEntityId, velocity: Vec3) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.update_velocity(velocity);
        }
    }

    pub fn update_entity_head_yaw(&mut self, entity_id: MinecraftEntityId, head_yaw: f32) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.update_head_yaw(head_yaw);
        }
    }
}

impl Default for EntitySync {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy system that processes entity-related packets from the server
pub fn handle_entity_packets(
    mut entity_sync: ResMut<EntitySync>,
    // TODO: Get packets from ServerConnection
) {
    // TODO: Process entity spawn/despawn/update packets
    // This will be connected to the persistent connection packet stream
}

/// Plugin for entity synchronization
pub struct EntitySyncPlugin;

impl Plugin for EntitySyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitySync>()
            .add_systems(Update, handle_entity_packets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_sync_default() {
        let entity_sync = EntitySync::default();
        assert_eq!(entity_sync.entity_count(), 0);
    }

    #[test]
    fn test_spawn_and_get_entity() {
        let mut entity_sync = EntitySync::new();
        let entity_id = MinecraftEntityId(42);
        let uuid = Uuid::new_v4();

        entity_sync.spawn_entity(
            entity_id,
            uuid,
            EntityKind::Creeper,
            Vec3::new(5.0, 70.0, 5.0),
            45.0,
            30.0,
            45.0,
            0,
        );

        let entity = entity_sync.get_entity(entity_id).unwrap();
        assert_eq!(entity.id, 42);
        assert_eq!(entity.uuid, uuid);
        assert_eq!(entity.entity_type, EntityKind::Creeper);
        assert_eq!(entity.position, Vec3::new(5.0, 70.0, 5.0));
        assert_eq!(entity.yaw, 45.0);
        assert_eq!(entity.pitch, 30.0);
    }
}
