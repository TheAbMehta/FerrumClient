use crate::{ChunkMesh, Face, MeshQuad, CHUNK_SIZE, CHUNK_SIZE_CB};

const CS: usize = CHUNK_SIZE;
const CS2: usize = CS * CS;

#[inline]
fn voxel_at(voxels: &[u32; CHUNK_SIZE_CB], x: usize, y: usize, z: usize) -> u32 {
    voxels[z * CS2 + y * CS + x]
}

/// Binary greedy meshing for a 32x32x32 chunk.
///
/// 1. Face culling: build 32-bit column masks per (row, layer) for each of 6 directions.
///    A face is exposed when the neighbor in that direction is air (0) or out of bounds.
/// 2. Greedy merging: sweep 2D slices per face direction, use trailing_zeros to find
///    exposed faces, extend right/forward while block type and merge count match.
pub fn mesh(voxels: &[u32; CHUNK_SIZE_CB]) -> ChunkMesh {
    let mut result = ChunkMesh::new();

    // face_masks[face][layer * CS + row] = 32-bit mask of exposed faces along that column
    // face 0: +X, 1: -X, 2: +Y, 3: -Y, 4: +Z, 5: -Z
    let mut face_masks = [[0u32; CS2]; 6];

    build_face_masks(voxels, &mut face_masks);
    greedy_merge(voxels, &face_masks, &mut result);

    result
}

fn build_face_masks(voxels: &[u32; CHUNK_SIZE_CB], masks: &mut [[u32; CS2]; 6]) {
    // Build opaque column masks along each axis, then derive face masks via bitwise ops.
    // opaque_x[z * CS + y] = 32-bit mask where bit i is set if voxel(i, y, z) != 0
    let mut opaque_x = [0u32; CS2];
    // opaque_y[z * CS + x] = 32-bit mask where bit i is set if voxel(x, i, z) != 0
    let mut opaque_y = [0u32; CS2];
    // opaque_z[y * CS + x] = 32-bit mask where bit i is set if voxel(x, y, i) != 0
    let mut opaque_z = [0u32; CS2];

    for z in 0..CS {
        for y in 0..CS {
            let row_base = z * CS2 + y * CS;
            let mut col_x = 0u32;
            for x in 0..CS {
                if voxels[row_base + x] != 0 {
                    col_x |= 1 << x;
                }
            }
            opaque_x[z * CS + y] = col_x;

            for x in 0..CS {
                if voxels[row_base + x] != 0 {
                    opaque_y[z * CS + x] |= 1 << y;
                    opaque_z[y * CS + x] |= 1 << z;
                }
            }
        }
    }

    // Face masks via bitwise column operations:
    // +X: solid here AND (neighbor to right is air or out of bounds)
    //     = opaque & ~(opaque << 1)  (bit 31 naturally has no left-shift neighbor)
    // -X: solid here AND (neighbor to left is air or out of bounds)
    //     = opaque & ~(opaque >> 1)  (bit 0 naturally has no right-shift neighbor)
    for i in 0..CS2 {
        let col = opaque_x[i];
        masks[0][i] = col & !(col << 1); // +X
        masks[1][i] = col & !(col >> 1); // -X

        let col = opaque_y[i];
        masks[2][i] = col & !(col << 1); // +Y
        masks[3][i] = col & !(col >> 1); // -Y

        let col = opaque_z[i];
        masks[4][i] = col & !(col << 1); // +Z
        masks[5][i] = col & !(col >> 1); // -Z
    }
}

/// Mask layout per face (all use [layer * CS + row] with bits along the third axis):
///   Face 0,1 (+X,-X): layer=z, row=y, bits=x
///   Face 2,3 (+Y,-Y): layer=z, row=x, bits=y
///   Face 4,5 (+Z,-Z): layer=y, row=x, bits=z
fn greedy_merge(voxels: &[u32; CHUNK_SIZE_CB], masks: &[[u32; CS2]; 6], result: &mut ChunkMesh) {
    let mut forward_merged = [0u8; CS];

    for face_idx in 0..6 {
        let face = match face_idx {
            0 => Face::Right,
            1 => Face::Left,
            2 => Face::Up,
            3 => Face::Down,
            4 => Face::Front,
            5 => Face::Back,
            _ => unreachable!(),
        };

        merge_face(
            voxels,
            &masks[face_idx],
            face,
            face_idx,
            &mut forward_merged,
            result,
        );
    }
}

