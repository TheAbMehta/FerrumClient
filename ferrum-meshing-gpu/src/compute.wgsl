// GPU Greedy Meshing Compute Shader
//
// Input: 32x32x32 chunk of block IDs (u32 per voxel, 0 = air)
// Output: Packed quad data (position, size, normal, block type)
//
// Algorithm:
//   1. Face culling: For each voxel, check 6 neighbors. If neighbor is air (0),
//      the face is visible. Store visibility as bitmasks per column.
//   2. Greedy merging: For each layer, merge adjacent visible faces into larger
//      quads using a row-by-row sweep with run-length encoding.
//
// Each quad is packed into 2 u32s:
//   word0: x(5) | y(5) | z(5) | w(5) | h(5) | face(3) | padding(4)
//   word1: block_type (u32)

const CHUNK_SIZE: u32 = 32u;
const CHUNK_SIZE_SQ: u32 = 1024u;  // 32*32
const CHUNK_SIZE_CB: u32 = 32768u; // 32*32*32

// Maximum quads we can output. Worst case: checkerboard = ~16384 visible faces per axis pair.
// 6 faces * ~5500 quads each = ~33000. Round up generously.
const MAX_QUADS: u32 = 65536u;

// Voxel data: 32^3 block IDs
@group(0) @binding(0)
var<storage, read> voxels: array<u32>;

// Output quad buffer: each quad is 2 u32s (8 bytes)
@group(0) @binding(1)
var<storage, read_write> quads: array<u32>;

// Atomic counter for number of quads written
@group(0) @binding(2)
var<storage, read_write> quad_count: atomic<u32>;

// Workgroup shared memory for face masks.
// For a given axis and layer, we store one u32 bitmask per column (32 bits = 32 voxels along that axis).
// We process one face direction per dispatch.
var<workgroup> face_masks: array<u32, 1024>; // 32x32 columns

fn voxel_index(x: u32, y: u32, z: u32) -> u32 {
    return z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x;
}

fn is_air(x: i32, y: i32, z: i32) -> bool {
    if x < 0 || x >= i32(CHUNK_SIZE) || y < 0 || y >= i32(CHUNK_SIZE) || z < 0 || z >= i32(CHUNK_SIZE) {
        return true;
    }
    return voxels[voxel_index(u32(x), u32(y), u32(z))] == 0u;
}

fn get_block(x: u32, y: u32, z: u32) -> u32 {
    return voxels[voxel_index(x, y, z)];
}

// Pack a quad into 2 u32s
fn pack_quad(x: u32, y: u32, z: u32, w: u32, h: u32, face: u32) -> vec2<u32> {
    let word0 = (x & 0x1Fu) | ((y & 0x1Fu) << 5u) | ((z & 0x1Fu) << 10u)
              | ((w & 0x1Fu) << 15u) | ((h & 0x1Fu) << 20u) | ((face & 0x7u) << 25u);
    return vec2<u32>(word0, 0u);
}

fn emit_quad(x: u32, y: u32, z: u32, w: u32, h: u32, face: u32, block_type: u32) {
    let idx = atomicAdd(&quad_count, 1u);
    if idx >= MAX_QUADS {
        return;
    }
    let word0 = (x & 0x1Fu) | ((y & 0x1Fu) << 5u) | ((z & 0x1Fu) << 10u)
              | ((w & 0x1Fu) << 15u) | ((h & 0x1Fu) << 20u) | ((face & 0x7u) << 25u);
    quads[idx * 2u] = word0;
    quads[idx * 2u + 1u] = block_type;
}

// Face directions:
// 0: +X (right)   1: -X (left)
// 2: +Y (up)      3: -Y (down)
// 4: +Z (front)   5: -Z (back)

// Each workgroup processes one layer along the face normal axis.
// global_invocation_id.x = column index within the layer (0..1023 for 32x32)
// global_invocation_id.y = face direction (0..5)
// global_invocation_id.z = unused

// We use workgroup_size(256, 1, 1) and dispatch (4, 6, 1) = 1024 threads per face, 6 faces
// Each thread handles one column in the 32x32 layer grid.

