use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 32;

pub struct LightingEngine {
    block_light: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    sky_light: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl LightingEngine {
    pub fn new() -> Self {
        Self {
            block_light: [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            sky_light: [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
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
}

impl Default for LightingEngine {
    fn default() -> Self {
        Self::new()
    }
}
