// GPU 2D Greedy Meshing Compute Shader — Batch Processing
//
// Supports N chunks per dispatch via wgid.z as chunk index.
// Buffers are laid out as arrays indexed by chunk_id:
//   voxels:        [chunk_id * 32768 + voxel_index]
//   face_mask_buf: [chunk_id * 6144 + face * 1024 + layer * 32 + row]
//   quads:         [chunk_id * MAX_QUADS * 2 + quad_idx * 2]
//   quad_count:    [chunk_id]
//
// Pass 1 (face_culling): Dispatch (4, 6, N) @ workgroup_size(256)
// Pass 2 (greedy_merge): Dispatch (32, 6, N) @ workgroup_size(32)
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
const FACE_MASK_STRIDE: u32 = 6144u; // 6 * 1024

@group(0) @binding(0)
var<storage, read> voxels: array<u32>;

@group(0) @binding(1)
var<storage, read_write> quads: array<u32>;

@group(0) @binding(2)
var<storage, read_write> quad_counts: array<atomic<u32>>;

@group(0) @binding(3)
var<storage, read_write> face_mask_buf: array<u32>;

fn voxel_index(chunk: u32, x: u32, y: u32, z: u32) -> u32 {
    return chunk * CHUNK_SIZE_CB + z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x;
}

fn get_block_for_face(chunk: u32, face: u32, layer: u32, row: u32, bit: u32) -> u32 {
    var x: u32; var y: u32; var z: u32;
    switch face {
        case 0u, 1u: { x = bit; y = row; z = layer; }
        case 2u, 3u: { x = row; y = bit; z = layer; }
        case 4u, 5u: { x = row; y = layer; z = bit; }
        default:     { x = 0u; y = 0u; z = 0u; }
    }
    return voxels[voxel_index(chunk, x, y, z)];
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
// Dispatch: (4, 6, N) with workgroup_size(256)

@compute @workgroup_size(256, 1, 1)
fn face_culling(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    let chunk = wgid.z;
    let face = wgid.y;
    let col_idx = lid.x + wgid.x * 256u;
    if col_idx >= CHUNK_SIZE_SQ {
        return;
    }

    let layer = col_idx / CHUNK_SIZE;
    let row = col_idx % CHUNK_SIZE;

    var opaque: u32 = 0u;
    for (var depth: u32 = 0u; depth < CHUNK_SIZE; depth = depth + 1u) {
        if get_block_for_face(chunk, face, layer, row, depth) != 0u {
            opaque = opaque | (1u << depth);
        }
    }

    var mask: u32;
    if (face & 1u) == 0u {
        mask = opaque & ~(opaque << 1u);
    } else {
        mask = opaque & ~(opaque >> 1u);
    }

    face_mask_buf[chunk * FACE_MASK_STRIDE + face * CHUNK_SIZE_SQ + layer * CHUNK_SIZE + row] = mask;
}

// ============================================================================
// Pass 2: Parallel Greedy Merge (1D right merge, all 32 threads active)
// ============================================================================
// Dispatch: (32, 6, N) with workgroup_size(32)
// Each thread handles one row: right-merges along bit axis, splits on block type.
// No forward merge (height=1 always) — trades quad count for parallelism.

var<workgroup> wg_quads: array<u32, 1024>;
var<workgroup> wg_count: atomic<u32>;
var<workgroup> wg_base: u32;

@compute @workgroup_size(32, 1, 1)
fn greedy_merge(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    let layer = wgid.x;
    let face = wgid.y;
    let chunk = wgid.z;
    let row = lid.x;

    if lid.x == 0u {
        atomicStore(&wg_count, 0u);
    }

    workgroupBarrier();

    let mask_addr = chunk * FACE_MASK_STRIDE + face * CHUNK_SIZE_SQ + layer * CHUNK_SIZE + row;
    var bits = face_mask_buf[mask_addr];

    // Each thread processes its own row: right merge along bit axis
    while bits != 0u {
        let bit_pos = countTrailingZeros(bits);
        let block = get_block_for_face(chunk, face, layer, row, bit_pos);

        // Right merge: extend along bit axis while same block type (cap at 31)
        var right_count: u32 = 1u;
        var next_bit = bit_pos + 1u;
        while next_bit < CHUNK_SIZE && right_count < 31u {
            if (bits & (1u << next_bit)) == 0u {
                break;
            }
            if block != get_block_for_face(chunk, face, layer, row, next_bit) {
                break;
            }
            right_count = right_count + 1u;
            next_bit = next_bit + 1u;
        }

        // Clear merged bits
        let end_bit = bit_pos + right_count;
        var clear_mask: u32;
        if end_bit >= 32u {
            clear_mask = ~((1u << bit_pos) - 1u);
        } else {
            clear_mask = ((1u << end_bit) - 1u) & ~((1u << bit_pos) - 1u);
        }
        bits = bits & ~clear_mask;

        let coords = face_quad_xyz(face, layer, row, bit_pos);
        let wh = face_quad_wh(face, right_count, 1u);

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

    workgroupBarrier();

    // Reserve global buffer space (single atomic per workgroup)
    if lid.x == 0u {
        let count = min(atomicLoad(&wg_count), 512u);
        if count > 0u {
            wg_base = atomicAdd(&quad_counts[chunk], count);
        } else {
            wg_base = 0u;
        }
    }

    workgroupBarrier();

    // All 32 threads flush quads to global buffer
    let total = min(atomicLoad(&wg_count), 512u);
    let quad_base = chunk * MAX_QUADS * 2u;
    var qi = lid.x;
    while qi < total {
        let gi = wg_base + qi;
        if gi < MAX_QUADS {
            quads[quad_base + gi * 2u] = wg_quads[qi * 2u];
            quads[quad_base + gi * 2u + 1u] = wg_quads[qi * 2u + 1u];
        }
        qi = qi + 32u;
    }
}
