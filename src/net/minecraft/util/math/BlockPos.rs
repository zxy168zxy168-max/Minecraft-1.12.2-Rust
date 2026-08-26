use super::Vec3d::Vec3d;
use super::Vec3i::Vec3i;
use super::{floor_f64, EnumFacing};
use core::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ORIGIN: Self = Self::new(0, 0, 0);
    pub const NUM_X_BITS: u32 = 26;
    pub const NUM_Z_BITS: u32 = 26;
    pub const NUM_Y_BITS: u32 = 12;
    pub const Y_SHIFT: u32 = Self::NUM_Z_BITS;
    pub const X_SHIFT: u32 = Self::Y_SHIFT + Self::NUM_Y_BITS;
    pub const X_MASK: u64 = (1_u64 << Self::NUM_X_BITS) - 1;
    pub const Y_MASK: u64 = (1_u64 << Self::NUM_Y_BITS) - 1;
    pub const Z_MASK: u64 = (1_u64 << Self::NUM_Z_BITS) - 1;

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(floor_f64(x), floor_f64(y), floor_f64(z))
    }

    pub fn from_vec3d(value: Vec3d) -> Self {
        Self::from_f64(value.x, value.y, value.z)
    }

    pub const fn from_vec3i(value: Vec3i) -> Self {
        Self::new(value.x, value.y, value.z)
    }

    pub fn add_f64(self, x: f64, y: f64, z: f64) -> Self {
        if x == 0.0 && y == 0.0 && z == 0.0 {
            self
        } else {
            Self::from_f64(self.x as f64 + x, self.y as f64 + y, self.z as f64 + z)
        }
    }

    pub const fn add(self, x: i32, y: i32, z: i32) -> Self {
        if x == 0 && y == 0 && z == 0 {
            self
        } else {
            Self::new(
                self.x.wrapping_add(x),
                self.y.wrapping_add(y),
                self.z.wrapping_add(z),
            )
        }
    }

    pub const fn subtract(self, value: Vec3i) -> Self {
        self.add(
            value.x.wrapping_neg(),
            value.y.wrapping_neg(),
            value.z.wrapping_neg(),
        )
    }

    pub const fn offset(self, facing: EnumFacing, amount: i32) -> Self {
        if amount == 0 {
            return self;
        }
        let (x, y, z) = facing.offsets();
        self.add(
            x.wrapping_mul(amount),
            y.wrapping_mul(amount),
            z.wrapping_mul(amount),
        )
    }

    pub const fn up(self, amount: i32) -> Self {
        self.offset(EnumFacing::Up, amount)
    }
    pub const fn down(self, amount: i32) -> Self {
        self.offset(EnumFacing::Down, amount)
    }
    pub const fn north(self, amount: i32) -> Self {
        self.offset(EnumFacing::North, amount)
    }
    pub const fn south(self, amount: i32) -> Self {
        self.offset(EnumFacing::South, amount)
    }
    pub const fn west(self, amount: i32) -> Self {
        self.offset(EnumFacing::West, amount)
    }
    pub const fn east(self, amount: i32) -> Self {
        self.offset(EnumFacing::East, amount)
    }

    pub const fn cross_product(self, value: Vec3i) -> Self {
        Self::new(
            self.y
                .wrapping_mul(value.z)
                .wrapping_sub(self.z.wrapping_mul(value.y)),
            self.z
                .wrapping_mul(value.x)
                .wrapping_sub(self.x.wrapping_mul(value.z)),
            self.x
                .wrapping_mul(value.y)
                .wrapping_sub(self.y.wrapping_mul(value.x)),
        )
    }

    pub const fn to_long(self) -> i64 {
        ((((self.x as i64) & Self::X_MASK as i64) << Self::X_SHIFT)
            | (((self.y as i64) & Self::Y_MASK as i64) << Self::Y_SHIFT)
            | ((self.z as i64) & Self::Z_MASK as i64)) as i64
    }

    pub const fn from_long(serialized: i64) -> Self {
        let x = ((serialized << (64 - Self::X_SHIFT - Self::NUM_X_BITS)) >> (64 - Self::NUM_X_BITS))
            as i32;
        let y = ((serialized << (64 - Self::Y_SHIFT - Self::NUM_Y_BITS)) >> (64 - Self::NUM_Y_BITS))
            as i32;
        let z = ((serialized << (64 - Self::NUM_Z_BITS)) >> (64 - Self::NUM_Z_BITS)) as i32;
        Self::new(x, y, z)
    }

    pub fn all_in_box(from: Self, to: Self) -> BlockPosBoxIter {
        BlockPosBoxIter::new(from, to)
    }
}

pub struct BlockPosBoxIter {
    min: BlockPos,
    max: BlockPos,
    current: Option<BlockPos>,
}

impl BlockPosBoxIter {
    fn new(from: BlockPos, to: BlockPos) -> Self {
        let min = BlockPos::new(from.x.min(to.x), from.y.min(to.y), from.z.min(to.z));
        let max = BlockPos::new(from.x.max(to.x), from.y.max(to.y), from.z.max(to.z));
        Self {
            min,
            max,
            current: None,
        }
    }
}

impl Iterator for BlockPosBoxIter {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.current {
            None => self.min,
            Some(current) if current == self.max => return None,
            Some(current) => {
                let mut x = current.x;
                let mut y = current.y;
                let mut z = current.z;
                if x < self.max.x {
                    x += 1;
                } else if y < self.max.y {
                    x = self.min.x;
                    y += 1;
                } else {
                    x = self.min.x;
                    y = self.min.y;
                    z += 1;
                }
                BlockPos::new(x, y, z)
            }
        };
        self.current = Some(next);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_roundtrip_matches_112_layout() {
        for position in [
            BlockPos::ORIGIN,
            BlockPos::new(30_000_000, 255, -30_000_000),
            BlockPos::new(-1, -64, 1),
        ] {
            assert_eq!(BlockPos::from_long(position.to_long()), position);
        }
    }

    #[test]
    fn box_iteration_uses_x_then_y_then_z_order() {
        let values: Vec<_> =
            BlockPos::all_in_box(BlockPos::ORIGIN, BlockPos::new(1, 1, 0)).collect();
        assert_eq!(
            values,
            vec![
                BlockPos::new(0, 0, 0),
                BlockPos::new(1, 0, 0),
                BlockPos::new(0, 1, 0),
                BlockPos::new(1, 1, 0),
            ]
        );
    }
}

impl Ord for BlockPos {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y
            .cmp(&other.y)
            .then_with(|| self.z.cmp(&other.z))
            .then_with(|| self.x.cmp(&other.x))
    }
}
impl PartialOrd for BlockPos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
