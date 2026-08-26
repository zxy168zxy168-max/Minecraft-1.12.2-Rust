#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxisDirection {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumFacing {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl EnumFacing {
    pub const VALUES: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    pub const fn index(self) -> i32 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    /// MCP `EnumFacing#getFront`: protocol/block metadata order is
    /// DOWN, UP, NORTH, SOUTH, WEST, EAST and wraps through the six-value
    /// array for out-of-range indices.
    pub fn getFront(index: i32) -> Self {
        Self::VALUES[index.rem_euclid(Self::VALUES.len() as i32) as usize]
    }

    pub const fn offsets(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    pub const fn axis(self) -> Axis {
        match self {
            Self::Down | Self::Up => Axis::Y,
            Self::North | Self::South => Axis::Z,
            Self::West | Self::East => Axis::X,
        }
    }

    pub const fn axis_direction(self) -> AxisDirection {
        match self {
            Self::Down | Self::North | Self::West => AxisDirection::Negative,
            Self::Up | Self::South | Self::East => AxisDirection::Positive,
        }
    }

    pub const fn rotateY(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            other => other,
        }
    }

    pub const fn rotateYCCW(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            other => other,
        }
    }

    /// MCP `EnumFacing#getHorizontal`: 0=SOUTH, 1=WEST, 2=NORTH, 3=EAST.
    pub fn getHorizontal(index: i32) -> Self {
        match index.rem_euclid(4) {
            0 => Self::South,
            1 => Self::West,
            2 => Self::North,
            _ => Self::East,
        }
    }

    pub const fn horizontalIndex(self) -> Option<u8> {
        match self {
            Self::South => Some(0),
            Self::West => Some(1),
            Self::North => Some(2),
            Self::East => Some(3),
            _ => None,
        }
    }

    /// MCP `EnumFacing#fromAngle`: 0=SOUTH, 90=WEST, 180=NORTH, 270=EAST.
    pub fn fromAngle(angle: f64) -> Self {
        match crate::net::minecraft::util::math::MathHelper::floor_f64(angle / 90.0 + 0.5) & 3 {
            0 => Self::South,
            1 => Self::West,
            2 => Self::North,
            _ => Self::East,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_angle_matches_mcp_cardinal_rounding() {
        assert_eq!(EnumFacing::fromAngle(0.0), EnumFacing::South);
        assert_eq!(EnumFacing::fromAngle(89.9), EnumFacing::West);
        assert_eq!(EnumFacing::fromAngle(180.0), EnumFacing::North);
        assert_eq!(EnumFacing::fromAngle(-90.0), EnumFacing::East);
    }
    #[test]
    fn get_horizontal_matches_protocol_index() {
        assert_eq!(EnumFacing::getHorizontal(0), EnumFacing::South);
        assert_eq!(EnumFacing::getHorizontal(1), EnumFacing::West);
        assert_eq!(EnumFacing::getHorizontal(2), EnumFacing::North);
        assert_eq!(EnumFacing::getHorizontal(3), EnumFacing::East);
        assert_eq!(EnumFacing::getHorizontal(-1), EnumFacing::East);
    }

    #[test]
    fn get_front_matches_block_metadata_order() {
        assert_eq!(EnumFacing::getFront(0), EnumFacing::Down);
        assert_eq!(EnumFacing::getFront(1), EnumFacing::Up);
        assert_eq!(EnumFacing::getFront(2), EnumFacing::North);
        assert_eq!(EnumFacing::getFront(5), EnumFacing::East);
        assert_eq!(EnumFacing::getFront(-1), EnumFacing::East);
    }
}
