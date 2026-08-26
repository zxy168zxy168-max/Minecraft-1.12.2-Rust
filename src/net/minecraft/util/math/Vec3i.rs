use super::Vec3d::Vec3d;
use core::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self::new(0, 0, 0);
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(
            super::floor_f64(x),
            super::floor_f64(y),
            super::floor_f64(z),
        )
    }
    pub fn cross_product(self, other: Self) -> Self {
        Self::new(
            self.y
                .wrapping_mul(other.z)
                .wrapping_sub(self.z.wrapping_mul(other.y)),
            self.z
                .wrapping_mul(other.x)
                .wrapping_sub(self.x.wrapping_mul(other.z)),
            self.x
                .wrapping_mul(other.y)
                .wrapping_sub(self.y.wrapping_mul(other.x)),
        )
    }
    pub fn distance(self, x: i32, y: i32, z: i32) -> f64 {
        let dx = self.x.wrapping_sub(x) as f64;
        let dy = self.y.wrapping_sub(y) as f64;
        let dz = self.z.wrapping_sub(z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    pub fn distance_sq_coords(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x as f64 - x;
        let dy = self.y as f64 - y;
        let dz = self.z as f64 - z;
        dx * dx + dy * dy + dz * dz
    }
    pub fn distance_sq_to_center(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x as f64 + 0.5 - x;
        let dy = self.y as f64 + 0.5 - y;
        let dz = self.z as f64 + 0.5 - z;
        dx * dx + dy * dy + dz * dz
    }
    pub fn distance_sq(self, other: Self) -> f64 {
        self.distance_sq_coords(other.x as f64, other.y as f64, other.z as f64)
    }
    pub fn to_vec3d(self) -> Vec3d {
        Vec3d::new(self.x as f64, self.y as f64, self.z as f64)
    }
}

impl Ord for Vec3i {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y
            .cmp(&other.y)
            .then_with(|| self.z.cmp(&other.z))
            .then_with(|| self.x.cmp(&other.x))
    }
}
impl PartialOrd for Vec3i {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mcp_order_is_y_then_z_then_x() {
        let mut values = [
            Vec3i::new(0, 1, 0),
            Vec3i::new(5, 0, 1),
            Vec3i::new(1, 0, 0),
        ];
        values.sort();
        assert_eq!(
            values,
            [
                Vec3i::new(1, 0, 0),
                Vec3i::new(5, 0, 1),
                Vec3i::new(0, 1, 0)
            ]
        );
    }
}
