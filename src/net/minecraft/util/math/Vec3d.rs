use super::Vec3i::Vec3i;
use core::ops::{Add, Mul, Sub};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    const INTERSECTION_EPSILON_SQ: f64 = 1.000_000_011_686_097_4E-7;

    pub fn new(mut x: f64, mut y: f64, mut z: f64) -> Self {
        if x == -0.0 {
            x = 0.0;
        }
        if y == -0.0 {
            y = 0.0;
        }
        if z == -0.0 {
            z = 0.0;
        }
        Self { x, y, z }
    }

    pub fn from_vec3i(value: Vec3i) -> Self {
        Self::new(value.x as f64, value.y as f64, value.z as f64)
    }

    pub fn subtract_reverse(self, vector: Self) -> Self {
        vector - self
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length < 1.0E-4 {
            Self::ZERO
        } else {
            self.scale(1.0 / length)
        }
    }

    pub fn dot(self, vector: Self) -> f64 {
        self.x * vector.x + self.y * vector.y + self.z * vector.z
    }

    pub fn cross(self, vector: Self) -> Self {
        Self::new(
            self.y * vector.z - self.z * vector.y,
            self.z * vector.x - self.x * vector.z,
            self.x * vector.y - self.y * vector.x,
        )
    }

    pub fn add_vector(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn subtract_vector(self, x: f64, y: f64, z: f64) -> Self {
        self.add_vector(-x, -y, -z)
    }

    pub fn distance_to(self, vector: Self) -> f64 {
        self.square_distance_to(vector).sqrt()
    }

    pub fn square_distance_to(self, vector: Self) -> f64 {
        self.square_distance_to_coords(vector.x, vector.y, vector.z)
    }

    pub fn square_distance_to_coords(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = x - self.x;
        let dy = y - self.y;
        let dz = z - self.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn intermediate_x(self, vector: Self, x: f64) -> Option<Self> {
        let delta = vector - self;
        if delta.x * delta.x < Self::INTERSECTION_EPSILON_SQ {
            return None;
        }
        let factor = (x - self.x) / delta.x;
        (0.0..=1.0)
            .contains(&factor)
            .then(|| self + delta.scale(factor))
    }

    pub fn intermediate_y(self, vector: Self, y: f64) -> Option<Self> {
        let delta = vector - self;
        if delta.y * delta.y < Self::INTERSECTION_EPSILON_SQ {
            return None;
        }
        let factor = (y - self.y) / delta.y;
        (0.0..=1.0)
            .contains(&factor)
            .then(|| self + delta.scale(factor))
    }

    pub fn intermediate_z(self, vector: Self, z: f64) -> Option<Self> {
        let delta = vector - self;
        if delta.z * delta.z < Self::INTERSECTION_EPSILON_SQ {
            return None;
        }
        let factor = (z - self.z) / delta.z;
        (0.0..=1.0)
            .contains(&factor)
            .then(|| self + delta.scale(factor))
    }

    pub fn rotate_pitch(self, pitch: f32) -> Self {
        let cos = super::cos(pitch) as f64;
        let sin = super::sin(pitch) as f64;
        Self::new(
            self.x,
            self.y * cos + self.z * sin,
            self.z * cos - self.y * sin,
        )
    }

    pub fn rotate_yaw(self, yaw: f32) -> Self {
        let cos = super::cos(yaw) as f64;
        let sin = super::sin(yaw) as f64;
        Self::new(
            self.x * cos + self.z * sin,
            self.y,
            self.z * cos - self.x * sin,
        )
    }
}

impl Add for Vec3d {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3d {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3d {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_negative_zero() {
        let vector = Vec3d::new(-0.0, -0.0, -0.0);
        assert_eq!(vector.x.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn tiny_vector_normalizes_to_zero() {
        assert_eq!(Vec3d::new(0.00001, 0.0, 0.0).normalize(), Vec3d::ZERO);
    }
}
