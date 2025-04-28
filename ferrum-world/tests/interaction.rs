use ferrum_core::BlockId;
use ferrum_world::{BlockInteraction, Chunk};
use glam::Vec3;

#[test]
fn test_chunk_creation() {
    let chunk = Chunk::new();
    // All blocks should be air (BlockId(0)) by default
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                assert_eq!(chunk.get_block(x, y, z), BlockId::new(0));
            }
        }
    }
}

#[test]
fn test_set_and_get_block() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);

    chunk.set_block(5, 10, 15, stone);
    assert_eq!(chunk.get_block(5, 10, 15), stone);

    // Other blocks should still be air
    assert_eq!(chunk.get_block(0, 0, 0), BlockId::new(0));
    assert_eq!(chunk.get_block(31, 31, 31), BlockId::new(0));
}

#[test]
fn test_break_block() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);

    // Place a stone block
    chunk.set_block(10, 10, 10, stone);
    assert_eq!(chunk.get_block(10, 10, 10), stone);

    // Break it (should become air)
    chunk.break_block(10, 10, 10);
    assert_eq!(chunk.get_block(10, 10, 10), BlockId::new(0));
}

#[test]
fn test_place_block_on_air() {
    let mut chunk = Chunk::new();
    let dirt = BlockId::new(3);

    // Should succeed placing on air
    assert!(chunk.place_block(5, 5, 5, dirt));
    assert_eq!(chunk.get_block(5, 5, 5), dirt);
}

#[test]
fn test_place_block_on_existing_block_fails() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);
    let dirt = BlockId::new(3);

    // Place stone first
    chunk.set_block(5, 5, 5, stone);

    // Trying to place dirt on stone should fail
    assert!(!chunk.place_block(5, 5, 5, dirt));
    assert_eq!(chunk.get_block(5, 5, 5), stone); // Should still be stone
}

#[test]
fn test_raycast_hits_block() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);

    // Place a stone block at (10, 10, 10)
    chunk.set_block(10, 10, 10, stone);

    // Raycast from (5, 10, 10) looking in +X direction
    let origin = Vec3::new(5.0, 10.5, 10.5);
    let direction = Vec3::new(1.0, 0.0, 0.0);

    let hit = chunk.raycast(origin, direction, 10.0);
    assert!(hit.is_some());

    let (x, y, z) = hit.unwrap();
    assert_eq!((x, y, z), (10, 10, 10));
}

#[test]
fn test_raycast_misses_block() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);

    // Place a stone block at (10, 10, 10)
    chunk.set_block(10, 10, 10, stone);

    // Raycast from (5, 5, 5) looking in -X direction (away from block)
    let origin = Vec3::new(5.0, 5.0, 5.0);
    let direction = Vec3::new(-1.0, 0.0, 0.0);

    let hit = chunk.raycast(origin, direction, 10.0);
    assert!(hit.is_none());
}

#[test]
fn test_raycast_max_distance() {
    let mut chunk = Chunk::new();
    let stone = BlockId::new(1);

    // Place a stone block at (20, 10, 10)
    chunk.set_block(20, 10, 10, stone);

    // Raycast from (5, 10, 10) with max distance of 10 (won't reach block at distance 15)
    let origin = Vec3::new(5.0, 10.5, 10.5);
    let direction = Vec3::new(1.0, 0.0, 0.0);

    let hit = chunk.raycast(origin, direction, 10.0);
    assert!(hit.is_none());
}

#[test]
fn test_out_of_bounds_returns_air() {
    let chunk = Chunk::new();

    // Out of bounds coordinates should return air
    assert_eq!(chunk.get_block(32, 0, 0), BlockId::new(0));
    assert_eq!(chunk.get_block(0, 32, 0), BlockId::new(0));
    assert_eq!(chunk.get_block(0, 0, 32), BlockId::new(0));
}
