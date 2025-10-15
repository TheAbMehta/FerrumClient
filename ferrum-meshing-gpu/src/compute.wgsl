// GPU 2D Greedy Meshing Compute Shader
//
// Two-pass architecture:
//   Pass 1 (face_culling): Binary face mask generation using bitwise ops.
//     Dispatch (4, 6, 1) @ workgroup_size(256) = 1024 threads per face.
//     Each thread builds one 32-bit column mask using opaque & ~(opaque << 1).
//     Output: face_mask_buf[face * 1024 + layer * 32 + row]
//
//   Pass 2 (greedy_merge): Full 2D greedy merge per (face, layer) slice.
//     Dispatch (32, 6, 1) @ workgroup_size(32).
//     Thread 0 runs the complete 2D greedy algorithm (forward+right merge)
//     on the 32x32 slice using bit manipulation. Other threads help flush
//     the per-workgroup quad buffer to global memory.
//
// Quad packing (2x u32):
//   word0: x(5) | y(5) | z(5) | w(5) | h(5) | face(3) | padding(4)
//   word1: block_type
//
// Face directions: 0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z

const CHUNK_SIZE: u32 = 32u;
const CHUNK_SIZE_SQ: u32 = 1024u;
const CHUNK_SIZE_CB: u32 = 32768u;
const MAX_QUADS: u32 = 65536u;

@group(0) @binding(0)
var<storage, read> voxels: array<u32>;

@group(0) @binding(1)
var<storage, read_write> quads: array<u32>;

@group(0) @binding(2)
var<storage, read_write> quad_count: atomic<u32>;

// Intermediate face mask buffer: 6 faces * 1024 entries = 6144 u32s
@group(0) @binding(3)
var<storage, read_write> face_mask_buf: array<u32>;

fn voxel_index(x: u32, y: u32, z: u32) -> u32 {
    return z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x;
}

// ============================================================================
// Coordinate mapping for face directions
// ============================================================================
// Mask layout: face_mask_buf[face * 1024 + layer * 32 + row]
//   Face 0,1 (+X,-X): layer=z, row=y, bits=x
//   Face 2,3 (+Y,-Y): layer=z, row=x, bits=y
//   Face 4,5 (+Z,-Z): layer=y, row=x, bits=z

fn get_block_for_face(face: u32, layer: u32, row: u32, bit: u32) -> u32 {
    var x: u32; var y: u32; var z: u32;
    switch face {
        case 0u, 1u: { x = bit; y = row; z = layer; }
        case 2u, 3u: { x = row; y = bit; z = layer; }
        case 4u, 5u: { x = row; y = layer; z = bit; }
        default:     { x = 0u; y = 0u; z = 0u; }
    }
    return voxels[voxel_index(x, y, z)];
}

fn face_quad_xyz(face: u32, layer: u32, row_start: u32, bit: u32) -> vec3<u32> {
    switch face {
        case 0u, 1u: { return vec3<u32>(bit, row_start, layer); }
        case 2u, 3u: { return vec3<u32>(row_start, bit, layer); }
        case 4u, 5u: { return vec3<u32>(row_start, layer, bit); }
        default:     { return vec3<u32>(0u, 0u, 0u); }
    }
}

fn face_quad_wh(face: u32, width_along_bit: u32, length_along_row: u32) -> vec2<u32> {
    // width_along_bit = extent along the bit axis
    // length_along_row = extent along the row axis
    switch face {
        case 0u, 1u: { return vec2<u32>(width_along_bit, length_along_row); }
        case 2u, 3u: { return vec2<u32>(length_along_row, width_along_bit); }
        case 4u, 5u: { return vec2<u32>(length_along_row, width_along_bit); }
        default:     { return vec2<u32>(0u, 0u); }
    }
}

// ============================================================================
// Pass 1: Face Culling — Binary face mask generation
// ============================================================================
// Dispatch: (4, 6, 1) with workgroup_size(256)
// 4 * 256 = 1024 threads per face, 6 face directions.
// Each thread handles one (layer, row) column and builds a 32-bit mask.

@compute @workgroup_size(256, 1, 1)
fn face_culling(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    let face = wgid.y;
    let col_idx = lid.x + wgid.x * 256u;
    if col_idx >= CHUNK_SIZE_SQ {
        return;
    }

    let layer = col_idx / CHUNK_SIZE;
    let row = col_idx % CHUNK_SIZE;

    // Build opaque column mask: bit i set if block at depth i is non-air
    var opaque: u32 = 0u;
    for (var depth: u32 = 0u; depth < CHUNK_SIZE; depth = depth + 1u) {
        if get_block_for_face(face, layer, row, depth) != 0u {
            opaque = opaque | (1u << depth);
        }
    }

    // Derive face mask using binary ops:
    // +dir (face 0,2,4): exposed at depth d if solid at d and air at d+1
    //   = opaque & ~(opaque << 1)  (bit 31 naturally exposed)
    // -dir (face 1,3,5): exposed at depth d if solid at d and air at d-1
    //   = opaque & ~(opaque >> 1)  (bit 0 naturally exposed)
    var mask: u32;
    if (face & 1u) == 0u {
        mask = opaque & ~(opaque << 1u);
    } else {
        mask = opaque & ~(opaque >> 1u);
    }

    face_mask_buf[face * CHUNK_SIZE_SQ + layer * CHUNK_SIZE + row] = mask;
}

