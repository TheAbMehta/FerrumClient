use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 32;

pub struct LightingEngine {
    block_light: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    sky_light: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    opaque: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl LightingEngine {
    pub fn new() -> Self {
        Self {
            block_light: [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            sky_light: [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            opaque: [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }

    pub fn set_opaque(&mut self, x: usize, y: usize, z: usize, is_opaque: bool) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }
        self.opaque[x][y][z] = is_opaque;
    }

    pub fn get_opaque(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return false;
        }
        self.opaque[x][y][z]
    }

    pub fn get_block_light(&self, x: usize, y: usize, z: usize) -> u8 {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return 0;
        }
        self.block_light[x][y][z]
    }

    pub fn set_block_light(&mut self, x: usize, y: usize, z: usize, value: u8) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }
        self.block_light[x][y][z] = value.min(15);
    }

    pub fn get_sky_light(&self, x: usize, y: usize, z: usize) -> u8 {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return 0;
        }
        self.sky_light[x][y][z]
    }

    pub fn set_sky_light(&mut self, x: usize, y: usize, z: usize, value: u8) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }
        self.sky_light[x][y][z] = value.min(15);
    }

    pub fn get_combined_light(&self, x: usize, y: usize, z: usize) -> u8 {
        self.get_block_light(x, y, z)
            .max(self.get_sky_light(x, y, z))
    }

    pub fn propagate_block_light(
        &mut self,
        opaque: &[[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    ) {
        let mut result = [[[0u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut queue = VecDeque::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.block_light[x][y][z] > 0 && !opaque[x][y][z] {
                        result[x][y][z] = self.block_light[x][y][z];
                        queue.push_back((x, y, z));
                    }
                }
            }
        }

        while let Some((x, y, z)) = queue.pop_front() {
            let current_light = result[x][y][z];
            if current_light <= 1 {
                continue;
            }

            let new_light = current_light - 1;
            let neighbors = [
                (x + 1, y, z),
                (x.wrapping_sub(1), y, z),
                (x, y + 1, z),
                (x, y.wrapping_sub(1), z),
                (x, y, z + 1),
                (x, y, z.wrapping_sub(1)),
            ];

            for (nx, ny, nz) in neighbors {
                if nx >= CHUNK_SIZE || ny >= CHUNK_SIZE || nz >= CHUNK_SIZE {
                    continue;
                }

                if opaque[nx][ny][nz] {
                    continue;
                }

                if result[nx][ny][nz] < new_light {
                    result[nx][ny][nz] = new_light;
                    queue.push_back((nx, ny, nz));
                }
            }
        }

        self.block_light = result;
    }

    pub fn propagate_sky_light(&mut self, opaque: &[[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]) {
        let mut result = [[[0u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut queue = VecDeque::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.sky_light[x][y][z] > 0 && !opaque[x][y][z] {
                        result[x][y][z] = self.sky_light[x][y][z];
                        queue.push_back((x, y, z));
                    }
                }
            }
        }

        while let Some((x, y, z)) = queue.pop_front() {
            let current_light = result[x][y][z];

            let neighbors = [
                (x + 1, y, z, current_light.saturating_sub(1)),
                (x.wrapping_sub(1), y, z, current_light.saturating_sub(1)),
                (x, y + 1, z, current_light.saturating_sub(1)),
                (x, y.wrapping_sub(1), z, current_light),
                (x, y, z + 1, current_light.saturating_sub(1)),
                (x, y, z.wrapping_sub(1), current_light.saturating_sub(1)),
            ];

            for (nx, ny, nz, new_light) in neighbors {
                if nx >= CHUNK_SIZE || ny >= CHUNK_SIZE || nz >= CHUNK_SIZE {
                    continue;
                }

                if new_light == 0 {
                    continue;
                }

                if opaque[nx][ny][nz] {
                    continue;
                }

                if result[nx][ny][nz] < new_light {
                    result[nx][ny][nz] = new_light;
                    queue.push_back((nx, ny, nz));
                }
            }
        }

        self.sky_light = result;
    }

    pub fn get_smooth_light(&self, x: usize, y: usize, z: usize, _face: usize) -> u8 {
        let x0 = x.saturating_sub(1);
        let y0 = y.saturating_sub(1);

        let l0 = self.get_combined_light(x0, y0, z);
        let l1 = self.get_combined_light(x, y0, z);
        let l2 = self.get_combined_light(x0, y, z);
        let l3 = self.get_combined_light(x, y, z);

        ((l0 as u32 + l1 as u32 + l2 as u32 + l3 as u32) / 4) as u8
    }

    pub fn calculate_ambient_occlusion_with_opaque(
        &self,
        opaque: &[[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        x: usize,
        y: usize,
        z: usize,
        face: usize,
        corner: usize,
    ) -> f32 {
        let (dx1, dy1, dz1, dx2, dy2, dz2, dx3, dy3, dz3) = match (face, corner) {
            (0, 0) => (1, 0, 0, 0, 1, 0, 1, 1, 0),
            (0, 1) => (1, 0, 0, 0, 0, 1, 1, 0, 1),
            (0, 2) => (1, 0, 0, 0, 1, 0, 1, 1, 0),
            (0, 3) => (1, 0, 0, 0, 0, 1, 1, 0, 1),
            (1, 0) => (-1, 0, 0, 0, 0, 1, -1, 0, 1),
            (1, 1) => (-1, 0, 0, 0, 0, -1, -1, 0, -1),
            (1, 2) => (-1, 0, 0, 0, 1, 0, -1, 1, 0),
            (1, 3) => (-1, 0, 0, 0, 1, 0, -1, 1, 0),
            (2, 0) => (0, 0, 1, 0, 1, 0, 0, 1, 1),
            (2, 1) => (0, 0, 1, 1, 0, 0, 1, 0, 1),
            (2, 2) => (0, 0, 1, 0, 1, 0, 0, 1, 1),
            (2, 3) => (0, 0, 1, -1, 0, 0, -1, 0, 1),
            (3, 0) => (0, 0, -1, 1, 0, 0, 1, 0, -1),
            (3, 1) => (0, 0, -1, 0, 0, -1, 0, 0, -1),
            (3, 2) => (0, 0, -1, 0, 1, 0, 0, 1, -1),
            (3, 3) => (0, 0, -1, -1, 0, 0, -1, 0, -1),
            (4, 0) => (0, 1, 0, 0, 0, 1, 0, 1, 1),
            (4, 1) => (0, 1, 0, 1, 0, 0, 1, 1, 0),
            (4, 2) => (0, 1, 0, 0, 0, 1, 0, 1, 1),
            (4, 3) => (0, 1, 0, -1, 0, 0, -1, 1, 0),
            (5, 0) => (0, -1, 0, 0, 0, 1, 0, -1, 1),
            (5, 1) => (0, -1, 0, 1, 0, 0, 1, -1, 0),
            (5, 2) => (0, -1, 0, 0, 0, -1, 0, -1, -1),
            (5, 3) => (0, -1, 0, -1, 0, 0, -1, -1, 0),
            _ => return 1.0,
        };

        let check_opaque = |dx: i32, dy: i32, dz: i32| -> bool {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            let nz = (z as i32 + dz) as usize;

            if nx >= CHUNK_SIZE || ny >= CHUNK_SIZE || nz >= CHUNK_SIZE {
                return false;
            }

            opaque[nx][ny][nz]
        };

        let side1 = check_opaque(dx1, dy1, dz1);
        let side2 = check_opaque(dx2, dy2, dz2);
        let diagonal = check_opaque(dx3, dy3, dz3);

        let occluded_count = (side1 as u32) + (side2 as u32) + (diagonal as u32);

        if side1 && side2 && diagonal {
            0.0
        } else if side1 && side2 {
            0.5
        } else {
            1.0 - (occluded_count as f32 / 4.0)
        }
    }

    pub fn calculate_ambient_occlusion(
        &self,
        x: usize,
        y: usize,
        z: usize,
        face: usize,
        corner: usize,
    ) -> f32 {
        self.calculate_ambient_occlusion_with_opaque(&self.opaque, x, y, z, face, corner)
    }
}

impl Default for LightingEngine {
    fn default() -> Self {
        Self::new()
    }
}
