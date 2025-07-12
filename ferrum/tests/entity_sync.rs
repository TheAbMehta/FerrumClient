use azalea_core::entity_id::MinecraftEntityId;
use azalea_core::position::Vec3;
use azalea_registry::builtin::EntityKind;
use ferrum::network::entity_sync::EntitySync;
use uuid::Uuid;

#[test]
fn test_entity_sync_creation() {
    let entity_sync = EntitySync::new();
    assert_eq!(entity_sync.entity_count(), 0);
}

#[test]
fn test_spawn_entity() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);
    let uuid = Uuid::new_v4();

    entity_sync.spawn_entity(
        entity_id,
        uuid,
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    assert_eq!(entity_sync.entity_count(), 1);
    assert!(entity_sync.has_entity(entity_id));

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.id, 123);
    assert_eq!(entity.uuid, uuid);
    assert_eq!(entity.entity_type, EntityKind::Zombie);
    assert_eq!(entity.position, Vec3::new(10.0, 64.0, 10.0));
}

#[test]
fn test_despawn_entity() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    assert_eq!(entity_sync.entity_count(), 1);

    entity_sync.despawn_entity(entity_id);

    assert_eq!(entity_sync.entity_count(), 0);
    assert!(!entity_sync.has_entity(entity_id));
}

#[test]
fn test_update_entity_position() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.update_entity_position(entity_id, Vec3::new(20.0, 64.0, 20.0));

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.position, Vec3::new(20.0, 64.0, 20.0));
}

#[test]
fn test_update_entity_rotation() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.update_entity_rotation(entity_id, 90.0, 45.0);

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.yaw, 90.0);
    assert_eq!(entity.pitch, 45.0);
}

#[test]
fn test_update_entity_position_and_rotation() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.update_entity_position_and_rotation(
        entity_id,
        Vec3::new(20.0, 64.0, 20.0),
        90.0,
        45.0,
    );

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.position, Vec3::new(20.0, 64.0, 20.0));
    assert_eq!(entity.yaw, 90.0);
    assert_eq!(entity.pitch, 45.0);
}

#[test]
fn test_update_entity_velocity() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.update_entity_velocity(entity_id, Vec3::new(1.0, 0.5, 0.0));

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.velocity, Vec3::new(1.0, 0.5, 0.0));
}

#[test]
fn test_update_entity_head_yaw() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(123);

    entity_sync.spawn_entity(
        entity_id,
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.update_entity_head_yaw(entity_id, 180.0);

    let entity = entity_sync.get_entity(entity_id).unwrap();
    assert_eq!(entity.head_yaw, 180.0);
}

#[test]
fn test_multiple_entities() {
    let mut entity_sync = EntitySync::new();

    entity_sync.spawn_entity(
        MinecraftEntityId(1),
        Uuid::new_v4(),
        EntityKind::Zombie,
        Vec3::new(10.0, 64.0, 10.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.spawn_entity(
        MinecraftEntityId(2),
        Uuid::new_v4(),
        EntityKind::Chicken,
        Vec3::new(20.0, 64.0, 20.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    entity_sync.spawn_entity(
        MinecraftEntityId(3),
        Uuid::new_v4(),
        EntityKind::Item,
        Vec3::new(30.0, 64.0, 30.0),
        0.0,
        0.0,
        0.0,
        0,
    );

    assert_eq!(entity_sync.entity_count(), 3);

    entity_sync.despawn_entity(MinecraftEntityId(2));

    assert_eq!(entity_sync.entity_count(), 2);
    assert!(entity_sync.has_entity(MinecraftEntityId(1)));
    assert!(!entity_sync.has_entity(MinecraftEntityId(2)));
    assert!(entity_sync.has_entity(MinecraftEntityId(3)));
}

#[test]
fn test_get_nonexistent_entity() {
    let entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(999);

    assert!(entity_sync.get_entity(entity_id).is_none());
}

#[test]
fn test_update_nonexistent_entity_position() {
    let mut entity_sync = EntitySync::new();
    let entity_id = MinecraftEntityId(999);

    entity_sync.update_entity_position(entity_id, Vec3::new(20.0, 64.0, 20.0));

    assert!(!entity_sync.has_entity(entity_id));
}
