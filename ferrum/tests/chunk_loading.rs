use ferrum_world::{Chunk, ChunkPos, World};

#[test]
fn test_world_creation() {
    let world = World::new();
    assert!(world.is_empty(), "Newly created world should be empty");
}

#[test]
fn test_store_chunk() {
    let mut world = World::new();
    let pos = ChunkPos { x: 0, z: 0 };
    let chunk = Chunk::new();

    world.set_chunk(pos, chunk);

    assert!(
        !world.is_empty(),
        "World should contain chunk after insertion"
    );
    assert!(
        world.has_chunk(pos),
        "World should have chunk at position (0, 0)"
    );
}

#[test]
fn test_get_chunk() {
    let mut world = World::new();
    let pos = ChunkPos { x: 5, z: -3 };
    let chunk = Chunk::new();

    world.set_chunk(pos, chunk);

    let retrieved = world.get_chunk(pos);
    assert!(
        retrieved.is_some(),
        "Should be able to retrieve stored chunk"
    );
}

#[test]
fn test_get_nonexistent_chunk() {
    let world = World::new();
    let pos = ChunkPos { x: 10, z: 10 };

    let retrieved = world.get_chunk(pos);
    assert!(retrieved.is_none(), "Nonexistent chunk should return None");
}

#[test]
fn test_remove_chunk() {
    let mut world = World::new();
    let pos = ChunkPos { x: 2, z: 4 };
    let chunk = Chunk::new();

    world.set_chunk(pos, chunk);
    assert!(world.has_chunk(pos), "Chunk should exist before removal");

    let removed = world.remove_chunk(pos);
    assert!(removed.is_some(), "Removed chunk should be returned");
    assert!(
        !world.has_chunk(pos),
        "Chunk should not exist after removal"
    );
}

#[test]
fn test_remove_nonexistent_chunk() {
    let mut world = World::new();
    let pos = ChunkPos { x: 99, z: 99 };

    let removed = world.remove_chunk(pos);
    assert!(
        removed.is_none(),
        "Removing nonexistent chunk should return None"
    );
}

#[test]
fn test_multiple_chunks() {
    let mut world = World::new();

    for x in -5..5 {
        for z in -5..5 {
            let pos = ChunkPos { x, z };
            world.set_chunk(pos, Chunk::new());
        }
    }

    assert_eq!(world.chunk_count(), 100, "Should have 10x10 = 100 chunks");

    for x in -5..5 {
        for z in -5..5 {
            let pos = ChunkPos { x, z };
            assert!(world.has_chunk(pos), "Chunk at ({}, {}) should exist", x, z);
        }
    }
}

#[test]
fn test_chunk_pos_equality() {
    let pos1 = ChunkPos { x: 10, z: 20 };
    let pos2 = ChunkPos { x: 10, z: 20 };
    let pos3 = ChunkPos { x: 10, z: 21 };

    assert_eq!(pos1, pos2, "Same positions should be equal");
    assert_ne!(pos1, pos3, "Different positions should not be equal");
}

#[test]
fn test_chunk_pos_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    let pos1 = ChunkPos { x: 1, z: 2 };
    let pos2 = ChunkPos { x: 1, z: 2 };

    set.insert(pos1);
    assert!(set.contains(&pos2), "HashSet should find equal ChunkPos");
}
