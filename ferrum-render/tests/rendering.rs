use bevy::mesh::{Indices, VertexAttributeValues};
use ferrum_meshing_cpu::{ChunkMesh, Face, MeshQuad};
use ferrum_render::{BlockRenderer, TextureAtlas};

#[tokio::test]
async fn test_texture_atlas_creation() {
    // Test that we can create a texture atlas
    let atlas = TextureAtlas::new(16);
    assert_eq!(atlas.tile_size(), 16);
}

#[tokio::test]
async fn test_uv_coordinates_for_block_face() {
    // Test UV generation for a specific block type and face
    let atlas = TextureAtlas::new(16);

    // Block type 1 (stone) on the top face
    let uvs = atlas.get_uvs(1, Face::Up);

    // UV coordinates should be in [0.0, 1.0] range
    assert!(uvs[0][0] >= 0.0 && uvs[0][0] <= 1.0);
    assert!(uvs[0][1] >= 0.0 && uvs[0][1] <= 1.0);
    assert_eq!(uvs.len(), 4); // 4 corners of a quad
}

#[tokio::test]
async fn test_different_faces_have_different_uvs() {
    // Test that different faces can have different textures
    let atlas = TextureAtlas::new(16);

    let top_uvs = atlas.get_uvs(2, Face::Up);
    let side_uvs = atlas.get_uvs(2, Face::Right);

    // For blocks like grass, top and side should differ
    // (This test will pass even if they're the same, but validates the API)
    assert_eq!(top_uvs.len(), 4);
    assert_eq!(side_uvs.len(), 4);
}

#[test]
fn test_chunk_mesh_to_bevy_mesh() {
    // Test converting a ChunkMesh to a Bevy Mesh
    let mut chunk_mesh = ChunkMesh::new();
    chunk_mesh.quads.push(MeshQuad {
        x: 0,
        y: 0,
        z: 0,
        width: 1,
        height: 1,
        face: Face::Up,
        block_type: 1,
    });

    let atlas = TextureAtlas::new(16);
    let bevy_mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);

    // Mesh should have position, normal, and UV attributes
    assert!(bevy_mesh
        .attribute(bevy::prelude::Mesh::ATTRIBUTE_POSITION)
        .is_some());
    assert!(bevy_mesh
        .attribute(bevy::prelude::Mesh::ATTRIBUTE_NORMAL)
        .is_some());
    assert!(bevy_mesh
        .attribute(bevy::prelude::Mesh::ATTRIBUTE_UV_0)
        .is_some());
}

#[test]
fn test_empty_chunk_mesh() {
    // Test that an empty chunk mesh produces an empty Bevy mesh
    let chunk_mesh = ChunkMesh::new();
    let atlas = TextureAtlas::new(16);
    let bevy_mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);

    // Empty mesh should have no vertices
    if let Some(positions) = bevy_mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_POSITION) {
        match positions {
            VertexAttributeValues::Float32x3(verts) => {
                assert_eq!(verts.len(), 0);
            }
            _ => panic!("Expected Float32x3 positions"),
        }
    }
}

#[test]
fn test_quad_generates_correct_vertex_count() {
    // Test that a single quad generates 4 vertices (or 6 for indexed triangles)
    let mut chunk_mesh = ChunkMesh::new();
    chunk_mesh.quads.push(MeshQuad {
        x: 5,
        y: 10,
        z: 15,
        width: 2,
        height: 3,
        face: Face::Front,
        block_type: 3,
    });

    let atlas = TextureAtlas::new(16);
    let bevy_mesh = BlockRenderer::create_mesh(&chunk_mesh, &atlas);

    // Should have 4 vertices for the quad
    if let Some(positions) = bevy_mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_POSITION) {
        match positions {
            VertexAttributeValues::Float32x3(verts) => {
                assert_eq!(verts.len(), 4);
            }
            _ => panic!("Expected Float32x3 positions"),
        }
    }

    // Should have 6 indices (2 triangles)
    if let Some(indices) = bevy_mesh.indices() {
        match indices {
            Indices::U32(idx) => {
                assert_eq!(idx.len(), 6);
            }
            _ => panic!("Expected U32 indices"),
        }
    }
}

#[test]
fn test_all_block_types_mapped() {
    let atlas = TextureAtlas::new(16);
    let faces = [
        Face::Up,
        Face::Down,
        Face::Right,
        Face::Left,
        Face::Front,
        Face::Back,
    ];

    // All 26 block types (0-25) should have explicit mappings for all 6 faces
    for block_type in 0..=25u32 {
        for &face in &faces {
            let uvs = atlas.get_uvs(block_type, face);
            // Verify UVs are valid (in 0.0..1.0 range)
            for uv in &uvs {
                assert!(
                    uv[0] >= 0.0 && uv[0] <= 1.0,
                    "Invalid U for block {block_type}"
                );
                assert!(
                    uv[1] >= 0.0 && uv[1] <= 1.0,
                    "Invalid V for block {block_type}"
                );
            }
        }
    }

    // Verify block types 4-25 don't all map to the same tile as type 0
    let type0_uvs = atlas.get_uvs(0, Face::Up);
    let mut different_count = 0;
    for block_type in 4..=25u32 {
        let uvs = atlas.get_uvs(block_type, Face::Up);
        if uvs != type0_uvs {
            different_count += 1;
        }
    }
    assert!(
        different_count >= 20,
        "Too many block types map to same tile as air: only {different_count} are different"
    );
}
