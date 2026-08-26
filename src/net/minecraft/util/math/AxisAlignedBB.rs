use super::BlockPos::BlockPos;
use super::EnumFacing;
use super::Vec3d::Vec3d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisAlignedBB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AxisAlignedBB {
    pub fn new(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self {
            min_x: x1.min(x2),
            min_y: y1.min(y2),
            min_z: z1.min(z2),
            max_x: x1.max(x2),
            max_y: y1.max(y2),
            max_z: z1.max(z2),
        }
    }

    pub fn from_block(position: BlockPos) -> Self {
        Self::new(
            position.x as f64,
            position.y as f64,
            position.z as f64,
            position.x as f64 + 1.0,
            position.y as f64 + 1.0,
            position.z as f64 + 1.0,
        )
    }

    pub fn from_vectors(minimum: Vec3d, maximum: Vec3d) -> Self {
        Self::new(
            minimum.x, minimum.y, minimum.z, maximum.x, maximum.y, maximum.z,
        )
    }

    pub fn set_max_y(self, maximum_y: f64) -> Self {
        Self::new(
            self.min_x, self.min_y, self.min_z, self.max_x, maximum_y, self.max_z,
        )
    }

    /// Port of MCP `func_191195_a`.
    pub fn contract_directional(self, x: f64, y: f64, z: f64) -> Self {
        let (mut min_x, mut min_y, mut min_z) = (self.min_x, self.min_y, self.min_z);
        let (mut max_x, mut max_y, mut max_z) = (self.max_x, self.max_y, self.max_z);
        if x < 0.0 {
            min_x -= x;
        } else if x > 0.0 {
            max_x -= x;
        }
        if y < 0.0 {
            min_y -= y;
        } else if y > 0.0 {
            max_y -= y;
        }
        if z < 0.0 {
            min_z -= z;
        } else if z > 0.0 {
            max_z -= z;
        }
        Self::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn add_coord(self, x: f64, y: f64, z: f64) -> Self {
        let (mut min_x, mut min_y, mut min_z) = (self.min_x, self.min_y, self.min_z);
        let (mut max_x, mut max_y, mut max_z) = (self.max_x, self.max_y, self.max_z);
        if x < 0.0 {
            min_x += x;
        } else if x > 0.0 {
            max_x += x;
        }
        if y < 0.0 {
            min_y += y;
        } else if y > 0.0 {
            max_y += y;
        }
        if z < 0.0 {
            min_z += z;
        } else if z > 0.0 {
            max_z += z;
        }
        Self::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn expand(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            self.min_x - x,
            self.min_y - y,
            self.min_z - z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    pub fn expand_xyz(self, value: f64) -> Self {
        self.expand(value, value, value)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self::new(
            self.min_x.max(other.min_x),
            self.min_y.max(other.min_y),
            self.min_z.max(other.min_z),
            self.max_x.min(other.max_x),
            self.max_y.min(other.max_y),
            self.max_z.min(other.max_z),
        )
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.min_z.min(other.min_z),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
            self.max_z.max(other.max_z),
        )
    }

    pub fn offset(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            self.min_x + x,
            self.min_y + y,
            self.min_z + z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    pub fn intersects(self, other: Self) -> bool {
        other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z
    }

    pub fn contains(self, point: Vec3d) -> bool {
        point.x > self.min_x
            && point.x < self.max_x
            && point.y > self.min_y
            && point.y < self.max_y
            && point.z > self.min_z
            && point.z < self.max_z
    }

    pub fn average_edge_length(self) -> f64 {
        ((self.max_x - self.min_x) + (self.max_y - self.min_y) + (self.max_z - self.min_z)) / 3.0
    }

    pub fn calculate_x_offset(self, other: Self, mut offset_x: f64) -> f64 {
        if other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z
        {
            if offset_x > 0.0 && other.max_x <= self.min_x {
                let delta = self.min_x - other.max_x;
                if delta < offset_x {
                    offset_x = delta;
                }
            } else if offset_x < 0.0 && other.min_x >= self.max_x {
                let delta = self.max_x - other.min_x;
                if delta > offset_x {
                    offset_x = delta;
                }
            }
        }
        offset_x
    }

    pub fn calculate_y_offset(self, other: Self, mut offset_y: f64) -> f64 {
        if other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_z > self.min_z
            && other.min_z < self.max_z
        {
            if offset_y > 0.0 && other.max_y <= self.min_y {
                let delta = self.min_y - other.max_y;
                if delta < offset_y {
                    offset_y = delta;
                }
            } else if offset_y < 0.0 && other.min_y >= self.max_y {
                let delta = self.max_y - other.min_y;
                if delta > offset_y {
                    offset_y = delta;
                }
            }
        }
        offset_y
    }

    pub fn calculate_z_offset(self, other: Self, mut offset_z: f64) -> f64 {
        if other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y
        {
            if offset_z > 0.0 && other.max_z <= self.min_z {
                let delta = self.min_z - other.max_z;
                if delta < offset_z {
                    offset_z = delta;
                }
            } else if offset_z < 0.0 && other.min_z >= self.max_z {
                let delta = self.max_z - other.min_z;
                if delta > offset_z {
                    offset_z = delta;
                }
            }
        }
        offset_z
    }

    /// Port of MCP 1.12.2 `AxisAlignedBB.calculateIntercept`.
    pub fn calculate_intercept(self, vec_a: Vec3d, vec_b: Vec3d) -> Option<(Vec3d, EnumFacing)> {
        let mut hit = self.collide_with_x_plane(self.min_x, vec_a, vec_b);
        let mut facing = EnumFacing::West;

        let mut candidate = self.collide_with_x_plane(self.max_x, vec_a, vec_b);
        if candidate.is_some_and(|value| Self::is_closer(vec_a, hit, value)) {
            hit = candidate;
            facing = EnumFacing::East;
        }
        candidate = self.collide_with_y_plane(self.min_y, vec_a, vec_b);
        if candidate.is_some_and(|value| Self::is_closer(vec_a, hit, value)) {
            hit = candidate;
            facing = EnumFacing::Down;
        }
        candidate = self.collide_with_y_plane(self.max_y, vec_a, vec_b);
        if candidate.is_some_and(|value| Self::is_closer(vec_a, hit, value)) {
            hit = candidate;
            facing = EnumFacing::Up;
        }
        candidate = self.collide_with_z_plane(self.min_z, vec_a, vec_b);
        if candidate.is_some_and(|value| Self::is_closer(vec_a, hit, value)) {
            hit = candidate;
            facing = EnumFacing::North;
        }
        candidate = self.collide_with_z_plane(self.max_z, vec_a, vec_b);
        if candidate.is_some_and(|value| Self::is_closer(vec_a, hit, value)) {
            hit = candidate;
            facing = EnumFacing::South;
        }
        hit.map(|value| (value, facing))
    }

    fn is_closer(origin: Vec3d, current: Option<Vec3d>, candidate: Vec3d) -> bool {
        current.map_or(true, |value| {
            origin.square_distance_to(candidate) < origin.square_distance_to(value)
        })
    }

    fn collide_with_x_plane(self, plane: f64, from: Vec3d, to: Vec3d) -> Option<Vec3d> {
        from.intermediate_x(to, plane).filter(|value| {
            value.y >= self.min_y
                && value.y <= self.max_y
                && value.z >= self.min_z
                && value.z <= self.max_z
        })
    }

    fn collide_with_y_plane(self, plane: f64, from: Vec3d, to: Vec3d) -> Option<Vec3d> {
        from.intermediate_y(to, plane).filter(|value| {
            value.x >= self.min_x
                && value.x <= self.max_x
                && value.z >= self.min_z
                && value.z <= self.max_z
        })
    }

    fn collide_with_z_plane(self, plane: f64, from: Vec3d, to: Vec3d) -> Option<Vec3d> {
        from.intermediate_z(to, plane).filter(|value| {
            value.x >= self.min_x
                && value.x <= self.max_x
                && value.y >= self.min_y
                && value.y <= self.max_y
        })
    }

    pub fn side_center(self, facing: EnumFacing) -> Vec3d {
        let center_x = (self.min_x + self.max_x) * 0.5;
        let center_y = (self.min_y + self.max_y) * 0.5;
        let center_z = (self.min_z + self.max_z) * 0.5;
        match facing {
            EnumFacing::Down => Vec3d::new(center_x, self.min_y, center_z),
            EnumFacing::Up => Vec3d::new(center_x, self.max_y, center_z),
            EnumFacing::North => Vec3d::new(center_x, center_y, self.min_z),
            EnumFacing::South => Vec3d::new(center_x, center_y, self.max_z),
            EnumFacing::West => Vec3d::new(self.min_x, center_y, center_z),
            EnumFacing::East => Vec3d::new(self.max_x, center_y, center_z),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touching_boxes_do_not_intersect() {
        let left = AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let right = AxisAlignedBB::new(1.0, 0.0, 0.0, 2.0, 1.0, 1.0);
        assert!(!left.intersects(right));
    }
}
