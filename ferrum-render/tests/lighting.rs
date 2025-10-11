use ferrum_render::lighting::{LightingEngine, CHUNK_SIZE};

#[test]
fn test_lighting_engine_creation() {
    let lighting = LightingEngine::new();
    // All light values should start at 0
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                assert_eq!(lighting.get_block_light(x, y, z), 0);
                assert_eq!(lighting.get_sky_light(x, y, z), 0);
            }
        }
    }
}

#[test]
fn test_set_and_get_block_light() {
    let mut lighting = LightingEngine::new();
    lighting.set_block_light(5, 10, 15, 14);
    assert_eq!(lighting.get_block_light(5, 10, 15), 14);
    assert_eq!(lighting.get_block_light(5, 10, 14), 0); // Adjacent block unaffected
}

#[test]
fn test_set_and_get_sky_light() {
    let mut lighting = LightingEngine::new();
    lighting.set_sky_light(5, 10, 15, 15);
    assert_eq!(lighting.get_sky_light(5, 10, 15), 15);
    assert_eq!(lighting.get_sky_light(5, 10, 14), 0); // Adjacent block unaffected
}

#[test]
fn test_light_values_clamped_to_15() {
    let mut lighting = LightingEngine::new();
    lighting.set_block_light(0, 0, 0, 20); // Try to set > 15
    assert_eq!(lighting.get_block_light(0, 0, 0), 15); // Should clamp to 15
}

#[test]
fn test_out_of_bounds_returns_zero() {
    let lighting = LightingEngine::new();
    assert_eq!(lighting.get_block_light(100, 100, 100), 0);
    assert_eq!(lighting.get_sky_light(100, 100, 100), 0);
}

#[test]
fn test_block_light_propagation_single_source() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Place torch at center (light level 14)
    lighting.set_block_light(16, 16, 16, 14);
    lighting.propagate_block_light(&opaque);

    // Adjacent blocks should be 13
    assert_eq!(lighting.get_block_light(17, 16, 16), 13);
    assert_eq!(lighting.get_block_light(15, 16, 16), 13);
    assert_eq!(lighting.get_block_light(16, 17, 16), 13);
    assert_eq!(lighting.get_block_light(16, 15, 16), 13);
    assert_eq!(lighting.get_block_light(16, 16, 17), 13);
    assert_eq!(lighting.get_block_light(16, 16, 15), 13);

    // Diagonal blocks should be 12 (14 - 1 - 1)
    assert_eq!(lighting.get_block_light(17, 17, 16), 12);
    assert_eq!(lighting.get_block_light(15, 15, 16), 12);
}

#[test]
fn test_block_light_propagation_stops_at_zero() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Place weak light source (level 3)
    lighting.set_block_light(16, 16, 16, 3);
    lighting.propagate_block_light(&opaque);

    // Should propagate 3 blocks away
    assert_eq!(lighting.get_block_light(19, 16, 16), 0); // 3 blocks away = 0
    assert_eq!(lighting.get_block_light(18, 16, 16), 1); // 2 blocks away = 1
    assert_eq!(lighting.get_block_light(17, 16, 16), 2); // 1 block away = 2
}

#[test]
fn test_opaque_blocks_stop_light() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Place torch
    lighting.set_block_light(16, 16, 16, 14);

    // Place opaque block adjacent to torch
    opaque[17][16][16] = true;

    lighting.propagate_block_light(&opaque);

    // Opaque block itself should have no light
    assert_eq!(lighting.get_block_light(17, 16, 16), 0);

    // Light can go around the opaque block (Minecraft flood-fill behavior)
    assert_eq!(lighting.get_block_light(18, 16, 16), 10);

    // Light should propagate in other directions normally
    assert_eq!(lighting.get_block_light(15, 16, 16), 13);
}

#[test]
fn test_sky_light_propagation_downward() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Set sky light at top of chunk (y=31)
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            lighting.set_sky_light(x, 31, z, 15);
        }
    }

    lighting.propagate_sky_light(&opaque);

    // Sky light should propagate downward without loss (in air)
    assert_eq!(lighting.get_sky_light(16, 30, 16), 15);
    assert_eq!(lighting.get_sky_light(16, 20, 16), 15);
    assert_eq!(lighting.get_sky_light(16, 10, 16), 15);
    assert_eq!(lighting.get_sky_light(16, 0, 16), 15);
}