@compute @workgroup_size(256, 1, 1)
fn face_culling(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    // Column index within the 32x32 grid
    let col = lid.x + wgid.x * 256u;
    if col >= CHUNK_SIZE_SQ {
        return;
    }

    let face = wgid.y;
    let row = col / CHUNK_SIZE;  // 0..31
    let col_in_row = col % CHUNK_SIZE;  // 0..31

    // Build face mask for this column.
    // The mask has one bit per voxel along the face normal axis.
    var mask: u32 = 0u;

    for (var depth: u32 = 0u; depth < CHUNK_SIZE; depth = depth + 1u) {
        // Map (row, col_in_row, depth) to (x, y, z) based on face direction
        var x: u32;
        var y: u32;
        var z: u32;

        // For faces along X axis (0,1): iterate depth along X, row=Z, col=Y
        // For faces along Y axis (2,3): iterate depth along Y, row=Z, col=X
        // For faces along Z axis (4,5): iterate depth along Z, row=Y, col=X
        switch face {
            case 0u, 1u: {
                x = depth;
                y = col_in_row;
                z = row;
            }
            case 2u, 3u: {
                x = col_in_row;
                y = depth;
                z = row;
            }
            case 4u, 5u: {
                x = col_in_row;
                y = row;
                z = depth;
            }
            default: {
                x = 0u;
                y = 0u;
                z = 0u;
            }
        }

        let block = get_block(x, y, z);
        if block == 0u {
            continue;
        }

        // Check if the face is visible (neighbor in face direction is air)
        var nx: i32 = i32(x);
        var ny: i32 = i32(y);
        var nz: i32 = i32(z);

        switch face {
            case 0u: { nx = nx + 1; }
            case 1u: { nx = nx - 1; }
            case 2u: { ny = ny + 1; }
            case 3u: { ny = ny - 1; }
            case 4u: { nz = nz + 1; }
            case 5u: { nz = nz - 1; }
            default: {}
        }

        if is_air(nx, ny, nz) {
            mask = mask | (1u << depth);
        }
    }

    // Store face mask. Layout: face_masks[face * CHUNK_SIZE_SQ + col]
    // But since each workgroup handles one face, we use shared memory indexed by col.
    face_masks[col] = mask;

    workgroupBarrier();

    // Now do greedy merging for this column.
    // We process the bitmask to find runs of visible faces and merge them.
    // Each thread handles one column; merging happens along the depth axis first,
    // then we try to merge across columns in the row direction.

    // For simplicity in this first pass, we emit individual quads per visible face
    // and do greedy merging along the depth (bit) axis only.
    // Cross-column merging requires synchronization and is done in a second pass.

    var bits = mask;
    while bits != 0u {
        let bit_pos = countTrailingZeros(bits);

        // Find run length along depth
        var run_len: u32 = 0u;
        var test_bits = bits >> bit_pos;
        while (test_bits & 1u) != 0u {
            run_len = run_len + 1u;
            test_bits = test_bits >> 1u;
        }

        // Clear the run from bits
        let run_mask = ((1u << run_len) - 1u) << bit_pos;
        bits = bits & ~run_mask;

        // Get block type at start of run
        var sx: u32;
        var sy: u32;
        var sz: u32;
        switch face {
            case 0u, 1u: { sx = bit_pos; sy = col_in_row; sz = row; }
            case 2u, 3u: { sx = col_in_row; sy = bit_pos; sz = row; }
            case 4u, 5u: { sx = col_in_row; sy = row; sz = bit_pos; }
            default: { sx = 0u; sy = 0u; sz = 0u; }
        }
        let block_type = get_block(sx, sy, sz);

        // Check if all blocks in the run are the same type
        var same_type = true;
        for (var i: u32 = 1u; i < run_len; i = i + 1u) {
            var cx: u32;
            var cy: u32;
            var cz: u32;
            switch face {
                case 0u, 1u: { cx = bit_pos + i; cy = col_in_row; cz = row; }
                case 2u, 3u: { cx = col_in_row; cy = bit_pos + i; cz = row; }
                case 4u, 5u: { cx = col_in_row; cy = row; cz = bit_pos + i; }
                default: { cx = 0u; cy = 0u; cz = 0u; }
            }
            if get_block(cx, cy, cz) != block_type {
                same_type = false;
                break;
            }
        }

        if !same_type {
            // Emit individual quads for each voxel in the run
            for (var i: u32 = 0u; i < run_len; i = i + 1u) {
                var ix: u32;
                var iy: u32;
                var iz: u32;
                switch face {
                    case 0u, 1u: { ix = bit_pos + i; iy = col_in_row; iz = row; }
                    case 2u, 3u: { ix = col_in_row; iy = bit_pos + i; iz = row; }
                    case 4u, 5u: { ix = col_in_row; iy = row; iz = bit_pos + i; }
                    default: { ix = 0u; iy = 0u; iz = 0u; }
                }
                let bt = get_block(ix, iy, iz);
                emit_quad(ix, iy, iz, 1u, 1u, face, bt);
            }
        } else {
            // Emit merged quad along depth axis
            // Width=1 (single column), Height=run_len (along depth)
            // The quad position is at the start of the run
            switch face {
                case 0u, 1u: {
                    emit_quad(sx, sy, sz, 1u, run_len, face, block_type);
                }
                case 2u, 3u: {
                    emit_quad(sx, sy, sz, 1u, run_len, face, block_type);
                }
                case 4u, 5u: {
                    emit_quad(sx, sy, sz, 1u, run_len, face, block_type);
                }
                default: {}
            }
        }
    }
}

// Second pass: merge quads across columns (greedy merge in 2D).
// This is done by sorting quads by position and merging adjacent ones.
// For the prototype, we skip this and rely on the depth-axis merging only.
// Full greedy merging would require a more complex approach with shared memory
// coordination between threads.
