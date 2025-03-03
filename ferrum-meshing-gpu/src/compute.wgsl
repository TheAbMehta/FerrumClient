// GPU Greedy Meshing Compute Shader
//
// Quad packing (2x u32):
//   word0: x(5) | y(5) | z(5) | w(5) | h(5) | face(3) | padding(4)
//   word1: block_type
//
// Face directions: 0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z
//
// Dispatch: (4, 6, 1) with workgroup_size(256)
//   4 * 256 = 1024 threads per face, 6 face directions

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

var<workgroup> face_masks: array<u32, 1024>;

fn voxel_index(x: u32, y: u32, z: u32) -> u32 {
    return z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x;
}

fn is_air_at(x: i32, y: i32, z: i32) -> bool {
    if x < 0 || x >= i32(CHUNK_SIZE) || y < 0 || y >= i32(CHUNK_SIZE) || z < 0 || z >= i32(CHUNK_SIZE) {
        return true;
    }
    return voxels[voxel_index(u32(x), u32(y), u32(z))] == 0u;
}

fn get_block(x: u32, y: u32, z: u32) -> u32 {
    return voxels[voxel_index(x, y, z)];
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

// Coordinate mapping: (row, col, depth) -> (x, y, z) per face axis
//   faces 0,1 (+-X): depth=X, row=Z, col=Y
//   faces 2,3 (+-Y): depth=Y, row=Z, col=X
//   faces 4,5 (+-Z): depth=Z, row=Y, col=X
fn to_world(face: u32, row: u32, col: u32, depth: u32) -> vec3<u32> {
    switch face {
        case 0u, 1u: { return vec3<u32>(depth, col, row); }
        case 2u, 3u: { return vec3<u32>(col, depth, row); }
        case 4u, 5u: { return vec3<u32>(col, row, depth); }
        default:     { return vec3<u32>(0u, 0u, 0u); }
    }
}

fn neighbor_offset(face: u32) -> vec3<i32> {
    switch face {
        case 0u: { return vec3<i32>(1, 0, 0); }
        case 1u: { return vec3<i32>(-1, 0, 0); }
        case 2u: { return vec3<i32>(0, 1, 0); }
        case 3u: { return vec3<i32>(0, -1, 0); }
        case 4u: { return vec3<i32>(0, 0, 1); }
        case 5u: { return vec3<i32>(0, 0, -1); }
        default: { return vec3<i32>(0, 0, 0); }
    }
}

@compute @workgroup_size(256, 1, 1)
fn face_culling(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    let col_idx = lid.x + wgid.x * 256u;
    if col_idx >= CHUNK_SIZE_SQ {
        return;
    }

    let face = wgid.y;
    let row = col_idx / CHUNK_SIZE;
    let col = col_idx % CHUNK_SIZE;
    let noff = neighbor_offset(face);

    // Phase 1: Build face visibility bitmask for this column
    var mask: u32 = 0u;
    for (var depth: u32 = 0u; depth < CHUNK_SIZE; depth = depth + 1u) {
        let pos = to_world(face, row, col, depth);
        let block = get_block(pos.x, pos.y, pos.z);
        if block == 0u {
            continue;
        }
        let nx = i32(pos.x) + noff.x;
        let ny = i32(pos.y) + noff.y;
        let nz = i32(pos.z) + noff.z;
        if is_air_at(nx, ny, nz) {
            mask = mask | (1u << depth);
        }
    }

    face_masks[col_idx] = mask;
    workgroupBarrier();

    // Phase 2: Greedy merge along depth axis, splitting on block type changes
    var bits = mask;
    while bits != 0u {
        let start = countTrailingZeros(bits);

        var run_len: u32 = 0u;
        var shifted = bits >> start;
        while (shifted & 1u) != 0u {
            run_len = run_len + 1u;
            shifted = shifted >> 1u;
        }

        bits = bits & ~(((1u << run_len) - 1u) << start);

        let start_pos = to_world(face, row, col, start);
        var current_type = get_block(start_pos.x, start_pos.y, start_pos.z);
        var actual_run: u32 = 1u;

        for (var i: u32 = 1u; i < run_len; i = i + 1u) {
            let p = to_world(face, row, col, start + i);
            let bt = get_block(p.x, p.y, p.z);
            if bt == current_type {
                actual_run = actual_run + 1u;
            } else {
                let emit_pos = to_world(face, row, col, start + i - actual_run);
                emit_quad(emit_pos.x, emit_pos.y, emit_pos.z, 1u, actual_run, face, current_type);
                current_type = bt;
                actual_run = 1u;
            }
        }
        let emit_pos = to_world(face, row, col, start + run_len - actual_run);
        emit_quad(emit_pos.x, emit_pos.y, emit_pos.z, 1u, actual_run, face, current_type);
    }
}
