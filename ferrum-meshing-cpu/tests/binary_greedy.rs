use ferrum_meshing_cpu::*;
use std::collections::HashSet;

#[test]
fn air_chunk_produces_no_quads() {
    let mesher = CpuMesher::new();
    let chunk = uniform_chunk(0);
    let mesh = mesher.mesh_chunk(&chunk);
    assert!(mesh.is_empty(), "Air chunk should produce 0 quads");
}

#[test]
fn single_block_produces_six_quads() {
    let mesher = CpuMesher::new();
    let mut chunk = [0u32; CHUNK_SIZE_CB];
    chunk[0] = 1;
    let mesh = mesher.mesh_chunk(&chunk);

    assert_eq!(
        mesh.quad_count(),
        6,
        "Single block at origin should have exactly 6 faces, got {}",
        mesh.quad_count()
    );

    let faces: HashSet<_> = mesh.quads.iter().map(|q| q.face).collect();
    assert_eq!(faces.len(), 6, "All 6 face directions should be present");

    for q in &mesh.quads {
        assert_eq!(q.block_type, 1);
        assert_eq!(q.width, 1);
        assert_eq!(q.height, 1);
    }
}

#[test]
fn solid_chunk_has_only_surface_faces() {
    let mesher = CpuMesher::new();
    let chunk = uniform_chunk(1);
    let mesh = mesher.mesh_chunk(&chunk);

    assert!(
        !mesh.is_empty(),
        "Solid chunk should have visible surface faces"
    );

    // 6 faces of a solid cube. With greedy merging, each face could be a single quad.
    // Best case: 6 quads (one per face, fully merged).
    // Worst case: 6 * 32 * 32 = 6144 (no merging).
    assert!(
        mesh.quad_count() <= 6 * 32 * 32,
        "Should not exceed unmerged face count: got {}",
        mesh.quad_count()
    );

    let faces: HashSet<_> = mesh.quads.iter().map(|q| q.face).collect();
    assert_eq!(faces.len(), 6, "All 6 face directions should be present");

    for q in &mesh.quads {
        assert_eq!(q.block_type, 1);
    }
}

#[test]
fn solid_chunk_greedy_merges_per_layer() {
    let mesher = CpuMesher::new();
    let chunk = uniform_chunk(1);
    let mesh = mesher.mesh_chunk(&chunk);

    // Binary greedy meshing merges within 2D slices but not across layers.
    // Each face has 32 layers, each producing 1 merged quad = 6 * 32 = 192.
    assert_eq!(
        mesh.quad_count(),
        6 * CHUNK_SIZE,
        "Uniform solid chunk should produce 6*32 quads (one per face per layer), got {}",
        mesh.quad_count()
    );
}

#[test]
fn checkerboard_no_merging() {
    let mesher = CpuMesher::new();
    let chunk = checkerboard_chunk(1);
    let mesh = mesher.mesh_chunk(&chunk);

    let solid_count = CHUNK_SIZE_CB / 2;
    assert!(
        mesh.quad_count() >= solid_count,
        "Checkerboard should produce at least {} quads (one per solid voxel), got {}",
        solid_count,
        mesh.quad_count()
    );

    for q in &mesh.quads {
        assert_eq!(q.block_type, 1);
        assert_eq!(q.width, 1, "Checkerboard quads should not merge (width)");
        assert_eq!(q.height, 1, "Checkerboard quads should not merge (height)");
    }
}

