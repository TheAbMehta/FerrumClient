use ferrum_meshing_gpu::*;

fn get_mesher() -> GpuChunkMesher {
    GpuChunkMesher::new().expect("Failed to create GPU mesher - is a GPU available?")
}

#[test]
fn uniform_air_produces_no_quads() {
    let mesher = get_mesher();
    let chunk = uniform_chunk(0);
    let quads = mesher.mesh_chunk(&chunk);
    assert_eq!(quads.len(), 0, "Air chunk should produce 0 quads");
}

#[test]
fn uniform_stone_produces_only_surface_quads() {
    let mesher = get_mesher();
    let chunk = uniform_chunk(1);
    let quads = mesher.mesh_chunk(&chunk);

    // A solid 32x32x32 cube has 6 faces, each 32x32.
    // With greedy merging along depth axis, each face should produce
    // at most 32 rows of merged quads (one per row, merged along depth).
    // The exact count depends on merging strategy, but it must be > 0
    // (surface faces are visible) and reasonable (< 6*32*32 = 6144 unmerged).
    assert!(
        !quads.is_empty(),
        "Solid chunk should have visible surface faces"
    );
    assert!(
        quads.len() <= 6 * 32 * 32,
        "Should not exceed unmerged face count: got {}",
        quads.len()
    );

    for quad in &quads {
        assert_eq!(
            quad.block_type, 1,
            "All quads should be stone (block_type=1)"
        );
    }

    let mut faces_seen = [false; 6];
    for quad in &quads {
        let f = quad.face() as usize;
        assert!(f < 6, "Invalid face direction: {}", f);
        faces_seen[f] = true;
    }
    for (i, seen) in faces_seen.iter().enumerate() {
        assert!(
            seen,
            "Face direction {} should be present for solid cube",
            i
        );
    }
}

#[test]
fn checkerboard_produces_maximum_quads() {
    let mesher = get_mesher();
    let chunk = checkerboard_chunk(1);
    let quads = mesher.mesh_chunk(&chunk);

    // Checkerboard: every solid voxel has all 6 faces exposed (all neighbors are air).
    // 32^3 / 2 = 16384 solid voxels, each with 6 faces = 98304 total faces.
    // With depth-axis merging, some faces may merge but checkerboard prevents most merging.
    // We expect a large number of quads.
    let solid_count = 16384; // 32^3 / 2
    assert!(
        quads.len() >= solid_count,
        "Checkerboard should produce at least {} quads (one per solid voxel), got {}",
        solid_count,
        quads.len()
    );

    for quad in &quads {
        assert_eq!(quad.block_type, 1);
    }

    for quad in &quads {
        assert_eq!(
            quad.width(),
            1,
            "Checkerboard quads should not merge (width)"
        );
        assert_eq!(
            quad.height(),
            1,
            "Checkerboard quads should not merge (height)"
        );
    }
}

#[test]
fn realistic_terrain_produces_reasonable_quads() {
    let mesher = get_mesher();
    let chunk = terrain_chunk();
    let quads = mesher.mesh_chunk(&chunk);

    println!("Generated {} quads for terrain chunk", quads.len());

    assert!(
        quads.len() > 100,
        "Terrain should produce significant geometry, got {}",
        quads.len()
    );
    assert!(
        quads.len() < 100_000,
        "Terrain quad count should be reasonable, got {}",
        quads.len()
    );

    for quad in &quads {
        assert!(quad.x() < CHUNK_SIZE as u32, "x={} out of bounds", quad.x());
        assert!(quad.y() < CHUNK_SIZE as u32, "y={} out of bounds", quad.y());
        assert!(quad.z() < CHUNK_SIZE as u32, "z={} out of bounds", quad.z());
        assert!(quad.face() < 6, "Invalid face: {}", quad.face());
        assert!(quad.width() > 0, "Width must be > 0");
        assert!(quad.height() > 0, "Height must be > 0");
        assert!(quad.block_type > 0, "Block type must be non-air");
    }

    let mut block_types_seen = std::collections::HashSet::new();
    for quad in &quads {
        block_types_seen.insert(quad.block_type);
    }
    assert!(
        block_types_seen.len() >= 2,
        "Terrain should have multiple block types, got {:?}",
        block_types_seen
    );
}

#[test]
fn all_normals_are_axis_aligned() {
    let mesher = get_mesher();
    let chunk = terrain_chunk();
    let quads = mesher.mesh_chunk(&chunk);

    for quad in &quads {
        let face = quad.face();
        assert!(
            face <= 5,
            "Face {} is not a valid axis-aligned direction",
            face
        );
    }
}

#[test]
fn no_duplicate_quads() {
    let mesher = get_mesher();

    let mut chunk = [0u32; CHUNK_SIZE_CB];
    chunk[0] = 1;
    let quads = mesher.mesh_chunk(&chunk);

    assert_eq!(
        quads.len(),
        6,
        "Single block should have exactly 6 quads, got {}",
        quads.len()
    );

    for i in 0..quads.len() {
        for j in (i + 1)..quads.len() {
            assert_ne!(
                quads[i], quads[j],
                "Found duplicate quads at indices {} and {}",
                i, j
            );
        }
    }
}
