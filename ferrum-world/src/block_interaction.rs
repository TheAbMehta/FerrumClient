use crate::Chunk;
use ferrum_core::BlockId;
use glam::Vec3;

pub trait BlockInteraction {
    fn break_block(&mut self, x: usize, y: usize, z: usize);
    fn place_block(&mut self, x: usize, y: usize, z: usize, block_id: BlockId) -> bool;
    fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(usize, usize, usize)>;
}

impl BlockInteraction for Chunk {
    fn break_block(&mut self, x: usize, y: usize, z: usize) {
        self.set_block(x, y, z, BlockId::new(0));
    }

    fn place_block(&mut self, x: usize, y: usize, z: usize, block_id: BlockId) -> bool {
        let current = self.get_block(x, y, z);
        if current.as_u16() == 0 {
            self.set_block(x, y, z, block_id);
            true
        } else {
            false
        }
    }

    fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(usize, usize, usize)> {
        let dir = direction.normalize();
        let step = 0.1;
        let max_steps = (max_distance / step) as usize;

        for i in 0..max_steps {
            let t = i as f32 * step;
            let pos = origin + dir * t;

            let x = pos.x.floor() as i32;
            let y = pos.y.floor() as i32;
            let z = pos.z.floor() as i32;

            if x < 0 || y < 0 || z < 0 || x >= 32 || y >= 32 || z >= 32 {
                continue;
            }

            let block = self.get_block(x as usize, y as usize, z as usize);
            if block.as_u16() != 0 {
                return Some((x as usize, y as usize, z as usize));
            }
        }

        None
    }
}