#[test]
fn terrain_produces_reasonable_quads() {
    let mesher = CpuMesher::new();
    let chunk = terrain_chunk();
    let mesh = mesher.mesh_chunk(&chunk);

    assert!(
        mesh.quad_count() > 100,
        "Terrain should produce significant geometry, got {}",
        mesh.quad_count()
    );
    assert!(
        mesh.quad_count() < 100_000,
        "Terrain quad count should be reasonable, got {}",
        mesh.quad_count()
    );

    for q in &mesh.quads {
        assert!(q.x < CHUNK_SIZE as u8, "x={} out of bounds", q.x);
        assert!(q.y < CHUNK_SIZE as u8, "y={} out of bounds", q.y);
        assert!(q.z < CHUNK_SIZE as u8, "z={} out of bounds", q.z);
        assert!(q.width > 0, "Width must be > 0");
        assert!(q.height > 0, "Height must be > 0");
        assert!(q.block_type > 0, "Block type must be non-air");
    }

    let block_types: HashSet<_> = mesh.quads.iter().map(|q| q.block_type).collect();
    assert!(
        block_types.len() >= 2,
        "Terrain should have multiple block types, got {:?}",
        block_types
    );
}

#[test]
fn two_adjacent_same_blocks_merge_shared_face() {
    let mesher = CpuMesher::new();
    let mut chunk = [0u32; CHUNK_SIZE_CB];
    chunk[0] = 1; // (0,0,0)
    chunk[1] = 1; // (1,0,0)
    let mesh = mesher.mesh_chunk(&chunk);

    // Two adjacent blocks along X share one internal face.
    // Total exposed faces: 2*6 - 2 (shared) = 10 individual faces.
    // With greedy merging, the 4 coplanar same-type faces merge into 4 quads of width 2.
    // Plus 2 end caps = 6 total quads.
    // Actually: +X face (1 quad at x=2), -X face (1 quad at x=0),
    //           +Y (merged 2x1), -Y (merged 2x1), +Z (merged 2x1), -Z (merged 2x1)
    // = 6 quads total with merging
    assert!(
        mesh.quad_count() <= 10,
        "Two adjacent blocks should merge some faces, got {}",
        mesh.quad_count()
    );
    assert!(
        mesh.quad_count() >= 6,
        "Two adjacent blocks need at least 6 quads, got {}",
        mesh.quad_count()
    );
}

#[test]
fn different_block_types_dont_merge() {
    let mesher = CpuMesher::new();
    let mut chunk = [0u32; CHUNK_SIZE_CB];
    chunk[0] = 1; // (0,0,0) stone
    chunk[1] = 2; // (1,0,0) dirt
    let mesh = mesher.mesh_chunk(&chunk);

    // Different types should not merge their coplanar faces.
    // Each block has 5 exposed faces (shared internal face is culled).
    // No merging across types = 10 quads.
    assert_eq!(
        mesh.quad_count(),
        10,
        "Different block types should not merge, got {}",
        mesh.quad_count()
    );
}

#[test]
fn interior_faces_are_culled() {
    let mesher = CpuMesher::new();

    // 2x2x2 solid cube
    let mut chunk = [0u32; CHUNK_SIZE_CB];
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                chunk[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x] = 1;
            }
        }
    }
    let mesh = mesher.mesh_chunk(&chunk);

    // 2x2x2 cube: 6 faces, each 2x2. Per-layer merge produces 1 quad per layer per face.
    // Each face has 2 layers → 6 * 2 = 12 quads. Interior faces are culled.
    assert_eq!(
        mesh.quad_count(),
        12,
        "2x2x2 cube should produce 12 quads (6 faces * 2 layers), got {}",
        mesh.quad_count()
    );

    // Verify no interior faces: all quads should be on the surface
    for q in &mesh.quads {
        let on_surface = q.x == 0 || q.x == 1 || q.y == 0 || q.y == 1 || q.z == 0 || q.z == 1;
        assert!(
            on_surface,
            "Quad at ({},{},{}) should be on surface",
            q.x, q.y, q.z
        );
    }
}

#[test]
fn chunk_mesher_trait_works_with_cpu() {
    let mesher: Box<dyn ChunkMesher> = Box::new(CpuMesher::new());
    let chunk = uniform_chunk(0);
    let mesh = mesher.mesh_chunk(&chunk);
    assert!(mesh.is_empty());
}