// ============================================================================
// Pass 2: 2D Greedy Merge
// ============================================================================
// Dispatch: (32, 6, 1) with workgroup_size(32)
// Each workgroup handles one (face, layer) pair = one 32x32 slice.
// Thread 0 runs the full 2D greedy merge algorithm (ported from CPU code).
// Other 31 threads assist with flushing quads to global memory.

// Per-workgroup quad buffer (max 512 quads per slice, 2 words each)
var<workgroup> wg_quads: array<u32, 1024>;
var<workgroup> wg_count: atomic<u32>;
var<workgroup> wg_base: u32;

@compute @workgroup_size(32, 1, 1)
fn greedy_merge(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    let layer = wgid.x;  // depth slice (0..31)
    let face = wgid.y;   // face direction (0..5)

    if lid.x == 0u {
        atomicStore(&wg_count, 0u);
    }

    workgroupBarrier();

    // Thread 0 performs the full 2D greedy merge for this slice.
    // Algorithm: identical to CPU binary_greedy.rs merge_face().
    // For each row (0..31), scan bits, try forward merge to next row,
    // then right merge along bit axis. Emit merged quads.
    if lid.x == 0u {
        // Load masks into private registers
        var masks: array<u32, 32>;
        for (var i: u32 = 0u; i < 32u; i = i + 1u) {
            masks[i] = face_mask_buf[face * CHUNK_SIZE_SQ + layer * CHUNK_SIZE + i];
        }

        // Forward merge counters per bit position
        var fwd: array<u32, 32>;
        for (var i: u32 = 0u; i < 32u; i = i + 1u) {
            fwd[i] = 0u;
        }

        for (var row: u32 = 0u; row < CHUNK_SIZE; row = row + 1u) {
            var bits = masks[row];
            if bits == 0u {
                continue;
            }

            var next_bits: u32 = 0u;
            if row + 1u < CHUNK_SIZE {
                next_bits = masks[row + 1u];
            }

            while bits != 0u {
                let bit_pos = countTrailingZeros(bits);
                let block = get_block_for_face(face, layer, row, bit_pos);

                // Forward merge: if next row has same bit set with same block type,
                // increment forward counter and skip emission.
                // Cap at 30 so length (fwd+1) stays <= 31 (5-bit packing limit).
                if fwd[bit_pos] < 30u
                    && (next_bits & (1u << bit_pos)) != 0u
                    && block == get_block_for_face(face, layer, row + 1u, bit_pos) {
                    fwd[bit_pos] = fwd[bit_pos] + 1u;
                    bits = bits & ~(1u << bit_pos);
                    continue;
                }

                // Right merge: extend along bit axis while same block type
                // and same forward merge count. Cap at 31 (5-bit packing limit).
                var right_count: u32 = 1u;
                var next_bit = bit_pos + 1u;
                while next_bit < CHUNK_SIZE && right_count < 31u {
                    if (bits & (1u << next_bit)) == 0u {
                        break;
                    }
                    if fwd[bit_pos] != fwd[next_bit] {
                        break;
                    }
                    if block != get_block_for_face(face, layer, row, next_bit) {
                        break;
                    }
                    // Consume this bit and reset its forward counter
                    fwd[next_bit] = 0u;
                    right_count = right_count + 1u;
                    next_bit = next_bit + 1u;
                }

                // Clear merged bits from current row
                let end_bit = bit_pos + right_count;
                var clear_mask: u32;
                if end_bit >= 32u {
                    clear_mask = ~((1u << bit_pos) - 1u);
                } else {
                    clear_mask = ((1u << end_bit) - 1u) & ~((1u << bit_pos) - 1u);
                }
                bits = bits & ~clear_mask;

                // Compute quad dimensions
                let row_start = row - fwd[bit_pos];
                let length = fwd[bit_pos] + 1u;  // along row axis
                let width = right_count;           // along bit axis

                fwd[bit_pos] = 0u;

                // Pack and store quad
                let coords = face_quad_xyz(face, layer, row_start, bit_pos);
                let wh = face_quad_wh(face, width, length);

                let word0 = (coords.x & 0x1Fu) | ((coords.y & 0x1Fu) << 5u)
                          | ((coords.z & 0x1Fu) << 10u)
                          | ((wh.x & 0x1Fu) << 15u) | ((wh.y & 0x1Fu) << 20u)
                          | ((face & 0x7u) << 25u);

                let local_idx = atomicAdd(&wg_count, 1u);
                if local_idx < 512u {
                    wg_quads[local_idx * 2u] = word0;
                    wg_quads[local_idx * 2u + 1u] = block;
                }
            }
        }
    }

    workgroupBarrier();

    // Reserve global buffer space (single atomic per workgroup)
    if lid.x == 0u {
        let count = min(atomicLoad(&wg_count), 512u);
        if count > 0u {
            wg_base = atomicAdd(&quad_count, count);
        } else {
            wg_base = 0u;
        }
    }

    workgroupBarrier();

    // All 32 threads cooperate to flush quads to global buffer
    let total = min(atomicLoad(&wg_count), 512u);
    var qi = lid.x;
    while qi < total {
        let gi = wg_base + qi;
        if gi < MAX_QUADS {
            quads[gi * 2u] = wg_quads[qi * 2u];
            quads[gi * 2u + 1u] = wg_quads[qi * 2u + 1u];
        }
        qi = qi + 32u;
    }
}
