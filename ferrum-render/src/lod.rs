//! Level of Detail (LOD) system for chunk rendering.
//!
//! Supports 4 detail levels to enable 48-64 chunk render distances:
//!
//! - **LOD 0** (0-16 chunks): Full detail — uses existing binary greedy meshing.
//! - **LOD 1** (17-32 chunks): Reduced detail — 2x2x2 downsampling, simplified meshing.
//! - **LOD 2** (33-48 chunks): Low detail — 4x4x4 downsampling, large quads only.
//! - **LOD 3** (49-64 chunks): Minimal detail — 8x8x8 downsampling, silhouette only.
//!
//! Each LOD level downsamples the voxel data by its scale factor, then runs a
//! simplified greedy mesh on the reduced grid. The resulting quads are scaled back
//! to chunk coordinates so they can be rendered with the same pipeline.

use ferrum_meshing_cpu::{ChunkMesh, Face, MeshQuad, CHUNK_SIZE, CHUNK_SIZE_CB, CHUNK_SIZE_SQ};

/// LOD level identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum LodLevel {
    /// Full detail (0-16 chunks). Uses existing meshing unchanged.
    Full = 0,
    /// Reduced detail (17-32 chunks). 2x2x2 downsampling.
    Reduced = 1,
    /// Low detail (33-48 chunks). 4x4x4 downsampling.
    Low = 2,
    /// Minimal detail (49-64 chunks). 8x8x8 downsampling, silhouette.
    Minimal = 3,
}

impl LodLevel {
    /// Downsampling scale factor: how many voxels per LOD cell along each axis.
    pub fn scale(self) -> usize {
        match self {
            LodLevel::Full => 1,
            LodLevel::Reduced => 2,
            LodLevel::Low => 4,
            LodLevel::Minimal => 8,
        }
    }

    /// Effective grid size after downsampling (CHUNK_SIZE / scale).
    pub fn grid_size(self) -> usize {
        CHUNK_SIZE / self.scale()
    }

    /// All LOD levels in order.
    pub fn all() -> [LodLevel; 4] {
        [
            LodLevel::Full,
            LodLevel::Reduced,
            LodLevel::Low,
            LodLevel::Minimal,
        ]
    }
}

/// Configuration for LOD distance thresholds.
///
/// Distances are in chunk units (not blocks). A chunk at distance 20 means
/// the chunk center is 20 chunks away from the camera.
#[derive(Clone, Debug)]
pub struct LodConfig {
    /// Maximum distance (inclusive) for full detail. Default: 16.
    pub full_max: f32,
    /// Maximum distance (inclusive) for reduced detail. Default: 32.
    pub reduced_max: f32,
    /// Maximum distance (inclusive) for low detail. Default: 48.
    pub low_max: f32,
    /// Maximum render distance. Chunks beyond this are not rendered. Default: 64.
    pub max_render_distance: f32,
    /// Width of the transition zone between LOD levels (in chunks). Default: 2.0.
    pub transition_width: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            full_max: 16.0,
            reduced_max: 32.0,
            low_max: 48.0,
            max_render_distance: 64.0,
            transition_width: 2.0,
        }
    }
}

impl LodConfig {
    /// Create a config with custom distance thresholds.
    pub fn new(full_max: f32, reduced_max: f32, low_max: f32, max_render_distance: f32) -> Self {
        Self {
            full_max,
            reduced_max,
            low_max,
            max_render_distance,
            transition_width: 2.0,
        }
    }

    /// Determine the LOD level for a chunk at the given distance (in chunk units).
    pub fn select_lod(&self, distance: f32) -> Option<LodLevel> {
        if distance > self.max_render_distance {
            None // Beyond render distance
        } else if distance <= self.full_max {
            Some(LodLevel::Full)
        } else if distance <= self.reduced_max {
            Some(LodLevel::Reduced)
        } else if distance <= self.low_max {
            Some(LodLevel::Low)
        } else {
            Some(LodLevel::Minimal)
        }
    }