/// Unified greedy merge for all 6 faces.
/// For each face, iterates: layer (outer) -> row (forward merge) -> bit_pos (right merge).
/// The get_block and emit_quad functions handle the axis remapping per face.
fn merge_face(
    voxels: &[u32; CHUNK_SIZE_CB],
    masks: &[u32; CS2],
    face: Face,
    face_idx: usize,
    forward_merged: &mut [u8; CS],
    result: &mut ChunkMesh,
) {
    for layer in 0..CS {
        let base = layer * CS;

        for row in 0..CS {
            let mut bits = masks[base + row];
            if bits == 0 {
                continue;
            }

            let next_bits = if row + 1 < CS {
                masks[base + row + 1]
            } else {
                0
            };

            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;

                let block = get_block(voxels, face_idx, layer, row, bit_pos);

                // Forward merge: extend one more row if same block type
                if (next_bits >> bit_pos & 1) != 0
                    && block == get_block(voxels, face_idx, layer, row + 1, bit_pos)
                {
                    forward_merged[bit_pos] += 1;
                    bits &= !(1 << bit_pos);
                    continue;
                }

                // Right merge: extend along the bit axis while same type and same forward count
                let mut right_merged: u8 = 1;
                for right in (bit_pos + 1)..CS {
                    if (bits >> right & 1) == 0
                        || forward_merged[bit_pos] != forward_merged[right]
                        || block != get_block(voxels, face_idx, layer, row, right)
                    {
                        break;
                    }
                    forward_merged[right] = 0;
                    right_merged += 1;
                }

                // Clear merged bits [bit_pos .. bit_pos + right_merged)
                let end = bit_pos + right_merged as usize;
                let clear_mask = if end >= 32 {
                    !((1u32 << bit_pos) - 1)
                } else {
                    ((1u32 << end) - 1) & !((1u32 << bit_pos) - 1)
                };
                bits &= !clear_mask;

                let row_start = row - forward_merged[bit_pos] as usize;
                let length = forward_merged[bit_pos] as u8 + 1;
                let width = right_merged;

                forward_merged[bit_pos] = 0;

                emit_quad(
                    result, face, face_idx, layer, row_start, bit_pos, width, length, block,
                );
            }
        }
    }
}

/// Look up the block type for a given face's coordinate system.
///
/// Face 0,1: layer=z, row=y, bit=x -> voxel_at(x=bit, y=row, z=layer)
/// Face 2,3: layer=z, row=x, bit=y -> voxel_at(x=row, y=bit, z=layer)
/// Face 4,5: layer=y, row=x, bit=z -> voxel_at(x=row, y=layer, z=bit)
#[inline]
fn get_block(
    voxels: &[u32; CHUNK_SIZE_CB],
    face_idx: usize,
    layer: usize,
    row: usize,
    bit_pos: usize,
) -> u32 {
    match face_idx {
        0 | 1 => voxel_at(voxels, bit_pos, row, layer),
        2 | 3 => voxel_at(voxels, row, bit_pos, layer),
        4 | 5 => voxel_at(voxels, row, layer, bit_pos),
        _ => unreachable!(),
    }
}

/// Emit a quad with the correct (x, y, z, width, height) for the given face.
///
/// Face 0,1: (x=bit, y=row_start, z=layer, w=width_along_x, h=length_along_y)
/// Face 2,3: (x=row_start, y=bit, z=layer, w=length_along_x, h=width_along_y)
/// Face 4,5: (x=row_start, y=layer, z=bit, w=length_along_x, h=width_along_z)
#[inline]
fn emit_quad(
    result: &mut ChunkMesh,
    face: Face,
    face_idx: usize,
    layer: usize,
    row_start: usize,
    bit_pos: usize,
    width: u8,
    length: u8,
    block: u32,
) {
    let (qx, qy, qz, qw, qh) = match face_idx {
        0 | 1 => (bit_pos as u8, row_start as u8, layer as u8, width, length),
        2 | 3 => (row_start as u8, bit_pos as u8, layer as u8, length, width),
        4 | 5 => (row_start as u8, layer as u8, bit_pos as u8, length, width),
        _ => unreachable!(),
    };

    result.quads.push(MeshQuad {
        x: qx,
        y: qy,
        z: qz,
        width: qw,
        height: qh,
        face,
        block_type: block,
    });
}
