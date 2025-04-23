use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn penetration(&self, other: &Aabb) -> Option<Vec3> {
        if !self.intersects(other) {
            return None;
        }

        let dx1 = other.max.x - self.min.x;
        let dx2 = self.max.x - other.min.x;
        let dy1 = other.max.y - self.min.y;
        let dy2 = self.max.y - other.min.y;
        let dz1 = other.max.z - self.min.z;
        let dz2 = self.max.z - other.min.z;

        let dx = if dx1 < dx2 { dx1 } else { -dx2 };
        let dy = if dy1 < dy2 { dy1 } else { -dy2 };
        let dz = if dz1 < dz2 { dz1 } else { -dz2 };

        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        let abs_dz = dz.abs();

        if abs_dx < abs_dy && abs_dx < abs_dz {
            Some(Vec3::new(dx, 0.0, 0.0))
        } else if abs_dy < abs_dz {
            Some(Vec3::new(0.0, dy, 0.0))
        } else {
            Some(Vec3::new(0.0, 0.0, dz))
        }
    }
}