    /// Compute the LOD transition info for smooth blending.
    ///
    /// Returns `(primary_lod, blend_factor)` where `blend_factor` is 0.0 when
    /// fully at the primary LOD and approaches 1.0 near the boundary to the
    /// next (lower detail) LOD. This allows the renderer to cross-fade or
    /// dither between LOD levels.
    pub fn select_lod_with_blend(&self, distance: f32) -> Option<LodTransition> {
        let lod = self.select_lod(distance)?;

        let boundary = match lod {
            LodLevel::Full => self.full_max,
            LodLevel::Reduced => self.reduced_max,
            LodLevel::Low => self.low_max,
            LodLevel::Minimal => self.max_render_distance,
        };

        let next_lod = match lod {
            LodLevel::Full => Some(LodLevel::Reduced),
            LodLevel::Reduced => Some(LodLevel::Low),
            LodLevel::Low => Some(LodLevel::Minimal),
            LodLevel::Minimal => None,
        };

        let half_width = self.transition_width / 2.0;
        let transition_start = boundary - half_width;

        let blend = if distance <= transition_start || self.transition_width <= 0.0 {
            0.0
        } else {
            ((distance - transition_start) / self.transition_width).clamp(0.0, 1.0)
        };

        Some(LodTransition {
            primary: lod,
            next: next_lod,
            blend,
        })
    }
}

/// Transition state between two LOD levels for smooth blending.
#[derive(Clone, Debug)]
pub struct LodTransition {
    /// The primary (higher detail) LOD level.
    pub primary: LodLevel,
    /// The next (lower detail) LOD level, if any.
    pub next: Option<LodLevel>,
    /// Blend factor: 0.0 = fully primary, 1.0 = fully next.
    pub blend: f32,
}

impl LodTransition {
    /// Whether this transition is actively blending between two levels.
    pub fn is_blending(&self) -> bool {
        self.blend > 0.0 && self.blend < 1.0 && self.next.is_some()
    }
}

/// LOD mesh generator.
///
/// Produces simplified `ChunkMesh` instances for each LOD level by downsampling
/// the voxel data and running a greedy merge on the reduced grid.
pub struct LodMesher;

impl LodMesher {
    /// Generate a mesh at the specified LOD level.
    ///
    /// - `LodLevel::Full`: Returns `None` — caller should use the standard mesher.
    /// - Other levels: Downsamples and produces a simplified mesh.
    pub fn mesh_chunk_lod(voxels: &[u32; CHUNK_SIZE_CB], lod: LodLevel) -> Option<ChunkMesh> {
        match lod {
            LodLevel::Full => None, // Use standard mesher
            LodLevel::Reduced => Some(Self::mesh_downsampled(voxels, 2)),
            LodLevel::Low => Some(Self::mesh_downsampled(voxels, 4)),
            LodLevel::Minimal => Some(Self::mesh_downsampled(voxels, 8)),
        }
    }

    /// Downsample voxels by `scale` and produce a greedy-merged mesh.
    ///
    /// Each `scale x scale x scale` block of voxels is reduced to a single cell
    /// using majority-vote (most common non-air block type wins). The resulting
    /// reduced grid is then meshed with face culling and greedy merging, and the
    /// output quads are scaled back to full chunk coordinates.
    fn mesh_downsampled(voxels: &[u32; CHUNK_SIZE_CB], scale: usize) -> ChunkMesh {
        let grid_size = CHUNK_SIZE / scale;
        let grid_len = grid_size * grid_size * grid_size;

        let mut grid = vec![0u32; grid_len];
        for gz in 0..grid_size {
            for gy in 0..grid_size {
                for gx in 0..grid_size {
                    grid[gz * grid_size * grid_size + gy * grid_size + gx] =
                        Self::downsample_cell(voxels, gx, gy, gz, scale);
                }
            }
        }

        let mut mesh = ChunkMesh::new();
        Self::mesh_reduced_grid(&grid, grid_size, scale, &mut mesh);
        mesh
    }

