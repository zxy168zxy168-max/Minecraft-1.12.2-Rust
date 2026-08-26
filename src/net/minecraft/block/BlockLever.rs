use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockButton;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Metadata-backed port of MCP 1.12.2 `BlockLever.EnumOrientation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumOrientation {
    DownX,
    East,
    West,
    South,
    North,
    UpZ,
    UpX,
    DownZ,
}

pub const fn isBlockLever(state: IBlockState) -> bool {
    state.getBlockId() == 69
}
impl EnumOrientation {
    pub const fn metadata(self) -> i32 {
        match self {
            Self::DownX => 0,
            Self::East => 1,
            Self::West => 2,
            Self::South => 3,
            Self::North => 4,
            Self::UpZ => 5,
            Self::UpX => 6,
            Self::DownZ => 7,
        }
    }
    pub fn forFacings(clicked: EnumFacing, entityFacing: EnumFacing) -> Self {
        match clicked {
            EnumFacing::Down => {
                if entityFacing.axis() == Axis::X {
                    Self::DownX
                } else {
                    Self::DownZ
                }
            }
            EnumFacing::Up => {
                if entityFacing.axis() == Axis::X {
                    Self::UpX
                } else {
                    Self::UpZ
                }
            }
            EnumFacing::North => Self::North,
            EnumFacing::South => Self::South,
            EnumFacing::West => Self::West,
            EnumFacing::East => Self::East,
        }
    }
}

/// MCP `BlockLever#onBlockPlaced`, including its horizontal fallback order.
pub fn onBlockPlacedState<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    facingIn: EnumFacing,
    placerYaw: f32,
) -> IBlockState {
    let entityFacing = EnumFacing::fromAngle(placerYaw as f64);
    let chosen = if BlockButton::canPlaceBlock(world, pos, facingIn) {
        Some(facingIn)
    } else {
        [
            EnumFacing::North,
            EnumFacing::East,
            EnumFacing::South,
            EnumFacing::West,
        ]
        .into_iter()
        .find(|f| *f != facingIn && BlockButton::canPlaceBlock(world, pos, *f))
    };
    let orientation = if let Some(face) = chosen {
        EnumOrientation::forFacings(face, entityFacing)
    } else if world.getBlockState(pos.down(1)).isTopSolid() {
        EnumOrientation::forFacings(EnumFacing::Up, entityFacing)
    } else {
        EnumOrientation::North
    };
    IBlockState::fromGlobalStateId((69 << 4) | orientation.metadata())
}

pub const fn orientation(state: IBlockState) -> EnumOrientation {
    match state.getMetadata() & 7 {
        0 => EnumOrientation::DownX,
        1 => EnumOrientation::East,
        2 => EnumOrientation::West,
        3 => EnumOrientation::South,
        4 => EnumOrientation::North,
        5 => EnumOrientation::UpZ,
        6 => EnumOrientation::UpX,
        _ => EnumOrientation::DownZ,
    }
}

pub const fn isPowered(state: IBlockState) -> bool {
    state.getMetadata() & 8 != 0
}

/// Exact local-space result of `BlockLever#getBoundingBox`.
pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    match orientation(state) {
        EnumOrientation::East => AxisAlignedBB::new(
            0.0,
            0.20000000298023224,
            0.3125,
            0.375,
            0.800000011920929,
            0.6875,
        ),
        EnumOrientation::West => AxisAlignedBB::new(
            0.625,
            0.20000000298023224,
            0.3125,
            1.0,
            0.800000011920929,
            0.6875,
        ),
        EnumOrientation::South => AxisAlignedBB::new(
            0.3125,
            0.20000000298023224,
            0.0,
            0.6875,
            0.800000011920929,
            0.375,
        ),
        EnumOrientation::North => AxisAlignedBB::new(
            0.3125,
            0.20000000298023224,
            0.625,
            0.6875,
            0.800000011920929,
            1.0,
        ),
        EnumOrientation::UpZ | EnumOrientation::UpX => {
            AxisAlignedBB::new(0.25, 0.0, 0.25, 0.75, 0.6000000238418579, 0.75)
        }
        EnumOrientation::DownX | EnumOrientation::DownZ => {
            AxisAlignedBB::new(0.25, 0.4000000059604645, 0.25, 0.75, 1.0, 0.75)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_orientation_and_bounds_match_mcp() {
        let east = IBlockState::fromGlobalStateId((69 << 4) | 1);
        assert_eq!(orientation(east), EnumOrientation::East);
        assert_eq!(getBoundingBox(east).max_x, 0.375);
        let downZPowered = IBlockState::fromGlobalStateId((69 << 4) | 15);
        assert_eq!(orientation(downZPowered), EnumOrientation::DownZ);
        assert!(isPowered(downZPowered));
        assert_eq!(getBoundingBox(downZPowered).min_y, 0.4000000059604645);
    }
}
