use crate::texture_atlas::TextureAtlas;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use ferrum_meshing_cpu::{ChunkMesh, Face};

pub struct BlockRenderer;

impl BlockRenderer {
    pub fn create_mesh(chunk_mesh: &ChunkMesh, atlas: &TextureAtlas) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        let mut vertex_count = 0u32;

        for quad in &chunk_mesh.quads {
            let base_pos = [quad.x as f32, quad.y as f32, quad.z as f32];
            let width = quad.width as f32;
            let height = quad.height as f32;

            let (quad_positions, normal) = match quad.face {
                Face::Up => (
                    [
                        [base_pos[0], base_pos[1] + 1.0, base_pos[2]],
                        [base_pos[0] + width, base_pos[1] + 1.0, base_pos[2]],
                        [base_pos[0] + width, base_pos[1] + 1.0, base_pos[2] + height],
                        [base_pos[0], base_pos[1] + 1.0, base_pos[2] + height],
                    ],
                    [0.0, 1.0, 0.0],
                ),
                Face::Down => (
                    [
                        [base_pos[0], base_pos[1], base_pos[2] + height],
                        [base_pos[0] + width, base_pos[1], base_pos[2] + height],
                        [base_pos[0] + width, base_pos[1], base_pos[2]],
                        [base_pos[0], base_pos[1], base_pos[2]],
                    ],
                    [0.0, -1.0, 0.0],
                ),
                Face::Right => (
                    [
                        [base_pos[0] + 1.0, base_pos[1], base_pos[2]],
                        [base_pos[0] + 1.0, base_pos[1], base_pos[2] + width],
                        [base_pos[0] + 1.0, base_pos[1] + height, base_pos[2] + width],
                        [base_pos[0] + 1.0, base_pos[1] + height, base_pos[2]],
                    ],
                    [1.0, 0.0, 0.0],
                ),
                Face::Left => (
                    [
                        [base_pos[0], base_pos[1], base_pos[2] + width],
                        [base_pos[0], base_pos[1], base_pos[2]],
                        [base_pos[0], base_pos[1] + height, base_pos[2]],
                        [base_pos[0], base_pos[1] + height, base_pos[2] + width],
                    ],
                    [-1.0, 0.0, 0.0],
                ),
                Face::Front => (
                    [
                        [base_pos[0], base_pos[1], base_pos[2] + 1.0],
                        [base_pos[0] + width, base_pos[1], base_pos[2] + 1.0],
                        [base_pos[0] + width, base_pos[1] + height, base_pos[2] + 1.0],
                        [base_pos[0], base_pos[1] + height, base_pos[2] + 1.0],
                    ],
                    [0.0, 0.0, 1.0],
                ),
                Face::Back => (
                    [
                        [base_pos[0] + width, base_pos[1], base_pos[2]],
                        [base_pos[0], base_pos[1], base_pos[2]],
                        [base_pos[0], base_pos[1] + height, base_pos[2]],
                        [base_pos[0] + width, base_pos[1] + height, base_pos[2]],
                    ],
                    [0.0, 0.0, -1.0],
                ),
            };

            let quad_uvs = atlas.get_uvs(quad.block_type, quad.face);

            positions.extend_from_slice(&quad_positions);
            normals.extend_from_slice(&[normal; 4]);
            uvs.extend_from_slice(&quad_uvs);

            indices.extend_from_slice(&[
                vertex_count,
                vertex_count + 1,
                vertex_count + 2,
                vertex_count,
                vertex_count + 2,
                vertex_count + 3,
            ]);

            vertex_count += 4;
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}