    /// Determine the block type for a downsampled cell via majority vote.
    ///
    /// Counts non-air block types in the `scale^3` region and picks the most
    /// common one. If more than half the voxels are air, the cell is air.
    fn downsample_cell(
        voxels: &[u32; CHUNK_SIZE_CB],
        gx: usize,
        gy: usize,
        gz: usize,
        scale: usize,
    ) -> u32 {
        let base_x = gx * scale;
        let base_y = gy * scale;
        let base_z = gz * scale;

        let total = scale * scale * scale;
        let mut air_count = 0u32;
        let mut types = [0u32; 4];
        let mut counts = [0u32; 4];
        let mut num_types = 0usize;

        for dz in 0..scale {
            for dy in 0..scale {
                for dx in 0..scale {
                    let x = base_x + dx;
                    let y = base_y + dy;
                    let z = base_z + dz;
                    let block = voxels[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x];

                    if block == 0 {
                        air_count += 1;
                        continue;
                    }

                    let mut found = false;
                    for i in 0..num_types {
                        if types[i] == block {
                            counts[i] += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found && num_types < 4 {
                        types[num_types] = block;
                        counts[num_types] = 1;
                        num_types += 1;
                    }
                }
            }
        }

        if air_count > (total as u32) / 2 {
            return 0;
        }

        let mut best_type = 0u32;
        let mut best_count = 0u32;
        for i in 0..num_types {
            if counts[i] > best_count {
                best_count = counts[i];
                best_type = types[i];
            }
        }
        best_type
    }

    /// Run face culling and greedy merge on a reduced-resolution grid.
    ///
    /// Output quads have coordinates and dimensions scaled by `scale` to map
    /// back to full chunk space.
    fn mesh_reduced_grid(grid: &[u32], grid_size: usize, scale: usize, result: &mut ChunkMesh) {
        let gs = grid_size;
        let gs2 = gs * gs;

        for face_idx in 0..6u8 {
            let face = match face_idx {
                0 => Face::Right,
                1 => Face::Left,
                2 => Face::Up,
                3 => Face::Down,
                4 => Face::Front,
                5 => Face::Back,
                _ => unreachable!(),
            };

            for layer in 0..gs {
                let mut mask = vec![false; gs2];

                for row in 0..gs {
                    for col in 0..gs {
                        let (x, y, z) = Self::remap_coords(face_idx, layer, row, col);
                        let block = grid[z * gs2 + y * gs + x];
                        if block == 0 {
                            continue;
                        }

                        let neighbor_block = Self::get_neighbor(grid, gs, face_idx, x, y, z);
                        if neighbor_block == 0 {
                            mask[row * gs + col] = true;
                        }
                    }
                }

                let mut visited = vec![false; gs2];
                for row in 0..gs {
                    for col in 0..gs {
                        if !mask[row * gs + col] || visited[row * gs + col] {
                            continue;
                        }

                        let (x, y, z) = Self::remap_coords(face_idx, layer, row, col);
                        let block = grid[z * gs2 + y * gs + x];

                        let mut width = 1usize;
                        while col + width < gs
                            && mask[row * gs + col + width]
                            && !visited[row * gs + col + width]
                        {
                            let (nx, ny, nz) =
                                Self::remap_coords(face_idx, layer, row, col + width);
                            if grid[nz * gs2 + ny * gs + nx] != block {
                                break;
                            }
                            width += 1;
                        }

                        let mut height = 1usize;
                        'outer: while row + height < gs {
                            for c in col..col + width {
                                if !mask[(row + height) * gs + c]
                                    || visited[(row + height) * gs + c]
                                {
                                    break 'outer;
                                }
                                let (nx, ny, nz) =
                                    Self::remap_coords(face_idx, layer, row + height, c);
                                if grid[nz * gs2 + ny * gs + nx] != block {
                                    break 'outer;
                                }
                            }
                            height += 1;
                        }

                        for r in row..row + height {
                            for c in col..col + width {
                                visited[r * gs + c] = true;
                            }
                        }

                        let (qx, qy, qz, qw, qh) =
                            Self::emit_scaled_quad(face_idx, layer, row, col, width, height, scale);

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
                }
            }
        }
    }

    /// Map (face, layer, row, col) to (x, y, z) in grid space.
    ///
    /// Matches the axis remapping used by the binary greedy mesher:
    /// - Face 0,1 (+X,-X): layer=z, row=y, col=x
    /// - Face 2,3 (+Y,-Y): layer=z, row=x, col=y
    /// - Face 4,5 (+Z,-Z): layer=y, row=x, col=z
    #[inline]
    fn remap_coords(face_idx: u8, layer: usize, row: usize, col: usize) -> (usize, usize, usize) {
        match face_idx {
            0 | 1 => (col, row, layer), // x=col, y=row, z=layer
            2 | 3 => (row, col, layer), // x=row, y=col, z=layer
            4 | 5 => (row, layer, col), // x=row, y=layer, z=col
            _ => unreachable!(),
        }
    }

    /// Get the neighbor block in the face direction. Returns 0 if out of bounds.
    #[inline]
    fn get_neighbor(grid: &[u32], gs: usize, face_idx: u8, x: usize, y: usize, z: usize) -> u32 {
        let gs2 = gs * gs;
        let (nx, ny, nz) = match face_idx {
            0 => {
                if x + 1 >= gs {
                    return 0;
                }
                (x + 1, y, z)
            }
            1 => {
                if x == 0 {
                    return 0;
                }
                (x - 1, y, z)
            }
            2 => {
                if y + 1 >= gs {
                    return 0;
                }
                (x, y + 1, z)
            }
            3 => {
                if y == 0 {
                    return 0;
                }
                (x, y - 1, z)
            }
            4 => {
                if z + 1 >= gs {
                    return 0;
                }
                (x, y, z + 1)
            }
            5 => {
                if z == 0 {
                    return 0;
                }
                (x, y, z - 1)
            }
            _ => unreachable!(),
        };
        grid[nz * gs2 + ny * gs + nx]
    }

    /// Compute the scaled quad position and dimensions for output.
    ///
    /// Maps from reduced grid coordinates back to full chunk coordinates.
    #[inline]
    fn emit_scaled_quad(
        face_idx: u8,
        layer: usize,
        row: usize,
        col: usize,
        width: usize,
        height: usize,
        scale: usize,
    ) -> (u8, u8, u8, u8, u8) {
        let s = scale;
        match face_idx {
            // Face 0,1: x=col, y=row, z=layer; width along x, height along y
            0 | 1 => (
                (col * s) as u8,
                (row * s) as u8,
                (layer * s) as u8,
                (width * s) as u8,
                (height * s) as u8,
            ),
            // Face 2,3: x=row, y=col, z=layer; width along x (from row), height along y (from col)
            2 | 3 => (
                (row * s) as u8,
                (col * s) as u8,
                (layer * s) as u8,
                (height * s) as u8,
                (width * s) as u8,
            ),
            // Face 4,5: x=row, y=layer, z=col; width along x (from row), height along z (from col)
            4 | 5 => (
                (row * s) as u8,
                (layer * s) as u8,
                (col * s) as u8,
                (height * s) as u8,
                (width * s) as u8,
            ),
            _ => unreachable!(),
        }
    }
}

/// Statistics about LOD mesh generation for performance monitoring.
#[derive(Clone, Debug, Default)]
pub struct LodStats {
    /// Number of chunks at each LOD level.
    pub chunks_per_level: [u32; 4],
    /// Total quads generated at each LOD level.
    pub quads_per_level: [u32; 4],
}

impl LodStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a chunk mesh at the given LOD level.
    pub fn record(&mut self, lod: LodLevel, quad_count: u32) {
        let idx = lod as usize;
        self.chunks_per_level[idx] += 1;
        self.quads_per_level[idx] += quad_count;
    }

    /// Total chunks across all LOD levels.
    pub fn total_chunks(&self) -> u32 {
        self.chunks_per_level.iter().sum()
    }

    /// Total quads across all LOD levels.
    pub fn total_quads(&self) -> u32 {
        self.quads_per_level.iter().sum()
    }

    /// Average quad reduction ratio compared to all-LOD-0.
    /// Returns the ratio of actual quads to estimated full-detail quads.
    pub fn reduction_ratio(&self, avg_full_detail_quads: u32) -> f32 {
        if avg_full_detail_quads == 0 || self.total_chunks() == 0 {
            return 1.0;
        }
        let estimated_full = self.total_chunks() as f32 * avg_full_detail_quads as f32;
        self.total_quads() as f32 / estimated_full
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_meshing_cpu::{terrain_chunk, uniform_chunk, ChunkMesher};

    #[test]
    fn lod_level_scales() {
        assert_eq!(LodLevel::Full.scale(), 1);
        assert_eq!(LodLevel::Reduced.scale(), 2);
        assert_eq!(LodLevel::Low.scale(), 4);
        assert_eq!(LodLevel::Minimal.scale(), 8);
    }

    #[test]
    fn lod_level_grid_sizes() {
        assert_eq!(LodLevel::Full.grid_size(), 32);
        assert_eq!(LodLevel::Reduced.grid_size(), 16);
        assert_eq!(LodLevel::Low.grid_size(), 8);
        assert_eq!(LodLevel::Minimal.grid_size(), 4);
    }

    #[test]
    fn lod_level_ordering() {
        assert!(LodLevel::Full < LodLevel::Reduced);
        assert!(LodLevel::Reduced < LodLevel::Low);
        assert!(LodLevel::Low < LodLevel::Minimal);
    }

    #[test]
    fn default_config_thresholds() {
        let config = LodConfig::default();
        assert_eq!(config.select_lod(0.0), Some(LodLevel::Full));
        assert_eq!(config.select_lod(16.0), Some(LodLevel::Full));
        assert_eq!(config.select_lod(17.0), Some(LodLevel::Reduced));
        assert_eq!(config.select_lod(32.0), Some(LodLevel::Reduced));
        assert_eq!(config.select_lod(33.0), Some(LodLevel::Low));
        assert_eq!(config.select_lod(48.0), Some(LodLevel::Low));
        assert_eq!(config.select_lod(49.0), Some(LodLevel::Minimal));
        assert_eq!(config.select_lod(64.0), Some(LodLevel::Minimal));
        assert_eq!(config.select_lod(65.0), None);
    }

    #[test]
    fn custom_config_thresholds() {
        let config = LodConfig::new(8.0, 16.0, 24.0, 32.0);
        assert_eq!(config.select_lod(5.0), Some(LodLevel::Full));
        assert_eq!(config.select_lod(12.0), Some(LodLevel::Reduced));
        assert_eq!(config.select_lod(20.0), Some(LodLevel::Low));
        assert_eq!(config.select_lod(28.0), Some(LodLevel::Minimal));
        assert_eq!(config.select_lod(33.0), None);
    }

    #[test]
    fn blend_factor_zero_in_center() {
        let config = LodConfig::default();
        let transition = config.select_lod_with_blend(8.0).unwrap();
        assert_eq!(transition.primary, LodLevel::Full);
        assert_eq!(transition.blend, 0.0);
        assert!(!transition.is_blending());
    }

    #[test]
    fn blend_factor_nonzero_near_boundary() {
        let config = LodConfig::default();
        let transition = config.select_lod_with_blend(15.5).unwrap();
        assert_eq!(transition.primary, LodLevel::Full);
        assert_eq!(transition.next, Some(LodLevel::Reduced));
        assert!(transition.blend > 0.0);
        assert!(transition.blend < 1.0);
        assert!(transition.is_blending());
    }

    #[test]
    fn blend_factor_at_exact_boundary() {
        let config = LodConfig::default();
        let transition = config.select_lod_with_blend(16.0).unwrap();
        assert_eq!(transition.primary, LodLevel::Full);
        assert!((transition.blend - 0.5).abs() < 0.01);
    }

    #[test]
    fn beyond_render_distance_returns_none() {
        let config = LodConfig::default();
        assert!(config.select_lod_with_blend(100.0).is_none());
    }

    #[test]
    fn lod_full_returns_none() {
        let chunk = uniform_chunk(1);
        assert!(LodMesher::mesh_chunk_lod(&chunk, LodLevel::Full).is_none());
    }

    #[test]
    fn lod_air_chunk_produces_no_quads() {
        let chunk = uniform_chunk(0);
        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            assert!(
                mesh.is_empty(),
                "Air chunk at {:?} should produce 0 quads, got {}",
                lod,
                mesh.quad_count()
            );
        }
    }

    #[test]
    fn lod_solid_chunk_produces_surface_quads() {
        let chunk = uniform_chunk(1);
        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            assert!(
                !mesh.is_empty(),
                "Solid chunk at {:?} should produce quads",
                lod
            );

            for q in &mesh.quads {
                assert_eq!(q.block_type, 1, "Block type mismatch at {:?}", lod);
            }

            let faces: std::collections::HashSet<_> = mesh.quads.iter().map(|q| q.face).collect();
            assert_eq!(
                faces.len(),
                6,
                "Solid chunk at {:?} should have all 6 faces, got {}",
                lod,
                faces.len()
            );
        }
    }