#[test]
fn test_sky_light_blocked_by_opaque() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Set sky light at top
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            lighting.set_sky_light(x, 31, z, 15);
        }
    }

    // Place opaque block at y=20
    opaque[16][20][16] = true;

    lighting.propagate_sky_light(&opaque);

    // Sky light should be full above opaque block
    assert_eq!(lighting.get_sky_light(16, 21, 16), 15);

    // Opaque block itself has no light
    assert_eq!(lighting.get_sky_light(16, 20, 16), 0);

    // Light can reach below by going around (flood-fill behavior)
    assert_eq!(lighting.get_sky_light(16, 19, 16), 14);
}

#[test]
fn test_smooth_lighting_averages_neighbors() {
    let mut lighting = LightingEngine::new();

    // Set up a gradient of light values
    lighting.set_block_light(0, 0, 0, 12);
    lighting.set_block_light(1, 0, 0, 8);
    lighting.set_block_light(0, 1, 0, 4);
    lighting.set_block_light(1, 1, 0, 0);

    // Get smooth light for vertex at (1, 1, 0)
    // Should average the 4 adjacent blocks
    let smooth = lighting.get_smooth_light(1, 1, 0, 0); // Face::Right (index 0)

    // Average of 12, 8, 4, 0 = 24 / 4 = 6
    assert_eq!(smooth, 6);
}

#[test]
fn test_smooth_lighting_uses_max_of_block_and_sky() {
    let mut lighting = LightingEngine::new();

    // Set different values for block and sky light
    lighting.set_block_light(0, 0, 0, 8);
    lighting.set_sky_light(0, 0, 0, 12);

    lighting.set_block_light(1, 0, 0, 4);
    lighting.set_sky_light(1, 0, 0, 10);

    // Smooth lighting should use max(block, sky) for each position
    let smooth = lighting.get_smooth_light(0, 0, 0, 0);

    // Should use max values: 12, 10, ... (depends on face and neighbors)
    assert!(smooth >= 8); // At least the average of some max values
}

#[test]
fn test_get_combined_light_returns_max() {
    let mut lighting = LightingEngine::new();

    lighting.set_block_light(5, 5, 5, 10);
    lighting.set_sky_light(5, 5, 5, 8);

    assert_eq!(lighting.get_combined_light(5, 5, 5), 10); // max(10, 8)

    lighting.set_sky_light(5, 5, 5, 14);
    assert_eq!(lighting.get_combined_light(5, 5, 5), 14); // max(10, 14)
}

#[test]
fn test_multiple_light_sources_combine() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    // Place two torches
    lighting.set_block_light(10, 16, 16, 14);
    lighting.set_block_light(22, 16, 16, 14);

    lighting.propagate_block_light(&opaque);

    // Block between them should have light from both sources
    let mid_light = lighting.get_block_light(16, 16, 16);

    // Should be lit from both sides (at least 8 from each torch)
    assert!(mid_light >= 8);
}

#[test]
fn test_light_propagation_is_symmetric() {
    let mut lighting = LightingEngine::new();
    let mut opaque = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    lighting.set_block_light(16, 16, 16, 14);
    lighting.propagate_block_light(&opaque);

    // Light should propagate equally in all 6 directions
    let light_px = lighting.get_block_light(17, 16, 16);
    let light_nx = lighting.get_block_light(15, 16, 16);
    let light_py = lighting.get_block_light(16, 17, 16);
    let light_ny = lighting.get_block_light(16, 15, 16);
    let light_pz = lighting.get_block_light(16, 16, 17);
    let light_nz = lighting.get_block_light(16, 16, 15);

    assert_eq!(light_px, 13);
    assert_eq!(light_nx, 13);
    assert_eq!(light_py, 13);
    assert_eq!(light_ny, 13);
    assert_eq!(light_pz, 13);
    assert_eq!(light_nz, 13);
}