    #[test]
    fn lod_reduces_quad_count() {
        let chunk = terrain_chunk();

        let full_mesh = ferrum_meshing_cpu::CpuMesher::new().mesh_chunk(&chunk);
        let reduced = LodMesher::mesh_chunk_lod(&chunk, LodLevel::Reduced).unwrap();
        let low = LodMesher::mesh_chunk_lod(&chunk, LodLevel::Low).unwrap();
        let minimal = LodMesher::mesh_chunk_lod(&chunk, LodLevel::Minimal).unwrap();

        assert!(
            reduced.quad_count() < full_mesh.quad_count(),
            "Reduced ({}) should have fewer quads than Full ({})",
            reduced.quad_count(),
            full_mesh.quad_count()
        );
        assert!(
            low.quad_count() < reduced.quad_count(),
            "Low ({}) should have fewer quads than Reduced ({})",
            low.quad_count(),
            reduced.quad_count()
        );
        assert!(
            minimal.quad_count() < low.quad_count(),
            "Minimal ({}) should have fewer quads than Low ({})",
            minimal.quad_count(),
            low.quad_count()
        );
    }

    #[test]
    fn lod_quads_within_chunk_bounds() {
        let chunk = terrain_chunk();
        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            for q in &mesh.quads {
                assert!(q.x < CHUNK_SIZE as u8, "{:?}: x={} out of bounds", lod, q.x);
                assert!(q.y < CHUNK_SIZE as u8, "{:?}: y={} out of bounds", lod, q.y);
                assert!(q.z < CHUNK_SIZE as u8, "{:?}: z={} out of bounds", lod, q.z);
                assert!(q.width > 0, "{:?}: width must be > 0", lod);
                assert!(q.height > 0, "{:?}: height must be > 0", lod);
                assert!(q.block_type > 0, "{:?}: block type must be non-air", lod);
            }
        }
    }

    #[test]
    fn lod_quad_dimensions_are_multiples_of_scale() {
        let chunk = uniform_chunk(1);
        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let scale = lod.scale() as u8;
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            for q in &mesh.quads {
                assert_eq!(
                    q.x % scale,
                    0,
                    "{:?}: x={} not aligned to scale {}",
                    lod,
                    q.x,
                    scale
                );
                assert_eq!(
                    q.y % scale,
                    0,
                    "{:?}: y={} not aligned to scale {}",
                    lod,
                    q.y,
                    scale
                );
                assert_eq!(
                    q.z % scale,
                    0,
                    "{:?}: z={} not aligned to scale {}",
                    lod,
                    q.z,
                    scale
                );
                assert_eq!(
                    q.width % scale,
                    0,
                    "{:?}: width={} not multiple of scale {}",
                    lod,
                    q.width,
                    scale
                );
                assert_eq!(
                    q.height % scale,
                    0,
                    "{:?}: height={} not multiple of scale {}",
                    lod,
                    q.height,
                    scale
                );
            }
        }
    }

    #[test]
    fn lod_solid_chunk_quad_count_per_level() {
        let chunk = uniform_chunk(1);

        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            let expected = 6 * lod.grid_size();
            assert_eq!(
                mesh.quad_count(),
                expected,
                "{:?}: expected {} quads for solid chunk, got {}",
                lod,
                expected,
                mesh.quad_count()
            );
        }
    }

    #[test]
    fn lod_preserves_block_types_in_terrain() {
        let chunk = terrain_chunk();
        for lod in [LodLevel::Reduced, LodLevel::Low, LodLevel::Minimal] {
            let mesh = LodMesher::mesh_chunk_lod(&chunk, lod).unwrap();
            let block_types: std::collections::HashSet<_> =
                mesh.quads.iter().map(|q| q.block_type).collect();
            assert!(
                !block_types.is_empty(),
                "{:?}: should have block types",
                lod
            );
            for &bt in &block_types {
                assert!(
                    bt >= 1 && bt <= 3,
                    "{:?}: unexpected block type {}",
                    lod,
                    bt
                );
            }
        }
    }

    #[test]
    fn lod_stats_tracking() {
        let mut stats = LodStats::new();
        stats.record(LodLevel::Full, 1000);
        stats.record(LodLevel::Full, 800);
        stats.record(LodLevel::Reduced, 200);
        stats.record(LodLevel::Low, 50);
        stats.record(LodLevel::Minimal, 10);

        assert_eq!(stats.total_chunks(), 5);
        assert_eq!(stats.total_quads(), 2060);
        assert_eq!(stats.chunks_per_level[0], 2);
        assert_eq!(stats.chunks_per_level[1], 1);
    }

    #[test]
    fn lod_stats_reduction_ratio() {
        let mut stats = LodStats::new();
        for _ in 0..5 {
            stats.record(LodLevel::Full, 100);
        }
        for _ in 0..5 {
            stats.record(LodLevel::Reduced, 25);
        }
        let ratio = stats.reduction_ratio(100);
        assert!((ratio - 0.625).abs() < 0.01);
    }
}
